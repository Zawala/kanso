// Isolation between accounts (after b4e2733's isolation suite). Three
// signed-up accounts share one Worker and use the same few document and file
// IDs, revisions and sealed values. Every answer one gets is the one its own
// library predicts, whatever the others did; one account's answers don't
// depend on anyone else's operations; R2 holds each account's files under its
// own prefix; and device revocation stays inside the account.

import { env } from "cloudflare:workers";
import fc from "fast-check";
import { beforeEach, describe, expect, it } from "vitest";
import { type Client, type Creds, type Device, callJson, client, newDevice, r2Keys, resetServer, signupBody } from "./helpers";

const runs = (n: number) => ({ numRuns: Math.min(n, Number(env.FAST_CHECK_RUNS)) });

const USERS = ["alice", "bob", "carol"] as const;
type UserIndex = 0 | 1 | 2;
const id22 = (c: string) => c.repeat(22);
const DOC_IDS = [`note:${id22("A")}`, `tag:${id22("B")}`, `file:${id22("C")}`, `file:${id22("D")}`] as const;
const FILE_IDS = [`file:${id22("C")}`, `file:${id22("D")}`] as const;

const rev = (n: number) => n.toString(16).padStart(32, "0");
/** The same (n, title) gives the same sealed text, for every account. */
const sealedFor = (n: number, title: string) => `S${n.toString(36)}_${title}`.padEnd(60, "x");

/** Fixed credentials, so two runs from an empty server are indistinguishable but for the epoch. */
function fixedCreds(i: number): { creds: Creds; device: Device } {
	const hex = (c: string, n: number) => c.repeat(n);
	const digit = String(i + 1);
	return {
		creds: {
			username: USERS[i],
			accountId: `a_${hex(digit, 32)}`,
			salt: `${digit}`.repeat(22),
			authKey: `${digit}`.repeat(42) + "w",
			wrappedKey: `${digit}`.repeat(98),
			recoveryAuthKey: `${"abc"[i]}`.repeat(42) + "w",
			recoveryWrappedKey: `${"abc"[i]}`.repeat(98),
			keyId: hex(digit, 16),
		},
		// The same device ID in every account, as a hostile client could choose.
		device: { id: `d_${"e".repeat(32)}`, proof: `${digit}`.repeat(42) + "A", sealedName: null },
	};
}

/** Signs the three accounts up on an empty server; their clients and epochs, in USERS order. */
async function people(): Promise<{ clients: Client[]; epochs: string[]; creds: Creds[] }> {
	await resetServer();
	const clients: Client[] = [];
	const epochs: string[] = [];
	const creds: Creds[] = [];
	for (const i of [0, 1, 2]) {
		const fixed = fixedCreds(i);
		const r = await callJson("/v2/auth/signup", { json: signupBody(fixed.creds, fixed.device), ip: `192.0.2.${i}` });
		expect(r.status, JSON.stringify(r.body)).toBe(201);
		clients.push(client(r.body.token));
		epochs.push(r.body.epoch);
		creds.push(fixed.creds);
	}
	return { clients, epochs, creds };
}

const genUser = fc.constantFrom<UserIndex>(0, 1, 2);
const genDoc = fc.record({
	id: fc.constantFrom(...DOC_IDS),
	// "neighbour": the base is another account's current revision of the same ID.
	base: fc.constantFrom("current", "current", "null", "stale", "neighbour"),
	deleted: fc.boolean(),
	title: fc.stringMatching(/^[A-Za-z0-9_-]{0,4}$/),
});
const genOp = fc.oneof(
	fc.record({ kind: fc.constant("push" as const), user: genUser, docs: fc.array(genDoc, { minLength: 1, maxLength: 3 }) }),
	fc.record({ kind: fc.constant("changes" as const), user: genUser, since: fc.nat({ max: 8 }), limit: fc.integer({ min: 1, max: 4 }) }),
	fc.record({ kind: fc.constant("put" as const), user: genUser, id: fc.constantFrom(...FILE_IDS), bytes: fc.uint8Array({ minLength: 41, maxLength: 64 }) }),
	fc.record({ kind: fc.constant("get" as const), user: genUser, id: fc.constantFrom(...FILE_IDS) }),
);
type Op = typeof genOp extends fc.Arbitrary<infer T> ? T : never;

interface ModelDoc {
	rev: string;
	deleted: boolean;
	sealed: string;
	seq: number;
}

/** One account's library as the protocol describes it, knowing nothing of anyone else's. */
class Library {
	docs = new Map<string, ModelDoc>();
	counter = 0;
	files = new Map<string, Uint8Array>();

