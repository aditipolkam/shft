use std::{
    io::{self, IsTerminal, Write},
    path::{Path, PathBuf},
};

use crate::{
    codecs::ImageFormat,
    config::RunConfig,
    error::{Result, ShftError},
};

pub fn destination(
    input: &Path,
    input_root: &Path,
    format: ImageFormat,
    config: &RunConfig,
) -> Result<PathBuf> {
    if let Some(output) = &config.output {
        return Ok(output.clone());
    }
    let stem = input
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            ShftError::InvalidArguments(format!(
                "cannot derive an output name from `{}`",
                input.display()
            ))
        })?;
    let mut name = format!("{}{}.{}", stem, config.suffix, format.extension());
    let mut parent = input
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    if let Some(output_dir) = &config.output_dir {
        let relative_parent = if input_root.is_dir() {
            input
                .parent()
                .and_then(|p| p.strip_prefix(input_root).ok())
                .unwrap_or_else(|| Path::new(""))
        } else {
            Path::new("")
        };
        parent = output_dir.join(relative_parent);
    }
    let mut output = parent.join(&name);
    if output == input && !config.overwrite {
        name = format!("{}{}-optimized.{}", stem, config.suffix, format.extension());
        output = parent.join(name);
    }
    if output.exists() && !config.overwrite {
        if can_prompt(config) {
            return prompt_for_destination(&output, input, format);
        }
        return Err(ShftError::OutputExists(output));
    }
    Ok(output)
}

fn can_prompt(config: &RunConfig) -> bool {
    config.output.is_none()
        && config.inputs.len() == 1
        && config.inputs[0].is_file()
        && !config.json
        && !config.quiet
        && io::stdin().is_terminal()
        && io::stderr().is_terminal()
}

fn prompt_for_destination(existing: &Path, input: &Path, format: ImageFormat) -> Result<PathBuf> {
    let parent = existing.parent().unwrap_or_else(|| Path::new("."));
    loop {
        eprint!(
            "Output `{}` already exists.\nEnter a custom output name (blank to cancel): ",
            existing.display()
        );
        io::stderr().flush().map_err(ShftError::Prompt)?;
        let mut value = String::new();
        io::stdin()
            .read_line(&mut value)
            .map_err(ShftError::Prompt)?;
        let value = value.trim();
        if value.is_empty() {
            return Err(ShftError::OutputExists(existing.to_path_buf()));
        }
        let candidate = normalize_custom_name(value, parent, format);
        if ImageFormat::from_path(&candidate) != Some(format) {
            eprintln!("Please use a .{} extension.", format.extension());
            continue;
        }
        if candidate == input {
            eprintln!("That name would overwrite the input. Enter a different name.");
            continue;
        }
        if candidate.exists() {
            eprintln!(
                "`{}` already exists. Enter a different name.",
                candidate.display()
            );
            continue;
        }
        return Ok(candidate);
    }
}

fn normalize_custom_name(value: &str, parent: &Path, format: ImageFormat) -> PathBuf {
    let entered = PathBuf::from(value);
    let has_directory = entered.is_absolute()
        || entered
            .parent()
            .is_some_and(|path| !path.as_os_str().is_empty());
    let mut candidate = if has_directory {
        entered
    } else {
        parent.join(entered)
    };
    if candidate.extension().is_none() {
        candidate.set_extension(format.extension());
    }
    candidate
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ResizeOptions, RunConfig};
    fn config(input: PathBuf) -> RunConfig {
        RunConfig {
            inputs: vec![input],
            explicit_format: None,
            quality: None,
            compress: false,
            resize: ResizeOptions {
                width: None,
                height: None,
                bounds: None,
                scale: None,
                max_width: None,
                max_height: None,
                allow_upscale: false,
            },
            strip_metadata: false,
            max_size: None,
            min_quality: 20,
            background: None,
            overwrite: false,
            output: None,
            output_dir: None,
            suffix: String::new(),
            recursive: false,
            jobs: 1,
            fail_fast: false,
            dry_run: false,
            smart: false,
            quiet: false,
            json: false,
            no_color: false,
            preset: None,
        }
    }
    #[test]
    fn same_format_gets_safe_suffix() {
        let input = PathBuf::from("photo.jpg");
        assert_eq!(
            destination(
                &input,
                Path::new("."),
                ImageFormat::Jpg,
                &config(input.clone())
            )
            .unwrap(),
            PathBuf::from("photo-optimized.jpg")
        );
    }
    #[test]
    fn conversion_changes_extension() {
        let input = PathBuf::from("photo.png");
        assert_eq!(
            destination(
                &input,
                Path::new("."),
                ImageFormat::Webp,
                &config(input.clone())
            )
            .unwrap(),
            PathBuf::from("photo.webp")
        );
    }

    #[test]
    fn custom_name_inherits_directory_and_extension() {
        assert_eq!(
            normalize_custom_name("vacation", Path::new("photos"), ImageFormat::Jpg),
            PathBuf::from("photos/vacation.jpg")
        );
    }

    #[test]
    fn custom_absolute_path_is_preserved() {
        assert_eq!(
            normalize_custom_name("/tmp/vacation.jpg", Path::new("photos"), ImageFormat::Jpg),
            PathBuf::from("/tmp/vacation.jpg")
        );
    }
}
