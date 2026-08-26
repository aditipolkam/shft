use image::{imageops::FilterType, DynamicImage, GenericImageView};

use crate::config::ResizeOptions;

pub fn dimensions(source_width: u32, source_height: u32, options: &ResizeOptions) -> (u32, u32) {
    let sw = source_width as f64;
    let sh = source_height as f64;
    let mut ratio = 1.0_f64;

    if let Some(scale) = options.scale {
        ratio = scale / 100.0;
    } else if let Some(bounds) = options.bounds {
        ratio = (bounds.width as f64 / sw).min(bounds.height as f64 / sh);
    } else if options.width.is_some() || options.height.is_some() {
        ratio = match (options.width, options.height) {
            (Some(width), Some(height)) => (width as f64 / sw).min(height as f64 / sh),
            (Some(width), None) => width as f64 / sw,
            (None, Some(height)) => height as f64 / sh,
            (None, None) => 1.0,
        };
    }

    if let Some(max_width) = options.max_width {
        ratio = ratio.min(max_width as f64 / sw);
    }
    if let Some(max_height) = options.max_height {
        ratio = ratio.min(max_height as f64 / sh);
    }
    if !options.allow_upscale {
        ratio = ratio.min(1.0);
    }
    let width = (sw * ratio).round().max(1.0) as u32;
    let height = (sh * ratio).round().max(1.0) as u32;
    (width, height)
}

pub fn apply(image: DynamicImage, options: &ResizeOptions) -> DynamicImage {
    let (source_width, source_height) = image.dimensions();
    let (width, height) = dimensions(source_width, source_height, options);
    if (width, height) == (source_width, source_height) {
        image
    } else {
        image.resize_exact(width, height, FilterType::Lanczos3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn options() -> ResizeOptions {
        ResizeOptions {
            width: None,
            height: None,
            bounds: None,
            scale: None,
            max_width: None,
            max_height: None,
            allow_upscale: false,
        }
    }
    #[test]
    fn preserves_aspect_ratio_for_width() {
        let mut o = options();
        o.width = Some(1000);
        assert_eq!(dimensions(4000, 3000, &o), (1000, 750));
    }
    #[test]
    fn fits_inside_bounds() {
        let mut o = options();
        o.bounds = Some(crate::config::ResizeBox {
            width: 1200,
            height: 800,
        });
        assert_eq!(dimensions(4000, 3000, &o), (1067, 800));
    }
    #[test]
    fn does_not_upscale_by_default() {
        let mut o = options();
        o.width = Some(1000);
        assert_eq!(dimensions(400, 300, &o), (400, 300));
    }
    #[test]
    fn can_upscale() {
        let mut o = options();
        o.width = Some(800);
        o.allow_upscale = true;
        assert_eq!(dimensions(400, 300, &o), (800, 600));
    }
}
