import { Button } from "@/components/ui/button";
import { X } from "@phosphor-icons/react";

export interface ImageEntry {
  id: string;
  fileName: string;
  svgInput: string;
  previewSvg: string | null;
}

interface FileListProps {
  entries: ImageEntry[];
  onRemove: (id: string) => void;
}

export function FileList({ entries, onRemove }: FileListProps) {
  if (entries.length === 0) return null;

  return (
    <div className="space-y-1">
      <h3 className="text-sm font-medium">Files ({entries.length})</h3>
      <div className="max-h-40 space-y-0.5 overflow-y-auto">
        {entries.map((entry) => (
          <div
            key={entry.id}
            className="flex items-center gap-2 rounded px-2 py-1 text-sm"
          >
            {entry.previewSvg && (
              <div
                className="h-6 w-6 shrink-0 rounded border border-border bg-white"
                dangerouslySetInnerHTML={{ __html: entry.previewSvg }}
              />
            )}
            <span className="min-w-0 flex-1 truncate">{entry.fileName}</span>
            <Button
              variant="ghost"
              size="icon-xs"
              onClick={() => onRemove(entry.id)}
            >
              <X size={12} />
            </Button>
          </div>
        ))}
      </div>
    </div>
  );
}
