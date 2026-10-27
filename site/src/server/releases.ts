/**
 * Fetches Kanso's recent releases from GitHub's public API for the release
 * notes page. Answered like `latestRelease`: kept in the edge cache for ten
 * minutes, and any failure (network, timeout, rate limit, JSON that isn't a
 * list) is null, and the page then links to the releases page on GitHub.
 */
import { REPOSITORY } from "../lib/release";
import { RELEASE_NOTES_COUNT, type ReleaseNote, parseReleaseNotes } from "../lib/releases";
import { RELEASE_CACHE_SECONDS, RELEASE_TIMEOUT_MS, type ReleaseDeps } from "./release";

// Drafts and prereleases come back in the list and are dropped by the parser, so ask for a few extra.
export const RELEASE_LIST_API = `https://api.github.com/repos/${REPOSITORY}/releases?per_page=${RELEASE_NOTES_COUNT + 10}`;

export async function recentReleases(deps: ReleaseDeps): Promise<ReleaseNote[] | null> {
  const request = new Request(RELEASE_LIST_API, {
    headers: { accept: "application/vnd.github+json", "user-agent": "kansoapp.app", "x-github-api-version": "2022-11-28" },
  });
  const log = deps.log ?? (() => {});
  try {
    const cached = await deps.cache?.match(request);
    if (cached) return parseReleaseNotes(await cached.json());
    const response = await deps.fetch(request, { signal: AbortSignal.timeout(RELEASE_TIMEOUT_MS) });
    if (!response.ok) {
      log(`releases: github ${response.status}`);
      return null;
    }
    const json: unknown = await response.json();
    if (!Array.isArray(json)) {
      log("releases: unparsable");
      return null;
    }
    await deps.cache?.put(
      request,
      new Response(JSON.stringify(json), {
        headers: { "content-type": "application/json", "cache-control": `public, max-age=${RELEASE_CACHE_SECONDS}` },
      }),
    );
    return parseReleaseNotes(json);
  } catch (err) {
    log(`releases: ${err instanceof Error ? err.name : "error"}`);
    return null;
  }
}
