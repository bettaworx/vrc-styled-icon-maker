use serde::Deserialize;
use std::fmt::Write as _;
use thiserror::Error;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct NormalizerConfig {
    pub fill_color: String,
    pub remove_background: bool,
    pub force_monochrome: bool,
}

impl Default for NormalizerConfig {
    fn default() -> Self {
        Self {
            fill_color: "#ffffff".into(),
            remove_background: true,
            force_monochrome: true,
        }
    }
}

#[derive(Debug, Error)]
pub enum NormalizeError {
    #[error("failed to parse SVG: {0}")]
    Parse(String),
}

struct ViewBox {
    min_x: f64,
    min_y: f64,
    width: f64,
    height: f64,
}

const RENDERABLE_ELEMENTS: &[&str] = &[
    "path", "rect", "circle", "ellipse", "line", "polyline", "polygon", "text", "use", "image",
];

const NON_INHERIT_CONTAINERS: &[&str] = &[
    "defs",
    "clipPath",
    "mask",
    "filter",
    "linearGradient",
    "radialGradient",
    "pattern",
    "symbol",
    "marker",
];

fn parse_viewbox(s: &str) -> Option<ViewBox> {
    let parts: Vec<f64> = s.split_whitespace().filter_map(|p| p.parse().ok()).collect();
    if parts.len() >= 4 {
        Some(ViewBox {
            min_x: parts[0],
            min_y: parts[1],
            width: parts[2],
            height: parts[3],
        })
    } else {
        None
    }
}

fn parse_length(s: &str) -> Option<f64> {
    s.trim_end_matches("px")
        .trim_end_matches("pt")
        .trim_end_matches("em")
        .trim_end_matches("ex")
        .parse()
        .ok()
}

fn is_skip_color(val: &str) -> bool {
    val == "none" || val.starts_with("url(") || val == "inherit" || val == "currentColor"
}

fn replace_color_in_style_attr(style: &str, fill_color: &str) -> String {
    let mut result = String::new();
    for decl in style.split(';') {
        let decl = decl.trim();
        if decl.is_empty() {
            continue;
        }
        if !result.is_empty() {
            result.push_str("; ");
        }
        if let Some(colon) = decl.find(':') {
            let prop = decl[..colon].trim();
            let val = decl[colon + 1..].trim();
            if (prop == "fill" || prop == "stroke") && !is_skip_color(val) {
                write!(result, "{prop}: {fill_color}").unwrap();
            } else {
                result.push_str(decl);
            }
        } else {
            result.push_str(decl);
        }
    }
    result
}

fn replace_colors_in_style_block(css: &str, fill_color: &str) -> String {
    let mut result = String::with_capacity(css.len());
    let mut rest = css;

    while !rest.is_empty() {
        let mut earliest: Option<(usize, &str)> = None;
        for prop in &["fill", "stroke"] {
            if let Some(pos) = rest.find(prop) {
                if earliest.is_none() || pos < earliest.unwrap().0 {
                    earliest = Some((pos, prop));
                }
            }
        }

        let Some((pos, prop)) = earliest else {
            result.push_str(rest);
            break;
        };

        let after_prop = &rest[pos + prop.len()..];

        if after_prop.starts_with('-') {
            result.push_str(&rest[..pos + prop.len()]);
            rest = after_prop;
            continue;
        }

        let trimmed = after_prop.trim_start();
        if !trimmed.starts_with(':') {
            result.push_str(&rest[..pos + prop.len()]);
            rest = after_prop;
            continue;
        }

        let after_colon = trimmed[1..].trim_start();
        let end = after_colon
            .find(|c: char| c == ';' || c == '}')
            .unwrap_or(after_colon.len());
        let val = after_colon[..end].trim();

        let total_consumed = pos + prop.len() + (after_prop.len() - after_colon.len()) + end;
        if is_skip_color(val) {
            result.push_str(&rest[..total_consumed]);
        } else {
            result.push_str(&rest[..pos]);
            result.push_str(prop);
            result.push_str(": ");
            result.push_str(fill_color);
        }
        rest = &rest[total_consumed..];
    }

    result
}

