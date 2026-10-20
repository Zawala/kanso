/**
 * The header and footer every page shares. The header stays at the top with a
 * hairline once the page scrolls; it carries the one download link so it is
 * always a click away. Links are plain anchors and router links, no menus.
 */
import { Link } from "@tanstack/react-router";
import { useEffect, useState } from "react";
import { cn } from "../lib/cn";
import { RELEASES_URL, REPOSITORY_URL } from "../lib/release";
import { Enso } from "./Enso";
import { Button } from "./ui/button";

export function Wordmark({ className }: { className?: string }) {
  return (
    <Link to="/" className={cn("group inline-flex items-center gap-2.5 text-ink", className)} aria-label="Kanso, home">
      <Enso animate={false} className="size-7 text-ink transition-transform duration-500 ease-[var(--ease-out-quint)] group-hover:rotate-[-24deg]" />
      <span className="font-display text-[1.45rem] font-extrabold tracking-tight">Kanso</span>
    </Link>
  );
}

export function SiteHeader() {
  const [scrolled, setScrolled] = useState(false);

  useEffect(() => {
    const update = () => setScrolled(window.scrollY > 8);
    update();
    window.addEventListener("scroll", update, { passive: true });
    return () => window.removeEventListener("scroll", update);
  }, []);

  return (
    <header
      data-scrolled={scrolled}
      className={cn(
        "sticky top-0 z-40 border-b bg-paper/95 transition-colors duration-300",
        scrolled ? "border-rule" : "border-transparent",
      )}
    >
      <div className="page flex h-16 items-center justify-between gap-6">
        <Wordmark />
        <nav aria-label="Main" className="flex items-center gap-1 text-[0.9375rem] sm:gap-2">
          <a href="/#features" className="hidden rounded-full px-3 py-2 text-ink-soft transition-colors hover:text-ink sm:inline-block">
            Features
          </a>
          <Link to="/privacy" className="hidden rounded-full px-3 py-2 text-ink-soft transition-colors hover:text-ink sm:inline-block">
            Privacy
          </Link>
          <a href={REPOSITORY_URL} className="rounded-full px-3 py-2 text-ink-soft transition-colors hover:text-ink">
            GitHub
          </a>
          <Button asChild size="sm" className="ml-1">
            <Link to="/download">Download</Link>
          </Button>
        </nav>
      </div>
    </header>
  );
}

export function SiteFooter() {
  return (
    <footer className="border-t border-rule">
      <div className="page grid gap-10 py-14 sm:grid-cols-[1fr_auto] sm:items-end">
        <div>
          <Wordmark />
          <p className="mt-4 max-w-sm text-[0.9375rem] text-ink-faint">
            <span lang="ja" className="font-display text-ink">簡素</span> — simplicity through leaving things out. Free and open source under the AGPL&#8209;3.0.
          </p>
        </div>
        <nav aria-label="Footer" className="flex flex-wrap gap-x-7 gap-y-3 text-[0.9375rem] text-ink-soft">
          <Link to="/download" className="ink-link">
            Download
          </Link>
          <Link to="/privacy" className="ink-link">
            Privacy
          </Link>
          <a href={RELEASES_URL} className="ink-link">
            Release notes
          </a>
          <a href={REPOSITORY_URL} className="ink-link">
            Source code
          </a>
        </nav>
      </div>
    </footer>
  );
}
