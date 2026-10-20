/**
 * Every build of the latest release, grouped by system, with what each needs
 * and how to install it. The visitor's own system comes first. When the
 * release can't be read, the page says so and links to GitHub's releases.
 */
import { createFileRoute } from "@tanstack/react-router";
import { ArrowDown, ArrowUpRight } from "lucide-react";
import { Button } from "../components/ui/button";
import { cn } from "../lib/cn";
import { PLATFORMS, PLATFORM_NAMES, type Platform } from "../lib/platform";
import { RELEASES_URL, type Release, buildsFor, formatDate, formatSize } from "../lib/release";
import { getSiteData } from "../server/fns";

export const Route = createFileRoute("/download")({
  loader: () => getSiteData(),
  head: () => ({
    meta: [
      { title: "Download Kanso" },
      { name: "description", content: "Download Kanso for macOS, Windows or Linux. Free and open source; it updates itself." },
    ],
    links: [{ rel: "canonical", href: "https://kansoapp.app/download" }],
  }),
  component: DownloadPage,
});

const INSTALL: Record<Platform, readonly string[]> = {
  macos: [
    "Open the disk image and drag Kanso to Applications.",
    "It is signed and notarized by Apple, so it opens like any other app.",
  ],
  windows: [
    "Run the installer. It installs for you only and doesn't ask for an administrator.",
    "It isn't code-signed yet: if SmartScreen stops it, choose More info, then Run anyway.",
  ],
  linux: [
    "Unpack the archive and run ./install.sh, which adds Kanso to your applications menu.",
    "Kanso needs Vulkan drivers (Mesa's are fine) and a running Secret Service to stay signed in.",
  ],
};

function DownloadPage() {
  const { release, platform } = Route.useLoaderData();
  const order = platform ? [platform, ...PLATFORMS.filter((p) => p !== platform)] : [...PLATFORMS];
  return (
    <section className="page pt-[clamp(3.5rem,9vw,7rem)] pb-[clamp(5rem,12vw,9rem)]">
      <p className="text-[0.875rem] font-medium text-fg-3">Download</p>
      <h1 className="mt-3 max-w-3xl text-[clamp(2.2rem,1.6rem+2.6vw,3.6rem)] leading-[1.05] font-semibold tracking-[-0.035em]">Kanso, for your computer.</h1>
      {release ? (
        <p className="text-[clamp(1.1rem,1rem+0.4vw,1.3rem)] leading-normal mt-6 max-w-2xl text-fg-2">
          Version {release.version}, released {formatDate(release.publishedAt)}. Once it's installed, Kanso updates itself.{" "}
          <a href={release.pageUrl} className="ink-link">
            What's new
          </a>
        </p>
      ) : (
        <p className="text-[clamp(1.1rem,1rem+0.4vw,1.3rem)] leading-normal mt-6 max-w-2xl text-fg-2">
          The list of builds couldn't be loaded just now. Every build is on{" "}
          <a href={RELEASES_URL} className="ink-link">
            Kanso's releases page on GitHub
          </a>
          .
        </p>
      )}

      {release && (
        <div className="mt-[clamp(3rem,7vw,5rem)] grid gap-px overflow-hidden rounded-[0.9rem] border border-line bg-line">
          {order.map((p) => (
            <PlatformRow key={p} platform={p} release={release} mine={p === platform} />
          ))}
        </div>
      )}

      {release?.checksumsUrl && (
        <p className="mt-8 max-w-2xl text-[0.9375rem] text-fg-3">
          Each download's SHA-256 is in{" "}
          <a href={release.checksumsUrl} className="ink-link">
            SHA256SUMS
          </a>
          . Check the ones you have with <code className="font-mono text-[0.875rem]">shasum -a 256 -c SHA256SUMS --ignore-missing</code>.
        </p>
      )}
      <p className="mt-4 max-w-2xl text-[0.9375rem] text-fg-3">
        Prefer to build it yourself?{" "}
        <a href={`${RELEASES_URL.replace(/\/releases$/, "")}#readme`} className="ink-link">
          The source and build steps are on GitHub
        </a>
        .
      </p>
    </section>
  );
}

function PlatformRow({ platform, release, mine }: { platform: Platform; release: Release; mine: boolean }) {
  const builds = buildsFor(release, platform);
  return (
    <div className={cn("grid gap-8 bg-elevated p-6 sm:p-9 lg:grid-cols-12", mine && "bg-sidebar")}>
      <div className="lg:col-span-4">
        <h2 className="flex items-center gap-3 text-[clamp(1.5rem,1.25rem+0.9vw,2rem)] font-semibold tracking-[-0.02em]">
          {PLATFORM_NAMES[platform]}
          {mine && <span className="rounded-full bg-accent/12 px-2 py-0.5 font-[family-name:var(--font-sans)] text-[0.75rem] font-medium text-accent">Your system</span>}
        </h2>
        <ul className="mt-4 grid gap-2 text-[0.9375rem] text-fg-2">
          {INSTALL[platform].map((line) => (
            <li key={line}>{line}</li>
          ))}
        </ul>
      </div>
      <div className="grid content-start gap-3 lg:col-span-7 lg:col-start-6">
        {builds.length === 0 && (
          <p className="text-fg-2">
            No build in this release.{" "}
            <a href={RELEASES_URL} className="ink-link">
              Earlier releases
            </a>
          </p>
        )}
        {builds.map((build, i) => (
          <div key={build.kind} className="flex flex-wrap items-center justify-between gap-4 border-b border-line pb-3 last:border-0 last:pb-0">
            <div>
              <p className="font-medium text-fg">{build.label}</p>
              <p className="text-[0.875rem] text-fg-3">
                {build.detail} · {formatSize(build.size)}
              </p>
            </div>
            <Button asChild variant={i === 0 && mine ? "primary" : "outline"} size="md">
              <a href={build.url} download>
                Download <ArrowDown />
              </a>
            </Button>
          </div>
        ))}
      </div>
      <a href={release.pageUrl} className="sr-only">
        Release notes for {release.version} <ArrowUpRight />
      </a>
    </div>
  );
}
