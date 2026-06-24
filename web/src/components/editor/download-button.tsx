import { Button } from "@/components/ui/button";
import { DownloadSimple, CircleNotch } from "@phosphor-icons/react";

interface DownloadButtonProps {
  onClick: () => void;
  disabled: boolean;
  processing: boolean;
  count: number;
}

export function DownloadButton({ onClick, disabled, processing, count }: DownloadButtonProps) {
  return (
    <Button onClick={onClick} disabled={disabled} className="w-full">
      {processing ? (
        <CircleNotch size={16} className="animate-spin" />
      ) : (
        <DownloadSimple size={16} />
      )}
      {count > 1 ? `Download All (${count})` : "Download PNG"}
    </Button>
  );
}
