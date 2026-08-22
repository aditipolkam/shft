use std::{error::Error, fs, io::IsTerminal, path::Path, process::ExitCode};

use clap::Parser;
use image::GenericImageView;
use shft::{
    cli::{Cli, Command},
    codecs::ImageFormat,
    config::RunConfig,
    error::{Result, ShftError},
    image_ops::transparency,
    pipeline, reporting,
};

fn main() -> ExitCode {
    match execute() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("error: {error}");
            let mut source = error.source();
            while let Some(cause) = source {
                eprintln!("caused by: {cause}");
                source = cause.source();
            }
            ExitCode::from(2)
        }
    }
}

fn execute() -> Result<ExitCode> {
    let cli = Cli::parse();
    if let Some(Command::Preview { input }) = cli.command {
        preview(&input)?;
        return Ok(ExitCode::SUCCESS);
    }
    let config = RunConfig::resolve(cli.transform)?;
    let colors = !config.no_color
        && !config.json
        && std::env::var_os("NO_COLOR").is_none()
        && std::io::stdout().is_terminal();
    console::set_colors_enabled(colors);
    let report = pipeline::run(&config)?;
    if config.json {
        reporting::json::print(&report)?;
    } else if !config.quiet {
        reporting::terminal::print(&report, colors);
    }
    if config.quiet {
        for error in &report.errors {
            eprintln!("error: {}: {}", error.input.display(), error.error);
        }
    }
    Ok(if report.failed == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

fn preview(path: &Path) -> Result<()> {
    let data = fs::read(path).map_err(|source| ShftError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    let format = ImageFormat::from_path(path)
        .ok_or_else(|| ShftError::UnsupportedFormat(path.to_path_buf()))?;
    let image = shft::codecs::decode(&data, format, path)?;
    let (width, height) = image.dimensions();
    println!(
        "{}\n  Format: {}\n  Dimensions: {}×{}\n  File size: {}\n  Transparency: {}",
        path.display(),
        format,
        width,
        height,
        reporting::terminal::bytes(data.len() as u64),
        if transparency::has_alpha(&image) {
            "yes"
        } else {
            "no"
        }
    );
    if format.is_heif() {
        println!("  HEIF orientation transforms: applied");
    } else {
        println!(
            "  EXIF orientation: {}",
            shft::image_ops::metadata::orientation(&data)
        );
    }
    Ok(())
}
