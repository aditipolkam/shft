use image::{codecs::jpeg::JpegEncoder, DynamicImage};

pub fn encode(image: &DynamicImage, quality: u8) -> Result<Vec<u8>, String> {
    let rgb = image.to_rgb8();
    let mut bytes = Vec::new();
    JpegEncoder::new_with_quality(&mut bytes, quality)
        .encode_image(&rgb)
        .map_err(|error| error.to_string())?;
    Ok(bytes)
}
