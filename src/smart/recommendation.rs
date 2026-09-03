use crate::{codecs::ImageFormat, smart::analyzer::Analysis};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Recommendation {
    pub format: ImageFormat,
    pub quality: Option<u8>,
    pub max_dimension: Option<u32>,
    pub strip_metadata: bool,
}

pub fn recommend(analysis: &Analysis) -> Recommendation {
    if analysis.has_alpha {
        Recommendation {
            format: ImageFormat::Webp,
            quality: Some(90),
            max_dimension: if analysis.width.max(analysis.height) > 2560 {
                Some(2560)
            } else {
                None
            },
            strip_metadata: true,
        }
    } else if analysis.source_format == ImageFormat::Png
        || analysis.source_format.is_heif()
        || analysis.input_bytes > 1_000_000
        || analysis.width.max(analysis.height) > 2560
    {
        Recommendation {
            format: ImageFormat::Webp,
            quality: Some(82),
            max_dimension: Some(2560),
            strip_metadata: true,
        }
    } else {
        Recommendation {
            format: analysis.source_format,
            quality: analysis.source_format.default_quality(),
            max_dimension: None,
            strip_metadata: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::smart::analyzer::Analysis;
    #[test]
    fn photographic_png_becomes_webp() {
        let a = Analysis {
            source_format: ImageFormat::Png,
            width: 6000,
            height: 4000,
            input_bytes: 8_000_000,
            has_alpha: false,
        };
        let r = recommend(&a);
        assert_eq!(r.format, ImageFormat::Webp);
        assert_eq!(r.quality, Some(82));
        assert_eq!(r.max_dimension, Some(2560));
    }
    #[test]
    fn transparent_asset_uses_high_quality() {
        let a = Analysis {
            source_format: ImageFormat::Png,
            width: 1200,
            height: 600,
            input_bytes: 500_000,
            has_alpha: true,
        };
        assert_eq!(recommend(&a).quality, Some(90));
    }
}
