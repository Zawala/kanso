// Published notes (docs/ACCOUNTS.md › Published notes): only the owner writes
// or deletes a share, accounts can't touch each other's, only sealed bytes are
// stored and served, size and quota caps hold, expiry and unpublishing stop
// the link, account deletion takes the shares with it, public reads are
// rate-limited, and the viewer page's CSP pins its own inline code.

import { env } from "cloudflare:workers";
import fc from "fast-check";
import { beforeEach, describe, expect, it } from "vitest";
import { runCron } from "../src/index";
import { reconcile } from "../src/purge";
import { MAX_SEALED_SHARE, shareKey } from "../src/shares";
import { VIEWER_HTML, VIEWER_SCRIPT, VIEWER_STYLE } from "../src/viewer";
import fixture from "./fixtures/share.json";
import { START, accountRow, base64url, call, callJson, client, r2Keys, recomputed, resetServer, runAdmin, signUp, testEnv } from "./helpers";

const HOUR = 3_600_000;
const DAY = 86_400_000;
const runs = (n: number) => ({ numRuns: Math.min(n, Number(env.FAST_CHECK_RUNS)) });

const random = (n: number): Uint8Array => crypto.getRandomValues(new Uint8Array(n));
const newShareId = (): string => base64url(random(16));
const fromHex = (hex: string): Uint8Array => Uint8Array.from(hex.match(/../g)!.map((h) => parseInt(h, 16)));
const fromB64url = (text: string): Uint8Array => Uint8Array.from(atob(text.replace(/-/g, "+").replace(/_/g, "/")), (c) => c.charCodeAt(0));

/** What the device uploads: 0x01, a nonce, and AES-256-GCM over `html` with the share's AAD. */
async function seal(id: string, html: string, key = random(32)): Promise<{ bytes: Uint8Array; key: Uint8Array }> {
	const nonce = random(12);
	const aad = new Uint8Array([...new TextEncoder().encode("kanso-share"), 1, ...new TextEncoder().encode(id)]);
	const k = await crypto.subtle.importKey("raw", key, { name: "AES-GCM" }, false, ["encrypt"]);
	const ct = new Uint8Array(await crypto.subtle.encrypt({ name: "AES-GCM", iv: nonce, additionalData: aad }, k, new TextEncoder().encode(html)));
	return { bytes: new Uint8Array([1, ...nonce, ...ct]), key };
}

/** Opens sealed bytes as the viewer page does. */
async function open(id: string, bytes: Uint8Array, key: Uint8Array): Promise<string> {
	const aad = new TextEncoder().encode(`kanso-share\u0001${id}`);
	const k = await crypto.subtle.importKey("raw", key, { name: "AES-GCM" }, false, ["decrypt"]);
	const plain = await crypto.subtle.decrypt({ name: "AES-GCM", iv: bytes.slice(1, 13), additionalData: aad, tagLength: 128 }, k, bytes.slice(13));
	return new TextDecoder().decode(plain);
}

async function publicGet(id: string, init: { ip?: string; now?: number; env?: ReturnType<typeof testEnv>; headers?: Record<string, string> } = {}) {
	const response = await call(`/v2/public/${id}`, { ip: init.ip ?? "203.0.113.7", now: init.now, env: init.env, headers: init.headers });
	return { status: response.status, headers: response.headers, bytes: new Uint8Array(await response.arrayBuffer()) };
}

const shareRows = async () => (await env.DB.prepare("SELECT * FROM shares ORDER BY id").all()).results;
const purgeRows = async () => (await env.DB.prepare("SELECT id FROM share_purges ORDER BY id").all<{ id: string }>()).results.map((r) => r.id);
const retiredRows = async () => (await env.DB.prepare("SELECT id FROM retired_shares ORDER BY id").all<{ id: string }>()).results.map((r) => r.id);

beforeEach(() => resetServer());

