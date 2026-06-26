import { Button } from "@/components/ui/button";
import { DownloadSimple, CircleNotch, DotsThree } from "@phosphor-icons/react";
import { DropdownMenu } from "radix-ui";

interface DownloadButtonProps {
  onClick: () => void;
  onDownloadSvg: () => void;
  disabled: boolean;
  processing: boolean;
  count: number;
}

export function DownloadButton({ onClick, onDownloadSvg, disabled, processing, count }: DownloadButtonProps) {
  return (
    <div className="flex w-full gap-1">
      <Button onClick={onClick} disabled={disabled} className="flex-1">
        {processing ? (
          <CircleNotch size={16} className="animate-spin" />
        ) : (
          <DownloadSimple size={16} />
        )}
        {count > 1 ? `Export All as PNG (${count})` : "Export as PNG"}
      </Button>
      <DropdownMenu.Root>
        <DropdownMenu.Trigger asChild>
          <Button variant="outline" size="icon" disabled={disabled} className="shrink-0">
            <DotsThree size={16} weight="bold" />
          </Button>
        </DropdownMenu.Trigger>
        <DropdownMenu.Portal>
          <DropdownMenu.Content
            align="end"
            sideOffset={4}
            className="z-50 min-w-36 rounded-md border border-border bg-popover p-1 shadow-md"
          >
            <DropdownMenu.Item
              onSelect={onDownloadSvg}
              className="flex cursor-pointer items-center gap-2 rounded-sm px-2 py-1.5 text-sm outline-none hover:bg-accent focus:bg-accent"
            >
              <DownloadSimple size={14} />
              {count > 1 ? `Export All as SVG (${count})` : "Export as SVG"}
            </DropdownMenu.Item>
          </DropdownMenu.Content>
        </DropdownMenu.Portal>
      </DropdownMenu.Root>
    </div>
  );
}