fn is_background_rect(
    node: &roxmltree::Node,
    vb: &ViewBox,
) -> bool {
    if !node.is_element() || node.tag_name().name() != "rect" {
        return false;
    }

    let w = node
        .attribute("width")
        .and_then(parse_length)
        .unwrap_or(0.0);
    let h = node
        .attribute("height")
        .and_then(parse_length)
        .unwrap_or(0.0);
    let x = node
        .attribute("x")
        .and_then(parse_length)
        .unwrap_or(vb.min_x);
    let y = node
        .attribute("y")
        .and_then(parse_length)
        .unwrap_or(vb.min_y);

    let eps = 0.5;
    (w - vb.width).abs() < eps
        && (h - vb.height).abs() < eps
        && (x - vb.min_x).abs() < eps
        && (y - vb.min_y).abs() < eps
}

struct InheritedStyle {
    fill: Option<String>,
    stroke: Option<String>,
}

fn serialize_node(
    node: &roxmltree::Node,
    inherited: &InheritedStyle,
    config: &NormalizerConfig,
    vb: &ViewBox,
    is_root_svg: bool,
    in_defs: bool,
    out: &mut String,
) {
    match node.node_type() {
        roxmltree::NodeType::Element => {
            let tag = node.tag_name().name();

            if is_root_svg && tag == "svg" {
                let current_fill = node.attribute("fill").map(String::from);
                let current_stroke = node.attribute("stroke").map(String::from);
                let child_inherited = InheritedStyle {
                    fill: current_fill.or_else(|| inherited.fill.clone()),
                    stroke: current_stroke.or_else(|| inherited.stroke.clone()),
                };

                write!(
                    out,
                    r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="{} {} {} {}">"#,
                    vb.width, vb.height, vb.min_x, vb.min_y, vb.width, vb.height
                )
                .unwrap();

                for child in node.children() {
                    if config.remove_background && !in_defs && is_background_rect(&child, vb) {
                        continue;
                    }
                    serialize_node(&child, &child_inherited, config, vb, false, in_defs, out);
                }

                out.push_str("</svg>");
                return;
            }

            let entering_defs = NON_INHERIT_CONTAINERS.contains(&tag);
            let child_in_defs = in_defs || entering_defs;
            let is_renderable = RENDERABLE_ELEMENTS.contains(&tag);
            let is_container = tag == "g" || tag == "a" || tag == "svg";

            out.push('<');
            out.push_str(tag);

            let mut has_fill = false;
            let mut has_stroke = false;

            for attr in node.attributes() {
                let name = attr.name();
                let val = attr.value();

                if name == "xmlns" || name.starts_with("xmlns:") {
                    continue;
                }

                if name == "fill" {
                    has_fill = true;
                    let output_val = if config.force_monochrome && !child_in_defs && !is_skip_color(val)
                    {
                        &config.fill_color
                    } else {
                        val
                    };
                    write!(out, r#" fill="{output_val}""#).unwrap();
                    continue;
                }

                if name == "stroke" {
                    has_stroke = true;
                    let output_val = if config.force_monochrome && !child_in_defs && !is_skip_color(val)
                    {
                        &config.fill_color
                    } else {
                        val
                    };
                    write!(out, r#" stroke="{output_val}""#).unwrap();
                    continue;
                }

                if name == "style" && config.force_monochrome && !child_in_defs {
                    let new_style = replace_color_in_style_attr(val, &config.fill_color);
                    write!(out, r#" style="{new_style}""#).unwrap();
                    continue;
                }

                write!(out, r#" {name}="{val}""#).unwrap();
            }

            if !child_in_defs && is_renderable && !has_fill {
                if let Some(ref inherited_fill) = inherited.fill {
                    let val = if config.force_monochrome && !is_skip_color(inherited_fill) {
                        &config.fill_color
                    } else {
                        inherited_fill
                    };
                    write!(out, r#" fill="{val}""#).unwrap();
                }
            }

            if !child_in_defs && is_renderable && !has_stroke {
                if let Some(ref inherited_stroke) = inherited.stroke {
                    if inherited_stroke != "none" {
                        let val = if config.force_monochrome && !is_skip_color(inherited_stroke) {
                            &config.fill_color
                        } else {
                            inherited_stroke
                        };
                        write!(out, r#" stroke="{val}""#).unwrap();
                    }
                }
            }

            let has_children = node.children().any(|c| {
                c.is_element() || (c.is_text() && !c.text().unwrap_or("").trim().is_empty())
            });

            if !has_children && !tag_needs_closing(tag) {
                out.push_str("/>");
            } else {
                out.push('>');

                let child_inherited = if is_container || entering_defs {
                    let fill = node
                        .attribute("fill")
                        .map(String::from)
                        .or_else(|| inherited.fill.clone());
                    let stroke = node
                        .attribute("stroke")
                        .map(String::from)
                        .or_else(|| inherited.stroke.clone());
                    InheritedStyle { fill, stroke }
                } else {
                    InheritedStyle {
                        fill: inherited.fill.clone(),
                        stroke: inherited.stroke.clone(),
                    }
                };

                for child in node.children() {
                    if config.remove_background
                        && !child_in_defs
                        && is_root_child(node)
                        && is_background_rect(&child, vb)
                    {
                        continue;
                    }

                    if child.is_element() && child.tag_name().name() == "style" && config.force_monochrome && !child_in_defs {
                        out.push_str("<style");
                        for attr in child.attributes() {
                            write!(out, r#" {}="{}""#, attr.name(), attr.value()).unwrap();
                        }
                        out.push('>');
                        let css_text: String = child
                            .children()
                            .filter_map(|c| c.text())
                            .collect();
                        let new_css = replace_colors_in_style_block(&css_text, &config.fill_color);
                        out.push_str(&new_css);
                        out.push_str("</style>");
                        continue;
                    }

                    serialize_node(&child, &child_inherited, config, vb, false, child_in_defs, out);
                }

                write!(out, "</{tag}>").unwrap();
            }
        }
        roxmltree::NodeType::Text => {
            if let Some(text) = node.text() {
                out.push_str(text);
            }
        }
        _ => {}
    }
}

fn tag_needs_closing(tag: &str) -> bool {
    matches!(tag, "style" | "script" | "text" | "tspan" | "title" | "desc" | "defs" | "g" | "a" | "svg")
}

fn is_root_child(node: &roxmltree::Node) -> bool {
    node.parent().map_or(false, |p| p.tag_name().name() == "svg")
}

pub fn normalize(svg_input: &str, config: &NormalizerConfig) -> Result<String, NormalizeError> {
    let doc = roxmltree::Document::parse(svg_input)
        .map_err(|e| NormalizeError::Parse(e.to_string()))?;

    let root = doc.root_element();
    if root.tag_name().name() != "svg" {
        return Err(NormalizeError::Parse("root element is not <svg>".into()));
    }

    let vb = if let Some(vb_str) = root.attribute("viewBox") {
        parse_viewbox(vb_str).unwrap_or(ViewBox {
            min_x: 0.0,
            min_y: 0.0,
            width: 100.0,
            height: 100.0,
        })
    } else {
        let w = root
            .attribute("width")
            .and_then(parse_length)
            .unwrap_or(100.0);
        let h = root
            .attribute("height")
            .and_then(parse_length)
            .unwrap_or(100.0);
        ViewBox {
            min_x: 0.0,
            min_y: 0.0,
            width: w,
            height: h,
        }
    };

    let inherited = InheritedStyle {
        fill: None,
        stroke: None,
    };

    let mut out = String::with_capacity(svg_input.len());
    serialize_node(
        &root.into(),
        &inherited,
        config,
        &vb,
        true,
        false,
        &mut out,
    );

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewbox_offset_preserved() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 -960 960 960"><path d="M0 0"/></svg>"#;
        let result = normalize(svg, &NormalizerConfig::default()).unwrap();
        assert!(result.contains(r#"viewBox="0 -960 960 960""#));
        assert!(!result.contains("translate"));
    }

    #[test]
    fn viewbox_zero_origin_preserved() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 200"><rect width="50" height="50" fill="red"/></svg>"#;
        let result = normalize(svg, &NormalizerConfig::default()).unwrap();
        assert!(result.contains(r#"viewBox="0 0 200 200""#));
    }

    #[test]
    fn root_fill_propagated() {
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" fill="#1f1f1f"><path d="M0 0"/></svg>"##;
        let config = NormalizerConfig {
            force_monochrome: false,
            ..Default::default()
        };
        let result = normalize(svg, &config).unwrap();
        assert!(result.contains(r##"fill="#1f1f1f""##));
        assert!(result.contains("<path"));
    }

    #[test]
    fn explicit_fill_not_overridden_by_inheritance() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" fill="red"><path fill="blue" d="M0 0"/></svg>"#;
        let config = NormalizerConfig {
            force_monochrome: false,
            ..Default::default()
        };
        let result = normalize(svg, &config).unwrap();
        assert!(result.contains(r#"fill="blue""#));
        assert!(!result.contains(r#"<path.*fill="red""#));
    }

    #[test]
    fn background_rect_removed() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 200"><rect width="200" height="200" fill="white"/><path d="M10 10" fill="black"/></svg>"#;
        let result = normalize(svg, &NormalizerConfig::default()).unwrap();
        assert!(!result.contains("<rect"));
        assert!(result.contains("<path"));
    }

    #[test]
    fn non_background_rect_kept() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 200"><rect width="50" height="50" fill="red"/></svg>"#;
        let result = normalize(svg, &NormalizerConfig::default()).unwrap();
        assert!(result.contains("<rect"));
    }

    #[test]
    fn force_monochrome_replaces_colors() {
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100"><path fill="#333" d="M0 0"/></svg>"##;
        let result = normalize(svg, &NormalizerConfig::default()).unwrap();
        assert!(result.contains(r##"fill="#ffffff""##));
        assert!(!result.contains(r##"fill="#333""##));
    }

    #[test]
    fn fill_none_preserved() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100"><path fill="none" stroke="black" d="M0 0"/></svg>"#;
        let result = normalize(svg, &NormalizerConfig::default()).unwrap();
        assert!(result.contains(r#"fill="none""#));
    }

    #[test]
    fn css_style_attr_colors_replaced() {
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100"><path style="fill: #333; stroke-width: 2px" d="M0 0"/></svg>"##;
        let result = normalize(svg, &NormalizerConfig::default()).unwrap();
        assert!(result.contains("fill: #ffffff"));
        assert!(result.contains("stroke-width: 2px"));
    }

    #[test]
    fn google_material_icon_normalized() {
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" height="24px" viewBox="0 -960 960 960" width="24px" fill="#1f1f1f"><path d="m480-340 180-180-57-56-123 123-123-123-57 56 180 180Zm0 260q-83 0-156-31.5T197-197q-54-54-85.5-127T80-480q0-83 31.5-156T197-763q54-54 127-85.5T480-880q83 0 156 31.5T763-763q54 54 85.5 127T880-480q0 83-31.5 156T763-197q-54 54-127 85.5T480-80Z"/></svg>"##;
        let result = normalize(svg, &NormalizerConfig::default()).unwrap();

        assert!(result.contains(r#"viewBox="0 -960 960 960""#));
        assert!(result.contains(r##"fill="#ffffff""##));
        assert!(!result.contains(r##"fill="#1f1f1f""##));
    }

    #[test]
    fn defs_content_not_color_replaced() {
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100"><defs><linearGradient id="g"><stop stop-color="red"/></linearGradient></defs><path fill="url(#g)" d="M0 0"/></svg>"##;
        let result = normalize(svg, &NormalizerConfig::default()).unwrap();
        assert!(result.contains(r##"stop-color="red""##));
        assert!(result.contains(r##"fill="url(#g)""##));
    }
}
