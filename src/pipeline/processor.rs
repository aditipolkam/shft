use std::{fs, path::Path, time::Instant};

use image::GenericImageView;

use crate::{
    codecs::{self, ImageFormat},
    config::RunConfig,
    error::{Result, ShftError},
    image_ops::{metadata, resize, transparency},
    output::{naming, writer},
    pipeline::options::TransformPlan,
    presets::Preset,
    reporting::FileResult,
    size_target::optimizer,
    smart::{analyzer, recommendation},
};

pub fn process(path: &Path, input_root: &Path, config: &RunConfig) -> Result<FileResult> {
    let started = Instant::now();
    let source_format = ImageFormat::from_path(path)
        .ok_or_else(|| ShftError::UnsupportedFormat(path.to_path_buf()))?;
    let bytes = fs::read(path).map_err(|source| ShftError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    let decoded = codecs::decode(&bytes, source_format, path)?;
    let mut image = if source_format.is_heif() {
        decoded
    } else {
        metadata::apply_orientation(decoded, metadata::orientation(&bytes))
    };
    let (source_width, source_height) = image.dimensions();
    let analysis = analyzer::analyze(&image, source_format, bytes.len() as u64);
    let smart = if config.smart {
        Some(recommendation::recommend(&analysis))
    } else {
        None
    };
    let email_format = if config.preset == Some(Preset::Email) {
        Some(if analysis.has_alpha {
            ImageFormat::Webp
        } else {
            ImageFormat::Jpg
        })
    } else {
        None
    };
    let default_output = default_output_format(source_format, analysis.has_alpha);
    let output_format = config
        .explicit_format
        .or(email_format)
        .or(smart.map(|item| item.format))
        .unwrap_or(default_output);
    let mut resize_options = config.resize.clone();
    let has_resolved_resize = resize_options.width.is_some()
        || resize_options.height.is_some()
        || resize_options.bounds.is_some()
        || resize_options.scale.is_some()
        || resize_options.max_width.is_some()
        || resize_options.max_height.is_some();
    if let Some(max) = smart
        .and_then(|item| item.max_dimension)
        .filter(|_| !has_resolved_resize)
    {
        if resize_options.max_width.is_none() {
            resize_options.max_width = Some(max)
        }
        if resize_options.max_height.is_none() {
            resize_options.max_height = Some(max)
        }
    }
    let quality = config
        .quality
        .or(smart.and_then(|item| item.quality))
        .or_else(|| {
            if config.compress {
                output_format.compression_quality()
            } else {
                output_format.default_quality()
            }
        });
    let output = naming::destination(path, input_root, output_format, config)?;
    let plan = TransformPlan {
        input: path.to_path_buf(),
        output: output.clone(),
        output_format,
        quality,
        resize: resize_options,
        strip_metadata: config.strip_metadata || smart.is_some_and(|item| item.strip_metadata),
        max_size: config.max_size,
        min_quality: config.min_quality,
        background: config.background,
        overwrite: config.overwrite,
        compress: config.compress,
    };
    image = resize::apply(image, &plan.resize);
    let has_alpha = transparency::has_alpha(&image);
    if has_alpha && !plan.output_format.supports_alpha() {
        image = match plan.background {
            Some(background) => transparency::flatten(&image, background),
            None => return Err(ShftError::TransparencyRequiresBackground),
        };
    }
    let (width, height) = image.dimensions();
    let (encoded, final_quality, mut warning) = encode(&image, &plan)?;
    if source_format.is_heif()
        && plan.output_format == ImageFormat::Png
        && encoded.len() as u64 > bytes.len() as u64
    {
        let growth = encoded.len() as f64 / bytes.len() as f64;
        let message = format!(
            "lossless PNG is {growth:.1}× larger than the HEIC source; use `jpg` or `webp` for photographic content"
        );
        push_warning(&mut warning, message);
    }
    if plan.compress && encoded.len() >= bytes.len() {
        push_warning(
            &mut warning,
            "compression did not reduce this file; its source encoding was already as efficient or more efficient".into(),
        );
    }
    if !config.dry_run {
        writer::atomic_write(&plan.output, &encoded, plan.overwrite)?;
    }
    let _metadata_policy = plan.strip_metadata;
    Ok(FileResult {
        input: plan.input,
        output: plan.output,
        input_bytes: bytes.len() as u64,
        output_bytes: if config.dry_run {
            None
        } else {
            Some(encoded.len() as u64)
        },
        source_width,
        source_height,
        width,
        height,
        format: plan.output_format,
        quality: final_quality,
        has_alpha,
        strip_metadata: plan.strip_metadata,
        elapsed_ms: started.elapsed().as_millis(),
        dry_run: config.dry_run,
        warning,
    })
}

fn push_warning(warning: &mut Option<String>, message: String) {
    *warning = Some(match warning.take() {
        Some(existing) => format!("{existing}; {message}"),
        None => message,
    });
}

fn default_output_format(source: ImageFormat, has_alpha: bool) -> ImageFormat {
    match (source.is_heif(), has_alpha) {
        (true, true) => ImageFormat::Png,
        (true, false) => ImageFormat::Jpg,
        (false, _) => source,
    }
}

fn encode(
    image: &image::DynamicImage,
    plan: &TransformPlan,
) -> Result<(Vec<u8>, Option<u8>, Option<String>)> {
    if let Some(target) = plan.max_size {
        if plan.output_format == ImageFormat::Png {
            let bytes = codecs::encode(image, plan.output_format, None, true, &plan.input)?;
            let warning=(bytes.len() as u64>target).then(||"target size could not be met: PNG quality is lossless; resize or choose JPEG/WebP".to_string());
            return Ok((bytes, None, warning));
        }
        let initial = plan
            .quality
            .or_else(|| plan.output_format.default_quality())
            .unwrap_or(85);
        let result = optimizer::optimize(target, plan.min_quality, initial, |quality| {
            codecs::encode(
                image,
                plan.output_format,
                Some(quality),
                plan.compress,
                &plan.input,
            )
            .map_err(|error| error.to_string())
        })
        .map_err(|source| ShftError::Encode {
            path: plan.input.clone(),
            format: plan.output_format.to_string(),
            detail: source,
        })?;
        let warning = (!result.met_target).then(|| {
            format!(
                "target size could not be met at minimum quality {}; consider resizing",
                plan.min_quality
            )
        });
        Ok((result.bytes, Some(result.quality), warning))
    } else {
        let bytes = codecs::encode(
            image,
            plan.output_format,
            plan.quality,
            plan.compress,
            &plan.input,
        )?;
        Ok((bytes, plan.quality, None))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heif_defaults_to_jpeg_for_opaque_images() {
        assert_eq!(
            default_output_format(ImageFormat::Heic, false),
            ImageFormat::Jpg
        );
    }

    #[test]
    fn heif_defaults_to_png_when_alpha_is_present() {
        assert_eq!(
            default_output_format(ImageFormat::Heif, true),
            ImageFormat::Png
        );
    }
}
