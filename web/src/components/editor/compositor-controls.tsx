import type { CompositorConfig } from "@/wasm/types";
import { DEFAULT_COMPOSITOR_CONFIG } from "@/wasm/types";
import { Label } from "@/components/ui/label";
import { Slider } from "@/components/ui/slider";
import { Input } from "@/components/ui/input";
import { Separator } from "@/components/ui/separator";
import { Button } from "@/components/ui/button";
import { ArrowCounterClockwise } from "@phosphor-icons/react";
import type { ChangeEvent } from "react";

interface CompositorControlsProps {
  config: CompositorConfig;
  onChange: (config: CompositorConfig) => void;
  disabled: boolean;
}

export function CompositorControls({ config, onChange, disabled }: CompositorControlsProps) {
  const onInput = (field: keyof CompositorConfig) => (e: ChangeEvent<HTMLInputElement>) => {
    onChange({ ...config, [field]: e.target.value });
  };

  const onNumber = (field: keyof CompositorConfig) => (e: ChangeEvent<HTMLInputElement>) => {
    onChange({ ...config, [field]: Number(e.target.value) });
  };

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-medium">Compositor</h3>
        <Button
          variant="ghost"
          size="icon-xs"
          disabled={disabled}
          onClick={() => onChange({ ...DEFAULT_COMPOSITOR_CONFIG })}
        >
          <ArrowCounterClockwise size={14} />
        </Button>
      </div>
      <Separator />

      <div className="flex items-center justify-between">
        <Label htmlFor="grad-start" className="text-xs">Gradient Start</Label>
        <div className="flex items-center gap-1.5">
          <input
            id="grad-start"
            type="color"
            disabled={disabled}
            value={config.gradient_start_color}
            onChange={onInput("gradient_start_color")}
            className="h-7 w-7 cursor-pointer rounded border border-border bg-transparent p-0.5 disabled:pointer-events-none disabled:opacity-50"
          />
          <Input
            disabled={disabled}
            value={config.gradient_start_color}
            onChange={onInput("gradient_start_color")}
            className="h-7 w-20 font-mono text-xs"
          />
        </div>
      </div>

      <div className="flex items-center justify-between">
        <Label htmlFor="grad-end" className="text-xs">Gradient End</Label>
        <div className="flex items-center gap-1.5">
          <input
            id="grad-end"
            type="color"
            disabled={disabled}
            value={config.gradient_end_color}
            onChange={onInput("gradient_end_color")}
            className="h-7 w-7 cursor-pointer rounded border border-border bg-transparent p-0.5 disabled:pointer-events-none disabled:opacity-50"
          />
          <Input
            disabled={disabled}
            value={config.gradient_end_color}
            onChange={onInput("gradient_end_color")}
            className="h-7 w-20 font-mono text-xs"
          />
        </div>
      </div>

      <div className="space-y-2.5">
        <div className="flex items-center justify-between">
          <Label className="text-xs">Shadow Blur</Label>
          <span className="text-xs text-muted-foreground">{config.shadow_blur.toFixed(1)}</span>
        </div>
        <Slider
          disabled={disabled}
          min={0} max={30} step={0.5}
          value={[config.shadow_blur]}
          onValueChange={([v]: number[]) => onChange({ ...config, shadow_blur: v })}
        />
      </div>

      <div className="space-y-2.5">
        <div className="flex items-center justify-between">
          <Label className="text-xs">Shadow Opacity</Label>
          <span className="text-xs text-muted-foreground">{config.shadow_opacity.toFixed(2)}</span>
        </div>
        <Slider
          disabled={disabled}
          min={0} max={1} step={0.01}
          value={[config.shadow_opacity]}
          onValueChange={([v]: number[]) => onChange({ ...config, shadow_opacity: v })}
        />
      </div>

      <div className="flex items-center justify-between">
        <Label htmlFor="shadow-color" className="text-xs">Shadow Color</Label>
        <div className="flex items-center gap-1.5">
          <input
            id="shadow-color"
            type="color"
            disabled={disabled}
            value={config.shadow_color}
            onChange={onInput("shadow_color")}
            className="h-7 w-7 cursor-pointer rounded border border-border bg-transparent p-0.5 disabled:pointer-events-none disabled:opacity-50"
          />
          <Input
            disabled={disabled}
            value={config.shadow_color}
            onChange={onInput("shadow_color")}
            className="h-7 w-20 font-mono text-xs"
          />
        </div>
      </div>

      <div className="grid grid-cols-2 gap-2">
        <div className="space-y-1">
          <Label htmlFor="offset-x" className="text-xs">Offset X</Label>
          <Input
            id="offset-x"
            type="number"
            disabled={disabled}
            value={config.shadow_offset_x}
            onChange={onNumber("shadow_offset_x")}
            className="h-7 text-xs"
          />
        </div>
        <div className="space-y-1">
          <Label htmlFor="offset-y" className="text-xs">Offset Y</Label>
          <Input
            id="offset-y"
            type="number"
            disabled={disabled}
            value={config.shadow_offset_y}
            onChange={onNumber("shadow_offset_y")}
            className="h-7 text-xs"
          />
        </div>
      </div>

      <div className="grid grid-cols-2 gap-2">
        <div className="space-y-1">
          <Label htmlFor="canvas-size" className="text-xs">Canvas Size</Label>
          <Input
            id="canvas-size"
            type="number"
            disabled={disabled}
            min={64}
            max={4096}
            value={config.canvas_size}
            onChange={onNumber("canvas_size")}
            className="h-7 text-xs"
          />
        </div>
        <div className="space-y-1">
          <Label htmlFor="padding" className="text-xs">Padding</Label>
          <Input
            id="padding"
            type="number"
            disabled={disabled}
            min={0}
            max={512}
            value={config.padding}
            onChange={onNumber("padding")}
            className="h-7 text-xs"
          />
        </div>
      </div>
    </div>
  );
}