describe("publishing", () => {
	it("stores exactly the sealed bytes at shares/<id>, serves them publicly, and counts them as a file", async () => {
		const a = await signUp("alice");
		const alice = client(a.token);
		const id = newShareId();
		const { bytes, key } = await seal(id, "<p>Hello</p>");
		const put = await alice.putShare(id, bytes);
		expect(put).toEqual({ status: 200, body: { ok: true, id, size: bytes.length, expires_at: null } });
		expect(await r2Keys()).toEqual([shareKey(id)]);
		const stored = await env.FILES.get(shareKey(id));
		expect(new Uint8Array(await stored!.arrayBuffer())).toEqual(bytes);
		const got = await publicGet(id);
		expect(got.status).toBe(200);
		expect(got.bytes).toEqual(bytes);
		expect(got.headers.get("content-type")).toBe("application/octet-stream");
		expect(got.headers.get("cache-control")).toBe("no-store");
		expect(got.headers.get("referrer-policy")).toBe("no-referrer");
		expect(got.headers.get("access-control-allow-origin")).toBeNull();
		expect(await open(id, got.bytes, key)).toBe("<p>Hello</p>");
		const row = await accountRow(a.creds.accountId);
		expect({ files: row!.files, file_bytes: row!.file_bytes }).toEqual({ files: 1, file_bytes: bytes.length + 1024 });
		expect(await recomputed(a.creds.accountId)).toMatchObject({ files: 1, file_bytes: bytes.length + 1024 });
	});

	it("replaces the account's own share in place, with the same ID", async () => {
		const a = await signUp("alice");
		const alice = client(a.token);
		const id = newShareId();
		const key = random(32);
		const first = await seal(id, "<p>One</p>", key);
		const second = await seal(id, "<p>Two, longer</p>", key);
		expect((await alice.putShare(id, first.bytes)).status).toBe(200);
		expect((await alice.putShare(id, second.bytes)).status).toBe(200);
		expect(await open(id, (await publicGet(id)).bytes, key)).toBe("<p>Two, longer</p>");
		const row = await accountRow(a.creds.accountId);
		expect({ files: row!.files, file_bytes: row!.file_bytes }).toEqual({ files: 1, file_bytes: second.bytes.length + 1024 });
		expect(await r2Keys()).toEqual([shareKey(id)]);
	});

	it("stores only ciphertext: no plaintext, wrong format byte, short body or bad checksum is accepted", async () => {
		const a = await signUp("alice");
		const alice = client(a.token);
		const id = newShareId();
		const html = new TextEncoder().encode("<!doctype html><title>Plain</title><p>not sealed</p>");
		expect(await alice.putShare(id, html)).toMatchObject({ status: 400, body: { error: "bad_request" } });
		expect(await alice.putShare(id, new Uint8Array([2, ...random(40)]))).toMatchObject({ status: 400 });
		expect(await alice.putShare(id, new Uint8Array([1, ...random(27)]))).toMatchObject({ status: 400 });
		const { bytes } = await seal(id, "x");
		expect(await alice.putShare(id, bytes, { sha: "0".repeat(64) })).toMatchObject({ status: 400, body: { error: "checksum_mismatch" } });
		expect((await call(`/v2/shares/${id}`, { token: a.token, method: "PUT", body: bytes, headers: { "content-type": "text/html" } })).status).toBe(415);
		expect(await shareRows()).toEqual([]);
		expect(await r2Keys()).toEqual([]);
		// The row holds no content, only who, how big, the ciphertext's checksum and until when.
		expect((await alice.putShare(id, bytes)).status).toBe(200);
		const row = (await shareRows())[0];
		expect(Object.keys(row).sort()).toEqual(["account_id", "created_at", "expires_at", "id", "sha256", "size", "updated_at"]);
		const digest = new Uint8Array(await crypto.subtle.digest("SHA-256", bytes));
		expect(row.sha256).toBe([...digest].map((b) => b.toString(16).padStart(2, "0")).join(""));
	});

	it("refuses share IDs that aren't 16 bytes of canonical base64url", async () => {
		const a = await signUp("alice");
		const alice = client(a.token);
		const { bytes } = await seal("x", "x");
		for (const id of ["short", "A".repeat(21), "A".repeat(23), "AAAAAAAAAAAAAAAAAAAAAB", "AAAAAAAAAAAAAAAAAAAA+A"]) {
			expect((await alice.putShare(encodeURIComponent(id), bytes)).status, id).toBe(400);
		}
	});

	it("caps a sealed note at 10 MiB plus the overhead, and charges it to the account's quotas", async () => {
		const a = await signUp("alice");
		const alice = client(a.token);
		const big = new Uint8Array(MAX_SEALED_SHARE + 1);
		big[0] = 1;
		expect(await alice.putShare(newShareId(), big)).toMatchObject({ status: 413, body: { error: "too_large" } });
		const max = big.subarray(0, MAX_SEALED_SHARE);
		expect((await alice.putShare(newShareId(), max)).status).toBe(200);
		await runAdmin(["quota", "alice", "--files", "1"]);
		const { bytes } = await seal("y", "y");
		expect(await alice.putShare(newShareId(), bytes)).toMatchObject({ status: 413, body: { error: "quota_exceeded", quota: "files", used: 1, limit: 1 } });
		expect((await shareRows()).length).toBe(1);
	});

	it("needs a session to write or delete", async () => {
		const a = await signUp("alice");
		const id = newShareId();
		const { bytes } = await seal(id, "x");
		expect((await client(a.token).putShare(id, bytes)).status).toBe(200);
		const anonymous = await call(`/v2/shares/${id}`, { method: "PUT", body: bytes, headers: { "content-type": "application/octet-stream" } });
		expect(anonymous.status).toBe(401);
		expect((await call(`/v2/shares/${id}`, { method: "DELETE" })).status).toBe(401);
		// Public reads are read-only.
		expect((await call(`/v2/public/${id}`, { method: "PUT", body: bytes })).status).toBe(405);
		expect((await call(`/v2/public/${id}`, { method: "DELETE" })).status).toBe(405);
	});

	it("keeps browsers off the owner's routes but lets the viewer page read", async () => {
		const a = await signUp("alice");
		const id = newShareId();
		const { bytes } = await seal(id, "x");
		const origin = { origin: "https://evil.example" };
		const put = await call(`/v2/shares/${id}`, {
			token: a.token,
			method: "PUT",
			body: bytes,
			headers: { ...origin, "content-type": "application/octet-stream" },
		});
		expect(put.status).toBe(403);
		expect((await client(a.token).putShare(id, bytes)).status).toBe(200);
		expect((await publicGet(id, { headers: origin })).status).toBe(200);
		expect((await call(`/p/${id}`, { headers: origin })).status).toBe(200);
	});
});

