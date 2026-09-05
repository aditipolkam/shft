pub mod json;
pub mod terminal;

use std::{path::PathBuf, time::Duration};

use serde::Serialize;

use crate::codecs::ImageFormat;

#[derive(Debug, Clone, Serialize)]
pub struct FileResult {
    pub input: PathBuf,
    pub output: PathBuf,
    pub input_bytes: u64,
    pub output_bytes: Option<u64>,
    pub source_width: u32,
    pub source_height: u32,
    pub width: u32,
    pub height: u32,
    pub format: ImageFormat,
    pub quality: Option<u8>,
    pub has_alpha: bool,
    pub strip_metadata: bool,
    pub elapsed_ms: u128,
    pub dry_run: bool,
    pub warning: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileFailure {
    pub input: PathBuf,
    pub error: String,
}

#[derive(Debug, Serialize)]
pub struct Report {
    pub processed: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub input_bytes: u64,
    pub output_bytes: u64,
    pub elapsed_ms: u128,
    pub dry_run: bool,
    pub files: Vec<FileResult>,
    pub errors: Vec<FileFailure>,
}

impl Report {
    pub fn new(
        files: Vec<FileResult>,
        errors: Vec<FileFailure>,
        elapsed: Duration,
        dry_run: bool,
    ) -> Self {
        let input_bytes = files.iter().map(|file| file.input_bytes).sum();
        let output_bytes = files.iter().filter_map(|file| file.output_bytes).sum();
        Self {
            processed: files.len() + errors.len(),
            succeeded: files.len(),
            failed: errors.len(),
            input_bytes,
            output_bytes,
            elapsed_ms: elapsed.as_millis(),
            dry_run,
            files,
            errors,
        }
    }
}
