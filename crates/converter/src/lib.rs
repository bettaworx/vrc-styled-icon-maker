use image::GenericImageView;
use serde::Deserialize;
use thiserror::Error;
use vtracer::{ColorImage, Config};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColorMode {
    Color,
    Binary,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Hierarchical {
    Stacked,
    Cutout,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct VtracerConfig {
    pub color_mode: ColorMode,
    pub hierarchical: Hierarchical,
    pub filter_speckle: usize,
    pub color_precision: i32,
    pub layer_difference: i32,
    pub corner_threshold: i32,
    pub length_threshold: f64,
    pub splice_threshold: i32,
    pub max_iterations: usize,
    pub path_precision: Option<u32>,
}

impl Default for VtracerConfig {
    fn default() -> Self {
        Self {
            color_mode: ColorMode::Binary,
            hierarchical: Hierarchical::Stacked,
            filter_speckle: 4,
            color_precision: 6,
            layer_difference: 16,
            corner_threshold: 60,
            length_threshold: 4.0,
            splice_threshold: 45,
            max_iterations: 10,
            path_precision: Some(2),
        }
    }
}

impl VtracerConfig {
    pub fn from_params_str(s: &str) -> Result<Self, serde_keyvalue::ParseError> {
        if s.is_empty() {
            return Ok(Self::default());
        }
        serde_keyvalue::from_key_values(s)
    }

    fn to_vtracer_config(&self) -> Config {
        Config {
            color_mode: match self.color_mode {
                ColorMode::Color => vtracer::ColorMode::Color,
                ColorMode::Binary => vtracer::ColorMode::Binary,
            },
            hierarchical: match self.hierarchical {
                Hierarchical::Stacked => vtracer::Hierarchical::Stacked,
                Hierarchical::Cutout => vtracer::Hierarchical::Cutout,
            },
            filter_speckle: self.filter_speckle,
            color_precision: self.color_precision,
            layer_difference: self.layer_difference,
            corner_threshold: self.corner_threshold,
            length_threshold: self.length_threshold,
            splice_threshold: self.splice_threshold,
            max_iterations: self.max_iterations,
            path_precision: self.path_precision,
            ..Config::default()
        }
    }
}

#[derive(Debug, Error)]
pub enum ConvertError {
    #[error("failed to decode image: {0}")]
    ImageDecode(String),
    #[error("vectorization failed: {0}")]
    Vectorize(String),
}

fn luminance(r: u8, g: u8, b: u8) -> f32 {
    0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32
}

fn edge_mean_luminance(pixels: &[u8], width: usize, height: usize) -> f32 {
    let mut sum = 0.0f64;
    let mut count = 0usize;
    for y in 0..height {
        for x in 0..width {
            if y == 0 || y == height - 1 || x == 0 || x == width - 1 {
                let i = (y * width + x) * 4;
                sum += luminance(pixels[i], pixels[i + 1], pixels[i + 2]) as f64;
                count += 1;
            }
        }
    }
    if count == 0 { 128.0 } else { (sum / count as f64) as f32 }
}

fn preprocess_for_tracing(pixels: &mut [u8], width: usize, height: usize) {
    const ALPHA_THRESHOLD: u8 = 10;

    let total = pixels.len() / 4;
    let transparent_count = pixels
        .chunks_exact(4)
        .filter(|c| c[3] < ALPHA_THRESHOLD)
        .count();

    let has_transparency = transparent_count > total / 20;

    if has_transparency {
        for chunk in pixels.chunks_exact_mut(4) {
            if chunk[3] < ALPHA_THRESHOLD {
                chunk.copy_from_slice(&[255, 255, 255, 255]);
            } else {
                chunk.copy_from_slice(&[0, 0, 0, 255]);
            }
        }
        return;
    }

    let edge_lum = edge_mean_luminance(pixels, width, height);
    if edge_lum >= 128.0 {
        return;
    }

    for chunk in pixels.chunks_exact_mut(4) {
        let lum = luminance(chunk[0], chunk[1], chunk[2]);
        if lum >= 128.0 {
            chunk.copy_from_slice(&[0, 0, 0, 255]);
        } else {
            chunk.copy_from_slice(&[255, 255, 255, 255]);
        }
    }
}

pub fn vectorize(png_data: &[u8], config: &VtracerConfig) -> Result<String, ConvertError> {
    let img = image::load_from_memory(png_data)
        .map_err(|e| ConvertError::ImageDecode(e.to_string()))?;

    let (img_width, img_height) = img.dimensions();
    let mut pixels = img.to_rgba8().into_raw();
    preprocess_for_tracing(&mut pixels, img_width as usize, img_height as usize);

    let color_image = ColorImage {
        pixels,
        width: img_width as usize,
        height: img_height as usize,
    };

    let svg_file = vtracer::convert(color_image, config.to_vtracer_config())
        .map_err(ConvertError::Vectorize)?;

    Ok(svg_file.to_string())
}


#[cfg(test)]
mod tests {
    use super::*;

    fn encode_png(img: &image::RgbaImage) -> Vec<u8> {
        let mut buf = Vec::new();
        let (w, h) = img.dimensions();
        let encoder = image::codecs::png::PngEncoder::new(std::io::Cursor::new(&mut buf));
        image::ImageEncoder::write_image(
            encoder,
            img.as_raw(),
            w,
            h,
            image::ColorType::Rgba8.into(),
        )
        .unwrap();
        buf
    }

    fn test_png() -> Vec<u8> {
        let mut img = image::RgbaImage::new(2, 2);
        for pixel in img.pixels_mut() {
            *pixel = image::Rgba([255, 0, 0, 255]);
        }
        encode_png(&img)
    }

    #[test]
    fn vectorize_produces_svg() {
        let png = test_png();
        let config = VtracerConfig::default();
        let result = vectorize(&png, &config).unwrap();
        assert!(result.contains("<svg"));
    }

    #[test]
    fn config_parse_empty() {
        let config = VtracerConfig::from_params_str("").unwrap();
        assert_eq!(config.filter_speckle, 4);
    }

    #[test]
    fn config_parse_values() {
        let config =
            VtracerConfig::from_params_str("color_mode=color,filter_speckle=2")
                .unwrap();
        assert!(matches!(config.color_mode, ColorMode::Color));
        assert_eq!(config.filter_speckle, 2);
    }

    #[test]
    fn config_parse_unknown_key() {
        let result = VtracerConfig::from_params_str("unknown=value");
        assert!(result.is_err());
    }

    #[test]
    fn invalid_image_data_returns_error() {
        let result = vectorize(&[0, 1, 2, 3], &VtracerConfig::default());
        assert!(result.is_err());
    }

    #[test]
    fn preprocess_converts_transparent_to_white() {
        // 2x2 image: 2 of 4 pixels transparent (>5%)
        let mut pixels = vec![
            255, 255, 255, 255, // white opaque (visible)
            0, 0, 0, 0,         // transparent
            128, 128, 128, 200, // gray visible
            0, 0, 0, 5,         // nearly transparent
        ];
        preprocess_for_tracing(&mut pixels, 2, 2);
        assert_eq!(&pixels[0..4], &[0, 0, 0, 255]);     // visible → black shape
        assert_eq!(&pixels[4..8], &[255, 255, 255, 255]); // transparent → white bg
        assert_eq!(&pixels[8..12], &[0, 0, 0, 255]);    // visible → black shape
        assert_eq!(&pixels[12..16], &[255, 255, 255, 255]); // nearly transparent → white bg
    }

    #[test]
    fn preprocess_skips_light_background_opaque() {
        // 2x2 all-white opaque: edge luminance high → no change
        let mut pixels = vec![
            255, 255, 255, 255,
            255, 255, 255, 255,
            255, 255, 255, 255,
            255, 255, 255, 255,
        ];
        let original = pixels.clone();
        preprocess_for_tracing(&mut pixels, 2, 2);
        assert_eq!(pixels, original);
    }

    #[test]
    fn preprocess_inverts_dark_background_opaque() {
        // 4x4: black border, white center → dark background detected, invert
        let mut pixels = Vec::new();
        for y in 0..4u32 {
            for x in 0..4u32 {
                if x >= 1 && x <= 2 && y >= 1 && y <= 2 {
                    pixels.extend_from_slice(&[255, 255, 255, 255]); // white icon
                } else {
                    pixels.extend_from_slice(&[0, 0, 0, 255]); // black background
                }
            }
        }
        preprocess_for_tracing(&mut pixels, 4, 4);
        // edge pixels (black bg) → white, center pixels (white icon) → black
        let edge_idx = 0; // (0,0) is an edge pixel
        assert_eq!(&pixels[edge_idx..edge_idx + 4], &[255, 255, 255, 255]);
        let center_idx = (1 * 4 + 1) * 4; // (1,1) is center
        assert_eq!(&pixels[center_idx..center_idx + 4], &[0, 0, 0, 255]);
    }

    #[test]
    fn vectorize_white_on_transparent() {
        let mut img = image::RgbaImage::new(4, 4);
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            if x >= 1 && x <= 2 && y >= 1 && y <= 2 {
                *pixel = image::Rgba([255, 255, 255, 255]);
            } else {
                *pixel = image::Rgba([0, 0, 0, 0]);
            }
        }
        let png = encode_png(&img);
        let result = vectorize(&png, &VtracerConfig::default()).unwrap();
        assert!(result.contains("<svg"));
    }

    #[test]
    fn vectorize_white_on_black_opaque() {
        let mut img = image::RgbaImage::new(8, 8);
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            if x >= 2 && x <= 5 && y >= 2 && y <= 5 {
                *pixel = image::Rgba([255, 255, 255, 255]);
            } else {
                *pixel = image::Rgba([0, 0, 0, 255]);
            }
        }
        let png = encode_png(&img);
        let result = vectorize(&png, &VtracerConfig::default()).unwrap();
        assert!(result.contains("<svg"));
    }
}
