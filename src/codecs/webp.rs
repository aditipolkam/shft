use image::DynamicImage;

pub fn encode(image: &DynamicImage, quality: u8) -> Result<Vec<u8>, String> {
    let encoder = webp::Encoder::from_image(image).map_err(str::to_owned)?;
    Ok(encoder.encode(quality as f32).to_vec())
}