describe("isolation between accounts", () => {
	it("refuses another account's writes and deletes, and leaves the share as its owner left it", async () => {
		const a = await signUp("alice");
		const b = await signUp("bob");
		const id = newShareId();
		const mine = await seal(id, "<p>Alice's</p>");
		expect((await client(a.token).putShare(id, mine.bytes)).status).toBe(200);
		const theirs = await seal(id, "<p>Bob's</p>");
		expect(await client(b.token).putShare(id, theirs.bytes)).toMatchObject({ status: 409, body: { error: "share_id_taken" } });
		expect((await client(b.token).deleteShare(id)).status).toBe(404);
		expect(await open(id, (await publicGet(id)).bytes, mine.key)).toBe("<p>Alice's</p>");
		expect((await accountRow(b.creds.accountId))!.files).toBe(0);
		expect((await accountRow(a.creds.accountId))!.files).toBe(1);
	});

	it("property: whatever two accounts do to a few IDs, each share is exactly its owner's last upload", async () => {
		const ids = [newShareId(), newShareId(), newShareId()];
		const op = fc.record({ who: fc.integer({ min: 0, max: 1 }), id: fc.integer({ min: 0, max: 2 }), kind: fc.constantFrom("put", "put", "delete") });
		await fc.assert(
			fc.asyncProperty(fc.array(op, { maxLength: 12 }), async (ops) => {
				await resetServer();
				const accounts = [await signUp("alice"), await signUp("bob")];
				const clients = accounts.map((x) => client(x.token));
				/** The model: who owns each ID, its bytes, and IDs no longer usable (unpublished). */
				const owner = new Map<number, { who: number; bytes: Uint8Array }>();
				const retired = new Set<number>();
				for (const [n, o] of ops.entries()) {
					const id = ids[o.id];
					if (o.kind === "put") {
						const { bytes } = await seal(id, `<p>${n}</p>`);
						const r = await clients[o.who].putShare(id, bytes);
						const current = owner.get(o.id);
						const allowed = !retired.has(o.id) && (current === undefined || current.who === o.who);
						expect(r.status).toBe(allowed ? 200 : 409);
						if (allowed) owner.set(o.id, { who: o.who, bytes });
					} else {
						const r = await clients[o.who].deleteShare(id);
						const current = owner.get(o.id);
						const allowed = current !== undefined && current.who === o.who;
						expect(r.status).toBe(allowed ? 204 : 404);
						if (allowed) {
							owner.delete(o.id);
							retired.add(o.id);
						}
					}
				}
				for (const [i, id] of ids.entries()) {
					const got = await publicGet(id);
					const want = owner.get(i);
					expect(got.status).toBe(want ? 200 : 404);
					if (want) expect(got.bytes).toEqual(want.bytes);
				}
				for (const [who, x] of accounts.entries()) {
					const row = await accountRow(x.creds.accountId);
					const owned = [...owner.values()].filter((v) => v.who === who);
					expect(row!.files).toBe(owned.length);
					expect(row!.file_bytes).toBe(owned.reduce((sum, v) => sum + v.bytes.length + 1024, 0));
				}
			}),
			runs(25),
		);
	});
});

