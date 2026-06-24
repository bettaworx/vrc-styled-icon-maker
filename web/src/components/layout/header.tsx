import { Link } from "@tanstack/react-router";
import { ArrowSquareOut, Check, List, Moon, Monitor, Sun } from "@phosphor-icons/react";
import { DropdownMenu } from "radix-ui";
import { Button } from "@/components/ui/button";
import { useTheme } from "@/hooks/use-theme";

const themeOptions = [
  { value: "auto", label: "Auto", icon: Monitor },
  { value: "light", label: "Light", icon: Sun },
  { value: "dark", label: "Dark", icon: Moon },
] as const;

function ThemeIcon({ theme }: { theme: string }) {
  const Icon = themeOptions.find((o) => o.value === theme)?.icon ?? Monitor;
  return <Icon size={18} />;
}

export function Header() {
  const { theme, setTheme } = useTheme();

  return (
    <header className="border-b border-border bg-background">
      <div className="w-full flex h-14 items-center justify-between px-4">
        <Link to="/" className="text-lg font-semibold tracking-tight">
          vrc-styled-icon-maker
        </Link>

        <div className="hidden md:flex items-center gap-4">
          <NavLinks />
          <ThemeDropdown theme={theme} setTheme={setTheme} />
        </div>

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
              <DropdownMenu.Separator className="my-1 h-px bg-border" />
              <DropdownMenu.Label className="px-3 py-1.5 text-xs font-medium text-muted-foreground">
                Theme
              </DropdownMenu.Label>
              {themeOptions.map((opt) => (
                <DropdownMenu.Item
                  key={opt.value}
                  className="flex items-center gap-2 rounded-sm px-3 py-2 text-sm outline-none cursor-pointer hover:bg-muted"
                  onSelect={() => setTheme(opt.value)}
                >
                  <opt.icon size={16} />
                  {opt.label}
                  {theme === opt.value && <Check size={14} className="ml-auto" />}
                </DropdownMenu.Item>
              ))}
            </DropdownMenu.Content>
          </DropdownMenu.Portal>
        </DropdownMenu.Root>
      </div>
    </header>
  );
}

function ThemeDropdown({
  theme,
  setTheme,
}: {
  theme: string;
  setTheme: (t: "auto" | "light" | "dark") => void;
}) {
  return (
    <DropdownMenu.Root>
      <DropdownMenu.Trigger asChild>
        <Button variant="ghost" size="icon" aria-label="Toggle theme">
          <ThemeIcon theme={theme} />
        </Button>
      </DropdownMenu.Trigger>
      <DropdownMenu.Portal>
        <DropdownMenu.Content
          align="end"
          sideOffset={4}
          className="z-50 min-w-32 rounded-md border border-border bg-popover p-1 text-popover-foreground shadow-md animate-in fade-in-0 zoom-in-95"
        >
          {themeOptions.map((opt) => (
            <DropdownMenu.Item
              key={opt.value}
              className="flex items-center gap-2 rounded-sm px-3 py-2 text-sm outline-none cursor-pointer hover:bg-muted"
              onSelect={() => setTheme(opt.value)}
            >
              <opt.icon size={16} />
              {opt.label}
              {theme === opt.value && <Check size={14} className="ml-auto" />}
            </DropdownMenu.Item>
          ))}
        </DropdownMenu.Content>
      </DropdownMenu.Portal>
    </DropdownMenu.Root>
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
