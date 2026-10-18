// Quotas, the global caps and the counters behind them: a model of every
// push and upload against small quotas, the reported quota order, the daily
// caps, the row-cost measurement and the admin overcommit refusal.

import { applyD1Migrations } from "cloudflare:test";
import { env } from "cloudflare:workers";
import fc from "fast-check";
import { beforeEach, describe, expect, it } from "vitest";
import { setQuota } from "../src/accounts";
import { sha256Hex } from "../src/crypto";
import { handle } from "../src/index";
import { MAX_SEALED_FILE } from "../src/protocol";
import {
	type Account,
	type Client,
	ORIGIN,
	SEEDED_SETTINGS,
	START,
	accountRow,
	client,
	recomputed,
	resetServer,
	runAdmin,
	sealed,
	setSetting,
	signUp,
	testEnv,
	testIO,
	wireId,
	wireRev,
} from "./helpers";

const runs = (n: number) => ({ numRuns: Math.min(n, Number(env.FAST_CHECK_RUNS)) });

const DOC_ROW = 512;
const FILE_ROW = 1024;

type Quota = "docs" | "doc_bytes" | "files" | "file_bytes" | "server";

interface Usage {
	docs: number;
	doc_bytes: number;
	files: number;
	file_bytes: number;
}

async function usageOf(accountId: string): Promise<Usage> {
	const row = await accountRow(accountId);
	return { docs: row!.docs, doc_bytes: row!.doc_bytes, files: row!.files, file_bytes: row!.file_bytes };
}

const bytes = (n: number, fill = 7) => new Uint8Array(n).fill(fill);

