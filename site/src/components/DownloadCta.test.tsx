/**
 * Tests for DownloadCta's three states: no release (GitHub link), no known
 * platform or no build for it (download page link), and a known platform
 * (direct build link with version, detail, size and "Other systems"), and
 * className passthrough. Router links are mocked to plain anchors.
 */
import "../test/router";
import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { RELEASES_URL, parseRelease } from "../lib/release";
import { DownloadCta } from "./DownloadCta";

const BASE = "https://github.com/iamngoni/kanso/releases/download/v0.5.2";
const release = parseRelease({
  tag_name: "v0.5.2",
  published_at: "2026-10-20T09:30:00Z",
  assets: [
    { name: "Kanso-0.5.2-macos-arm64.dmg", size: 52_400_000, browser_download_url: `${BASE}/Kanso-0.5.2-macos-arm64.dmg` },
    { name: "Kanso-0.5.2-windows-x64-setup.exe", size: 41_000_000, browser_download_url: `${BASE}/Kanso-0.5.2-windows-x64-setup.exe` },
  ],
});
if (release === null) throw new Error("fixture");

describe("DownloadCta", () => {
  it("links to GitHub's releases when there is no release", () => {
    render(<DownloadCta release={null} platform="macos" />);
    expect(screen.getByRole("link", { name: /Download from GitHub/ })).toHaveAttribute("href", RELEASES_URL);
    expect(screen.getByText(/Free and open source/)).toBeInTheDocument();
  });

  it("links to the download page when the platform is unknown", () => {
    render(<DownloadCta release={release} platform={null} />);
    expect(screen.getByRole("link", { name: /Download Kanso/ })).toHaveAttribute("href", "/download");
    expect(screen.getByText(/Version 0\.5\.2/)).toBeInTheDocument();
    expect(screen.queryByText("Other systems")).toBeNull();
  });

  it("links to the download page when the release has no build for the platform", () => {
    render(<DownloadCta release={release} platform="linux" />);
    expect(screen.getByRole("link", { name: /Download Kanso/ })).toHaveAttribute("href", "/download");
  });

  it("links straight to the build for a known platform", () => {
    render(<DownloadCta release={release} platform="windows" />);
    const link = screen.getByRole("link", { name: /Download for Windows/ });
    expect(link).toHaveAttribute("href", `${BASE}/Kanso-0.5.2-windows-x64-setup.exe`);
    expect(link).toHaveAttribute("download");
    expect(screen.getByRole("link", { name: "Other systems" })).toHaveAttribute("href", "/download");
    expect(screen.getByText(/Version 0\.5\.2 · 64-bit · installs for you only · 41 MB/)).toBeInTheDocument();
  });

  it("passes className to the wrapper in every state", () => {
    const { container, rerender } = render(<DownloadCta release={null} platform={null} className="mine" />);
    expect(container.firstElementChild).toHaveClass("mine");
    rerender(<DownloadCta release={release} platform={null} className="mine" />);
    expect(container.firstElementChild).toHaveClass("mine");
    rerender(<DownloadCta release={release} platform="macos" className="mine" />);
    expect(container.firstElementChild).toHaveClass("mine");
  });

  it("styles the main action as the primary button and the secondary link as ink", () => {
    render(<DownloadCta release={release} platform="macos" />);
    expect(screen.getByRole("link", { name: /Download for macOS/ })).toHaveClass("bg-fg", "h-12");
    expect(screen.getByRole("link", { name: "Other systems" })).toHaveClass("ink-link");
  });
});
