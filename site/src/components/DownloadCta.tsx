/**
 * The page's main action: the build for the visitor's system when we can tell
 * what it is, the download page when we can't, and GitHub's releases page
 * when the latest release couldn't be read. It never offers a build it hasn't
 * parsed from the release, and it says which version and size it offers.
 */
import { Link } from "@tanstack/react-router";
import { ArrowDown, ArrowUpRight } from "lucide-react";
import { cn } from "../lib/cn";
import { PLATFORM_NAMES, type Platform } from "../lib/platform";
import { RELEASES_URL, type Release, buildsFor, formatSize } from "../lib/release";
import { Button } from "./ui/button";

export interface DownloadCtaProps {
  release: Release | null;
  platform: Platform | null;
  /** "onSumi" on the dark band. */
  tone?: "paper" | "sumi";
  className?: string;
}

export function DownloadCta({ release, platform, tone = "paper", className }: DownloadCtaProps) {
  const variant = tone === "sumi" ? "onSumi" : "primary";
  const meta = cn("text-[0.9375rem]", tone === "sumi" ? "text-on-sumi-soft" : "text-ink-faint");
  const secondary = cn(
    "ink-link text-[0.9375rem] font-medium",
    tone === "sumi" ? "text-on-sumi" : "text-ink-soft hover:text-ink",
  );

  if (release === null) {
    return (
      <div className={cn("flex flex-col items-start gap-3", className)}>
        <Button asChild size="lg" variant={variant}>
          <a href={RELEASES_URL}>
            Download from GitHub <ArrowUpRight />
          </a>
        </Button>
        <p className={meta}>Free and open source · macOS, Windows and Linux</p>
      </div>
    );
  }

  const primary = platform ? buildsFor(release, platform)[0] : undefined;
  if (platform === null || primary === undefined) {
    return (
      <div className={cn("flex flex-col items-start gap-3", className)}>
        <Button asChild size="lg" variant={variant}>
          <Link to="/download">
            Download Kanso <ArrowDown />
          </Link>
        </Button>
        <p className={meta}>
          Version {release.version} · free and open source · macOS, Windows and Linux
        </p>
      </div>
    );
  }

  return (
    <div className={cn("flex flex-col items-start gap-3", className)}>
      <div className="flex flex-wrap items-center gap-x-6 gap-y-3">
        <Button asChild size="lg" variant={variant}>
          <a href={primary.url} download>
            Download for {PLATFORM_NAMES[platform]} <ArrowDown />
          </a>
        </Button>
        <Link to="/download" className={secondary}>
          Other systems
        </Link>
      </div>
      <p className={meta}>
        Version {release.version} · {primary.detail} · {formatSize(primary.size)}
      </p>
    </div>
  );
}
