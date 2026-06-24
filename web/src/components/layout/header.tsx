import { Link } from "@tanstack/react-router";
import { ArrowSquareOut, List } from "@phosphor-icons/react";
import { DropdownMenu } from "radix-ui";
import { Button } from "@/components/ui/button";

export function Header() {
  return (
    <header className="border-b border-border bg-background">
      <div className="w-full flex h-14 items-center justify-between px-4">
        <Link to="/" className="text-lg font-semibold tracking-tight">
          vrc-styled-icon-maker
        </Link>

        <nav className="hidden md:flex items-center gap-4">
          <NavLinks />
        </nav>

        <DropdownMenu.Root>
          <DropdownMenu.Trigger asChild>
            <Button variant="ghost" size="icon" className="md:hidden" aria-label="Menu">
              <List size={20} />
            </Button>
          </DropdownMenu.Trigger>
          <DropdownMenu.Portal>
            <DropdownMenu.Content
              align="end"
              sideOffset={4}
              className="z-50 min-w-40 rounded-md border border-border bg-popover p-1 text-popover-foreground shadow-md animate-in fade-in-0 zoom-in-95"
            >
              <DropdownMenu.Item asChild className="flex items-center rounded-sm px-3 py-2 text-sm outline-none cursor-pointer hover:bg-muted">
                <Link to="/">Home</Link>
              </DropdownMenu.Item>
              <DropdownMenu.Item asChild className="flex items-center rounded-sm px-3 py-2 text-sm outline-none cursor-pointer hover:bg-muted">
                <Link to="/about">About</Link>
              </DropdownMenu.Item>
              <DropdownMenu.Item asChild className="flex items-center gap-1.5 rounded-sm px-3 py-2 text-sm outline-none cursor-pointer hover:bg-muted">
                <a href="https://github.com/nekochanfood/VRCStyledIconMaker" target="_blank" rel="noopener noreferrer">
                  Source Code
                  <ArrowSquareOut size={14} />
                </a>
              </DropdownMenu.Item>
            </DropdownMenu.Content>
          </DropdownMenu.Portal>
        </DropdownMenu.Root>
      </div>
    </header>
  );
}

function NavLinks() {
  return (
    <>
      <Link to="/" className="text-sm text-muted-foreground transition-colors hover:text-foreground">
        Home
      </Link>
      <Link to="/about" className="text-sm text-muted-foreground transition-colors hover:text-foreground">
        About
      </Link>
      <a
        href="https://github.com/nekochanfood/VRCStyledIconMaker"
        target="_blank"
        rel="noopener noreferrer"
        className="flex items-center gap-1.5 text-sm text-muted-foreground transition-colors hover:text-foreground"
      >
        Source Code
        <ArrowSquareOut size={14} />
      </a>
    </>
  );
}
