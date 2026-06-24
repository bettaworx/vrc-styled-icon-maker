import type { NormalizerConfig } from "@/wasm/types";
import { DEFAULT_NORMALIZER_CONFIG } from "@/wasm/types";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Input } from "@/components/ui/input";
import { Separator } from "@/components/ui/separator";
import { Button } from "@/components/ui/button";
import { ArrowCounterClockwise } from "@phosphor-icons/react";
import type { ChangeEvent } from "react";

interface NormalizerControlsProps {
  config: NormalizerConfig;
  onChange: (config: NormalizerConfig) => void;
  disabled: boolean;
}

export function NormalizerControls({ config, onChange, disabled }: NormalizerControlsProps) {
  const onColor = (e: ChangeEvent<HTMLInputElement>) => {
    onChange({ ...config, fill_color: e.target.value });
  };

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-medium">Normalize</h3>
        <Button
          variant="ghost"
          size="icon-xs"
          disabled={disabled}
          onClick={() => onChange({ ...DEFAULT_NORMALIZER_CONFIG })}
        >
          <ArrowCounterClockwise size={14} />
        </Button>
      </div>
      <Separator />
      <div className="flex items-center justify-between">
        <Label htmlFor="fill-color" className="text-xs">Fill Color</Label>
        <div className="flex items-center gap-1.5">
          <input
            id="fill-color"
            type="color"
            disabled={disabled}
            value={config.fill_color}
            onChange={onColor}
            className="h-7 w-7 cursor-pointer rounded border border-border bg-transparent p-0.5 disabled:pointer-events-none disabled:opacity-50"
          />
          <Input
            disabled={disabled}
            value={config.fill_color}
            onChange={onColor}
            className="h-7 w-20 font-mono text-xs"
          />
        </div>
      </div>
      <div className="flex items-center justify-between">
        <Label htmlFor="remove-bg" className="text-xs">Remove Background</Label>
        <Switch
          id="remove-bg"
          disabled={disabled}
          checked={config.remove_background}
          onCheckedChange={(v: boolean) => onChange({ ...config, remove_background: v })}
        />
      </div>
      <div className="flex items-center justify-between">
        <Label htmlFor="monochrome" className="text-xs">Force Monochrome</Label>
        <Switch
          id="monochrome"
          disabled={disabled}
          checked={config.force_monochrome}
          onCheckedChange={(v: boolean) => onChange({ ...config, force_monochrome: v })}
        />
      </div>
    </div>
  );
}
