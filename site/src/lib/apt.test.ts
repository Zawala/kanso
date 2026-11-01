/**
 * Tests for the apt repository's redirects: each index file to the latest
 * release's flattened asset, each .deb to its tag's asset, and a 404 for
 * anything else under /apt, so the site never redirects where it wasn't meant to.
 */
import { describe, expect, it } from "vitest";
import { aptAssetName, aptResponse, aptTarget } from "./apt";
import { redirectFor } from "./redirects";

const LATEST = "https://github.com/iamngoni/kanso/releases/latest/download/";

const answer = (path: string) => {
  const response = redirectFor(new URL(`https://kansoapp.app${path}`));
  return response && { status: response.status, location: response.headers.get("location"), cache: response.headers.get("cache-control") };
};

describe("apt", () => {
  it("flattens an index path into its asset name", () => {
    expect(aptAssetName("stable/InRelease")).toBe("apt-stable-InRelease");
    expect(aptAssetName("stable/main/binary-arm64/Packages.gz")).toBe("apt-stable-main-binary-arm64-Packages.gz");
  });

  it("sends each index file to the latest release, uncached", () => {
    const files = [
      "stable/InRelease",
      "stable/Release",
      "stable/Release.gpg",
      "stable/main/binary-amd64/Packages",
      "stable/main/binary-amd64/Packages.gz",
      "stable/main/binary-arm64/Packages",
      "stable/main/binary-arm64/Packages.gz",
    ];
    for (const file of files) {
      expect(answer(`/apt/dists/${file}`), file).toEqual({ status: 302, location: LATEST + aptAssetName(file), cache: "no-store" });
    }
  });

  it("sends a .deb to its tag's release", () => {
    expect(answer("/apt/pool/v0.9.0/kanso_0.9.0-1_amd64.deb")).toEqual({
      status: 302,
      location: "https://github.com/iamngoni/kanso/releases/download/v0.9.0/kanso_0.9.0-1_amd64.deb",
      cache: "public, max-age=86400",
    });
    expect(answer("/apt/pool/v10.20.30/kanso_10.20.30~rc1_arm64.deb")?.status).toBe(302);
  });

  it("answers 404 for anything else under /apt", () => {
    const refused = [
      "/apt",
      "/apt/",
      "/apt/kanso.gpg", // served as a static file once it exists
      "/apt/dists/unstable/InRelease",
      "/apt/dists/stable/main/binary-i386/Packages",
      "/apt/dists/stable/main/binary-amd64/Packages.xz",
      "/apt/dists/stable/../../InRelease",
      "/apt/dists/stable/InRelease/",
      "/apt/pool/v0.9/kanso_0.9.0_amd64.deb",
      "/apt/pool/0.9.0/kanso_0.9.0_amd64.deb",
      "/apt/pool/v0.9.0-rc1/kanso_0.9.0_amd64.deb",
      "/apt/pool/v0.9.0/other_0.9.0_amd64.deb",
      "/apt/pool/v0.9.0/kanso_0.9.0_i386.deb",
      "/apt/pool/v0.9.0/kanso_0.9.0_amd64.deb/x",
      "/apt/pool/v0.9.0/kanso_0.9.0_amd64.deb.asc",
      "/apt/pool/v0.9.0/..%2F..%2Fkanso_0.9.0_amd64.deb",
      "/apt/pool/v0.9.0/kanso_%2Fevil_amd64.deb",
      "/apt/pool/v0.9.0/kanso_0.9.0_amd64.deb?x=https://evil.example",
    ];
    for (const path of refused) {
      const url = new URL(`https://kansoapp.app${path}`);
      if (url.pathname.startsWith("/apt/pool/v0.9.0/kanso_0.9.0_amd64.deb") && url.search) {
        // The query is ignored; the redirect stays on GitHub.
        expect(answer(path)?.location, path).toMatch(/^https:\/\/github\.com\/iamngoni\/kanso\//);
        continue;
      }
      expect(answer(path), path).toMatchObject({ status: 404, location: null });
    }
  });

  it("leaves every path outside /apt alone", () => {
    for (const path of ["/", "/apt-get", "/aptx/dists/stable/InRelease", "/download"]) {
      expect(aptTarget(path), path).toBeUndefined();
      expect(aptResponse(new URL(`https://kansoapp.app${path}`)), path).toBeNull();
    }
  });
});
