use image::{DynamicImage, GenericImageView};

use crate::{codecs::ImageFormat, image_ops::transparency};

#[derive(Debug, Clone)]
pub struct Analysis {
    pub source_format: ImageFormat,
    pub width: u32,
    pub height: u32,
    pub input_bytes: u64,
    pub has_alpha: bool,
}

pub fn analyze(image: &DynamicImage, source_format: ImageFormat, input_bytes: u64) -> Analysis {
    let (width, height) = image.dimensions();
    Analysis {
        source_format,
        width,
        height,
        input_bytes,
        has_alpha: transparency::has_alpha(image),
    }
}