describe("unpublishing and expiry", () => {
	it("unpublishing answers 404 at once, deletes the object, refunds the quota, and retires the ID for good", async () => {
		const a = await signUp("alice");
		const alice = client(a.token);
		const id = newShareId();
		const { bytes } = await seal(id, "x");
		await alice.putShare(id, bytes);
		expect((await alice.deleteShare(id)).status).toBe(204);
		expect((await publicGet(id)).status).toBe(404);
		expect(await r2Keys()).toEqual([]);
		expect((await accountRow(a.creds.accountId))!).toMatchObject({ files: 0, file_bytes: 0 });
		expect(await purgeRows()).toEqual([id]);
		expect(await retiredRows()).toEqual([id]);
		expect((await alice.deleteShare(id)).status).toBe(404);
		expect(await alice.putShare(id, bytes)).toMatchObject({ status: 409, body: { error: "share_retired" } });
		await runCron(testEnv(), START + HOUR);
		expect(await purgeRows()).toEqual([]);
		// The purge is done, but the ID stays retired.
		expect(await alice.putShare(id, bytes)).toMatchObject({ status: 409, body: { error: "share_retired" } });
		expect(await r2Keys()).toEqual([]);
	});

	it("nobody can take over an old link once it's purged: unpublished, expired, or of a deleted account", async () => {
		const a = await signUp("alice");
		const alice = client(a.token);
		const [unpublished, expired, deleted] = [newShareId(), newShareId(), newShareId()];
		await alice.putShare(unpublished, (await seal(unpublished, "a")).bytes);
		await alice.putShare(expired, (await seal(expired, "b")).bytes, { expiresAt: START + DAY });
		await alice.deleteShare(unpublished);
		await runCron(testEnv(), START + DAY + 1);
		const b = await signUp("bob");
		await client(b.token).putShare(deleted, (await seal(deleted, "c")).bytes);
		await callJson("/v2/account/delete", { token: b.token, json: { auth_key: b.creds.authKey, username: "bob" } });
		// Days later every purge is long done; a newcomer holding the old links tries to reuse them.
		await runCron(testEnv(), START + 3 * DAY);
		await runCron(testEnv(), START + 4 * DAY);
		expect(await purgeRows()).toEqual([]);
		const m = await signUp("mallory");
		const mallory = client(m.token);
		for (const id of [unpublished, expired, deleted]) {
			const planted = await seal(id, "<p>Planted</p>");
			expect(await mallory.putShare(id, planted.bytes), id).toMatchObject({ status: 409, body: { error: "share_retired" } });
			expect(await alice.putShare(id, planted.bytes), id).toMatchObject({ status: 409 });
			expect((await publicGet(id, { now: START + 4 * DAY })).status, id).toBe(404);
		}
		expect(await shareRows()).toEqual([]);
		expect(await r2Keys()).toEqual([]);
		expect((await accountRow(m.creds.accountId))!).toMatchObject({ files: 0, file_bytes: 0 });
	});

	it("serves until the expiry, then 410, and the cron unpublishes it", async () => {
		const a = await signUp("alice");
		const alice = client(a.token);
		const id = newShareId();
		const { bytes } = await seal(id, "x");
		const expiresAt = START + DAY;
		expect(await alice.putShare(id, bytes, { expiresAt })).toMatchObject({ status: 200, body: { expires_at: expiresAt } });
		expect((await publicGet(id, { now: expiresAt - 1 })).status).toBe(200);
		const expired = await publicGet(id, { now: expiresAt });
		expect(expired.status).toBe(410);
		expect(expired.headers.get("referrer-policy")).toBe("no-referrer");
		const report = await runCron(testEnv(), expiresAt + 1);
		expect(report).toMatchObject({ sharesExpired: 1, sharesPurged: 1 });
		expect(await shareRows()).toEqual([]);
		expect(await r2Keys()).toEqual([]);
		expect((await accountRow(a.creds.accountId))!).toMatchObject({ files: 0, file_bytes: 0 });
		expect((await publicGet(id, { now: expiresAt + 2 })).status).toBe(404);
	});

	it("an update moves the expiry, or removes it", async () => {
		const a = await signUp("alice");
		const alice = client(a.token);
		const id = newShareId();
		const { bytes } = await seal(id, "x");
		await alice.putShare(id, bytes, { expiresAt: START + HOUR });
		await alice.putShare(id, bytes);
		await runCron(testEnv(), START + 2 * HOUR);
		expect((await publicGet(id, { now: START + 2 * HOUR })).status).toBe(200);
	});

	it("refuses an expiry in the past, too far ahead, or not a number", async () => {
		const a = await signUp("alice");
		const alice = client(a.token);
		const id = newShareId();
		const { bytes } = await seal(id, "x");
		for (const expiresAt of [START, START - 1, START + 367 * DAY]) {
			expect((await alice.putShare(id, bytes, { expiresAt })).status, String(expiresAt)).toBe(400);
		}
		const r = await call(`/v2/shares/${id}`, {
			token: a.token,
			method: "PUT",
			body: bytes,
			headers: { "content-type": "application/octet-stream", "x-kanso-sha256": "0".repeat(64), "x-kanso-expires-at": "tomorrow" },
		});
		expect(r.status).toBe(400);
		expect(await shareRows()).toEqual([]);
	});
});

