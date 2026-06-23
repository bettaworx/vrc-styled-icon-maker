use serde::Deserialize;
use serde::de;
use thiserror::Error;

fn deserialize_f32<'de, D: de::Deserializer<'de>>(deserializer: D) -> Result<f32, D::Error> {
    struct F32Visitor;
    impl<'de> de::Visitor<'de> for F32Visitor {
        type Value = f32;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("a float or a string containing a float")
        }
        fn visit_f32<E: de::Error>(self, v: f32) -> Result<f32, E> { Ok(v) }
        fn visit_f64<E: de::Error>(self, v: f64) -> Result<f32, E> { Ok(v as f32) }
        fn visit_i64<E: de::Error>(self, v: i64) -> Result<f32, E> { Ok(v as f32) }
        fn visit_u64<E: de::Error>(self, v: u64) -> Result<f32, E> { Ok(v as f32) }
        fn visit_str<E: de::Error>(self, s: &str) -> Result<f32, E> {
            s.parse::<f32>().map_err(de::Error::custom)
        }
    }
    deserializer.deserialize_any(F32Visitor)
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CompositorConfig {
    pub canvas_size: u32,
    pub gradient_start_color: String,
    pub gradient_end_color: String,
    #[serde(deserialize_with = "deserialize_f32")]
    pub shadow_blur: f32,
    pub shadow_color: String,
    #[serde(deserialize_with = "deserialize_f32")]
    pub shadow_opacity: f32,
    pub padding: u32,
}

impl Default for CompositorConfig {
    fn default() -> Self {
        Self {
            canvas_size: 1024,
            gradient_start_color: "#ffffff".into(),
            gradient_end_color: "#c0c0c0".into(),
            shadow_blur: 12.0,
            shadow_color: "#000000".into(),
            shadow_opacity: 1.0,
            padding: 96,
        }
    }
}

impl CompositorConfig {
    pub fn from_params_str(s: &str) -> Result<Self, serde_keyvalue::ParseError> {
        if s.is_empty() {
            return Ok(Self::default());
        }
        serde_keyvalue::from_key_values(s)
    }
}

#[derive(Debug, Error)]
pub enum CompositorError {
    #[error("failed to parse input SVG: {0}")]
    SvgParse(String),
}

struct SvgInfo {
    min_x: f64,
    min_y: f64,
    width: f64,
    height: f64,
    inner_content: String,
}

fn extract_attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let search = format!("{name}=\"");
    let start = tag.find(&search)? + search.len();
    let end = start + tag[start..].find('"')?;
    Some(&tag[start..end])
}

fn parse_svg_input(svg: &str) -> Result<SvgInfo, CompositorError> {
    let svg_start = svg
        .find("<svg")
        .ok_or_else(|| CompositorError::SvgParse("no <svg> tag found".into()))?;

    let tag_end = svg[svg_start..]
        .find('>')
        .ok_or_else(|| CompositorError::SvgParse("unclosed <svg> tag".into()))?;
    let opening_tag = &svg[svg_start..svg_start + tag_end];

    let (min_x, min_y, width, height) = if let Some(vb) = extract_attr(opening_tag, "viewBox") {
        let parts: Vec<f64> = vb
            .split_whitespace()
            .filter_map(|s| s.parse().ok())
            .collect();
        if parts.len() >= 4 {
            (parts[0], parts[1], parts[2], parts[3])
        } else {
            (0.0, 0.0, 100.0, 100.0)
        }
    } else {
        let w = extract_attr(opening_tag, "width")
            .and_then(|s| s.trim_end_matches("px").parse::<f64>().ok())
            .unwrap_or(100.0);
        let h = extract_attr(opening_tag, "height")
            .and_then(|s| s.trim_end_matches("px").parse::<f64>().ok())
            .unwrap_or(100.0);
        (0.0, 0.0, w, h)
    };

    let content_start = svg_start + tag_end + 1;
    let content_end = svg
        .rfind("</svg>")
        .ok_or_else(|| CompositorError::SvgParse("no closing </svg> tag".into()))?;

    let inner_content = if content_start < content_end {
        svg[content_start..content_end].to_string()
    } else {
        String::new()
    };

    Ok(SvgInfo {
        min_x,
        min_y,
        width,
        height,
        inner_content,
    })
}

