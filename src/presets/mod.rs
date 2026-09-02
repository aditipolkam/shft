use clap::ValueEnum;

use crate::{codecs::ImageFormat, config::ResizeBox};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Preset {
    Web,
    Thumbnail,
    Avatar,
    Email,
}

#[derive(Debug, Clone)]
pub struct PresetDefaults {
    pub format: Option<ImageFormat>,
    pub quality: Option<u8>,
    pub bounds: Option<ResizeBox>,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
    pub strip_metadata: bool,
}

pub fn defaults(preset: Preset) -> PresetDefaults {
    match preset {
        Preset::Web => PresetDefaults {
            format: Some(ImageFormat::Webp),
            quality: Some(82),
            bounds: None,
            max_width: Some(2560),
            max_height: Some(2560),
            strip_metadata: true,
        },
        Preset::Thumbnail => PresetDefaults {
            format: Some(ImageFormat::Webp),
            quality: Some(75),
            bounds: Some(ResizeBox {
                width: 400,
                height: 400,
            }),
            max_width: None,
            max_height: None,
            strip_metadata: true,
        },
        Preset::Avatar => PresetDefaults {
            format: Some(ImageFormat::Webp),
            quality: Some(90),
            bounds: Some(ResizeBox {
                width: 512,
                height: 512,
            }),
            max_width: None,
            max_height: None,
            strip_metadata: true,
        },
        Preset::Email => PresetDefaults {
            format: None,
            quality: Some(72),
            bounds: None,
            max_width: Some(1600),
            max_height: None,
            strip_metadata: true,
        },
    }
}
