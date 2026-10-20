/**
 * The document every page renders into: fonts and styles, the shared header
 * and footer, social preview tags, and the not-found page. Pages set their
 * own titles and descriptions through `head`.
 */
import { HeadContent, Link, Outlet, Scripts, createRootRoute } from "@tanstack/react-router";
import type { ReactNode } from "react";
import { SiteFooter, SiteHeader } from "../components/SiteChrome";
import appCss from "../styles/app.css?url";

export const SITE_URL = "https://kansoapp.app";
export const SITE_DESCRIPTION =
  "Kanso is a fast, local-first Markdown notes app for macOS, Windows and Linux. Your notes live on your computer and sync end-to-end encrypted. Free and open source.";

export const Route = createRootRoute({
  head: () => ({
    meta: [
      { charSet: "utf-8" },
      { name: "viewport", content: "width=device-width, initial-scale=1, viewport-fit=cover" },
      { title: "Kanso — notes, kept simple" },
      { name: "description", content: SITE_DESCRIPTION },
      { name: "theme-color", content: "#f8f4ec", media: "(prefers-color-scheme: light)" },
      { name: "theme-color", content: "#1b1814", media: "(prefers-color-scheme: dark)" },
      { property: "og:type", content: "website" },
      { property: "og:site_name", content: "Kanso" },
      { property: "og:title", content: "Kanso — notes, kept simple" },
      { property: "og:description", content: SITE_DESCRIPTION },
      { property: "og:image", content: `${SITE_URL}/og.png` },
      { property: "og:image:width", content: "1200" },
      { property: "og:image:height", content: "630" },
      { name: "twitter:card", content: "summary_large_image" },
    ],
    links: [
      { rel: "stylesheet", href: appCss },
      { rel: "icon", type: "image/png", href: "/favicon.png" },
      { rel: "apple-touch-icon", href: "/apple-touch-icon.png" },
      { rel: "canonical", href: SITE_URL },
    ],
  }),
  shellComponent: RootDocument,
  component: Outlet,
  notFoundComponent: NotFound,
});

function RootDocument({ children }: { children: ReactNode }) {
  return (
    <html lang="en">
      <head>
        <HeadContent />
      </head>
      <body className="flex min-h-dvh flex-col">
        <a
          href="#main"
          className="sr-only focus:not-sr-only focus:fixed focus:top-3 focus:left-3 focus:z-50 focus:rounded-full focus:bg-ink focus:px-4 focus:py-2 focus:text-paper"
        >
          Skip to content
        </a>
        <SiteHeader />
        <main id="main" className="flex-1">
          {children}
        </main>
        <SiteFooter />
        <Scripts />
      </body>
    </html>
  );
}

function NotFound() {
  return (
    <section className="page py-[clamp(5rem,14vw,11rem)]">
      <p className="type-eyebrow text-shu">404</p>
      <h1 className="type-section mt-4 max-w-2xl">This page was left out.</h1>
      <p className="mt-6 max-w-xl text-ink-soft">
        Kanso is about leaving things out, but not this much. The page you asked for isn't here.
      </p>
      <Link to="/" className="ink-link mt-8 inline-block font-medium">
        Back to the beginning
      </Link>
    </section>
  );
}