fn compute_shape_bounds(svg_input: &str, info: &SvgInfo) -> (f64, f64, f64, f64) {
    let tree = resvg::usvg::Tree::from_str(svg_input, &resvg::usvg::Options::default());
    if let Ok(tree) = tree {
        let bbox = tree.root().stroke_bounding_box();
        let (l, t, r, b) = (
            bbox.left() as f64,
            bbox.top() as f64,
            bbox.right() as f64,
            bbox.bottom() as f64,
        );
        if r > l && b > t {
            return (l, t, r, b);
        }
    }
    (info.min_x, info.min_y, info.min_x + info.width, info.min_y + info.height)
}

pub fn compose(svg_input: &str, config: &CompositorConfig) -> Result<String, CompositorError> {
    let info = parse_svg_input(svg_input)?;

    let cs = config.canvas_size as f64;
    let padding = config.padding as f64;
    let available = cs - padding * 2.0;
    let scale = available / info.width.max(info.height);
    let scaled_w = info.width * scale;
    let scaled_h = info.height * scale;
    let tx = (cs - scaled_w) / 2.0 - info.min_x * scale;
    let ty = (cs - scaled_h) / 2.0 - info.min_y * scale;

    let margin = (config.shadow_blur * 3.0).ceil() as u32;
    let fx = -(margin as i32);
    let fy = -(margin as i32);
    let fw = config.canvas_size + margin * 2;
    let fh = config.canvas_size + margin * 2;

    let (_, bbox_top, _, bbox_bottom) = compute_shape_bounds(svg_input, &info);
    let grad_top = bbox_top * scale + ty;
    let grad_bottom = bbox_bottom * scale + ty;
    let canvas_mid = cs / 2.0;

    Ok(format!(
        r#"<svg viewBox="0 0 {cs} {cs}" width="100%" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink">
  <defs>
    <linearGradient id="vrc-icon-grad" x1="{cmx}" y1="{it}" x2="{cmx}" y2="{ib}" gradientUnits="userSpaceOnUse">
      <stop offset="0" stop-color="{gs}"/>
      <stop offset="1" stop-color="{ge}"/>
    </linearGradient>
    <filter id="vrc-shadow-filter" x="{fx}" y="{fy}" width="{fw}" height="{fh}" filterUnits="userSpaceOnUse">
      <feOffset dx="0" dy="0"/>
      <feGaussianBlur result="blur" stdDeviation="{sb}"/>
      <feFlood flood-color="{sc}" flood-opacity="{so}"/>
      <feComposite in2="blur" operator="in"/>
      <feComposite in="SourceGraphic"/>
    </filter>
    <mask id="vrc-icon-mask">
      <g transform="translate({tx},{ty}) scale({scale})">
        {inner}
      </g>
    </mask>
  </defs>
  <g filter="url(#vrc-shadow-filter)">
    <rect width="{cs}" height="{cs}" fill="url(#vrc-icon-grad)" mask="url(#vrc-icon-mask)"/>
  </g>
</svg>"#,
        cs = config.canvas_size,
        gs = config.gradient_start_color,
        ge = config.gradient_end_color,
        cmx = canvas_mid,
        it = grad_top,
        ib = grad_bottom,
        fx = fx,
        fy = fy,
        fw = fw,
        fh = fh,
        sb = config.shadow_blur,
        sc = config.shadow_color,
        so = config.shadow_opacity,
        tx = tx,
        ty = ty,
        scale = scale,
        inner = info.inner_content.trim(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIMPLE_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200" viewBox="0 0 200 200">
  <rect width="200" height="200" fill="red"/>
</svg>"#;

    const MULTI_COLOR_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200" viewBox="0 0 200 200">
  <rect width="100" height="200" fill="red"/>
  <rect x="100" width="100" height="200" fill="blue"/>
</svg>"#;

    const NONSQUARE_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="300" height="200" viewBox="0 0 300 200">
  <rect width="300" height="200" fill="#333333"/>
</svg>"##;

    const CSS_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
  <defs>
    <style>
      .cls-1 {
        fill: none;
        stroke: #000;
        stroke-width: 2px;
      }
    </style>
  </defs>
  <path class="cls-1" d="M4,14L13,2"/>
</svg>"#;

    #[test]
    fn compose_basic() {
        let result = compose(SIMPLE_SVG, &CompositorConfig::default()).unwrap();
        assert!(result.contains("vrc-shadow-filter"));
        assert!(result.contains("vrc-icon-grad"));
        assert!(result.contains("gradientUnits=\"userSpaceOnUse\""));
    }

    #[test]
    fn shadow_applied_to_icon_group() {
        let result = compose(SIMPLE_SVG, &CompositorConfig::default()).unwrap();
        assert!(result.contains(r#"filter="url(#vrc-shadow-filter)""#));
    }

    #[test]
    fn uses_mask_based_gradient() {
        let result = compose(SIMPLE_SVG, &CompositorConfig::default()).unwrap();
        assert!(result.contains("vrc-icon-mask"));
        assert!(result.contains(r#"mask="url(#vrc-icon-mask)""#));
        assert!(result.contains(r#"fill="url(#vrc-icon-grad)""#));
        assert!(result.contains(r#"fill="red""#));
    }

    #[test]
    fn inner_content_in_mask() {
        let result = compose(MULTI_COLOR_SVG, &CompositorConfig::default()).unwrap();
        assert!(result.contains(r#"fill="red""#));
        assert!(result.contains(r#"fill="blue""#));
        assert!(result.contains("<mask"));
    }

    #[test]
    fn nonsquare_scales_by_larger_dimension() {
        let result = compose(NONSQUARE_SVG, &CompositorConfig::default()).unwrap();
        assert!(result.contains("scale("));
        let info = parse_svg_input(NONSQUARE_SVG).unwrap();
        assert_eq!(info.width, 300.0);
        assert_eq!(info.height, 200.0);
    }

    #[test]
    fn gradient_spans_canvas() {
        let result = compose(SIMPLE_SVG, &CompositorConfig::default()).unwrap();
        assert!(result.contains(r#"x1="512""#));
        assert!(result.contains("userSpaceOnUse"));
    }

    #[test]
    fn css_content_preserved_in_mask() {
        let result = compose(CSS_SVG, &CompositorConfig::default()).unwrap();
        assert!(result.contains("stroke: #000"));
        assert!(result.contains("fill: none"));
        assert!(result.contains("stroke-width: 2px"));
    }

    #[test]
    fn custom_config_values() {
        let config = CompositorConfig {
            gradient_start_color: "#ff0000".into(),
            gradient_end_color: "#0000ff".into(),
            shadow_blur: 15.0,
            ..Default::default()
        };
        let result = compose(SIMPLE_SVG, &config).unwrap();
        assert!(result.contains(r##"stop-color="#ff0000""##));
        assert!(result.contains(r##"stop-color="#0000ff""##));
        assert!(result.contains(r#"stdDeviation="15""#));
    }

    #[test]
    fn config_parse_empty() {
        let config = CompositorConfig::from_params_str("").unwrap();
        assert_eq!(config.canvas_size, 1024);
        assert_eq!(config.padding, 96);
    }

    #[test]
    fn config_parse_values() {
        let config = CompositorConfig::from_params_str("padding=100,shadow_blur=0.5").unwrap();
        assert_eq!(config.padding, 100);
        assert_eq!(config.shadow_blur, 0.5);
    }

    #[test]
    fn config_parse_unknown_key() {
        let result = CompositorConfig::from_params_str("unknown=value");
        assert!(result.is_err());
    }

    #[test]
    fn svg_with_xml_declaration() {
        let svg = r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
  <rect width="100" height="100" fill="black"/>
</svg>"#;
        let result = compose(svg, &CompositorConfig::default()).unwrap();
        assert!(result.contains("vrc-icon-grad"));
    }

    #[test]
    fn svg_width_height_only() {
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="150" height="80">
  <rect width="150" height="80" fill="#aabbcc"/>
</svg>"##;
        let info = parse_svg_input(svg).unwrap();
        assert_eq!(info.width, 150.0);
        assert_eq!(info.height, 80.0);
    }
}