describe("the model", () => {
	// Two accounts push documents (new, edited, deleted, revived, and stale
	// bases that conflict), upload and replace files, and delete files with
	// tombstones, against small quotas and global caps that the owner lowers
	// below usage now and then. Each answer is the model's (accepted, or 413
	// naming the first quota in the spec's order with `used` and `limit`),
	// and afterwards every account's counters equal both the model and the
	// sums recomputed from its rows.
	it("prop_counters_equal_recomputed_sums and prop_growth_past_quota_is_refused_and_shrinking_always_passes", async () => {
		const DOC_SLOTS = 5;
		const FILE_SLOTS = 3;
		const genDoc = fc.record({
			slot: fc.integer({ min: 0, max: DOC_SLOTS + FILE_SLOTS - 1 }),
			deleted: fc.boolean(),
			len: fc.oneof(fc.integer({ min: 55, max: 120 }), fc.integer({ min: 55, max: 1500 })),
			// One in five sends a base the server doesn't hold: a null base for an existing document, or a made-up rev.
			stale: fc.integer({ min: 0, max: 9 }).map((n) => n < 2),
			staleNull: fc.boolean(),
		});
		const genOp = fc.oneof(
			{ weight: 5, arbitrary: fc.record({ kind: fc.constant("push" as const), who: fc.integer({ min: 0, max: 1 }), docs: fc.array(genDoc, { minLength: 1, maxLength: 3 }) }) },
			{
				weight: 3,
				arbitrary: fc.record({ kind: fc.constant("upload" as const), who: fc.integer({ min: 0, max: 1 }), slot: fc.integer({ min: 0, max: FILE_SLOTS - 1 }), size: fc.integer({ min: 41, max: 2500 }) }),
			},
			{
				weight: 1,
				arbitrary: fc.record({
					kind: fc.constant("quota" as const),
					docs: fc.integer({ min: 1, max: 6 }),
					doc_bytes: fc.integer({ min: 600, max: 8000 }),
					files: fc.integer({ min: 1, max: 3 }),
					file_bytes: fc.integer({ min: 1100, max: 6000 }),
				}),
			},
		);
		await fc.assert(
			fc.asyncProperty(fc.array(genOp, { minLength: 15, maxLength: 45 }), async (ops) => {
				await resetServer({ signup: "open", global_doc_bytes: "9000", global_file_bytes: "8000" });
				const accounts: Account[] = [await signUp("alice"), await signUp("bob")];
				const clients: Client[] = accounts.map((a) => client(a.token));
				const docIds = Array.from({ length: DOC_SLOTS }, () => wireId("note"));
				const fileIds = Array.from({ length: FILE_SLOTS }, () => wireId("file"));
				const idOf = (slot: number) => (slot < DOC_SLOTS ? docIds[slot] : fileIds[slot - DOC_SLOTS]);
				// Alice has small quotas; Bob the (large) defaults.
				const quotas = [
					{ docs: 3, doc_bytes: 6000, files: 2, file_bytes: 5000 },
					{ docs: 50_000, doc_bytes: 104_857_600, files: 10_000, file_bytes: 524_288_000 },
				];
				const setAlice = (q: (typeof quotas)[0]) =>
					env.DB.prepare("UPDATE accounts SET quota_docs = ?2, quota_doc_bytes = ?3, quota_files = ?4, quota_file_bytes = ?5 WHERE id = ?1")
						.bind(accounts[0].creds.accountId, q.docs, q.doc_bytes, q.files, q.file_bytes)
						.run();
				await setAlice(quotas[0]);

				interface Doc {
					rev: string;
					deleted: boolean;
					sealed: string;
				}
				const model = accounts.map(() => ({
					docs: new Map<string, Doc>(),
					files: new Map<string, number>(),
					lastSeq: 0,
				}));
				const usage = (m: (typeof model)[0]): Usage => ({
					docs: [...m.docs.values()].filter((d) => !d.deleted).length,
					doc_bytes: [...m.docs.values()].reduce((t, d) => t + d.sealed.length + DOC_ROW, 0),
					files: m.files.size,
					file_bytes: [...m.files.values()].reduce((t, s) => t + s + FILE_ROW, 0),
				});
				const globalDoc = 9000;
				const globalFile = 8000;

				for (const op of ops) {
					const label = JSON.stringify(op);
					if (op.kind === "quota") {
						quotas[0] = { docs: op.docs, doc_bytes: op.doc_bytes, files: op.files, file_bytes: op.file_bytes };
						await setAlice(quotas[0]);
						continue;
					}
					const m = model[op.who];
					const q = quotas[op.who];
					const before = usage(m);
					const others = usage(model[1 - op.who]);

					if (op.kind === "push") {
						// Simulate the batch in order on a copy.
						const docs = new Map(m.docs);
						let seq = m.lastSeq;
						let refusal: Quota | null = null;
						const expected: ({ status: "ok"; seq: number } | { status: "conflict"; current: Doc | null })[] = [];
						const wire = op.docs.map((d) => {
							const id = idOf(d.slot);
							const existing = docs.get(id);
							const base = d.stale ? (existing && d.staleNull ? null : wireRev()) : (existing?.rev ?? null);
							const doc = { id, rev: wireRev(), deleted: d.deleted, sealed: sealed(d.len), base_rev: base };
							const accepted = base === null ? !existing : existing?.rev === base;
							if (!accepted) {
								expected.push({ status: "conflict", current: existing ?? null });
								return doc;
							}
							if (refusal) return doc;
							const was = usage({ ...m, docs });
							docs.set(id, { rev: doc.rev, deleted: doc.deleted, sealed: doc.sealed });
							const now = usage({ ...m, docs });
							if (now.docs > was.docs && now.docs > q.docs) refusal = "docs";
							else if (now.doc_bytes > was.doc_bytes && now.doc_bytes > q.doc_bytes) refusal = "doc_bytes";
							else if (now.doc_bytes > was.doc_bytes && others.doc_bytes + now.doc_bytes > globalDoc) refusal = "server";
							seq += 1;
							expected.push({ status: "ok", seq });
							return doc;
						});
						const r = await clients[op.who].push(wire);
						if (refusal) {
							const [used, limit] =
								refusal === "server" ? [before.doc_bytes + others.doc_bytes, globalDoc] : [before[refusal as keyof Usage], q[refusal as keyof Usage]];
							expect(r.status, label).toBe(413);
							expect(r.body, label).toEqual({ error: "quota_exceeded", message: expect.any(String), quota: refusal, used, limit });
						} else {
							expect(r.status, `${label}: ${JSON.stringify(r.body)}`).toBe(200);
							const grew = (Object.keys(before) as (keyof Usage)[]).some((k) => usage({ ...m, docs })[k] > before[k]);
							if (!grew) expect(r.status, `${label}: shrinking passes`).toBe(200);
							expect(r.body.results.map((x: any) => x.status)).toEqual(expected.map((x) => x.status));
							r.body.results.forEach((x: any, i: number) => {
								const want = expected[i];
								if (want.status === "ok") expect(x.seq).toBe(want.seq);
								else if (want.current === null) expect(x.current).toBeNull();
								else expect(x.current).toEqual({ id: wire[i].id, rev: want.current.rev, deleted: want.current.deleted, sealed: want.current.sealed });
							});
							m.docs = docs;
							m.lastSeq = seq;
							// A file's bytes go when the last accepted version of its document in this push is a tombstone.
							const last = new Map<string, boolean>();
							wire.forEach((d, i) => {
								if (expected[i].status === "ok" && d.id.startsWith("file:")) last.set(d.id, d.deleted);
							});
							for (const [id, deleted] of last) if (deleted) m.files.delete(id);
						}
					} else {
						const id = fileIds[op.slot];
						const old = m.files.get(id);
						const files = new Map(m.files);
						files.set(id, op.size);
						const after = usage({ ...m, files });
						let refusal: Quota | null = null;
						if (after.files > before.files && after.files > q.files) refusal = "files";
						else if (after.file_bytes > before.file_bytes && after.file_bytes > q.file_bytes) refusal = "file_bytes";
						else if (after.file_bytes > before.file_bytes && others.file_bytes + after.file_bytes > globalFile) refusal = "server";
						const r = await clients[op.who].putFile(id, bytes(op.size, op.slot + 1));
						if (refusal) {
							const [used, limit] =
								refusal === "server" ? [before.file_bytes + others.file_bytes, globalFile] : [before[refusal as keyof Usage], q[refusal as keyof Usage]];
							expect(r.status, label).toBe(413);
							expect(r.body, label).toEqual({ error: "quota_exceeded", message: expect.any(String), quota: refusal, used, limit });
						} else {
							expect(r.status, `${label} (old ${old}): ${JSON.stringify(r.body)}`).toBe(200);
							expect(r.body).toEqual({ ok: true, size: op.size });
							m.files = files;
						}
					}

					for (const [i, a] of accounts.entries()) {
						const stored = await usageOf(a.creds.accountId);
						expect(stored, `${label}: account ${i} against the model`).toEqual(usage(model[i]));
						expect(stored, `${label}: account ${i} against its rows`).toEqual(await recomputed(a.creds.accountId));
					}
				}
				for (const [i, c] of clients.entries()) {
					const r = await c.account();
					expect(r.body.usage).toMatchObject(usage(model[i]));
					const feed = await c.changes(0, 1);
					expect(feed.body.head_seq).toBe(model[i].lastSeq);
				}
			}),
			runs(60),
		);
	});
});

