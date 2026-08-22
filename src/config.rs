use std::{path::PathBuf, str::FromStr};

use crate::{
    cli::TransformArgs,
    codecs::ImageFormat,
    error::{Result, ShftError},
    presets::{self, Preset},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResizeBox {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RgbaColor(pub [u8; 4]);

#[derive(Debug, Clone, PartialEq)]
pub struct ResizeOptions {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub bounds: Option<ResizeBox>,
    pub scale: Option<f64>,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
    pub allow_upscale: bool,
}

#[derive(Debug, Clone)]
pub struct RunConfig {
    pub inputs: Vec<PathBuf>,
    pub explicit_format: Option<ImageFormat>,
    pub quality: Option<u8>,
    pub compress: bool,
    pub resize: ResizeOptions,
    pub strip_metadata: bool,
    pub max_size: Option<u64>,
    pub min_quality: u8,
    pub background: Option<RgbaColor>,
    pub overwrite: bool,
    pub output: Option<PathBuf>,
    pub output_dir: Option<PathBuf>,
    pub suffix: String,
    pub recursive: bool,
    pub jobs: usize,
    pub fail_fast: bool,
    pub dry_run: bool,
    pub smart: bool,
    pub quiet: bool,
    pub json: bool,
    pub no_color: bool,
    pub preset: Option<Preset>,
}

impl RunConfig {
    pub fn resolve(args: TransformArgs) -> Result<Self> {
        let input = args.input.ok_or_else(|| {
            ShftError::InvalidArguments("missing <INPUT>; run `shft --help` for usage".into())
        })?;
        let mut inputs = vec![input];
        let positional_format = match args.format {
            Some(value) => match value.parse::<ImageFormat>() {
                Ok(format) => Some(format),
                Err(_) => {
                    inputs.push(PathBuf::from(value));
                    None
                }
            },
            None => None,
        };
        inputs.extend(args.additional_inputs);
        if positional_format.is_some() && args.to.is_some() && positional_format != args.to {
            return Err(ShftError::InvalidArguments(
                "positional FORMAT and --to disagree".into(),
            ));
        }
        if args.output.is_some() && (inputs.len() != 1 || inputs[0].is_dir()) {
            return Err(ShftError::InvalidArguments(
                "--output is only valid for a single input file; use --output-dir for directories"
                    .into(),
            ));
        }
        let preset = if args.web {
            Some(Preset::Web)
        } else {
            args.preset
        };
        let defaults = preset.map(presets::defaults);
        let has_explicit_resize = args.width.is_some()
            || args.height.is_some()
            || args.resize.is_some()
            || args.scale.is_some();
        let explicit_format = args
            .to
            .or(positional_format)
            .or(defaults.as_ref().and_then(|d| d.format));
        if explicit_format.is_some_and(|format| !format.can_encode()) {
            return Err(ShftError::InvalidArguments(
                "HEIC/HEIF are input-only formats; choose --to png, jpg, or webp".into(),
            ));
        }
        let quality = args.quality.or(defaults.as_ref().and_then(|d| d.quality));
        let resize = ResizeOptions {
            width: args.width,
            height: args.height,
            bounds: args.resize.or_else(|| {
                (!has_explicit_resize)
                    .then(|| defaults.as_ref().and_then(|defaults| defaults.bounds))
                    .flatten()
            }),
            scale: args.scale,
            max_width: args
                .max_width
                .or(defaults.as_ref().and_then(|d| d.max_width)),
            max_height: args
                .max_height
                .or(defaults.as_ref().and_then(|d| d.max_height)),
            allow_upscale: args.allow_upscale,
        };
        let resize_modes = [
            resize.width.is_some() || resize.height.is_some(),
            resize.bounds.is_some(),
            resize.scale.is_some(),
        ];
        if resize_modes.into_iter().filter(|enabled| *enabled).count() > 1 {
            return Err(ShftError::InvalidArguments(
                "use only one of --width/--height, --resize, or --scale".into(),
            ));
        }
        if args.min_quality > quality.unwrap_or(100) && args.max_size.is_some() {
            return Err(ShftError::InvalidArguments(
                "--min-quality cannot exceed the starting quality".into(),
            ));
        }
        Ok(Self {
            inputs,
            explicit_format,
            quality,
            compress: args.compress,
            resize,
            strip_metadata: if args.keep_metadata {
                false
            } else {
                args.strip_metadata || defaults.as_ref().is_some_and(|d| d.strip_metadata)
            },
            max_size: args.max_size,
            min_quality: args.min_quality,
            background: args.background,
            overwrite: args.overwrite,
            output: args.output,
            output_dir: args.output_dir,
            suffix: args.suffix,
            recursive: args.recursive,
            jobs: args
                .jobs
                .unwrap_or_else(|| std::thread::available_parallelism().map_or(1, usize::from)),
            fail_fast: args.fail_fast,
            dry_run: args.dry_run,
            smart: args.smart,
            quiet: args.quiet,
            json: args.json,
            no_color: args.no_color,
            preset,
        })
    }
}

pub fn parse_size(value: &str) -> std::result::Result<u64, String> {
    let normalized = value.trim().to_ascii_lowercase().replace(' ', "");
    let split = normalized
        .find(|ch: char| !ch.is_ascii_digit())
        .unwrap_or(normalized.len());
    let (number, unit) = normalized.split_at(split);
    let amount = u64::from_str(number).map_err(|_| format!("invalid size `{value}`"))?;
    if amount == 0 {
        return Err("size must be greater than zero".into());
    }
    let multiplier = match unit {
        "" | "b" | "bytes" => 1,
        "k" | "kb" => 1_000,
        "m" | "mb" => 1_000_000,
        _ => {
            return Err(format!(
                "unsupported size unit `{unit}`; use bytes, KB, or MB"
            ))
        }
    };
    amount
        .checked_mul(multiplier)
        .ok_or_else(|| "size is too large".into())
}

pub fn parse_resize(value: &str) -> std::result::Result<ResizeBox, String> {
    let normalized = value.to_ascii_lowercase();
    let (width, height) = normalized
        .split_once('x')
        .ok_or_else(|| "expected WIDTHxHEIGHT, for example 1200x800".to_string())?;
    let width = width
        .parse::<u32>()
        .map_err(|_| "resize width must be a positive integer".to_string())?;
    let height = height
        .parse::<u32>()
        .map_err(|_| "resize height must be a positive integer".to_string())?;
    if width == 0 || height == 0 {
        return Err("resize dimensions must be greater than zero".into());
    }
    Ok(ResizeBox { width, height })
}

pub fn parse_percent(value: &str) -> std::result::Result<f64, String> {
    let percent = value
        .trim()
        .trim_end_matches('%')
        .parse::<f64>()
        .map_err(|_| format!("invalid scale `{value}`"))?;
    if !percent.is_finite() || percent <= 0.0 {
        return Err("scale must be greater than zero".into());
    }
    Ok(percent)
}

pub fn parse_background(value: &str) -> std::result::Result<RgbaColor, String> {
    let hex = value.trim().trim_start_matches('#');
    if hex.len() != 6 {
        return Err("background must be a six-digit color such as #ffffff".into());
    }
    let rgb = u32::from_str_radix(hex, 16)
        .map_err(|_| "background must contain hexadecimal digits".to_string())?;
    Ok(RgbaColor([
        ((rgb >> 16) & 0xff) as u8,
        ((rgb >> 8) & 0xff) as u8,
        (rgb & 0xff) as u8,
        255,
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_sizes() {
        assert_eq!(parse_size("500KB"), Ok(500_000));
        assert_eq!(parse_size("1mb"), Ok(1_000_000));
        assert_eq!(parse_size("42"), Ok(42));
    }
    #[test]
    fn rejects_zero_size() {
        assert!(parse_size("0kb").is_err());
    }
    #[test]
    fn parses_resize_box() {
        assert_eq!(
            parse_resize("1200x800"),
            Ok(ResizeBox {
                width: 1200,
                height: 800
            })
        );
    }
    #[test]
    fn parses_background_color() {
        assert_eq!(
            parse_background("#ff0080"),
            Ok(RgbaColor([255, 0, 128, 255]))
        );
    }
}
