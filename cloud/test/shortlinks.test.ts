// Short links for published notes (docs/ACCOUNTS.md › Published notes › Short
// links), against a stand-in for Rerout's API: a share gets one link whose
// target never holds the key, its preview title follows the note's, a failing
// Rerout never fails publishing, and whatever stops a share being served
// deletes its link, now or on a later cron.

import { env } from "cloudflare:workers";
import fc from "fast-check";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { handle, runCron } from "../src/index";
import { DEFAULT_TITLE, MAX_TITLE_CHARS, PREVIEW_IMAGE_PATH, REROUT_API, isShortUrl, previewTitle, titleHeader } from "../src/rerout";
import { ORIGIN, START, base64url, call, callJson, resetServer, runAdmin, signUp, testEnv } from "./helpers";

const DAY = 86_400_000;
const KEY = "rrk_test_not_a_real_key";
const PROJECT = "prj_kanso_test";
const runs = (n: number) => ({ numRuns: Math.min(n, Number(env.FAST_CHECK_RUNS)) });

const random = (n: number): Uint8Array => crypto.getRandomValues(new Uint8Array(n));
const newShareId = (): string => base64url(random(16));

/** A sealed note under a fresh key, and the key as the link's fragment carries it. */
async function seal(id: string, html: string): Promise<{ bytes: Uint8Array; fragment: string }> {
	const key = random(32);
	const nonce = random(12);
	const aad = new Uint8Array([...new TextEncoder().encode("kanso-share"), 1, ...new TextEncoder().encode(id)]);
	const k = await crypto.subtle.importKey("raw", key, { name: "AES-GCM" }, false, ["encrypt"]);
	const ct = new Uint8Array(await crypto.subtle.encrypt({ name: "AES-GCM", iv: nonce, additionalData: aad }, k, new TextEncoder().encode(html)));
	return { bytes: new Uint8Array([1, ...nonce, ...ct]), fragment: base64url(key) };
}

const sha256Hex = async (bytes: Uint8Array) =>
	[...new Uint8Array(await crypto.subtle.digest("SHA-256", bytes))].map((b) => b.toString(16).padStart(2, "0")).join("");

interface Sent {
	method: string;
	url: string;
	authorization: string | null;
	body: string;
}

/** Rerout's link routes as the Worker uses them, with switches for failure. */
class FakeRerout {
	sent: Sent[] = [];
	links = new Map<string, { target_url: string; seo_title: string | null; deleted: boolean }>();
	/** Status every call answers instead, or "throw" for a network error. */
	failing: number | "throw" | null = null;
	/** What create answers, changed after the link is made. */
	tamper: ((link: Record<string, unknown>) => void) | null = null;
	/** Runs while a create is in flight, before it answers. */
	duringCreate: (() => Promise<void>) | null = null;
	private next = 0;