describe("examples", () => {
	let alice: Client;
	let a: Account;

	beforeEach(async () => {
		await resetServer();
		a = await signUp("alice");
		alice = client(a.token);
	});

	const quota = (change: Parameters<typeof setQuota>[2]) => setQuota(testIO().io, "alice", change, true);
	const doc = (len = 64, deleted = false) => ({ id: wireId(), rev: wireRev(), deleted, sealed: sealed(len), base_rev: null as string | null });

	it("names one quota, in the order docs, doc_bytes, server", async () => {
		await quota({ docs: 1, docBytes: 1000 });
		expect((await alice.push([doc(100)])).status).toBe(200);
		// Over both the count and the bytes: docs.
		let r = await alice.push([doc(2000)]);
		expect(r.body).toMatchObject({ quota: "docs", used: 1, limit: 1 });
		// Over the bytes only (a tombstone isn't live): doc_bytes.
		r = await alice.push([doc(2000, true)]);
		expect(r.body).toMatchObject({ quota: "doc_bytes", used: 612, limit: 1000 });
		// Within the account's quota, over the server's: server, with the server's sum and cap.
		await quota({ docBytes: 5000 });
		await setSetting("global_doc_bytes", "1000");
		r = await alice.push([doc(300, true)]);
		expect(r.body).toMatchObject({ quota: "server", used: 612, limit: 1000 });
		expect(await usageOf(a.creds.accountId)).toEqual({ docs: 1, doc_bytes: 612, files: 0, file_bytes: 0 });
	});

	it("names one quota, in the order files, file_bytes, server", async () => {
		await quota({ files: 1, fileBytes: 3000 });
		expect((await alice.putFile(wireId("file"), bytes(500))).status).toBe(200);
		let r = await alice.putFile(wireId("file"), bytes(5000));
		expect(r.body).toMatchObject({ quota: "files", used: 1, limit: 1 });
		const [only] = (await env.DB.prepare("SELECT id FROM account_files").all<{ id: string }>()).results;
		r = await alice.putFile(only.id, bytes(2500));
		expect(r.body).toMatchObject({ quota: "file_bytes", used: 1524, limit: 3000 });
		await setSetting("global_file_bytes", "2000");
		r = await alice.putFile(only.id, bytes(1000));
		expect(r.body).toMatchObject({ quota: "server", used: 1524, limit: 2000 });
		// Replacing with fewer bytes always passes, whatever is over.
		expect((await alice.putFile(only.id, bytes(100))).status).toBe(200);
		expect(await usageOf(a.creds.accountId)).toEqual({ docs: 0, doc_bytes: 0, files: 1, file_bytes: 1124 });
	});

	it("an account over a lowered quota can still delete, shrink and edit within its bytes", async () => {
		const docs = [doc(500), doc(500), doc(500)];
		expect((await alice.push(docs)).status).toBe(200);
		await quota({ docs: 1, docBytes: 600 });
		const edit = (d: (typeof docs)[0], len: number, deleted = false) => ({ ...d, rev: wireRev(), base_rev: d.rev, sealed: sealed(len), deleted });
		// Shrinking one document: fewer bytes, same count.
		const smaller = edit(docs[0], 100);
		expect((await alice.push([smaller])).status).toBe(200);
		// Deleting another.
		const gone = edit(docs[1], 55, true);
		expect((await alice.push([gone])).status).toBe(200);
		// Reviving it grows the count past 1: refused.
		expect((await alice.push([edit(gone, 55)])).body.quota).toBe("docs");
		expect(await usageOf(a.creds.accountId)).toEqual(await recomputed(a.creds.accountId));
	});

	it("the daily caps refuse with 429 until the next UTC midnight, then pass", async () => {
		await setSetting("global_writes_per_day", "3");
		const now = START + 10 * 3_600_000;
		const at = client(a.token, { now });
		expect((await at.push([doc(), doc()])).status).toBe(200);
		let r = await at.push([doc(), doc()]);
		expect(r.status).toBe(429);
		expect(r.body).toMatchObject({ error: "rate_limited", limit: "writes-global", retry_after: 14 * 3600 });
		expect(r.headers.get("retry-after")).toBe(String(14 * 3600));
		expect((await at.push([doc()])).status).toBe(200);
		expect((await at.push([doc()])).body.limit).toBe("writes-global");
		expect((await client(a.token, { now: START + 86_400_000 }).push([doc(), doc(), doc()])).status).toBe(200);

		await setSetting("global_upload_bytes_per_day", "1000");
		expect((await at.putFile(wireId("file"), bytes(600))).status).toBe(200);
		const upload = await at.putFile(wireId("file"), bytes(401));
		expect(upload.body).toMatchObject({ error: "rate_limited", limit: "uploads-global", retry_after: 14 * 3600 });
		expect((await at.putFile(wireId("file"), bytes(400))).status).toBe(200);
		// Replacements count too.
		const [first] = (await env.DB.prepare("SELECT id FROM account_files ORDER BY uploaded_at LIMIT 1").all<{ id: string }>()).results;
		expect((await at.putFile(first.id, bytes(41))).body.limit).toBe("uploads-global");
		expect((await client(a.token, { now: START + 86_400_000 }).putFile(first.id, bytes(999))).status).toBe(200);
	});

	it("the uploads-account rule caps one account's upload bytes per UTC day, counted before the body is read", async () => {
		const UPLOAD_BYTES_PER_DAY = Number(SEEDED_SETTINGS.upload_bytes_per_day);
		expect(UPLOAD_BYTES_PER_DAY).toBe(1_073_741_824);
		const key = `uploads-account:${a.creds.accountId}`;
		const near = (left: number) =>
			env.DB.prepare("INSERT OR REPLACE INTO limits (key, window_start, count) VALUES (?1, ?2, ?3)").bind(key, START, UPLOAD_BYTES_PER_DAY - left).run();
		const now = START + 10 * 3_600_000;
		const at = client(a.token, { now });
		const id = wireId("file");
		await near(1000);
		// Exactly the day's remaining bytes pass; one more byte doesn't, replacement or not.
		expect((await at.putFile(id, bytes(1000))).status).toBe(200);
		const r = await at.putFile(id, bytes(41));
		expect(r.status).toBe(429);
		expect(r.body).toMatchObject({ error: "rate_limited", limit: "uploads-account", retry_after: 14 * 3600 });
		// Other accounts have their own budget; the next UTC day starts afresh.
		const b = client((await signUp("bob")).token, { now });
		expect((await b.putFile(id, bytes(5000))).status).toBe(200);
		expect((await client(a.token, { now: START + 86_400_000 }).putFile(id, bytes(5000))).status).toBe(200);

		// Without Content-Length, the largest file is counted before anything is read.
		await near(MAX_SEALED_FILE - 1);
		const streamed = async () => {
			const body = bytes(64);
			const stream = new ReadableStream({
				start(controller) {
					controller.enqueue(body);
					controller.close();
				},
			});
			const response = await handle(
				new Request(`${ORIGIN}/v2/files/${id}`, {
					method: "PUT",
					body: stream,
					headers: { authorization: `Bearer ${a.token}`, "content-type": "application/octet-stream", "x-kanso-sha256": await sha256Hex(body) },
				}),
				testEnv(),
				now,
			);
			return response.status;
		};
		expect(await streamed()).toBe(429);
		await near(MAX_SEALED_FILE + 64);
		expect(await streamed()).toBe(200);

		// The budget is the upload_bytes_per_day setting, read as each upload counts.
		await setSetting("upload_bytes_per_day", "200");
		const later = client(a.token, { now: START + 2 * 86_400_000 });
		expect((await later.putFile(id, bytes(200))).status).toBe(200);
		expect((await later.putFile(id, bytes(41))).body.limit).toBe("uploads-account");
		await setSetting("upload_bytes_per_day", "0");
		expect((await client(b.token, { now: START + 2 * 86_400_000 }).putFile(id, bytes(41))).body.limit).toBe("uploads-account");
	});

	it("an account's own quota_writes_per_day overrides writes_per_day for it alone", async () => {
		await setSetting("writes_per_day", "2");
		const b = client((await signUp("bob")).token);
		await env.DB.prepare("UPDATE accounts SET quota_writes_per_day = 4 WHERE id = ?1").bind(a.creds.accountId).run();
		expect((await alice.push([doc(), doc(), doc(), doc()])).status).toBe(200);
		expect((await alice.push([doc()])).body.limit).toBe("writes-account");
		expect((await alice.account()).body.quota.writes_per_day).toBe(4);
		expect((await b.push([doc(), doc(), doc()])).body.limit).toBe("writes-account");
		expect((await b.account()).body.quota.writes_per_day).toBe(2);
	});

	it("the writes-account rule counts documents and uploads per account per UTC day", async () => {
		await setSetting("writes_per_day", "5");
		const b = client((await signUp("bob")).token);
		expect((await alice.push([doc(), doc(), doc(), doc()])).status).toBe(200);
		expect((await alice.putFile(wireId("file"), bytes(41))).status).toBe(200);
		const r = await alice.push([doc()]);
		expect(r.body).toMatchObject({ error: "rate_limited", limit: "writes-account" });
		expect((await alice.putFile(wireId("file"), bytes(41))).body.limit).toBe("writes-account");
		expect((await b.push([doc(), doc(), doc(), doc(), doc()])).status).toBe(200);
		expect((await alice.account()).body.usage.writes_today).toBe(7);
	});
});

