import { createFileRoute } from "@tanstack/react-router";
import { useState, useCallback, useRef, useEffect } from "react";
import { useWasm } from "@/hooks/use-wasm";
import { vectorize, normalize, compose, process_svg } from "@/wasm/init";
import {
  type CompositorConfig,
  type RenderOptions,
  DEFAULT_NORMALIZER_CONFIG,
  DEFAULT_COMPOSITOR_CONFIG,
  DEFAULT_RENDER_OPTIONS,
} from "@/wasm/types";
import { downloadPng, downloadSvg } from "@/lib/download";
import { EditorLayout } from "@/components/editor/editor-layout";
import { FileUpload } from "@/components/editor/file-upload";
import { FileList, type ImageEntry } from "@/components/editor/file-list";
import { CompositorControls } from "@/components/editor/compositor-controls";
import { RenderControls } from "@/components/editor/render-controls";
import { PreviewPanel } from "@/components/editor/preview-panel";
import { DownloadButton } from "@/components/editor/download-button";

export const Route = createFileRoute("/")({
  component: EditorPage,
});

let nextId = 0;

const SVG_IDS = ["vrc-icon-grad", "vrc-shadow-filter", "vrc-icon-mask"];

function uniquifySvgIds(svg: string, suffix: string): string {
  let result = svg;
  for (const id of SVG_IDS) {
    // replaceAll may not be available depending on TS lib target; use split/join for compatibility
    result = result.split(`"${id}"`).join(`"${id}-${suffix}"`);
    result = result.split(`(#${id})`).join(`(#${id}-${suffix})`);
  }
  return result;
}

function EditorPage() {
  const { ready, error: wasmError } = useWasm();
  const [entries, setEntries] = useState<ImageEntry[]>([]);
  const normConfig = DEFAULT_NORMALIZER_CONFIG;
  const [compConfig, setCompConfig] = useState<CompositorConfig>({ ...DEFAULT_COMPOSITOR_CONFIG });
  const [renderOpts, setRenderOpts] = useState<RenderOptions>({ ...DEFAULT_RENDER_OPTIONS });
  const [processing, setProcessing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const debounceRef = useRef<ReturnType<typeof setTimeout>>(null);

  const updatePreviews = useCallback(() => {
    if (!ready || entries.length === 0) return;
    try {
      setEntries((prev) =>
        prev.map((entry) => {
          try {
            const normalized = normalize(entry.svgInput, normConfig);
            const composed = compose(normalized, compConfig);
            return { ...entry, previewSvg: uniquifySvgIds(composed, entry.id) };
          } catch {
            return { ...entry, previewSvg: null };
          }
        }),
      );
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }, [ready, entries.length, normConfig, compConfig]);

  useEffect(() => {
    if (debounceRef.current) clearTimeout(debounceRef.current);
    debounceRef.current = setTimeout(updatePreviews, 150);
    return () => {
      if (debounceRef.current) clearTimeout(debounceRef.current);
    };
  }, [updatePreviews]);

  const addEntry = useCallback(
    (svgInput: string, fileName: string) => {
      if (!ready) return;
      const id = String(++nextId);
      try {
        const normalized = normalize(svgInput, normConfig);
        const composed = compose(normalized, compConfig);
        const entry: ImageEntry = { id, fileName, svgInput, previewSvg: uniquifySvgIds(composed, id) };
        setEntries((prev) => [...prev, entry]);
        setError(null);
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err));
      }
    },
    [ready, normConfig, compConfig],
  );

  const handleSvgLoad = useCallback(
    (content: string, name: string) => addEntry(content, name),
    [addEntry],
  );

  const handlePngLoad = useCallback(
    (bytes: Uint8Array, name: string) => {
      if (!ready) return;
      try {
        const svg = vectorize(bytes, undefined);
        addEntry(svg, name);
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err));
      }
    },
    [ready, addEntry],
  );

  const handleRemove = useCallback(
    (id: string) => {
      setEntries((prev) => prev.filter((e) => e.id !== id));
    },
    [],
  );

  const handleDownloadOne = useCallback(
    (id: string) => {
      if (!ready) return;
      const entry = entries.find((e) => e.id === id);
      if (!entry) return;
      try {
        const cleanOpts = Object.fromEntries(
          Object.entries(renderOpts).filter(([, v]) => v != null),
        );
        const pngBytes = process_svg(entry.svgInput, normConfig, compConfig, cleanOpts);
        const outName = entry.fileName.replace(/\.(svg|png)$/i, ".png");
        downloadPng(pngBytes, outName);
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err));
      }
    },
    [ready, entries, normConfig, compConfig, renderOpts],
  );

  const handleDownloadSvg = useCallback(() => {
    if (!ready || entries.length === 0) return;
    for (const entry of entries) {
      const normalized = normalize(entry.svgInput, normConfig);
      const composed = compose(normalized, compConfig);
      const outName = entry.fileName.replace(/\.(svg|png)$/i, ".svg");
      downloadSvg(composed, outName);
    }
  }, [ready, entries, normConfig, compConfig]);

  const handleDownload = useCallback(() => {
    if (!ready || entries.length === 0) return;
    setProcessing(true);
    try {
      const cleanOpts = Object.fromEntries(
        Object.entries(renderOpts).filter(([, v]) => v != null),
      );
      for (const entry of entries) {
        const pngBytes = process_svg(entry.svgInput, normConfig, compConfig, cleanOpts);
        const outName = entry.fileName.replace(/\.(svg|png)$/i, ".png");
        downloadPng(pngBytes, outName);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setProcessing(false);
    }
  }, [ready, entries, normConfig, compConfig, renderOpts]);

  const disabled = !ready;

  if (wasmError) {
    return (
      <div className="flex items-center justify-center p-12 text-destructive">
        Failed to load WebAssembly: {wasmError.message}
      </div>
    );
  }

  return (
    <EditorLayout
      sidebar={
        <div className="flex flex-col gap-6 p-4">
          <FileUpload onSvgLoad={handleSvgLoad} onPngLoad={handlePngLoad} disabled={!ready} />
          <FileList
            entries={entries}
            onRemove={handleRemove}
          />
          <CompositorControls config={compConfig} onChange={setCompConfig} disabled={disabled} />
          <RenderControls options={renderOpts} onChange={setRenderOpts} disabled={disabled} />
          <DownloadButton
            onClick={handleDownload}
            onDownloadSvg={handleDownloadSvg}
            disabled={disabled || entries.length === 0 || processing}
            processing={processing}
            count={entries.length}
          />
        </div>
      }
      preview={
        <PreviewPanel
          entries={entries}
          onRemove={handleRemove}
          onDownload={handleDownloadOne}
          loading={!ready}
          error={error}
        />
      }
    />
  );
}