describe("account deletion", () => {
	it("deleting the account unpublishes every share and deletes their objects", async () => {
		const a = await signUp("alice");
		const b = await signUp("bob");
		const ids = [newShareId(), newShareId()];
		for (const id of ids) await client(a.token).putShare(id, (await seal(id, "a")).bytes);
		const bobs = newShareId();
		await client(b.token).putShare(bobs, (await seal(bobs, "b")).bytes);
		const deleted = await callJson("/v2/account/delete", { token: a.token, json: { auth_key: a.creds.authKey, username: "alice" } });
		expect(deleted.status).toBe(200);
		for (const id of ids) expect((await publicGet(id)).status).toBe(404);
		expect(await r2Keys()).toEqual([shareKey(bobs)]);
		expect((await shareRows()).map((r) => r.id)).toEqual([bobs]);
		expect((await publicGet(bobs)).status).toBe(200);
	});

	it("the admin's delete does the same, and the cron empties R2", async () => {
		const a = await signUp("alice");
		const id = newShareId();
		await client(a.token).putShare(id, (await seal(id, "a")).bytes);
		expect((await runAdmin(["delete", "alice", "--confirm", "alice"])).ok).toBe(true);
		expect((await publicGet(id)).status).toBe(404);
		expect(await purgeRows()).toEqual([id]);
		await runCron(testEnv(), START + HOUR);
		expect(await r2Keys()).toEqual([]);
		expect(await purgeRows()).toEqual([]);
	});

	it("recount counts shares as files", async () => {
		const a = await signUp("alice");
		const id = newShareId();
		const { bytes } = await seal(id, "a");
		await client(a.token).putShare(id, bytes);
		await env.DB.prepare("UPDATE accounts SET files = 0, file_bytes = 0, recounted_at = 1 WHERE id = ?1").bind(a.creds.accountId).run();
		await runAdmin(["recount", "alice"]);
		expect((await accountRow(a.creds.accountId))!).toMatchObject({ files: 1, file_bytes: bytes.length + 1024 });
	});
});

