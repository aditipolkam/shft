use std::path::PathBuf;

use clap::{ArgAction, Args, Parser, Subcommand};

use crate::{
    codecs::ImageFormat,
    config::{parse_background, parse_percent, parse_resize, parse_size, ResizeBox, RgbaColor},
};

const AFTER_HELP: &str = "EXAMPLES:\n  shft photo.png webp\n  shft photo.jpg --compress\n  shft photo.png webp -q 80\n  shft photo.jpg --width 1200\n  shft photo.jpg --max-size 500kb\n  shft ./images --web -r\n  shft ./images --web --dry-run\n  shft preview photo.webp";

#[derive(Debug, Parser)]
#[command(name = "shft", version, about = "Fast, offline, safe-by-default image optimization", long_about = None, after_help = AFTER_HELP)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    #[command(flatten)]
    pub transform: TransformArgs,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Inspect an image without modifying it
    Preview { input: PathBuf },
}

#[derive(Debug, Args)]
pub struct TransformArgs {
    /// First image file or directory to process
    pub input: Option<PathBuf>,

    /// Output format (png, jpg/jpeg, webp), or another shell-expanded input
    pub format: Option<String>,

    /// Additional shell-expanded input files
    #[arg(value_name = "INPUT")]
    pub additional_inputs: Vec<PathBuf>,

    /// Explicit output format (png, jpg/jpeg, or webp)
    #[arg(long = "to", value_name = "FORMAT")]
    pub to: Option<ImageFormat>,

    /// Re-encode using sensible compression defaults
    #[arg(long)]
    pub compress: bool,

    /// Lossy output quality, from 1 (smallest) to 100 (highest quality)
    #[arg(short = 'q', long, value_parser = clap::value_parser!(u8).range(1..=100))]
    pub quality: Option<u8>,

    /// Resize to this width while preserving aspect ratio
    #[arg(long, value_parser = clap::value_parser!(u32).range(1..))]
    pub width: Option<u32>,

    /// Resize to this height while preserving aspect ratio
    #[arg(long, value_parser = clap::value_parser!(u32).range(1..))]
    pub height: Option<u32>,

    /// Fit within WIDTHxHEIGHT while preserving aspect ratio
    #[arg(long, value_parser = parse_resize, value_name = "WIDTHxHEIGHT")]
    pub resize: Option<ResizeBox>,

    /// Scale dimensions by PERCENT (for example 50 or 50%)
    #[arg(long, value_parser = parse_percent)]
    pub scale: Option<f64>,

    /// Constrain output width without cropping
    #[arg(long, value_parser = clap::value_parser!(u32).range(1..))]
    pub max_width: Option<u32>,

    /// Constrain output height without cropping
    #[arg(long, value_parser = clap::value_parser!(u32).range(1..))]
    pub max_height: Option<u32>,

    /// Permit resizing beyond the source dimensions
    #[arg(long)]
    pub allow_upscale: bool,

    /// Web preset: WebP, quality 82, max edge 2560, metadata stripped
    #[arg(long, conflicts_with = "preset")]
    pub web: bool,

    /// Named optimization preset
    #[arg(long, value_enum)]
    pub preset: Option<crate::presets::Preset>,

    /// Target maximum encoded size (bytes, KB, or MB)
    #[arg(long, value_parser = parse_size, value_name = "SIZE")]
    pub max_size: Option<u64>,

    /// Minimum quality used by target-size optimization
    #[arg(long, default_value_t = 20, value_parser = clap::value_parser!(u8).range(1..=100))]
    pub min_quality: u8,

    /// Deterministically choose a strategy from image properties
    #[arg(long)]
    pub smart: bool,

    /// Include supported images in nested directories
    #[arg(short = 'r', long)]
    pub recursive: bool,

    /// Maximum concurrent image workers
    #[arg(long, value_parser = parse_jobs)]
    pub jobs: Option<usize>,

    /// Stop scheduling useful work after the first file failure
    #[arg(long)]
    pub fail_fast: bool,

    /// Write one file to PATH
    #[arg(short = 'o', long, value_name = "PATH", conflicts_with = "output_dir")]
    pub output: Option<PathBuf>,

    /// Place outputs beneath DIR
    #[arg(long, value_name = "DIR")]
    pub output_dir: Option<PathBuf>,

    /// Append TEXT to output stems
    #[arg(long, default_value = "")]
    pub suffix: String,

    /// Replace existing destination files (using atomic writes)
    #[arg(long)]
    pub overwrite: bool,

    /// Show resolved transformations without writing files
    #[arg(long)]
    pub dry_run: bool,

    /// Remove metadata from encoded output
    #[arg(long, action = ArgAction::SetTrue, conflicts_with = "keep_metadata")]
    pub strip_metadata: bool,

    /// Preserve metadata where the selected codec supports it
    #[arg(long, action = ArgAction::SetTrue, conflicts_with = "strip_metadata")]
    pub keep_metadata: bool,

    /// Flatten transparency against a color when writing JPEG (#RRGGBB)
    #[arg(long, value_parser = parse_background)]
    pub background: Option<RgbaColor>,

    /// Suppress successful result output
    #[arg(long, conflicts_with = "json")]
    pub quiet: bool,

    /// Emit one structured JSON report
    #[arg(long, conflicts_with = "quiet")]
    pub json: bool,

    /// Disable terminal colors
    #[arg(long)]
    pub no_color: bool,
}

fn parse_jobs(value: &str) -> std::result::Result<usize, String> {
    let jobs = value
        .parse::<usize>()
        .map_err(|_| "jobs must be a positive integer".to_string())?;
    if jobs == 0 {
        return Err("jobs must be greater than zero".into());
    }
    Ok(jobs)
}