describe("the row cost", () => {
	// docs/ACCOUNTS.md › Settings: 10,000 random rows in a fresh database grow
	// it by no more than Σ(length(sealed) + 512).
	it("covers what a document row really takes, indexes included", async () => {
		const db = env.MIGRATION_DB;
		const { results } = await db
			.prepare("SELECT type, name FROM sqlite_master WHERE type IN ('table', 'view') AND name NOT LIKE 'sqlite_%' AND name NOT LIKE '_cf_%'")
			.all<{ type: string; name: string }>();
		for (const { name } of results.filter((r) => r.type === "view")) await db.exec(`DROP VIEW "${name}"`);
		const order = ["sessions", "account_docs", "account_files"];
		const tables = results.filter((r) => r.type === "table").map((r) => r.name);
		tables.sort((x, y) => Number(order.includes(y)) - Number(order.includes(x)));
		for (const name of tables) await db.exec(`DROP TABLE "${name}"`);
		await applyD1Migrations(db, env.TEST_MIGRATIONS);
		await db.prepare("UPDATE settings SET value = 'open' WHERE name = 'signup'").run();
		const accountId = `a_${"ab".repeat(16)}`;
		await db
			.prepare(
				`INSERT INTO accounts (id, username, kdf_m_kib, kdf_t, kdf_p, salt, auth_hash, wrapped_key, recovery_hash,
  recovery_wrapped_key, key_id, epoch, created_at) VALUES (?1, 'alice', 1048576, 3, 4, ?2, ?3, ?4, ?3, ?4, ?5, ?5, 0)`,
			)
			.bind(accountId, "A".repeat(22), "0".repeat(64), "A".repeat(98), "0".repeat(16))
			.run();

		// D1 refuses PRAGMA page_count (SQLITE_AUTH); every result's meta.size_after is
		// the same figure, the database file's size in bytes (page_count × page_size).
		const size = async () => (await db.prepare("SELECT 1").run()).meta.size_after;
		const start = await size();

		// Mostly short rows (tombstones and small notes), where the fixed cost matters most.
		const seed = Number(env.FAST_CHECK_SEED);
		const lengths = fc.sample(fc.oneof({ weight: 3, arbitrary: fc.integer({ min: 55, max: 120 }) }, { weight: 1, arbitrary: fc.integer({ min: 121, max: 4000 }) }), {
			numRuns: 10_000,
			seed,
		});
		let charged = 0;
		const insert = db.prepare("INSERT INTO account_docs (account_id, id, rev, seq, deleted, sealed, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)");
		for (let i = 0; i < lengths.length; i += 500) {
			const chunk = lengths.slice(i, i + 500).map((len, j) => {
				charged += len + DOC_ROW;
				return insert.bind(accountId, wireId(), wireRev(), i + j + 1, len < 70 ? 1 : 0, sealed(len), START + i + j);
			});
			await db.batch(chunk);
		}
		const grown = (await size()) - start;
		expect(grown).toBeGreaterThan(0);
		expect(grown).toBeLessThanOrEqual(charged);
	});
});

