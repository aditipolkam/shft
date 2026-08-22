use image::{
    codecs::png::{CompressionType, FilterType, PngEncoder},
    DynamicImage, ImageEncoder,
};

pub fn encode(image: &DynamicImage, optimize: bool) -> Result<Vec<u8>, String> {
    let rgba = image.to_rgba8();
    let mut bytes = Vec::new();
    let compression = if optimize {
        CompressionType::Best
    } else {
        CompressionType::Fast
    };
    let encoder = PngEncoder::new_with_quality(&mut bytes, compression, FilterType::Adaptive);
    if rgba.pixels().all(|pixel| pixel.0[3] == 255) {
        let rgb = image.to_rgb8();
        encoder
            .write_image(
                &rgb,
                rgb.width(),
                rgb.height(),
                image::ExtendedColorType::Rgb8,
            )
            .map_err(|error| error.to_string())?;
    } else {
        encoder
            .write_image(
                &rgba,
                rgba.width(),
                rgba.height(),
                image::ExtendedColorType::Rgba8,
            )
            .map_err(|error| error.to_string())?;
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{GenericImage, ImageBuffer, Rgba};

    #[test]
    fn omits_unused_alpha_channel() {
        let image = DynamicImage::ImageRgba8(ImageBuffer::from_pixel(4, 4, Rgba([1, 2, 3, 255])));
        let encoded = encode(&image, true).unwrap();
        let decoded = image::load_from_memory(&encoded).unwrap();
        assert!(!decoded.color().has_alpha());
    }

    #[test]
    fn optimized_encoding_is_no_larger_than_fast_encoding() {
        let mut image = DynamicImage::new_rgba8(128, 128);
        for y in 0..128 {
            for x in 0..128 {
                image.put_pixel(x, y, Rgba([(x % 16) as u8, (y % 16) as u8, 80, 255]));
            }
        }
        let fast = encode(&image, false).unwrap();
        let optimized = encode(&image, true).unwrap();
        assert!(optimized.len() <= fast.len());
    }
}
