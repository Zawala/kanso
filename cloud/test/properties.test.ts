// Property tests (fast-check, seed pinned in vitest.config.ts via FAST_CHECK_SEED).

import fc from "fast-check";
import { describe, expect, it } from "vitest";
import { TOKEN, call, changes, getFile, md5Hex, push, putFile, resetServer } from "./helpers";

interface ModelDoc {
	rev: string;
	deleted: boolean;
	fields: Record<string, unknown>;
	seq: number;
}

const IDS = ["note:a", "note:b", "file:c"] as const;

const genDoc = fc.record({
	id: fc.constantFrom(...IDS),
	base: fc.constantFrom("current", "current", "null", "stale"),
	deleted: fc.boolean(),
	title: fc.string({ maxLength: 6 }),
});

describe("push and changes follow the model", () => {
	// Model-based: a Map of current revisions and one counter. A document is
	// accepted iff its base is the model's current revision (or both are
	// absent); each accepted write takes the next seq; the feed lists every
	// document once, at its latest seq.
	it("prop_push_acceptance_and_seq_match_the_model", async () => {
		await fc.assert(
			fc.asyncProperty(fc.array(fc.array(genDoc, { maxLength: 4 }), { minLength: 1, maxLength: 8 }), async (pushes) => {
				await resetServer();
				const model = new Map<string, ModelDoc>();
				let counter = 0;
				let revs = 0;
				for (const batch of pushes) {
					// The client picks bases from what it knew before this push.
					const sent = batch.map((doc) => {
						revs += 1;
						const known = model.get(doc.id)?.rev ?? null;
						return {
							id: doc.id,
							rev: `${revs}-${revs.toString(16)}`,
							deleted: doc.deleted,
							fields: doc.deleted ? {} : { title: doc.title },
							base_rev: doc.base === "current" ? known : doc.base === "null" ? null : "1-0",
						};
					});
					const response = await push(sent);
					expect(response.status).toBe(200);
					sent.forEach((doc, i) => {
						const current = model.get(doc.id);
						const accepted = current ? doc.base_rev === current.rev : doc.base_rev === null;
						if (accepted) {
							counter += 1;
							model.set(doc.id, { rev: doc.rev, deleted: doc.deleted, fields: doc.fields, seq: counter });
							expect(response.body.results[i]).toEqual({ id: doc.id, status: "ok", seq: counter });
						} else {
							expect(response.body.results[i]).toEqual({
								id: doc.id,
								status: "conflict",
								current: current
									? { id: doc.id, rev: current.rev, deleted: current.deleted, fields: current.fields }
									: null,
							});
						}
					});
				}
				const feed = await changes(0, 500);
				const expected = [...model.entries()]
					.map(([id, d]) => ({ id, rev: d.rev, deleted: d.deleted, fields: d.fields, seq: d.seq }))
					.sort((a, b) => a.seq - b.seq);
				expect(feed.body).toEqual({ docs: expected, last_seq: expected.at(-1)?.seq ?? 0 });
			}),
		);
	});

	// Metamorphic: paging with any limit yields exactly the one-page feed.
	it("prop_paging_with_any_limit_concatenates_to_the_whole_feed", async () => {
		await fc.assert(
			fc.asyncProperty(
				fc.array(fc.integer({ min: 0, max: 7 }), { maxLength: 25 }),
				fc.integer({ min: 1, max: 6 }),
				async (writes, limit) => {
					await resetServer();
					const latest = new Map<string, string>();
					const docs = writes.map((k, n) => {
						const id = `note:d${k}`;
						const rev = `${n + 1}-${(n + 1).toString(16)}`;
						const doc = { id, rev, base_rev: latest.get(id) ?? null };
						latest.set(id, rev);
						return doc;
					});
					expect((await push(docs)).status).toBe(200);
					const whole = (await changes(0, 500)).body;
					expect(whole.docs.length).toBe(latest.size);
					for (const doc of whole.docs) expect(doc.rev).toBe(latest.get(doc.id));
					const seqs = whole.docs.map((d: { seq: number }) => d.seq);
					expect(seqs).toEqual([...seqs].sort((a, b) => a - b));
					expect(new Set(seqs).size).toBe(seqs.length);

					const paged: unknown[] = [];
					let since = 0;
					for (;;) {
						const page = (await changes(since, limit)).body;
						expect(page.docs.length).toBeLessThanOrEqual(limit);
						expect(page.last_seq).toBe(page.docs.at(-1)?.seq ?? since);
						if (page.docs.length === 0) break;
						paged.push(...page.docs);
						since = page.last_seq;
					}
					expect(paged).toEqual(whole.docs);
				},
			),
		);
	});
});

describe("auth", () => {
	const real = `Bearer ${TOKEN}`;
	const printable = fc.stringMatching(/^[\x21-\x7e][\x20-\x7e]{0,40}$/);
	const mutated = fc
		.tuple(fc.nat({ max: real.length }), fc.constantFrom("drop", "replace", "insert"), fc.stringMatching(/^[\x21-\x7e]$/))
		.map(([i, op, c]) =>
			op === "drop"
				? real.slice(0, i) + real.slice(i + 1)
				: op === "replace"
					? real.slice(0, i) + c + real.slice(i + 1)
					: real.slice(0, i) + c + real.slice(i),
		);

	// Invariant: only the exact bearer token gets through.
	it("prop_any_other_authorization_is_refused", async () => {
		await fc.assert(
			fc.asyncProperty(fc.oneof(printable, mutated), async (header) => {
				fc.pre(header.trim() !== real);
				const response = await call("/v1/changes", { token: null, headers: { authorization: header } });
				expect(response.status).toBe(401);
			}),
		);
		expect((await call("/v1/changes", { token: null, headers: { authorization: real } })).status).toBe(200);
	});
});

describe("files", () => {
	// Round trip, and an MD5 mismatch never replaces stored bytes.
	it("prop_files_round_trip_and_reject_a_wrong_md5", async () => {
		await fc.assert(
			fc.asyncProperty(fc.uint8Array({ maxLength: 4096 }), fc.uint8Array({ maxLength: 64 }), async (first, other) => {
				const md5 = await md5Hex(first);
				expect((await putFile("file:p", md5, first)).status).toBe(200);
				let stored = await getFile("file:p");
				expect(stored.md5).toBe(md5);
				expect(stored.bytes).toEqual(first);

				if ((await md5Hex(other)) !== md5) {
					expect((await putFile("file:p", md5, other)).status).toBe(400);
					stored = await getFile("file:p");
					expect(stored.bytes).toEqual(first);
				}
			}),
		);
	});
});
