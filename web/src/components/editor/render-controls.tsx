import type { RenderOptions } from "@/wasm/types";
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import { Separator } from "@/components/ui/separator";
import type { ChangeEvent } from "react";

interface RenderControlsProps {
  options: RenderOptions;
  onChange: (options: RenderOptions) => void;
  disabled: boolean;
}

export function RenderControls({ options, onChange, disabled }: RenderControlsProps) {
  const onDimension = (field: "width" | "height") => (e: ChangeEvent<HTMLInputElement>) => {
    const v = e.target.value;
    onChange({ ...options, [field]: v === "" ? null : Number(v) });
  };

  return (
    <div className="space-y-3">
      <h3 className="text-sm font-medium">Output</h3>
      <Separator />
      <div className="grid grid-cols-2 gap-2">
        <div className="space-y-1">
          <Label htmlFor="out-width" className="text-xs">Width</Label>
          <Input
            id="out-width"
            type="number"
            disabled={disabled}
            min={1}
            max={4096}
            placeholder="auto"
            value={options.width ?? ""}
            onChange={onDimension("width")}
            className="h-7 text-xs"
          />
        </div>
        <div className="space-y-1">
          <Label htmlFor="out-height" className="text-xs">Height</Label>
          <Input
            id="out-height"
            type="number"
            disabled={disabled}
            min={1}
            max={4096}
            placeholder="auto"
            value={options.height ?? ""}
            onChange={onDimension("height")}
            className="h-7 text-xs"
          />
        </div>
      </div>
    </div>
  );
}