describe("rate limits", () => {
	it("a public read never writes to D1, and a malformed ID never reaches it", async () => {
		const a = await signUp("alice");
		const id = newShareId();
		await client(a.token).putShare(id, (await seal(id, "x")).bytes);
		const statements: string[] = [];
		const recording = new Proxy(env.DB, {
			get(target, name) {
				if (name === "prepare") {
					return (sql: string) => {
						statements.push(sql);
						return target.prepare(sql);
					};
				}
				if (name === "batch" || name === "exec") throw new Error(`unexpected ${String(name)}`);
				const value = Reflect.get(target, name);
				return typeof value === "function" ? value.bind(target) : value;
			},
		});
		const watched = testEnv({ DB: recording });
		const before = await env.DB.prepare("SELECT (SELECT count(*) FROM limits) AS limits, (SELECT count(*) FROM daily) AS daily").first();
		for (let i = 0; i < 100; i++) expect((await publicGet(i % 2 ? id : newShareId(), { env: watched, ip: "198.51.100.1" })).status).toBe(i % 2 ? 200 : 404);
		expect(statements.length).toBeGreaterThan(0);
		for (const sql of statements) expect(sql.trimStart()).toMatch(/^SELECT\b/);
		statements.length = 0;
		for (const bad of ["not-a-share", "AAAAAAAAAAAAAAAAAAAAAB", "x".repeat(200)]) {
			expect((await publicGet(bad, { env: watched })).status, bad).toBe(404);
		}
		expect(statements).toEqual([]);
		expect(await env.DB.prepare("SELECT (SELECT count(*) FROM limits) AS limits, (SELECT count(*) FROM daily) AS daily").first()).toEqual(before);
	});

	it("the PUBLIC_LIMITER binding refuses public reads and the viewer page before D1", async () => {
		const a = await signUp("alice");
		const id = newShareId();
		await client(a.token).putShare(id, (await seal(id, "x")).bytes);
		const seen: string[] = [];
		const refusing = testEnv({
			PUBLIC_LIMITER: {
				async limit({ key }) {
					seen.push(key);
					return { success: false };
				},
			},
		});
		const before = (await env.DB.prepare("SELECT * FROM limits ORDER BY key").all()).results;
		expect((await publicGet(id, { env: refusing, ip: "2001:db8:abcd:12ff::1" })).status).toBe(429);
		expect((await call(`/p/${id}`, { env: refusing, ip: "2001:db8:abcd:12ff::1" })).status).toBe(429);
		expect(new Set(seen)).toEqual(new Set(["2001:db8:abcd:0:0:0:0:0/48"]));
		expect((await env.DB.prepare("SELECT * FROM limits ORDER BY key").all()).results).toEqual(before);
	});
});

/** A share ID every run's slice of `shares/` covers: each starts at some first character and runs on to `z`. */
const lastShareId = (): string => `z${newShareId().slice(1)}`;

