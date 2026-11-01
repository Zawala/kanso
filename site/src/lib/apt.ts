/**
 * The apt repository at kansoapp.app/apt (docs/APT.md). Every file of it is
 * an asset of a GitHub Release; the site only redirects to them, so nothing
 * large is served from the Worker and CI needs no Cloudflare credentials.
 *
 * - /apt/kanso.gpg, the signing key, is a static file (public/apt/kanso.gpg),
 *   served before the Worker runs; it never reaches this code.
 * - /apt/dists/<suite>/… goes to the latest release's `apt-<suite>-…` asset
 *   (the path after dists/, its slashes made dashes). The latest release
 *   leaves out drafts and prereleases, which is what `stable` wants.
 * - /apt/pool/<tag>/<file>.deb goes to that release's .deb.
 *
 * Only those exact paths redirect; anything else under /apt is a 404, so the
 * site is never an open redirect. Redirects can't swap content: apt checks
 * InRelease against the key, and Packages pins each .deb's SHA-256.
 */
import { REPOSITORY_URL } from "./release";

/** The suites published. `unstable` needs prerelease tags first. */
export const APT_SUITES = ["stable"] as const;
export const APT_ARCHITECTURES = ["amd64", "arm64"] as const;

const LATEST_DOWNLOAD = `${REPOSITORY_URL}/releases/latest/download/`;
const DOWNLOAD = `${REPOSITORY_URL}/releases/download/`;

const TAG = /^v\d+\.\d+\.\d+$/;
const DEB = /^kanso_[0-9A-Za-z.~+-]+_(amd64|arm64)\.deb$/;

/** Each suite's index files, as paths after dists/. */
function suiteFiles(suite: string): string[] {
  const files = [`${suite}/InRelease`, `${suite}/Release`, `${suite}/Release.gpg`];
  for (const arch of APT_ARCHITECTURES) {
    files.push(`${suite}/main/binary-${arch}/Packages`, `${suite}/main/binary-${arch}/Packages.gz`);
  }
  return files;
}

const INDEX_FILES: ReadonlySet<string> = new Set(APT_SUITES.flatMap(suiteFiles));

/** The release asset for an index file: `stable/main/binary-amd64/Packages` → `apt-stable-main-binary-amd64-Packages`. */
export function aptAssetName(afterDists: string): string {
  return `apt-${afterDists.split("/").join("-")}`;
}

/**
 * Where an /apt path redirects, with whether it may be cached, or null when it
 * isn't one of the repository's files. Paths outside /apt are `undefined`:
 * not this code's.
 */
export function aptTarget(pathname: string): { location: string; cacheable: boolean } | null | undefined {
  if (pathname !== "/apt" && !pathname.startsWith("/apt/")) return undefined;
  const rest = pathname.slice("/apt/".length);
  if (rest.startsWith("dists/")) {
    const afterDists = rest.slice("dists/".length);
    if (!INDEX_FILES.has(afterDists)) return null;
    // The index changes with every release: never cached, so apt update
    // sees a new one at once.
    return { location: LATEST_DOWNLOAD + aptAssetName(afterDists), cacheable: false };
  }
  const [first, tag, file, ...more] = rest.split("/");
  if (first === "pool" && tag !== undefined && file !== undefined && more.length === 0 && TAG.test(tag) && DEB.test(file)) {
    // A tag's .deb never changes.
    return { location: `${DOWNLOAD}${tag}/${file}`, cacheable: true };
  }
  return null;
}

/** The response for an /apt path, or null when the path isn't under /apt. */
export function aptResponse(url: URL): Response | null {
  const target = aptTarget(url.pathname);
  if (target === undefined) return null;
  if (target === null) {
    return new Response("Not found\n", {
      status: 404,
      headers: { "content-type": "text/plain; charset=utf-8", "cache-control": "no-store" },
    });
  }
  return new Response(null, {
    status: 302,
    headers: {
      location: target.location,
      "cache-control": target.cacheable ? "public, max-age=86400" : "no-store",
    },
  });
}
