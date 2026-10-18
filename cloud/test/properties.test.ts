// Properties of one account's sync (fast-check, seed pinned in
// vitest.config.ts): push and the feed against a model, paging, tokens and
// files; then conflicts whose current version is large, and "behind".

import { env } from "cloudflare:workers";
import fc from "fast-check";
import { beforeEach, describe, expect, it } from "vitest";
import { MAX_SEALED, PAGE_SEALED_BUDGET } from "../src/protocol";
import { type Account, type Client, call, client, resetServer, sealed, signUp, wireId, wireRev } from "./helpers";

const runs = (n: number) => ({ numRuns: Math.min(n, Number(env.FAST_CHECK_RUNS)) });

interface ModelDoc {
	rev: string;
	deleted: boolean;
	sealed: string;
	seq: number;
}

const IDS = [wireId("note"), wireId("book"), wireId("file")];
const rev = (n: number) => n.toString(16).padStart(32, "0");

const genDoc = fc.record({
	id: fc.constantFrom(...IDS),
	base: fc.constantFrom("current", "current", "null", "stale"),
	deleted: fc.boolean(),
	len: fc.integer({ min: 55, max: 400 }),
});

let a: Account;
let alice: Client;

async function fresh(): Promise<void> {
	await resetServer();
	a = await signUp("alice");
	alice = client(a.token);
}

describe("push and changes follow the model", () => {
	// Model-based: a Map of current revisions and one counter. A document is
	// accepted iff its base is the model's current revision (or both are
	// absent); each accepted write takes the next seq; the feed lists every
	// document once, at its latest seq, with head_seq the counter.
	it("prop_push_acceptance_and_seq_match_the_model", async () => {
		await fc.assert(
			fc.asyncProperty(fc.array(fc.array(genDoc, { maxLength: 4 }), { minLength: 1, maxLength: 8 }), async (pushes) => {
				await fresh();
				const model = new Map<string, ModelDoc>();
				let counter = 0;
				let revs = 0;
				for (const batch of pushes) {
					const sent = batch.map((doc) => {
						revs += 1;
						const known = model.get(doc.id)?.rev ?? null;
						return {
							id: doc.id,
							rev: rev(revs),
							deleted: doc.deleted,
							sealed: sealed(doc.len),
							base_rev: doc.base === "current" ? known : doc.base === "null" ? null : rev(0),
						};
					});
					const response = await alice.push(sent);
					expect(response.status).toBe(200);
					sent.forEach((doc, i) => {
						const current = model.get(doc.id);
						const accepted = current ? doc.base_rev === current.rev : doc.base_rev === null;
						if (accepted) {
							counter += 1;
							model.set(doc.id, { rev: doc.rev, deleted: doc.deleted, sealed: doc.sealed, seq: counter });
							expect(response.body.results[i]).toEqual({ id: doc.id, status: "ok", seq: counter });
						} else {
							expect(response.body.results[i]).toEqual({
								id: doc.id,
								status: "conflict",
								current: current ? { id: doc.id, rev: current.rev, deleted: current.deleted, sealed: current.sealed } : null,
							});
						}
					});
				}
				const feed = await alice.changes(0, 500);
				const expected = [...model.entries()]
					.map(([id, d]) => ({ id, rev: d.rev, deleted: d.deleted, sealed: d.sealed, seq: d.seq }))
					.sort((x, y) => x.seq - y.seq);
				expect(feed.body).toEqual({
					docs: expected,
					last_seq: expected.at(-1)?.seq ?? 0,
					more: false,
					head_seq: counter,
					epoch: a.epoch,
					key_id: a.creds.keyId,
				});
			}),
			runs(60),
		);
	});

	// Metamorphic: paging with any limit concatenates to the one-page feed,
	// `more` is true exactly while a page was full, and head_seq never moves.
	it("prop_paging_with_any_limit_concatenates_to_the_whole_feed", async () => {
		await fc.assert(
			fc.asyncProperty(fc.array(fc.integer({ min: 0, max: 7 }), { maxLength: 25 }), fc.integer({ min: 1, max: 6 }), async (writes, limit) => {
				await fresh();
				const ids = Array.from({ length: 8 }, () => wireId());
				const latest = new Map<string, string>();
				const docs = writes.map((k, n) => {
					const id = ids[k];
					const doc = { id, rev: rev(n + 1), deleted: false, sealed: sealed(60), base_rev: latest.get(id) ?? null };
					latest.set(id, doc.rev);
					return doc;
				});
				expect((await alice.push(docs)).status).toBe(200);
				const whole = (await alice.changes(0, 500)).body;
				expect(whole.docs.length).toBe(latest.size);
				expect(whole.head_seq).toBe(writes.length);
				for (const doc of whole.docs) expect(doc.rev).toBe(latest.get(doc.id));
				const seqs = whole.docs.map((d: { seq: number }) => d.seq);
				expect(seqs).toEqual([...seqs].sort((x, y) => x - y));
				expect(new Set(seqs).size).toBe(seqs.length);

				const paged: unknown[] = [];
				let since = 0;
				for (;;) {
					const page = (await alice.changes(since, limit)).body;
					expect(page.docs.length).toBeLessThanOrEqual(limit);
					expect(page.last_seq).toBe(page.docs.at(-1)?.seq ?? since);
					expect(page.more).toBe(page.docs.length === limit);
					expect(page.head_seq).toBe(writes.length);
					paged.push(...page.docs);
					since = page.last_seq;
					if (!page.more) break;
				}
				expect(paged).toEqual(whole.docs);
			}),
			runs(60),
		);
	});
});

