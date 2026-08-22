use std::path::PathBuf;

use crate::{
    codecs::ImageFormat,
    config::{ResizeOptions, RgbaColor},
};

#[derive(Debug, Clone)]
pub struct TransformPlan {
    pub input: PathBuf,
    pub output: PathBuf,
    pub output_format: ImageFormat,
    pub quality: Option<u8>,
    pub resize: ResizeOptions,
    pub strip_metadata: bool,
    pub max_size: Option<u64>,
    pub min_quality: u8,
    pub background: Option<RgbaColor>,
    pub overwrite: bool,
    pub compress: bool,
}