	async fetch(input: RequestInfo | URL, init?: RequestInit): Promise<Response> {
		const request = new Request(input, init);
		const body = request.method === "GET" || request.method === "DELETE" ? "" : await request.text();
		this.sent.push({ method: request.method, url: request.url, authorization: request.headers.get("authorization"), body });
		if (!request.url.startsWith(`${REROUT_API}/v1/links`)) throw new Error(`unexpected fetch: ${request.url}`);
		if (this.failing === "throw") throw new TypeError("network down");
		if (this.failing !== null) return Response.json({ code: "internal_error", message: "down" }, { status: this.failing });
		if (request.headers.get("authorization") !== `Bearer ${KEY}`) return Response.json({ code: "unauthorized" }, { status: 401 });
		const path = new URL(request.url).pathname;
		if (request.method === "POST" && path === "/v1/links") {
			const input = JSON.parse(body);
			const code = `c${(this.next += 1)}`;
			this.links.set(code, { target_url: input.target_url, seo_title: input.seo_title, deleted: false });
			if (this.duringCreate) await this.duringCreate();
			const host = typeof input.domain_hostname === "string" ? input.domain_hostname : "rerout.co";
			const link: Record<string, unknown> = { code, short_url: `https://${host}/${code}`, project_id: PROJECT, target_url: input.target_url, ...input };
			this.tamper?.(link);
			return Response.json(link, { status: 201 });
		}
		const code = decodeURIComponent(path.replace(/^\/v1\/links\//, ""));
		const link = this.links.get(code);
		if (!link || link.deleted) return Response.json({ code: "not_found" }, { status: 404 });
		if (request.method === "PATCH") {
			Object.assign(link, JSON.parse(body));
			return Response.json({ code, ...link });
		}
		if (request.method === "DELETE") {
			link.deleted = true;
			return Response.json({ deleted: true });
		}
		return Response.json({ code: "bad_request" }, { status: 400 });
	}

	live(): string[] {
		return [...this.links].filter(([, l]) => !l.deleted).map(([code]) => code);
	}
}

let rerout: FakeRerout;
const reroutEnv = () => testEnv({ REROUT_API_KEY: KEY, REROUT_PROJECT_ID: PROJECT });

beforeEach(async () => {
	await resetServer();
	rerout = new FakeRerout();
	vi.spyOn(globalThis, "fetch").mockImplementation((input, init) => rerout.fetch(input, init));
});
afterEach(() => vi.restoreAllMocks());

async function put(token: string, id: string, bytes: Uint8Array, options: { title?: string | null; expiresAt?: number; e?: ReturnType<typeof testEnv> } = {}) {
	const headers: Record<string, string> = { "content-type": "application/octet-stream", "x-kanso-sha256": await sha256Hex(bytes) };
	if (options.title !== null) headers["x-kanso-title"] = encodeURIComponent(options.title ?? "Groceries");
	if (options.expiresAt !== undefined) headers["x-kanso-expires-at"] = String(options.expiresAt);
	return callJson(`/v2/shares/${id}`, { token, method: "PUT", body: bytes, headers, env: options.e ?? reroutEnv() });
}

const unpublish = (token: string, id: string, e = reroutEnv()) => call(`/v2/shares/${id}`, { token, method: "DELETE", env: e });
const row = (id: string) => env.DB.prepare("SELECT short_code, short_url, short_title FROM shares WHERE id = ?1").bind(id).first();
const queued = async () =>
	(await env.DB.prepare("SELECT code, attempts FROM short_link_purges ORDER BY code").all<{ code: string; attempts: number }>()).results;

describe("making a short link", () => {
	it("makes links on the custom domain when one is set, and uses no link Rerout put on another host", async () => {
		const a = await signUp("alice");
		const onDomain = () => testEnv({ REROUT_API_KEY: KEY, REROUT_PROJECT_ID: PROJECT, REROUT_DOMAIN: " Go.KansoApp.app " });
		const id = newShareId();
		const r = await put(a.token, id, (await seal(id, "x")).bytes, { e: onDomain() });
		expect(r).toMatchObject({ status: 200, body: { short_url: "https://go.kansoapp.app/c1" } });
		expect(JSON.parse(rerout.sent[0].body)).toMatchObject({ domain_hostname: "go.kansoapp.app", target_url: `${ORIGIN}/p/${id}` });

		// Rerout answering on its own host (the domain not ready, say): deleted, the long link stays.
		rerout.tamper = (l) => (l.short_url = `https://rerout.co/${l.code}`);
		const other = newShareId();
		expect((await put(a.token, other, (await seal(other, "x")).bytes, { e: onDomain() })).body.short_url).toBeNull();
		expect(await row(other)).toMatchObject({ short_code: null });
		expect(rerout.live()).toEqual(["c1"]);
	});

	it("leads to SHARE_ORIGIN when it is an https origin, and to where the note was published otherwise", async () => {
		const a = await signUp("alice");
		const target = async (SHARE_ORIGIN: string | undefined) => {
			const before = rerout.sent.length;
			const id = newShareId();
			await put(a.token, id, (await seal(id, "x")).bytes, { e: testEnv({ REROUT_API_KEY: KEY, REROUT_PROJECT_ID: PROJECT, SHARE_ORIGIN }) });
			return { id, sent: JSON.parse(rerout.sent[before].body).target_url };
		};
		const shared = await target(" https://Share.KansoApp.app/ ");
		expect(shared.sent).toBe(`https://share.kansoapp.app/p/${shared.id}`);
		for (const bad of [undefined, "", "http://share.kansoapp.app", "not a url"]) {
			const fallback = await target(bad);
			expect(fallback.sent).toBe(`${ORIGIN}/p/${fallback.id}`);
		}
	});

	it("hands out a link on the custom domain only: any other host answered is refused", async () => {
		const a = await signUp("alice");
		const onDomain = testEnv({ REROUT_API_KEY: KEY, REROUT_PROJECT_ID: PROJECT, REROUT_DOMAIN: "go.kansoapp.app" });
		await fc.assert(
			fc.asyncProperty(fc.domain(), async (host) => {
				rerout.tamper = (l) => (l.short_url = `https://${host}/${l.code}`);
				const id = newShareId();
				const url = (await put(a.token, id, (await seal(id, "x")).bytes, { e: onDomain })).body.short_url;
				expect(url === null ? null : new URL(url).hostname).toBe(new URL(`https://${host}/`).hostname === "go.kansoapp.app" ? "go.kansoapp.app" : null);
			}),
			runs(40),
		);
	});

	it("gives a share one link to /p/<id> without the key, previewing the title over Kanso's image, noindex", async () => {
		const a = await signUp("alice");
		const id = newShareId();
		const { bytes, fragment } = await seal(id, "<p>Milk</p>");
		const r = await put(a.token, id, bytes, { title: "  Groceries\n for  Sunday " });
		expect(r).toMatchObject({ status: 200, body: { ok: true, id, short_url: "https://rerout.co/c1" } });
		expect(rerout.sent).toHaveLength(1);
		const [create] = rerout.sent;
		expect(create).toMatchObject({ method: "POST", url: `${REROUT_API}/v1/links`, authorization: `Bearer ${KEY}` });
		expect(JSON.parse(create.body)).toEqual({
			target_url: `${ORIGIN}/p/${id}`,
			seo_title: "Groceries for Sunday",
			seo_description: expect.stringContaining("Kanso"),
			seo_image_url: `${ORIGIN}${PREVIEW_IMAGE_PATH}`,
			seo_canonical_url: `${ORIGIN}/p/${id}`,
			seo_noindex: true,
		});
		expect(create.body).not.toContain("#");
		expect(create.body).not.toContain(fragment);
		expect(await row(id)).toEqual({ short_code: "c1", short_url: "https://rerout.co/c1", short_title: "Groceries for Sunday" });

		// Updating the note keeps its link; a new title retitles it, once.
		expect((await put(a.token, id, (await seal(id, "<p>Eggs</p>")).bytes, { title: "Groceries for Sunday" })).body.short_url).toBe("https://rerout.co/c1");
		expect(rerout.sent).toHaveLength(1);
		expect((await put(a.token, id, bytes, { title: "Shopping" })).body.short_url).toBe("https://rerout.co/c1");
		expect(rerout.sent.slice(1).map((s) => [s.method, s.url, JSON.parse(s.body)])).toEqual([["PATCH", `${REROUT_API}/v1/links/c1`, { seo_title: "Shopping" }]]);
		expect(rerout.links.get("c1")!.seo_title).toBe("Shopping");
		expect((await row(id))!.short_title).toBe("Shopping");
		expect(rerout.live()).toEqual(["c1"]);
	});

	it("previews 'A note' without a usable title", async () => {
		const a = await signUp("alice");
		for (const title of [null, "", "   ", "%E0%A4%A"]) {
			const id = newShareId();
			const headers: Record<string, string> = { "content-type": "application/octet-stream" };
			const { bytes } = await seal(id, "x");
			headers["x-kanso-sha256"] = await sha256Hex(bytes);
			if (title !== null) headers["x-kanso-title"] = title;
			expect((await callJson(`/v2/shares/${id}`, { token: a.token, method: "PUT", body: bytes, headers, env: reroutEnv() })).status).toBe(200);
			expect(JSON.parse(rerout.sent.at(-1)!.body).seo_title, String(title)).toBe(DEFAULT_TITLE);
		}
	});

	it("is off without REROUT_API_KEY, and on an origin that isn't https: the long link only", async () => {
		const a = await signUp("alice");
		const id = newShareId();
		const r = await put(a.token, id, (await seal(id, "x")).bytes, { e: testEnv() });
		expect(r).toMatchObject({ status: 200, body: { short_url: null } });
		// wrangler dev: a key, but Rerout can't redirect to http://127.0.0.1.
		const other = newShareId();
		const { bytes } = await seal(other, "x");
		const local = await handle(
			new Request(`http://127.0.0.1:8787/v2/shares/${other}`, {
				method: "PUT",
				body: bytes,
				headers: { authorization: `Bearer ${a.token}`, "content-type": "application/octet-stream", "x-kanso-sha256": await sha256Hex(bytes) },
			}),
			reroutEnv(),
			START,
		);
		expect(local.status).toBe(200);
		expect(await local.json()).toMatchObject({ short_url: null });
		expect(rerout.sent).toEqual([]);
		expect(await row(id)).toEqual({ short_code: null, short_url: null, short_title: null });
		expect((await unpublish(a.token, id, testEnv())).status).toBe(204);
		expect(await runCron(testEnv(), START + 1)).toMatchObject({ shortLinksPurged: 0 });
		expect(rerout.sent).toEqual([]);
	});

	it("never fails publishing when Rerout does, and the next upload tries again", async () => {
		const a = await signUp("alice");
		const id = newShareId();
		const { bytes } = await seal(id, "x");
		for (const failing of [500, 429, 401, "throw"] as const) {
			rerout.failing = failing;
			const r = await put(a.token, id, bytes);
			expect(r, String(failing)).toMatchObject({ status: 200, body: { ok: true, short_url: null } });
			expect(await row(id)).toMatchObject({ short_code: null });
		}
		rerout.failing = null;
		expect((await put(a.token, id, bytes)).body.short_url).toBe("https://rerout.co/c1");
		// The public route doesn't care either way.
		expect((await call(`/v2/public/${id}`, { env: reroutEnv() })).status).toBe(200);
	});

	it("deletes a link it can't use: another project's, a different target, or a short URL with a fragment or query", async () => {
		const a = await signUp("alice");
		const tampers: ((link: Record<string, unknown>) => void)[] = [
			(l) => (l.project_id = "prj_someone_else"),
			(l) => (l.target_url = "https://example.com/"),
			(l) => (l.short_url = `${l.short_url}#x`),
			(l) => (l.short_url = `${l.short_url}?x=1`),
			(l) => (l.short_url = "http://rerout.co/plain"),
		];
		for (const tamper of tampers) {
			rerout.tamper = tamper;
			const id = newShareId();
			expect((await put(a.token, id, (await seal(id, "x")).bytes)).body.short_url).toBeNull();
			expect(await row(id)).toMatchObject({ short_code: null });
		}
		expect(rerout.live()).toEqual([]);
	});

	it("keeps one link when two uploads race, and deletes the loser's", async () => {
		const a = await signUp("alice");
		const id = newShareId();
		const { bytes } = await seal(id, "x");
		expect((await put(a.token, id, bytes, { e: testEnv() })).status).toBe(200);
		// While this upload's create is in flight, another upload stores its link first.
		rerout.duringCreate = async () => {
			rerout.duringCreate = null;
			await env.DB.prepare("UPDATE shares SET short_code = 'w1', short_url = 'https://rerout.co/w1', short_title = 'Groceries' WHERE id = ?1")
				.bind(id)
				.run();
		};
		expect((await put(a.token, id, bytes)).body.short_url).toBe("https://rerout.co/w1");
		expect(rerout.live()).toEqual([]);
		expect(await queued()).toEqual([]);
	});
});

describe("deleting a short link", () => {
	it("deletes it at once on unpublish, and on the next cron when Rerout was down", async () => {
		const a = await signUp("alice");
		const [first, second] = [newShareId(), newShareId()];
		await put(a.token, first, (await seal(first, "x")).bytes);
		await put(a.token, second, (await seal(second, "y")).bytes);
		expect(rerout.live()).toEqual(["c1", "c2"]);
		expect((await unpublish(a.token, first)).status).toBe(204);
		expect(rerout.sent.at(-1)).toMatchObject({ method: "DELETE", url: `${REROUT_API}/v1/links/c1` });
		expect(rerout.live()).toEqual(["c2"]);
		expect(await queued()).toEqual([]);

		rerout.failing = 503;
		expect((await unpublish(a.token, second)).status).toBe(204);
		expect(await queued()).toEqual([{ code: "c2", attempts: 0 }]);
		expect(await runCron(reroutEnv(), START + 1)).toMatchObject({ shortLinksPurged: 0 });
		expect(await queued()).toEqual([{ code: "c2", attempts: 1 }]);
		rerout.failing = null;
		expect(await runCron(reroutEnv(), START + 2)).toMatchObject({ shortLinksPurged: 1 });
		expect(rerout.live()).toEqual([]);
		expect(await queued()).toEqual([]);
	});

	it("deletes the links of expired shares, taken-down shares and deleted accounts on the cron", async () => {
		const a = await signUp("alice");
		const b = await signUp("bob");
		const [expiring, takenDown, bobs] = [newShareId(), newShareId(), newShareId()];
		await put(a.token, expiring, (await seal(expiring, "x")).bytes, { expiresAt: START + DAY });
		await put(a.token, takenDown, (await seal(takenDown, "y")).bytes);
		await put(b.token, bobs, (await seal(bobs, "z")).bytes);
		expect(rerout.live()).toEqual(["c1", "c2", "c3"]);
		expect((await runAdmin(["unpublish", takenDown])).ok).toBe(true);
		expect(
			(await callJson("/v2/account/delete", { token: b.token, json: { auth_key: b.creds.authKey, username: "bob" }, env: reroutEnv() })).status,
		).toBe(200);
		const report = await runCron(reroutEnv(), START + DAY + 1);
		expect(report).toMatchObject({ sharesExpired: 1, shortLinksPurged: 3 });
		expect(rerout.live()).toEqual([]);
		expect(await queued()).toEqual([]);
	});

	it("counts a link Rerout no longer has as deleted, and keeps retrying the ones it refuses", async () => {
		await env.DB.prepare("INSERT INTO short_link_purges (code, requested_at) VALUES ('gone', 1), ('stuck', 2)").run();
		rerout.links.set("stuck", { target_url: "", seo_title: null, deleted: false });
		const original = rerout.fetch.bind(rerout);
		rerout.fetch = async (input, init) => (String(input).endsWith("/stuck") ? Response.json({ code: "forbidden" }, { status: 403 }) : original(input, init));
		expect(await runCron(reroutEnv(), START)).toMatchObject({ shortLinksPurged: 1 });
		expect(await queued()).toEqual([{ code: "stuck", attempts: 1 }]);
		// Without a key the queue waits.
		expect(await runCron(testEnv(), START + 1)).toMatchObject({ shortLinksPurged: 0 });
		expect(await queued()).toEqual([{ code: "stuck", attempts: 1 }]);
	});

	it("never sends a share's key anywhere, through publish, update, unpublish and the cron", async () => {
		const a = await signUp("alice");
		const fragments: string[] = [];
		await fc.assert(
			fc.asyncProperty(fc.string({ maxLength: 120 }), fc.boolean(), async (title, drop) => {
				const id = newShareId();
				const sealed = await seal(id, `<h1>${title}</h1>`);
				fragments.push(sealed.fragment);
				await put(a.token, id, sealed.bytes, { title });
				await put(a.token, id, (await seal(id, "again")).bytes, { title: `${title}!` });
				if (drop) await unpublish(a.token, id);
			}),
			runs(25),
		);
		await runCron(reroutEnv(), START + 1);
		expect(rerout.sent.length).toBeGreaterThan(0);
		for (const sent of rerout.sent) {
			expect(sent.url.startsWith(`${REROUT_API}/v1/links`)).toBe(true);
			expect(sent.url).not.toContain("#");
			if (sent.body !== "") expect(JSON.parse(sent.body).target_url ?? "").not.toContain("#");
			for (const fragment of fragments) expect(sent.url + sent.body).not.toContain(fragment);
		}
	});
});

describe("titles", () => {
	const codePoints = (s: string) => Array.from(s).length;

	it("examples", () => {
		expect(previewTitle("")).toBe(DEFAULT_TITLE);
		expect(previewTitle("\t\u200b\n")).toBe(DEFAULT_TITLE);
		expect(previewTitle(" Weekly\u0000plan ")).toBe("Weekly plan");
		expect(previewTitle("a".repeat(90))).toBe("a".repeat(90));
		expect(previewTitle("a".repeat(91))).toBe(`${"a".repeat(89)}…`);
		expect(previewTitle("😀".repeat(100))).toBe(`${"😀".repeat(89)}…`);
		expect(titleHeader(null)).toBe(DEFAULT_TITLE);
		expect(titleHeader("%")).toBe(DEFAULT_TITLE);
		expect(titleHeader(encodeURIComponent("Café ☕"))).toBe("Café ☕");
		expect(titleHeader("x".repeat(4097))).toBe(DEFAULT_TITLE);
		expect(isShortUrl("https://rerout.co/abc")).toBe(true);
		for (const bad of ["http://rerout.co/abc", "https://rerout.co/a#b", "https://rerout.co/a?b", "https://u:p@rerout.co/a", "https:// x", "rerout.co/abc", 7, null]) {
			expect(isShortUrl(bad), String(bad)).toBe(false);
		}
	});

	it("property: a preview title is 1 to 90 characters, single-spaced, without controls, and stable", () => {
		fc.assert(
			fc.property(fc.string({ unit: "grapheme", maxLength: 200 }), (text) => {
				const title = previewTitle(text);
				expect(codePoints(title)).toBeGreaterThanOrEqual(1);
				expect(codePoints(title)).toBeLessThanOrEqual(MAX_TITLE_CHARS);
				expect(title).toBe(title.trim());
				expect(title).not.toMatch(/\s\s|[\u0000-\u001f\u007f-\u009f]/);
				expect(previewTitle(title)).toBe(title);
				// The header carries any title intact.
				expect(titleHeader(encodeURIComponent(text))).toBe(title);
			}),
			runs(500),
		);
	});
});
