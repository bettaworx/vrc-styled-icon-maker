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

pub fn vectorize(png_data: &[u8], config: &VtracerConfig) -> Result<String, ConvertError> {
    let img = image::load_from_memory(png_data)
        .map_err(|e| ConvertError::ImageDecode(e.to_string()))?;

    let (img_width, img_height) = img.dimensions();
    let rgba = img.to_rgba8();

    let color_image = ColorImage {
        pixels: rgba.into_raw(),
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
}
