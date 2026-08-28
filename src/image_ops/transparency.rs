use image::{DynamicImage, GenericImageView, Rgba, RgbaImage};

use crate::config::RgbaColor;

pub fn has_alpha(image: &DynamicImage) -> bool {
    if !image.color().has_alpha() {
        return false;
    }
    image.to_rgba8().pixels().any(|pixel| pixel.0[3] < 255)
}

pub fn flatten(image: &DynamicImage, background: RgbaColor) -> DynamicImage {
    let (width, height) = image.dimensions();
    let mut output = RgbaImage::from_pixel(width, height, Rgba(background.0));
    image::imageops::overlay(&mut output, &image.to_rgba8(), 0, 0);
    DynamicImage::ImageRgba8(output)
}