describe("racing uploads", () => {
	it("serves nothing while R2's object isn't the one the row describes, and reconciliation adopts it after an hour", async () => {
		const a = await signUp("alice");
		const id = lastShareId();
		const first = await seal(id, "<p>First</p>");
		const second = await seal(id, "<p>The second upload, which is longer</p>");
		expect((await client(a.token).putShare(id, first.bytes)).status).toBe(200);
		// The other upload's R2 write lands last: the row says `first`, R2 holds `second`.
		const digest = new Uint8Array(await crypto.subtle.digest("SHA-256", second.bytes));
		await env.FILES.put(shareKey(id), second.bytes, { sha256: digest });
		expect((await publicGet(id)).status).toBe(404);
		// An object R2 never checked isn't served either.
		await env.FILES.put(shareKey(id), first.bytes);
		expect((await publicGet(id)).status).toBe(404);
		await env.FILES.put(shareKey(id), second.bytes, { sha256: digest });
		// Within the hour reconciliation leaves it (an upload may still be landing).
		expect((await reconcile(testEnv(), START + HOUR / 2)).sharesAdopted).toBe(0);
		const later = START + 2 * HOUR;
		expect(await reconcile(testEnv(), later)).toMatchObject({ sharesAdopted: 1, sharesDropped: 0 });
		expect(await reconcile(testEnv(), later + 15 * 60_000)).toMatchObject({ sharesAdopted: 0, sharesDropped: 0 });
		const got = await publicGet(id, { now: later });
		expect(got.status).toBe(200);
		expect(await open(id, got.bytes, second.key)).toBe("<p>The second upload, which is longer</p>");
		expect((await accountRow(a.creds.accountId))!).toMatchObject({ files: 1, file_bytes: second.bytes.length + 1024 });
		expect(await recomputed(a.creds.accountId)).toMatchObject({ files: 1, file_bytes: second.bytes.length + 1024 });
	});

	it("reconciliation deletes share objects with no row, once they're an hour old", async () => {
		const id = lastShareId();
		await env.FILES.put(shareKey(id), (await seal(id, "x")).bytes);
		expect((await reconcile(testEnv(), Date.now() + HOUR / 2)).orphansDeleted).toBe(0);
		expect(await r2Keys()).toEqual([shareKey(id)]);
		expect((await reconcile(testEnv(), Date.now() + 2 * HOUR)).orphansDeleted).toBe(1);
		expect(await r2Keys()).toEqual([]);
	});
});

describe("takedowns", () => {
	it("a disabled account's shares answer 404 until it is enabled again", async () => {
		const a = await signUp("alice");
		const id = newShareId();
		await client(a.token).putShare(id, (await seal(id, "x")).bytes);
		expect((await runAdmin(["disable", "alice"])).ok).toBe(true);
		expect((await publicGet(id)).status).toBe(404);
		expect((await runAdmin(["enable", "alice"])).ok).toBe(true);
		expect((await publicGet(id)).status).toBe(200);
	});

	it("admin shares lists an account's published notes, and admin unpublish takes one down for good", async () => {
		const a = await signUp("alice");
		const alice = client(a.token);
		const [kept, reported] = [newShareId(), newShareId()];
		await alice.putShare(kept, (await seal(kept, "kept")).bytes);
		const { bytes } = await seal(reported, "reported");
		await alice.putShare(reported, bytes, { expiresAt: START + DAY });
		const listed = await runAdmin(["shares", "alice"]);
		expect(listed.ok).toBe(true);
		expect(listed.message).toContain(kept);
		expect(listed.message).toContain(reported);
		expect((await runAdmin(["shares", "nobody"])).ok).toBe(false);

		const done = await runAdmin(["unpublish", reported]);
		expect(done).toMatchObject({ ok: true });
		expect(done.message).toContain("alice");
		expect((await publicGet(reported)).status).toBe(404);
		expect((await publicGet(kept)).status).toBe(200);
		expect(await purgeRows()).toEqual([reported]);
		expect(await retiredRows()).toEqual([reported]);
		expect((await accountRow(a.creds.accountId))!).toMatchObject({ files: 1 });
		expect(await recomputed(a.creds.accountId)).toMatchObject({ files: 1 });
		expect((await accountRow(a.creds.accountId))!.file_bytes).toBe((await recomputed(a.creds.accountId)).file_bytes);
		// Neither the owner nor anyone else can bring it back.
		expect(await alice.putShare(reported, bytes)).toMatchObject({ status: 409, body: { error: "share_retired" } });
		expect((await runAdmin(["unpublish", reported])).ok).toBe(false);
		await expect(runAdmin(["unpublish", "x'; DROP TABLE shares; --"])).rejects.toThrow(/not a share ID/);
		await runCron(testEnv(), START + HOUR);
		expect(await r2Keys()).toEqual([shareKey(kept)]);
	});
});

