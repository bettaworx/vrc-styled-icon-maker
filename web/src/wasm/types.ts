export interface NormalizerConfig {
  fill_color: string;
  remove_background: boolean;
  force_monochrome: boolean;
}

export const DEFAULT_NORMALIZER_CONFIG: NormalizerConfig = {
  fill_color: "#ffffff",
  remove_background: true,
  force_monochrome: true,
};

export interface CompositorConfig {
  canvas_size: number;
  gradient_start_color: string;
  gradient_end_color: string;
  shadow_blur: number;
  shadow_color: string;
  shadow_opacity: number;
  shadow_offset_x: number;
  shadow_offset_y: number;
  padding: number;
}

export const DEFAULT_COMPOSITOR_CONFIG: CompositorConfig = {
  canvas_size: 1024,
  gradient_start_color: "#ffffff",
  gradient_end_color: "#CCCCCC",
  shadow_blur: 10.0,
  shadow_color: "#000000",
  shadow_opacity: 0.5,
  shadow_offset_x: -4.0,
  shadow_offset_y: 4.0,
  padding: 108,
};

export interface RenderOptions {
  width?: number | null;
  height?: number | null;
  format: "png";
  background?: string | null;
}

export const DEFAULT_RENDER_OPTIONS: RenderOptions = {
  width: 256,
  height: 256,
  format: "png",
  background: null,
};