describe("tokens", () => {
	beforeEach(fresh);

	// Invariant: only the exact bearer token gets through; everything else is 401 unknown.
	it("prop_any_other_authorization_is_refused", async () => {
		const real = `Bearer ${a.token}`;
		const printable = fc.stringMatching(/^[\x21-\x7e][\x20-\x7e]{0,60}$/);
		const mutated = fc
			.tuple(fc.nat({ max: real.length }), fc.constantFrom("drop", "replace", "insert"), fc.stringMatching(/^[\x21-\x7e]$/))
			.map(([i, op, c]) =>
				op === "drop" ? real.slice(0, i) + real.slice(i + 1) : op === "replace" ? real.slice(0, i) + c + real.slice(i + 1) : real.slice(0, i) + c + real.slice(i),
			);
		await fc.assert(
			fc.asyncProperty(fc.oneof(printable, mutated), async (header) => {
				fc.pre(header.trim() !== real && header.trim().replace(/^bearer/i, "Bearer") !== real);
				const response = await call("/v2/changes", { headers: { authorization: header } });
				expect(response.status).toBe(401);
				const body = await response.json<{ error: string; reason: string }>();
				expect(body).toMatchObject({ error: "unauthorized", reason: "unknown" });
			}),
			runs(100),
		);
		expect((await call("/v2/changes", { headers: { authorization: real } })).status).toBe(200);
		expect((await call("/v2/changes")).status).toBe(401);
	});
});

describe("files", () => {
	beforeEach(fresh);

	// Round trip, and a wrong checksum never replaces stored bytes.
	it("prop_files_round_trip_and_reject_a_wrong_checksum", async () => {
		const id = wireId("file");
		await fc.assert(
			fc.asyncProperty(fc.uint8Array({ minLength: 41, maxLength: 4096 }), fc.uint8Array({ minLength: 41, maxLength: 64 }), async (first, other) => {
				expect((await alice.putFile(id, first)).body).toEqual({ ok: true, size: first.length });
				const wrong = await alice.putFile(id, other, "0".repeat(64));
				expect(wrong.status).toBe(400);
				expect(wrong.body.error).toBe("checksum_mismatch");
				const got = await alice.getFile(id);
				expect(got.status).toBe(200);
				expect(got.bytes).toEqual(first);
				expect((await alice.account()).body.usage).toMatchObject({ files: 1, file_bytes: first.length + 1024 });
			}),
			runs(40),
		);
	});
});

describe("large conflicts", () => {
	beforeEach(fresh);

	const big = (fill: string) => fill.repeat(MAX_SEALED);

	it("returns a large current version whole, and answers behind once the 8 MiB budget is spent", async () => {
		// Five documents at the largest sealed size, two per push (4 MiB bodies).
		const docs = ["A", "B", "C", "D", "E"].map((fill) => ({ id: wireId(), rev: wireRev(), deleted: false, sealed: big(fill), base_rev: null }));
		for (let i = 0; i < docs.length; i += 2) expect((await alice.push(docs.slice(i, i + 2))).status).toBe(200);
		// Stale edits of all five: the first four currents fit 8 MiB, the fifth doesn't.
		const stale = docs.map((d) => ({ id: d.id, rev: wireRev(), deleted: false, sealed: sealed(), base_rev: wireRev() }));
		const r = await alice.push(stale);
		expect(r.status).toBe(200);
		expect(4 * MAX_SEALED).toBeLessThanOrEqual(PAGE_SEALED_BUDGET);
		expect(5 * MAX_SEALED).toBeGreaterThan(PAGE_SEALED_BUDGET);
		expect(r.body.results.map((x: { status: string }) => x.status)).toEqual(["conflict", "conflict", "conflict", "conflict", "behind"]);
		r.body.results.slice(0, 4).forEach((x: any, i: number) => {
			expect(x).toEqual({ id: docs[i].id, status: "conflict", current: { id: docs[i].id, rev: docs[i].rev, deleted: false, sealed: docs[i].sealed } });
		});
		expect(r.body.results[4]).toEqual({ id: docs[4].id, status: "behind" });
	});

	it("answers behind for a large current version that a later document in the same push replaced", async () => {
		const doc = { id: wireId(), rev: wireRev(), deleted: false, sealed: big("Q"), base_rev: null };
		expect((await alice.push([doc])).status).toBe(200);
		const conflicting = { id: doc.id, rev: wireRev(), deleted: false, sealed: sealed(), base_rev: wireRev() };
		const replacing = { id: doc.id, rev: wireRev(), deleted: false, sealed: sealed(), base_rev: doc.rev };
		const r = await alice.push([conflicting, replacing]);
		expect(r.body.results).toEqual([
			{ id: doc.id, status: "behind" },
			{ id: doc.id, status: "ok", seq: 2 },
		]);
	});

	it("returns a small current version inline even when a large one is pending", async () => {
		const large = { id: wireId(), rev: wireRev(), deleted: false, sealed: big("L"), base_rev: null };
		const small = { id: wireId(), rev: wireRev(), deleted: true, sealed: sealed(70), base_rev: null };
		expect((await alice.push([large, small])).status).toBe(200);
		const r = await alice.push([
			{ id: small.id, rev: wireRev(), deleted: false, sealed: sealed(), base_rev: null },
			{ id: large.id, rev: wireRev(), deleted: false, sealed: sealed(), base_rev: null },
			{ id: wireId(), rev: wireRev(), deleted: false, sealed: sealed(), base_rev: wireRev() },
		]);
		expect(r.body.results).toEqual([
			{ id: small.id, status: "conflict", current: { id: small.id, rev: small.rev, deleted: true, sealed: small.sealed } },
			{ id: large.id, status: "conflict", current: { id: large.id, rev: large.rev, deleted: false, sealed: large.sealed } },
			{ id: expect.any(String), status: "conflict", current: null },
		]);
	});
});
