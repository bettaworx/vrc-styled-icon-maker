use wasm_bindgen::prelude::*;

fn parse_config<T: serde::de::DeserializeOwned + Default>(val: JsValue) -> Result<T, JsError> {
    if val.is_undefined() || val.is_null() {
        Ok(T::default())
    } else {
        serde_wasm_bindgen::from_value(val).map_err(|e| JsError::new(&e.to_string()))
    }
}

#[wasm_bindgen]
pub fn normalize(svg_input: &str, config: JsValue) -> Result<String, JsError> {
    let config: normalizer::NormalizerConfig = parse_config(config)?;
    normalizer::normalize(svg_input, &config).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen]
pub fn compose(svg_input: &str, config: JsValue) -> Result<String, JsError> {
    let config: compositor::CompositorConfig = parse_config(config)?;
    compositor::compose(svg_input, &config).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen]
pub fn render_svg(svg_data: &str, options: JsValue) -> Result<Vec<u8>, JsError> {
    let options: renderer::RenderOptions = parse_config(options)?;
    renderer::render_svg(svg_data, &options).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen]
pub fn process_svg(
    svg_input: &str,
    normalize_config: JsValue,
    compose_config: JsValue,
    render_options: JsValue,
) -> Result<Vec<u8>, JsError> {
    let norm_cfg: normalizer::NormalizerConfig = parse_config(normalize_config)?;
    let comp_cfg: compositor::CompositorConfig = parse_config(compose_config)?;
    let render_opts: renderer::RenderOptions = parse_config(render_options)?;

    let normalized =
        normalizer::normalize(svg_input, &norm_cfg).map_err(|e| JsError::new(&e.to_string()))?;
    let composed =
        compositor::compose(&normalized, &comp_cfg).map_err(|e| JsError::new(&e.to_string()))?;
    renderer::render_svg(&composed, &render_opts).map_err(|e| JsError::new(&e.to_string()))
}
