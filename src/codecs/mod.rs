use std::{fmt, str::FromStr};

use image::DynamicImage;
use serde::Serialize;

use crate::error::{Result, ShftError};

pub mod jpeg;
pub mod png;
pub mod webp;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageFormat {
    Png,
    Jpg,
    Webp,
    Heic,
    Heif,
}

impl ImageFormat {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpg => "jpg",
            Self::Webp => "webp",
            Self::Heic => "heic",
            Self::Heif => "heif",
        }
    }

    pub fn default_quality(self) -> Option<u8> {
        match self {
            Self::Png => None,
            Self::Jpg => Some(85),
            Self::Webp => Some(82),
            Self::Heic | Self::Heif => None,
        }
    }

    pub fn compression_quality(self) -> Option<u8> {
        match self {
            Self::Png | Self::Heic | Self::Heif => None,
            Self::Jpg | Self::Webp => Some(75),
        }
    }

    pub fn supports_alpha(self) -> bool {
        !matches!(self, Self::Jpg)
    }

    pub fn can_encode(self) -> bool {
        !matches!(self, Self::Heic | Self::Heif)
    }

    pub fn is_heif(self) -> bool {
        matches!(self, Self::Heic | Self::Heif)
    }

    pub fn from_path(path: &std::path::Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?;
        ext.parse().ok()
    }
}

impl fmt::Display for ImageFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.extension())
    }
}

impl FromStr for ImageFormat {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "png" => Ok(Self::Png),
            "jpg" | "jpeg" => Ok(Self::Jpg),
            "webp" => Ok(Self::Webp),
            "heic" => Ok(Self::Heic),
            "heif" => Ok(Self::Heif),
            "avif" => {
                Err("AVIF is not enabled in this build; supported formats: png, jpg, webp".into())
            }
            _ => Err(format!(
                "unknown format `{value}`; expected png, jpg, jpeg, or webp"
            )),
        }
    }
}

pub fn encode(
    image: &DynamicImage,
    format: ImageFormat,
    quality: Option<u8>,
    optimize: bool,
    path: &std::path::Path,
) -> Result<Vec<u8>> {
    match format {
        ImageFormat::Png => png::encode(image, optimize).map_err(|source| ShftError::Encode {
            path: path.to_path_buf(),
            format: format.to_string(),
            detail: source,
        }),
        ImageFormat::Jpg => {
            jpeg::encode(image, quality.unwrap_or(85)).map_err(|source| ShftError::Encode {
                path: path.to_path_buf(),
                format: format.to_string(),
                detail: source,
            })
        }
        ImageFormat::Webp => {
            webp::encode(image, quality.unwrap_or(82)).map_err(|source| ShftError::Encode {
                path: path.to_path_buf(),
                format: format.to_string(),
                detail: source,
            })
        }
        ImageFormat::Heic | ImageFormat::Heif => Err(ShftError::Encode {
            path: path.to_path_buf(),
            format: format.to_string(),
            detail: "HEIC/HEIF encoding is not supported; choose png, jpg, or webp".into(),
        }),
    }
}

pub fn decode(bytes: &[u8], format: ImageFormat, path: &std::path::Path) -> Result<DynamicImage> {
    if format.is_heif() {
        let decoded = heif_oxide::decode_bytes(bytes).map_err(|error| ShftError::Decode {
            path: path.to_path_buf(),
            detail: error.to_string(),
        })?;
        let rgba = image::RgbaImage::from_raw(decoded.width, decoded.height, decoded.to_rgba8())
            .ok_or_else(|| ShftError::Decode {
                path: path.to_path_buf(),
                detail: "decoder returned an invalid RGBA buffer".into(),
            })?;
        Ok(DynamicImage::ImageRgba8(rgba))
    } else {
        image::load_from_memory(bytes).map_err(|error| ShftError::Decode {
            path: path.to_path_buf(),
            detail: error.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_heic_and_heif_extensions() {
        assert_eq!(
            ImageFormat::from_path(std::path::Path::new("photo.HEIC")),
            Some(ImageFormat::Heic)
        );
        assert_eq!(
            ImageFormat::from_path(std::path::Path::new("photo.heif")),
            Some(ImageFormat::Heif)
        );
    }

    #[test]
    fn heif_formats_are_decode_only() {
        assert!(!ImageFormat::Heic.can_encode());
        assert!(!ImageFormat::Heif.can_encode());
    }

    #[test]
    fn compression_uses_a_smaller_lossy_default() {
        assert!(ImageFormat::Jpg.compression_quality() < ImageFormat::Jpg.default_quality());
        assert!(ImageFormat::Webp.compression_quality() < ImageFormat::Webp.default_quality());
    }
}
