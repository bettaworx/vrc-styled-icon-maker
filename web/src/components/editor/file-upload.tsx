import { useCallback, useRef, useState } from "react";
import { Button } from "@/components/ui/button";
import { UploadSimple } from "@phosphor-icons/react";

interface FileUploadProps {
  onSvgLoad: (content: string, name: string) => void;
  onPngLoad: (bytes: Uint8Array, name: string) => void;
  disabled: boolean;
}

export function FileUpload({ onSvgLoad, onPngLoad, disabled }: FileUploadProps) {
  const inputRef = useRef<HTMLInputElement>(null);
  const [dragOver, setDragOver] = useState(false);

  const readFile = useCallback(
    (file: File) => {
      const name = file.name.toLowerCase();
      if (name.endsWith(".svg")) {
        const reader = new FileReader();
        reader.onload = () => {
          const text = reader.result as string;
          if (text.includes("<svg")) {
            onSvgLoad(text, file.name);
          }
        };
        reader.readAsText(file);
      } else if (name.endsWith(".png")) {
        const reader = new FileReader();
        reader.onload = () => {
          const bytes = new Uint8Array(reader.result as ArrayBuffer);
          onPngLoad(bytes, file.name);
        };
        reader.readAsArrayBuffer(file);
      }
    },
    [onSvgLoad, onPngLoad],
  );

  const readFiles = useCallback(
    (files: FileList) => {
      for (let i = 0; i < files.length; i++) {
        readFile(files[i]);
      }
    },
    [readFile],
  );

  const handleDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      setDragOver(false);
      if (e.dataTransfer.files.length > 0) readFiles(e.dataTransfer.files);
    },
    [readFiles],
  );

  const handleChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      if (e.target.files && e.target.files.length > 0) {
        readFiles(e.target.files);
        e.target.value = "";
      }
    },
    [readFiles],
  );

  return (
    <div
      onDragOver={(e) => { e.preventDefault(); setDragOver(true); }}
      onDragLeave={() => setDragOver(false)}
      onDrop={handleDrop}
      className={`flex flex-col items-center gap-2 rounded-lg border-2 border-dashed p-6 text-center transition-colors ${
        dragOver ? "border-ring bg-accent" : "border-border"
      }`}
    >
      <UploadSimple size={24} className="text-muted-foreground" />
      <p className="text-sm text-muted-foreground">
        Drop SVG or PNG files here
      </p>
      <Button
        variant="outline"
        size="sm"
        disabled={disabled}
        onClick={() => inputRef.current?.click()}
      >
        Browse
      </Button>
      <input
        ref={inputRef}
        type="file"
        accept=".svg,.png"
        multiple
        className="hidden"
        onChange={handleChange}
      />
    </div>
  );
}
