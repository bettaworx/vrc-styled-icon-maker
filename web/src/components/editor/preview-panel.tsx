import { CircleNotch, DownloadSimple, X } from "@phosphor-icons/react";
import type { ImageEntry } from "@/components/editor/file-list";

interface PreviewPanelProps {
  entries: ImageEntry[];
  onRemove: (id: string) => void;
  onDownload: (id: string) => void;
  loading: boolean;
  error: string | null;
}

export function PreviewPanel({ entries, onRemove, onDownload, loading, error }: PreviewPanelProps) {
  if (loading) {
    return (
      <div className="flex aspect-video items-center justify-center bg-secondary text-muted-foreground md:aspect-auto md:h-full">
        <CircleNotch size={24} className="animate-spin" />
        <span className="ml-2 text-sm">Initializing...</span>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex aspect-video items-center justify-center bg-secondary p-8 md:aspect-auto md:h-full">
        <p className="max-w-md text-center text-sm text-destructive">{error}</p>
      </div>
    );
  }

  if (entries.length === 0) {
    return (
      <div className="flex aspect-video items-center justify-center bg-secondary text-muted-foreground md:aspect-auto md:h-full">
        <p className="text-sm">Import files to preview</p>
      </div>
    );
  }

  if (entries.length === 1) {
    const entry = entries[0];
    return (
      <div className="flex h-full items-center justify-center bg-secondary p-8">
        <div
          className="aspect-square w-full max-w-md rounded-lg bg-[repeating-conic-gradient(var(--color-checker)_0%_25%,transparent_0%_50%)] bg-[length:16px_16px] shadow-sm"
          dangerouslySetInnerHTML={{ __html: entry.previewSvg ?? "" }}
        />
      </div>
    );
  }

  return (
    <div className="h-full overflow-y-auto bg-secondary p-4">
      <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-4">
        {entries.map((entry) => (
          <div
            key={entry.id}
            className="group/card relative rounded-lg border-2 border-transparent p-1 transition-colors hover:border-border"
          >
            <button
              onClick={() => onDownload(entry.id)}
              className="absolute top-2 left-2 z-10 rounded-full bg-background/80 p-1 text-muted-foreground backdrop-blur-sm transition-colors hover:bg-background hover:text-foreground md:hidden md:group-hover/card:block"
            >
              <DownloadSimple size={12} />
            </button>
            <button
              onClick={() => onRemove(entry.id)}
              className="absolute top-2 right-2 z-10 rounded-full bg-background/80 p-1 text-muted-foreground backdrop-blur-sm transition-colors hover:bg-background hover:text-foreground md:hidden md:group-hover/card:block"
            >
              <X size={12} />
            </button>
            <div
              className="aspect-square w-full rounded bg-[repeating-conic-gradient(var(--color-checker)_0%_25%,transparent_0%_50%)] bg-[length:8px_8px]"
              dangerouslySetInnerHTML={{ __html: entry.previewSvg ?? "" }}
            />
            <p className="mt-1 truncate text-center text-xs text-muted-foreground">
              {entry.fileName}
            </p>
          </div>
        ))}
      </div>
    </div>
  );
}