describe("the viewer page", () => {
	const sha256 = async (text: string) => {
		const digest = new Uint8Array(await crypto.subtle.digest("SHA-256", new TextEncoder().encode(text)));
		return btoa(String.fromCharCode(...digest));
	};

	it("pins its inline script and style in the CSP by hash, and loads nothing else", async () => {
		const id = newShareId();
		const r = await call(`/p/${id}`);
		expect(r.status).toBe(200);
		expect(r.headers.get("content-type")).toBe("text/html; charset=utf-8");
		const html = await r.text();
		expect(html).toBe(VIEWER_HTML);
		const csp = r.headers.get("content-security-policy")!;
		expect(csp).toBeTruthy();
		const directives = Object.fromEntries(csp.split(";").map((d) => d.trim().split(/\s+/)).map(([k, ...v]) => [k, v]));
		// The hashes are of the page's inline code exactly as served.
		const scripts = [...html.matchAll(/<script>([\s\S]*?)<\/script>/g)].map((m) => m[1]);
		const styles = [...html.matchAll(/<style>([\s\S]*?)<\/style>/g)].map((m) => m[1]);
		expect(scripts).toEqual([VIEWER_SCRIPT]);
		expect(styles).toEqual([VIEWER_STYLE]);
		expect(directives["script-src"]).toEqual([`'sha256-${await sha256(scripts[0])}'`]);
		expect(directives["style-src"]).toEqual([`'sha256-${await sha256(styles[0])}'`, "data:"]);
		expect(directives["default-src"]).toEqual(["'none'"]);
		expect(directives["connect-src"]).toEqual(["'self'"]);
		expect(directives["img-src"]).toEqual(["data:"]);
		expect(directives["frame-ancestors"]).toEqual(["'none'"]);
		expect(directives["base-uri"]).toEqual(["'none'"]);
		// The note goes in by srcdoc, which frame-src doesn't govern; nothing else may load in a frame.
		expect(directives["frame-src"]).toEqual(["'none'"]);
		// The note's own markup loses scripts, styles and templates (declarative shadow roots too).
		expect(/querySelectorAll\("([^"]*)"\)\.forEach\(function\(el\)\{el\.remove/.exec(VIEWER_SCRIPT)![1].split(",")).toEqual(
			expect.arrayContaining(["script", "style", "template", "iframe", "object", "embed", "base", "form"]),
		);
		expect(csp).not.toMatch(/unsafe-inline|unsafe-eval|https?:|\*/);
		expect(r.headers.get("referrer-policy")).toBe("no-referrer");
		expect(r.headers.get("x-content-type-options")).toBe("nosniff");
		// No third-party code: no external src or href at all.
		const markup = html.replace(/<script>[\s\S]*?<\/script>/, "<script></script>");
		expect(markup).not.toMatch(/\s(src|href|srcset|action)=/);
		expect(markup).not.toMatch(/<(link|img|object|embed)\b/);
		expect(html).not.toMatch(/https?:\/\//);
		// The frame can't run scripts, and has no origin of its own.
		const sandbox = /<iframe[^>]*sandbox="([^"]*)"/.exec(html)![1].split(" ");
		expect(sandbox).not.toContain("allow-scripts");
		expect(sandbox).not.toContain("allow-same-origin");
		expect(sandbox).not.toContain("allow-top-navigation");
		// The key comes from the fragment, never the query or a request.
		expect(VIEWER_SCRIPT).toContain("location.hash");
		expect(VIEWER_SCRIPT).not.toContain("location.search");
	});

	it("is the same page for every share, and 404 for a malformed ID", async () => {
		expect(await (await call(`/p/${newShareId()}`)).text()).toBe(VIEWER_HTML);
		expect((await call("/p/not-a-share")).status).toBe(404);
		expect((await call(`/p/${newShareId()}/x`)).status).toBe(404);
	});

	it("opens what the Rust client seals (the shared fixture), with the viewer's AAD", async () => {
		const sealed = fromHex(fixture.sealed);
		expect(sealed[0]).toBe(1);
		expect(sealed.slice(1, 13)).toEqual(fromHex(fixture.nonce));
		expect(await open(fixture.share_id, sealed, fromB64url(fixture.key))).toBe(fixture.plaintext);
		// The ID is bound in: the same bytes under another ID don't open.
		await expect(open(newShareId(), sealed, fromB64url(fixture.key))).rejects.toThrow();
		// And the server accepts it as a share.
		const a = await signUp("alice");
		expect((await client(a.token).putShare(fixture.share_id, sealed)).status).toBe(200);
	});
});
