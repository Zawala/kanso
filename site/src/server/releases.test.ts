/**
 * Tests for recentReleases: fake fetch and cache objects cover success and
 * caching, cache hits, non-OK, thrown and timed-out requests, JSON that is
 * not a list, and logging.
 */
import { describe, expect, it, vi } from "vitest";
import type { ReleaseDeps } from "./release";
import { RELEASE_CACHE_SECONDS } from "./release";
import { RELEASE_LIST_API, recentReleases } from "./releases";

const json = [
  { tag_name: "v0.9.0", published_at: "2026-10-20T09:30:00Z", body: "- Mobile apps" },
  { tag_name: "v0.8.0", published_at: "2026-09-01T00:00:00Z", body: "" },
];
const ok = (body: unknown = json) => new Response(JSON.stringify(body), { status: 200 });

function setup(response: () => Promise<Response>, cached?: Response) {
  const fetch = vi.fn(response) as unknown as ReleaseDeps["fetch"] & ReturnType<typeof vi.fn>;
  const match = vi.fn(async () => cached);
  const put = vi.fn(async () => {});
  const log = vi.fn();
  return { fetch, match, put, log, deps: { fetch, cache: { match, put }, log } as ReleaseDeps };
}

describe("recentReleases", () => {
  it("fetches, parses and caches the list", async () => {
    const t = setup(async () => ok());
    const notes = await recentReleases(t.deps);
    expect(notes?.map((n) => n.version)).toEqual(["0.9.0", "0.8.0"]);
    const [request, init] = t.fetch.mock.calls[0] as [Request, RequestInit];
    expect(request.url).toBe(RELEASE_LIST_API);
    expect(init.signal).toBeInstanceOf(AbortSignal);
    const stored = (t.put.mock.calls[0] as unknown as [Request, Response])[1];
    expect(stored.headers.get("cache-control")).toBe(`public, max-age=${RELEASE_CACHE_SECONDS}`);
    expect(await stored.json()).toEqual(json);
    expect(t.log).not.toHaveBeenCalled();
  });

  it("serves a cache hit without fetching", async () => {
    const t = setup(async () => ok(), ok());
    expect((await recentReleases(t.deps))?.length).toBe(2);
    expect(t.fetch).not.toHaveBeenCalled();
    expect(t.put).not.toHaveBeenCalled();
  });

  it("works without a cache or a logger", async () => {
    expect((await recentReleases({ fetch: (async () => ok()) as unknown as typeof fetch }))?.length).toBe(2);
    expect(await recentReleases({ fetch: (async () => new Response("", { status: 500 })) as unknown as typeof fetch })).toBeNull();
  });

  it("returns null and logs the status for a non-OK response", async () => {
    const t = setup(async () => new Response("rate limited", { status: 403 }));
    expect(await recentReleases(t.deps)).toBeNull();
    expect(t.log).toHaveBeenCalledWith("releases: github 403");
    expect(t.put).not.toHaveBeenCalled();
  });

  it("returns null when fetch throws or times out", async () => {
    const boom = setup(async () => {
      throw new TypeError("network down");
    });
    expect(await recentReleases(boom.deps)).toBeNull();
    expect(boom.log).toHaveBeenCalledWith("releases: TypeError");
    const timeout = setup(async () => {
      throw Object.assign(new Error("timed out"), { name: "TimeoutError" });
    });
    expect(await recentReleases(timeout.deps)).toBeNull();
    expect(timeout.log).toHaveBeenCalledWith("releases: TimeoutError");
    const odd = setup(() => Promise.reject("string"));
    expect(await recentReleases(odd.deps)).toBeNull();
    expect(odd.log).toHaveBeenCalledWith("releases: error");
  });

  it("returns null for JSON that is not a list, without caching", async () => {
    const t = setup(async () => ok({ message: "nope" }));
    expect(await recentReleases(t.deps)).toBeNull();
    expect(t.log).toHaveBeenCalledWith("releases: unparsable");
    expect(t.put).not.toHaveBeenCalled();
  });

  it("returns null for invalid JSON", async () => {
    const t = setup(async () => new Response("<html>", { status: 200 }));
    expect(await recentReleases(t.deps)).toBeNull();
    expect(t.log).toHaveBeenCalledWith("releases: SyntaxError");
  });
});
