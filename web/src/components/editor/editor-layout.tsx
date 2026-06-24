import type { ReactNode } from "react";

interface EditorLayoutProps {
  sidebar: ReactNode;
  preview: ReactNode;
}

export function EditorLayout({ sidebar, preview }: EditorLayoutProps) {
  return (
    <div className="flex flex-col md:flex-row md:h-[calc(100vh-3.5rem)]">
      <div className="max-h-[50vh] overflow-y-auto md:max-h-none md:overflow-hidden md:order-2 md:flex-1 md:min-w-0">
        {preview}
      </div>
      <aside className="w-full shrink-0 overflow-y-auto border-t border-border md:order-1 md:w-80 md:border-t-0 md:border-r lg:w-96">
        {sidebar}
      </aside>
    </div>
  );
}
