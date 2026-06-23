use resvg::tiny_skia;
use resvg::usvg;
use serde::Deserialize;
use serde::de;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageFormat {
    Png,
}

fn deserialize_background<'de, D: de::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<[u8; 4]>, D::Error> {
    let s = String::deserialize(deserializer)?;
    parse_hex_color(&s).map(Some).map_err(de::Error::custom)
}

fn parse_hex_color(s: &str) -> Result<[u8; 4], String> {
    let s = s.strip_prefix('#').unwrap_or(s);
    match s.len() {
        6 => {
            let r = u8::from_str_radix(&s[0..2], 16).map_err(|e| e.to_string())?;
            let g = u8::from_str_radix(&s[2..4], 16).map_err(|e| e.to_string())?;
            let b = u8::from_str_radix(&s[4..6], 16).map_err(|e| e.to_string())?;
            Ok([r, g, b, 255])
        }
        8 => {
            let r = u8::from_str_radix(&s[0..2], 16).map_err(|e| e.to_string())?;
            let g = u8::from_str_radix(&s[2..4], 16).map_err(|e| e.to_string())?;
            let b = u8::from_str_radix(&s[4..6], 16).map_err(|e| e.to_string())?;
            let a = u8::from_str_radix(&s[6..8], 16).map_err(|e| e.to_string())?;
            Ok([r, g, b, a])
        }
        _ => Err(format!("expected 6 or 8 hex digits, got {}", s.len())),
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RenderOptions {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub format: ImageFormat,
    #[serde(deserialize_with = "deserialize_background")]
    pub background: Option<[u8; 4]>,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            width: None,
            height: None,
            format: ImageFormat::Png,
            background: None,
        }
    }
}

impl RenderOptions {
    pub fn from_params_str(s: &str) -> Result<Self, serde_keyvalue::ParseError> {
        if s.is_empty() {
            return Ok(Self::default());
        }
        serde_keyvalue::from_key_values(s)
    }
}

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("failed to parse SVG: {0}")]
    SvgParse(String),
    #[error("failed to create pixel buffer ({width}x{height})")]
    PixmapCreate { width: u32, height: u32 },
    #[error("failed to encode image: {0}")]
    Encode(String),
}

fn encode_png_fast(pixmap: &tiny_skia::Pixmap) -> Result<Vec<u8>, RenderError> {
    let width = pixmap.width();
    let height = pixmap.height();

    let data = pixmap.data();
    let mut rgba = Vec::with_capacity(data.len());
    for chunk in data.chunks(4) {
        let (r, g, b, a) = (chunk[0], chunk[1], chunk[2], chunk[3]);
        if a == 0 {
            rgba.extend_from_slice(&[0, 0, 0, 0]);
        } else {
            let a_f = a as f32 / 255.0;
            rgba.push((r as f32 / a_f).min(255.0) as u8);
            rgba.push((g as f32 / a_f).min(255.0) as u8);
            rgba.push((b as f32 / a_f).min(255.0) as u8);
            rgba.push(a);
        }
    }

    let mut buf = Vec::new();
    let mut encoder = png::Encoder::new(&mut buf, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_compression(png::Compression::Fast);
    let mut writer = encoder
        .write_header()
        .map_err(|e| RenderError::Encode(e.to_string()))?;
    writer
        .write_image_data(&rgba)
        .map_err(|e| RenderError::Encode(e.to_string()))?;
    drop(writer);
    Ok(buf)
}

pub fn render_svg(svg_data: &str, options: &RenderOptions) -> Result<Vec<u8>, RenderError> {
    let mut usvg_options = usvg::Options::default();

    let mut fontdb = usvg::fontdb::Database::new();
    if svg_data.contains("<text") || svg_data.contains("<tspan") {
        fontdb.load_system_fonts();
    }
    usvg_options.fontdb = std::sync::Arc::new(fontdb);

    let tree = usvg::Tree::from_str(svg_data, &usvg_options)
        .map_err(|e| RenderError::SvgParse(e.to_string()))?;

    let svg_size = tree.size();
    let (width, height) = match (options.width, options.height) {
        (Some(w), Some(h)) => (w, h),
        (Some(w), None) => {
            let aspect = svg_size.height() / svg_size.width();
            (w, (w as f32 * aspect).round() as u32)
        }
        (None, Some(h)) => {
            let aspect = svg_size.width() / svg_size.height();
            ((h as f32 * aspect).round() as u32, h)
        }
        (None, None) => (svg_size.width() as u32, svg_size.height() as u32),
    };

    let mut pixmap = tiny_skia::Pixmap::new(width, height)
        .ok_or(RenderError::PixmapCreate { width, height })?;

    if let Some([r, g, b, a]) = options.background {
        pixmap.fill(tiny_skia::Color::from_rgba8(r, g, b, a));
    }

    let transform = tiny_skia::Transform::from_scale(
        width as f32 / svg_size.width(),
        height as f32 / svg_size.height(),
    );

    resvg::render(&tree, transform, &mut pixmap.as_mut());

    match options.format {
        ImageFormat::Png => encode_png_fast(&pixmap),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
        <rect width="100" height="100" fill="red"/>
    </svg>"#;

    #[test]
    fn render_png_returns_valid_bytes() {
        let options = RenderOptions {
            width: Some(64),
            height: Some(64),
            format: ImageFormat::Png,
            background: None,
        };
        let result = render_svg(TEST_SVG, &options).unwrap();
        assert!(!result.is_empty());
        assert_eq!(&result[..4], &[0x89, b'P', b'N', b'G']);
    }

    #[test]
    fn render_with_background() {
        let options = RenderOptions {
            width: Some(32),
            height: Some(32),
            format: ImageFormat::Png,
            background: Some([255, 255, 255, 255]),
        };
        let result = render_svg(TEST_SVG, &options).unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn render_uses_svg_size_when_none() {
        let result = render_svg(TEST_SVG, &RenderOptions::default()).unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn render_invalid_svg_returns_error() {
        let result = render_svg("not valid svg", &RenderOptions::default());
        assert!(result.is_err());
    }

    #[test]
    fn render_zero_size_returns_error() {
        let options = RenderOptions {
            width: Some(0),
            height: Some(0),
            ..RenderOptions::default()
        };
        let result = render_svg(TEST_SVG, &options);
        assert!(result.is_err());
    }
}
