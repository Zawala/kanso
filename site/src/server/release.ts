/**
 * Fetches Kanso's latest release from GitHub's public API for the pages that
 * offer downloads. Answers are kept in the edge cache for ten minutes so
 * visitors don't spend the Worker's unauthenticated GitHub quota. Any failure
 * (network, timeout, rate limit, a release that doesn't parse) is null, and
 * the pages then link to the releases page on GitHub instead of a build.
 */
import { REPOSITORY, type Release, parseRelease } from "../lib/release";

export const LATEST_RELEASE_API = `https://api.github.com/repos/${REPOSITORY}/releases/latest`;
export const RELEASE_CACHE_SECONDS = 600;
export const RELEASE_TIMEOUT_MS = 4_000;

/** What `latestRelease` needs from its runtime; tests pass fakes. */
export interface ReleaseDeps {
  readonly fetch: typeof fetch;
  /** Cloudflare's `caches.default`; absent outside Workers. */
  readonly cache?: Pick<Cache, "match" | "put"> | undefined;
  /** Logs one line per failure, without request details. */
  readonly log?: ((line: string) => void) | undefined;
}

export async function latestRelease(deps: ReleaseDeps): Promise<Release | null> {
  const request = new Request(LATEST_RELEASE_API, {
    headers: { accept: "application/vnd.github+json", "user-agent": "kansoapp.app", "x-github-api-version": "2022-11-28" },
  });
  const log = deps.log ?? (() => {});
  try {
    const cached = await deps.cache?.match(request);
    if (cached) return parseRelease(await cached.json());
    const response = await deps.fetch(request, { signal: AbortSignal.timeout(RELEASE_TIMEOUT_MS) });
    if (!response.ok) {
      log(`release: github ${response.status}`);
      return null;
    }
    const json: unknown = await response.json();
    const release = parseRelease(json);
    if (release === null) {
      log("release: unparsable");
      return null;
    }
    await deps.cache?.put(
      request,
      new Response(JSON.stringify(json), {
        headers: { "content-type": "application/json", "cache-control": `public, max-age=${RELEASE_CACHE_SECONDS}` },
      }),
    );
    return release;
  } catch (err) {
    log(`release: ${err instanceof Error ? err.name : "error"}`);
    return null;
  }
}

/** The runtime's dependencies: global fetch, and the edge cache where there is one. */
export function runtimeDeps(): ReleaseDeps {
  const edge = (globalThis as { caches?: { default?: Cache } }).caches?.default;
  return { fetch: globalThis.fetch.bind(globalThis), cache: edge, log: (line) => console.warn(line) };
}