	constructor(
		readonly epoch: string,
		readonly keyId: string,
	) {}

	feed(since: number, limit: number) {
		const after = [...this.docs.entries()]
			.map(([id, d]) => ({ id, rev: d.rev, deleted: d.deleted, sealed: d.sealed, seq: d.seq }))
			.filter((d) => d.seq > since)
			.sort((a, b) => a.seq - b.seq);
		const docs = after.slice(0, limit);
		return { docs, last_seq: docs.at(-1)?.seq ?? since, more: after.length >= limit, head_seq: this.counter, epoch: this.epoch, key_id: this.keyId };
	}
}

describe("accounts never observe each other", () => {
	beforeEach(() => resetServer());

	// Model-based: one Library per account. Every answer must be the one the
	// caller's own Library predicts; at the end each feed is exactly its
	// Library and R2 holds exactly each account's files under its own prefix.
	it("prop_every_answer_matches_the_callers_own_library", async () => {
		await fc.assert(
			fc.asyncProperty(fc.array(genOp, { minLength: 1, maxLength: 25 }), async (ops) => {
				const { clients, epochs, creds } = await people();
				const libraries = USERS.map((_, i) => new Library(epochs[i], creds[i].keyId));
				let revs = 0;
				for (const op of ops) {
					const me = libraries[op.user];
					const remote = clients[op.user];
					if (op.kind === "push") {
						const sent = op.docs.map((doc) => {
							revs += 1;
							const neighbour = libraries.find((l, i) => i !== op.user && l.docs.has(doc.id));
							const base = {
								current: me.docs.get(doc.id)?.rev ?? null,
								null: null,
								stale: rev(0),
								neighbour: neighbour?.docs.get(doc.id)?.rev ?? rev(0),
							}[doc.base];
							return { id: doc.id, rev: rev(revs), deleted: doc.deleted, sealed: sealedFor(revs, doc.title), base_rev: base };
						});
						const response = await remote.push(sent);
						expect(response.status).toBe(200);
						const tombstoned = new Map<string, boolean>();
						sent.forEach((doc, i) => {
							const current = me.docs.get(doc.id);
							const accepted = current ? doc.base_rev === current.rev : doc.base_rev === null;
							if (accepted) {
								me.counter += 1;
								me.docs.set(doc.id, { rev: doc.rev, deleted: doc.deleted, sealed: doc.sealed, seq: me.counter });
								if (doc.id.startsWith("file:")) tombstoned.set(doc.id, doc.deleted);
								expect(response.body.results[i]).toEqual({ id: doc.id, status: "ok", seq: me.counter });
							} else {
								expect(response.body.results[i]).toEqual({
									id: doc.id,
									status: "conflict",
									current: current ? { id: doc.id, rev: current.rev, deleted: current.deleted, sealed: current.sealed } : null,
								});
							}
						});
						for (const [id, deleted] of tombstoned) if (deleted) me.files.delete(id);
					} else if (op.kind === "changes") {
						const response = await remote.changes(op.since, op.limit);
						expect(response.status).toBe(200);
						expect(response.body).toEqual(me.feed(op.since, op.limit));
					} else if (op.kind === "put") {
						const response = await remote.putFile(op.id, op.bytes);
						expect(response.status).toBe(200);
						expect(response.body).toEqual({ ok: true, size: op.bytes.length });
						me.files.set(op.id, op.bytes);
					} else {
						const file = await remote.getFile(op.id);
						const expected = me.files.get(op.id);
						if (expected) {
							expect(file.status).toBe(200);
							expect(file.bytes).toEqual(expected);
						} else {
							expect(file.status).toBe(404);
						}
					}
				}

				for (const [i, remote] of clients.entries()) {
					expect((await remote.changes(0, 500)).body).toEqual(libraries[i].feed(0, 500));
					expect((await remote.account()).body.usage).toMatchObject({
						docs: [...libraries[i].docs.values()].filter((d) => !d.deleted).length,
						files: libraries[i].files.size,
					});
				}
				const prefixes = creds.map((c) => `accounts/${c.accountId}/`);
				for (const a of prefixes) for (const b of prefixes) if (a !== b) expect(b.startsWith(a)).toBe(false);
				const expectedKeys = USERS.flatMap((_, i) => [...libraries[i].files.keys()].map((id) => `${prefixes[i]}${id}`));
				expect(await r2Keys()).toEqual(expectedKeys.sort());
			}),
			runs(60),
		);
	});

	// Metamorphic (non-interference): what one account sees depends only on
	// what it did. Its operations alone, and the same operations interleaved
	// with everyone else's, get identical answers. Every account numbers
	// revisions and seals values the same way, so the others often push the
	// observer's exact IDs, revisions and sealed text.
	it("prop_an_accounts_answers_do_not_depend_on_anyone_elses_operations", async () => {
		const genSeq = fc.array(genOp, { maxLength: 12 });
		await fc.assert(
			fc.asyncProperty(genUser, genSeq, genSeq, fc.array(fc.boolean(), { maxLength: 24 }), async (observer, mine, theirs, order) => {
				const own = mine.map((op) => ({ ...op, user: observer }));
				const others = theirs.map((op) => ({ ...op, user: op.user === observer ? (((op.user + 1) % 3) as UserIndex) : op.user }));

				const alone = await replay(observer, own);
				const merged: Op[] = [];
				let [i, j] = [0, 0];
				for (const takeMine of order) {
					if (takeMine && i < own.length) merged.push(own[i++]);
					else if (j < others.length) merged.push(others[j++]);
				}
				merged.push(...own.slice(i), ...others.slice(j));
				expect(await replay(observer, merged)).toEqual(alone);
			}),
			runs(60),
		);
	});
});

