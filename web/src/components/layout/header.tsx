import { Link } from "@tanstack/react-router";
import { ArrowSquareOut } from "@phosphor-icons/react";

export function Header() {
  return (
    <header className="border-b border-border bg-background">
      <div className="w-full flex h-14 items-center justify-between px-4">
        <Link to="/" className="text-lg font-semibold tracking-tight">
          vrc-styled-icon-maker
        </Link>
        <nav className="flex items-center gap-4">
          <Link
            to="/"
            className="text-sm text-muted-foreground transition-colors hover:text-foreground"
          >
            Home
          </Link>
          <Link
            to="/about"
            className="text-sm text-muted-foreground transition-colors hover:text-foreground"
          >
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
        </nav>
      </div>
    </header>
  );
}