describe("the overcommit check", () => {
	beforeEach(() => resetServer());

	it("quota, default-quota and max-accounts refuse to promise more than a global cap, unless forced", async () => {
		await signUp("alice");
		// 50 × 100 MiB = 5,242,880,000 of 6,000,000,000 promised.
		let r = await runAdmin(["quota", "alice", "--doc-mib", "823"]);
		expect(r.ok, r.message).toBe(false);
		expect(r.message).toContain("global_doc_bytes");
		r = await runAdmin(["quota", "alice", "--doc-mib", "822"]);
		expect(r.ok, r.message).toBe(true);
		expect(await env.DB.prepare("SELECT quota_doc_bytes FROM accounts").first("quota_doc_bytes")).toBe(822 * 1_048_576);
		// 50 × 1000 MiB of files would be 52,428,800,000 of 50,000,000,000.
		expect((await runAdmin(["default-quota", "--file-mib", "1000"])).message).toContain("global_file_bytes");
		expect((await runAdmin(["default-quota", "--file-mib", "900"])).ok).toBe(true);
		expect((await runAdmin(["max-accounts", "60"])).ok).toBe(false);
		expect(await env.DB.prepare("SELECT value FROM settings WHERE name = 'max_accounts'").first("value")).toBe("50");
		expect((await runAdmin(["max-accounts", "60", "--force"])).ok).toBe(true);
		expect(await env.DB.prepare("SELECT value FROM settings WHERE name = 'max_accounts'").first("value")).toBe("60");
		expect(await env.DB.prepare("SELECT value FROM settings WHERE name = 'quota_file_bytes'").first("value")).toBe(String(900 * 1_048_576));
		// Once overcommitted, every change that leaves it so is refused too, even one that lowers the promise.
		expect((await runAdmin(["quota", "alice", "--default"])).ok).toBe(false);
		expect((await runAdmin(["max-accounts", "50"])).ok).toBe(true);
		expect((await runAdmin(["quota", "alice", "--default"])).ok).toBe(true);
	});
});