/** Runs `ops` from an empty server; the observer's answers, in order, with its (random) epoch masked. */
async function replay(observer: UserIndex, ops: Op[]): Promise<unknown[]> {
	const { clients, epochs } = await people();
	// Each account's client remembers what it last learned of each ID, from its own answers only.
	const known = USERS.map(() => new Map<string, string | null>());
	const pushed = USERS.map(() => 0);
	const seen: unknown[] = [];
	const mask = (answer: { status: number; body: any }) => ({
		status: answer.status,
		body: answer.body?.epoch === epochs[observer] ? { ...answer.body, epoch: "<epoch>" } : answer.body,
	});
	for (const op of ops) {
		const remote = clients[op.user];
		let answer: unknown;
		if (op.kind === "push") {
			const sent = op.docs.map((doc) => {
				const n = ++pushed[op.user];
				const base = { current: known[op.user].get(doc.id) ?? null, null: null, stale: rev(0), neighbour: rev(1) }[doc.base];
				return { id: doc.id, rev: rev(n), deleted: doc.deleted, sealed: sealedFor(n, doc.title), base_rev: base };
			});
			const response = await remote.push(sent);
			for (const [k, result] of (response.body.results ?? []).entries()) {
				known[op.user].set(result.id, result.status === "ok" ? sent[k].rev : (result.current?.rev ?? null));
			}
			answer = mask(response);
		} else if (op.kind === "changes") {
			answer = mask(await remote.changes(op.since, op.limit));
		} else if (op.kind === "put") {
			answer = await remote.putFile(op.id, op.bytes);
		} else {
			answer = await remote.getFile(op.id);
		}
		if (op.user === observer) seen.push(answer);
	}
	seen.push(mask(await clients[observer].changes(0, 500)));
	seen.push((await clients[observer].account()).body.usage);
	return seen;
}

describe("devices", () => {
	beforeEach(() => resetServer());

	it("revoking a device ID touches only the caller's account, even when another account uses the same ID", async () => {
		const { clients, creds } = await people();
		const [alice, bob, carol] = clients;
		// Bob signs in a second Mac; Alice tries to sign it out by its ID.
		const bobsMac = newDevice();
		const second = await callJson("/v2/auth/login", { json: { username: "bob", auth_key: creds[1].authKey, device: { id: bobsMac.id, proof: bobsMac.proof } } });
		expect(second.status).toBe(200);
		expect((await alice.revoke(bobsMac.id)).status).toBe(404);
		expect((await client(second.body.token).account()).status).toBe(200);
		// Each account's device list holds only its own sessions.
		expect((await alice.devices()).body.devices.map((d: { id: string }) => d.id)).toEqual([`d_${"e".repeat(32)}`]);
		expect((await bob.devices()).body.devices).toHaveLength(2);
		// All three use d_eee…: Alice signing hers out leaves Bob's and Carol's.
		expect((await alice.revoke(`d_${"e".repeat(32)}`)).status).toBe(204);
		expect((await alice.account()).body.reason).toBe("signed_out");
		expect((await bob.account()).status).toBe(200);
		expect((await carol.account()).status).toBe(200);
		expect(await env.DB.prepare("SELECT count(*) AS n FROM sessions WHERE revoked_at IS NULL").first("n")).toBe(3);
	});
});
