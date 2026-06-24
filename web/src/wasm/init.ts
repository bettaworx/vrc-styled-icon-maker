import wasmInit, {
  vectorize,
  normalize,
  compose,
  render_svg,
  process_svg,
} from "@bettaworx/vrc-styled-icon-maker-wasm";

let initPromise: Promise<unknown> | null = null;

export function ensureWasmReady(): Promise<unknown> {
  if (!initPromise) {
    initPromise = wasmInit();
  }
  return initPromise;
}

export { vectorize, normalize, compose, render_svg, process_svg };
