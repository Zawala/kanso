// Push notifications (docs/ACCOUNTS.md › Push notifications): registering a
// token, which changes wake which devices, the 30 s coalescing, what clears a
// token, the exact APNs and FCM requests, and the AccountRoom alarm.
//
// Routes run against a stand-in AccountRoom namespace that only records what
// it is told to schedule, so no real alarm fires between a test's steps; the
// alarm itself is tested last, against the real room.

import { runDurableObjectAlarm, runInDurableObject } from "cloudflare:test";
import { env } from "cloudflare:workers";
import fc from "fast-check";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { AccountRoom } from "../src/account_room";
import { fromBase64url } from "../src/protocol";
import { PUSH_INTERVAL_MS, type Outcome, type PushTarget, type Send, deliverDue, newPushCache, pushSender, wakeAccount, wakeShare } from "../src/push";
import { runCron } from "../src/purge";
import type { Env } from "../src/env";
import { type Signed, another, first } from "./approval-helpers";
import { envelope, shareWith } from "./collab-helpers";
import { type Account, START, callJson, call, client, newCreds, newDevice, passwordBody, resetServer, sealed, signUp, testEnv, wireId, wireRev } from "./helpers";

const runs = (n: number) => ({ numRuns: Math.min(n, Number(env.FAST_CHECK_RUNS)) });
const DAY = 86_400_000;

// ------------------------------------------------------------------ helpers

const hex = (n: number): string => [...crypto.getRandomValues(new Uint8Array(n))].map((b) => b.toString(16).padStart(2, "0")).join("");
const apnsToken = (): string => hex(32);
const FCM_ALPHABET = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_:-";
const fcmToken = (): string => Array.from(crypto.getRandomValues(new Uint8Array(152)), (b) => FCM_ALPHABET[b % FCM_ALPHABET.length]).join("");

/** An AccountRoom namespace that records `schedule` calls and has no alarm. */
const scheduled: { accountId: string; at: number }[] = [];
let failSchedule = false;
const fakeRooms = {
	idFromName: (name: string) => name,
	get: (id: string) => ({
		schedule: async (accountId: string, at: number) => {
			if (failSchedule) throw new Error("room unreachable");
			scheduled.push({ accountId: accountId === id ? id : "?", at });
		},
	}),
} as unknown as DurableObjectNamespace<AccountRoom>;
const fake = (overrides: Partial<Env> = {}): Env => testEnv({ ACCOUNT_ROOM: fakeRooms, ...overrides });

interface Registered {
	platform: "apns" | "fcm";
	token: string;
	environment: "production" | "development";
}
const apns = (environment: "production" | "development" = "production"): Registered => ({ platform: "apns", token: apnsToken(), environment });
const fcm = (): Registered => ({ platform: "fcm", token: fcmToken(), environment: "production" });

const pushPath = (deviceId: string) => `/v2/account/devices/${deviceId}/push`;
const register = (s: Signed, body: unknown, id = s.device.id, now = START) => callJson(pushPath(id), { method: "PUT", token: s.token, json: body, env: fake(), now });
const unregister = (s: Signed, id = s.device.id) => callJson(pushPath(id), { method: "DELETE", token: s.token, env: fake() });

interface PushRow {
	push_platform: string | null;
	push_token: string | null;
	push_environment: string | null;
	push_updated_at: number | null;
	push_last_sent_at: number | null;
	push_due_at: number | null;
	revoked_at: number | null;
}
async function row(accountId: string, deviceId: string): Promise<PushRow> {
	return (await env.DB.prepare(
		`SELECT push_platform, push_token, push_environment, push_updated_at, push_last_sent_at, push_due_at, revoked_at
  FROM sessions WHERE account_id = ?1 AND device_id = ?2`,
	)
		.bind(accountId, deviceId)
		.first<PushRow>())!;
}
const tokenOf = async (a: Account, s: Signed): Promise<string | null> => (await row(a.creds.accountId, s.device.id)).push_token;
async function dueDevices(accountId?: string): Promise<string[]> {
	const { results } = await env.DB.prepare("SELECT device_id FROM sessions WHERE push_due_at IS NOT NULL AND (?1 IS NULL OR account_id = ?1) ORDER BY device_id")
		.bind(accountId ?? null)
		.all<{ device_id: string }>();
	return results.map((r) => r.device_id);
}

/** A sender that records what it is asked to send. */
function recorder(outcome: (target: PushTarget) => Outcome = () => "sent") {
	const sent: (PushTarget & { now: number })[] = [];
	const send: Send = async (target, now) => {
		sent.push({ ...target, now });
		return outcome(target);
	};
	return { send, sent };
}

/** A push of one new document by `s` at `now`; the result's status. */
async function change(s: Signed, now: number, docs = [{ id: wireId(), rev: wireRev(), deleted: false, sealed: sealed(64), base_rev: null as string | null }]) {
	const r = await client(s.token, { env: fake(), now }).push(docs);
	if (r.status !== 200) throw new Error(`push: ${r.status} ${JSON.stringify(r.body)}`);
	return r.body.results[0].status as string;
}

let alice: Account;
let a1: Signed;
let a2: Signed;
let a3: Signed;
let bob: Account;
let b1: Signed;

beforeEach(async () => {
	await resetServer();
	scheduled.length = 0;
	failSchedule = false;
	alice = await signUp("alice");
	a1 = first(alice);
	a2 = await another(alice);
	a3 = await another(alice);
	bob = await signUp("bob");
	b1 = first(bob);
});

afterEach(() => {
	vi.restoreAllMocks();
});

// ------------------------------------------------------------ registration

describe("registering a token", () => {
	it("stores it on the caller's session, 204, and DELETE clears it, 204 again when none is set", async () => {
		const reg = apns("development");
		expect((await register(a1, reg, a1.device.id, START + 5)).status).toBe(204);
		expect(await row(alice.creds.accountId, a1.device.id)).toMatchObject({
			push_platform: "apns",
			push_token: reg.token,
			push_environment: "development",
			push_updated_at: START + 5,
			push_due_at: null,
		});
		expect(await tokenOf(alice, a2)).toBeNull();
		expect((await unregister(a1)).status).toBe(204);
		expect(await row(alice.creds.accountId, a1.device.id)).toMatchObject({ push_platform: null, push_token: null, push_environment: null, push_updated_at: null });
		expect((await unregister(a1)).status).toBe(204);
		expect((await register(a1, fcm())).status).toBe(204);
		expect((await row(alice.creds.accountId, a1.device.id)).push_platform).toBe("fcm");
	});

	it("registering again replaces the token; the same token again is fine", async () => {
		const one = apns();
		expect((await register(a1, one)).status).toBe(204);
		expect((await register(a1, one)).status).toBe(204);
		const two = fcm();
		expect((await register(a1, two)).status).toBe(204);
		expect(await row(alice.creds.accountId, a1.device.id)).toMatchObject({ push_platform: "fcm", push_token: two.token, push_environment: "production" });
	});

	it("refuses a bad platform, environment, token shape or body with 400", async () => {
		const good = apns();
		const bad: unknown[] = [
			{ ...good, platform: "web" },
			{ ...good, platform: undefined },
			{ ...good, environment: "staging" },
			{ ...good, environment: undefined },
			{ ...fcm(), environment: "development" },
			{ ...good, token: undefined },
			{ ...good, token: 12 },
			{ ...good, token: good.token.toUpperCase() },
			{ ...good, token: good.token.slice(1) },
			{ ...good, token: hex(15) },
			{ ...good, token: hex(101) },
			{ ...good, token: `${good.token.slice(0, 62)}zz` },
			{ ...fcm(), token: fcmToken().slice(0, 99) },
			{ ...fcm(), token: `${fcmToken().slice(0, 150)}!!` },
			{ ...fcm(), token: "A".repeat(4097) },
			{ ...fcm(), token: good.token },
			[],
			"nope",
		];
		for (const body of bad) {
			const r = await register(a1, body);
			expect(r.status, JSON.stringify(body)).toBe(400);
			expect(r.body.error).toBe("bad_request");
		}
		expect(await tokenOf(alice, a1)).toBeNull();
		const notJson = await callJson(pushPath(a1.device.id), { method: "PUT", token: a1.token, body: "{", headers: { "content-type": "application/json" }, env: fake() });
		expect(notJson.status).toBe(400);
		expect((await callJson(pushPath(a1.device.id), { method: "PUT", token: a1.token, body: "{}", env: fake() })).status).toBe(415);
	});

	it("accepts the largest and smallest valid tokens", async () => {
		for (const reg of [
			{ platform: "apns", token: hex(16), environment: "production" },
			{ platform: "apns", token: hex(100), environment: "development" },
			{ platform: "fcm", token: "a".repeat(100), environment: "production" },
			{ platform: "fcm", token: "a".repeat(4096), environment: "production" },
		]) {
			expect((await register(a1, reg)).status, `${reg.platform} ${reg.token.length}`).toBe(204);
		}
	});

	it("property: a token is accepted exactly when it has its platform's shape", async () => {
		await fc.assert(
			fc.asyncProperty(fc.constantFrom("apns", "fcm"), fc.stringMatching(/^[0-9a-fA-F_:!-]{0,210}$/), fc.integer({ min: 0, max: 1 }), async (platform, text, big) => {
				const token = platform === "fcm" && big === 1 ? text.repeat(30).slice(0, 4100) : text;
				const valid = platform === "apns" ? /^(?:[0-9a-f]{2}){16,100}$/.test(token) : /^[A-Za-z0-9_:-]{100,4096}$/.test(token);
				const r = await register(a1, { platform, token, environment: "production" });
				expect(r.status).toBe(valid ? 204 : 400);
			}),
			runs(40),
		);
	});

	it("only the caller's own device: another device of the account, another account's, and a malformed ID are refused", async () => {
		const reg = apns();
		expect((await register(a1, reg, a2.device.id)).status).toBe(404);
		expect((await register(a1, reg, b1.device.id)).status).toBe(404);
		expect((await register(a1, reg, `d_${hex(16)}`)).status).toBe(404);
		expect((await register(a1, reg, "d_nope")).status).toBe(400);
		expect((await unregister(a1, a2.device.id)).status).toBe(404);
		expect((await unregister(a1, b1.device.id)).status).toBe(404);
		expect((await unregister(a1, "d_nope")).status).toBe(400);
		await register(a2, apns());
		expect(await tokenOf(alice, a2)).not.toBeNull();
		// Refused calls changed nothing.
		await unregister(a1, a2.device.id);
		expect(await tokenOf(alice, a2)).not.toBeNull();
		expect(await tokenOf(alice, a1)).toBeNull();
	});

	it("needs a live session; other methods are 405", async () => {
		expect((await callJson(pushPath(a1.device.id), { method: "PUT", json: apns(), env: fake() })).status).toBe(401);
		await client(a2.token).logout();
		expect((await register(a2, apns())).status).toBe(401);
		expect((await unregister(a2)).status).toBe(401);
		expect((await callJson(pushPath(a1.device.id), { token: a1.token, env: fake() })).status).toBe(405);
	});

	it("moves a token another session holds, in one step; a token lives on one session", async () => {
		const reg = apns();
		await register(a1, reg);
		await register(b1, reg);
		expect(await tokenOf(alice, a1)).toBeNull();
		expect(await tokenOf(bob, b1)).toBe(reg.token);
		await register(a2, reg);
		expect(await tokenOf(bob, b1)).toBeNull();
		expect(await tokenOf(alice, a2)).toBe(reg.token);
		const holders = await env.DB.prepare("SELECT count(*) AS n FROM sessions WHERE push_token = ?1").bind(reg.token).first<{ n: number }>();
		expect(holders!.n).toBe(1);
	});

	it("a moved token takes its pending wake with it: the old session is no longer due", async () => {
		const reg = apns();
		await register(a1, reg);
		await register(a2, apns());
		await change(a2, START);
		expect(await dueDevices(alice.creds.accountId)).toEqual([a1.device.id]);
		await register(a3, reg);
		expect(await dueDevices(alice.creds.accountId)).toEqual([]);
	});

	it("property: whatever the registrations, no token is on two sessions and every stored row holds its last registration", async () => {
		const sessions = () => [a1, a2, a3, b1];
		await fc.assert(
			fc.asyncProperty(fc.array(fc.record({ who: fc.nat(3), token: fc.nat(2), off: fc.boolean() }), { minLength: 1, maxLength: 10 }), async (ops) => {
				await env.DB.prepare("UPDATE sessions SET push_platform = NULL, push_token = NULL, push_environment = NULL, push_updated_at = NULL").run();
				const tokens = [apnsToken(), apnsToken(), apnsToken()];
				const model = new Map<number, number>();
				for (const op of ops) {
					const s = sessions()[op.who];
					if (op.off) {
						await unregister(s);
						model.delete(op.who);
					} else {
						await register(s, { platform: "apns", token: tokens[op.token], environment: "production" });
						for (const [who, t] of model) if (t === op.token) model.delete(who);
						model.set(op.who, op.token);
					}
				}
				for (const [i, s] of sessions().entries()) {
					const owner = i === 3 ? bob : alice;
					expect(await tokenOf(owner, s)).toBe(model.has(i) ? tokens[model.get(i)!] : null);
				}
			}),
			runs(25),
		);
	});
});

// ----------------------------------------------------------------- wakes

describe("which changes wake which devices", () => {
	beforeEach(async () => {
		await register(a1, apns());
		await register(a2, apns());
		await register(a3, fcm());
		await register(b1, apns());
	});

	it("a push from a1 marks a2 and a3 due, never a1, never account B", async () => {
		expect(await change(a1, START + 1000)).toBe("ok");
		expect(await dueDevices()).toEqual([a2.device.id, a3.device.id].sort());
		for (const s of [a2, a3]) expect((await row(alice.creds.accountId, s.device.id)).push_due_at).toBe(START + 1000);
		expect(scheduled).toEqual([{ accountId: alice.creds.accountId, at: START + 1000 }]);
		const sender = recorder();
		const delivery = await deliverDue(fake(), alice.creds.accountId, START + 1000, sender.send);
		expect(delivery).toMatchObject({ claimed: 2, sent: 2, next: null });
		expect(sender.sent.map((s) => s.deviceId).sort()).toEqual([a2.device.id, a3.device.id].sort());
		expect(sender.sent.find((s) => s.deviceId === a3.device.id)).toMatchObject({ platform: "fcm", environment: "production" });
		expect(await dueDevices()).toEqual([]);
	});

	it("never wakes a device without a token or a revoked one, and not the other way round for the sender", async () => {
		await unregister(a2);
		await call(`/v2/account/devices/${a3.device.id}`, { method: "DELETE", token: a1.token, env: fake() });
		expect(await change(a1, START)).toBe("ok");
		expect(await dueDevices()).toEqual([]);
		expect(scheduled).toEqual([]);
		// a1 is woken by a push from a device that has a token.
		await register(a2, apns());
		await change(a2, START);
		expect(await dueDevices()).toEqual([a1.device.id]);
	});

	it("a push whose documents all conflict, or that carries none, wakes nobody", async () => {
		const doc = { id: wireId(), rev: wireRev(), deleted: false, sealed: sealed(64), base_rev: null as string | null };
		expect(await change(a1, START, [doc])).toBe("ok");
		await deliverDue(fake(), alice.creds.accountId, START, recorder().send);
		scheduled.length = 0;
		expect(await change(a1, START + DAY, [{ ...doc, rev: wireRev() }])).toBe("conflict");
		expect(await dueDevices()).toEqual([]);
		expect((await client(a1.token, { env: fake(), now: START + DAY }).push([])).status).toBe(200);
		expect(await dueDevices()).toEqual([]);
		expect(scheduled).toEqual([]);
		// One ok among conflicts is enough.
		const fresh = { id: wireId(), rev: wireRev(), deleted: false, sealed: sealed(64), base_rev: null as string | null };
		const r = await client(a1.token, { env: fake(), now: START + DAY }).push([{ ...doc, rev: wireRev() }, fresh]);
		expect(r.body.results.map((x: any) => x.status)).toEqual(["conflict", "ok"]);
		expect(await dueDevices()).toEqual([a2.device.id, a3.device.id].sort());
	});

	it("a refused push (bad body) wakes nobody", async () => {
		const r = await callJson("/v2/push", { token: a1.token, json: { docs: [{ id: "nope" }] }, env: fake() });
		expect(r.status).toBe(400);
		expect(await dueDevices()).toEqual([]);
	});

	it("a disabled account is not woken", async () => {
		await env.DB.prepare("UPDATE accounts SET disabled_at = ?2 WHERE id = ?1").bind(alice.creds.accountId, START).run();
		const bobDoc = await change(b1, START);
		expect(bobDoc).toBe("ok");
		expect(await dueDevices()).toEqual([]);
		await env.DB.prepare("UPDATE accounts SET disabled_at = NULL WHERE id = ?1").bind(alice.creds.accountId).run();
		await change(b1, START);
		expect(await dueDevices()).toEqual([]);
		// A device marked due before its account was disabled is not sent to either.
		await change(a1, START);
		await env.DB.prepare("UPDATE accounts SET disabled_at = ?2 WHERE id = ?1").bind(alice.creds.accountId, START).run();
		const sender = recorder();
		expect(await deliverDue(fake(), alice.creds.accountId, START + DAY, sender.send)).toMatchObject({ claimed: 0, next: null });
		expect(sender.sent).toEqual([]);
	});

	it("a push wakes only devices of its own account, however many accounts have tokens", async () => {
		await change(b1, START);
		expect(await dueDevices()).toEqual([]);
		await register(await another(bob), apns());
		await change(b1, START);
		expect(await dueDevices(bob.creds.accountId)).toHaveLength(1);
		expect(await dueDevices(alice.creds.accountId)).toEqual([]);
	});

	it("a failing room never fails or changes the push; the next change re-schedules", async () => {
		failSchedule = true;
		const log = vi.spyOn(console, "error").mockImplementation(() => {});
		expect(await change(a1, START)).toBe("ok");
		expect(await dueDevices(alice.creds.accountId)).toHaveLength(2);
		expect(log.mock.calls.map((c) => JSON.parse(String(c[0])))).toContainEqual({ route: "push", error: "schedule_failed", kind: "Error" });
		expect(JSON.stringify(log.mock.calls)).not.toContain(alice.creds.accountId);
		failSchedule = false;
		await change(a1, START + 10);
		// The earlier due time is what the room is told.
		expect(scheduled).toEqual([{ accountId: alice.creds.accountId, at: START }]);
	});

	it("a D1 failure while marking is logged and never thrown", async () => {
		const log = vi.spyOn(console, "error").mockImplementation(() => {});
		const broken = { prepare: () => { throw new Error("D1 down"); } } as unknown as D1Database;
		await wakeAccount(fake({ DB: broken }), alice.creds.accountId, a1.device.id, START);
		await wakeShare(fake({ DB: broken }), "s", { accountId: alice.creds.accountId, deviceId: a1.device.id }, START);
		expect(log.mock.calls.map((c) => JSON.parse(String(c[0])))).toEqual([
			{ route: "push", error: "wake_failed", kind: "Error" },
			{ route: "push", error: "wake_failed", kind: "Error" },
		]);
	});
});

describe("shared notes wake the other members' devices", () => {
	it("an update wakes every other device of every member account, not the sender's device, not a non-member", async () => {
		const carol = await signUp("carol");
		const c1 = first(carol);
		const a2b = a2;
		for (const [s, reg] of [[a1, apns()], [a2b, apns()], [b1, apns()], [c1, apns()]] as const) await register(s, reg);
		const b2 = await another(bob);
		await register(b2, fcm());
		const id = await shareWith(alice, [{ account: bob, role: "edit" }]);
		const u = await envelope({ kind: "u", epoch: 1, shareId: id, by: alice, device: a1.device.id });
		const r = await callJson(`/v2/collab/shares/${id}/log`, {
			token: a1.token,
			method: "POST",
			body: u,
			headers: { "content-type": "application/octet-stream" },
			env: fake(),
			now: START + 50,
		});
		expect(r.status).toBe(200);
		expect(await dueDevices()).toEqual([a2b.device.id, b1.device.id, b2.device.id].sort());
		expect(scheduled.map((s) => s.accountId).sort()).toEqual([alice.creds.accountId, bob.creds.accountId].sort());
		for (const s of scheduled) expect(s.at).toBe(START + 50);
		// Bob's device writes: Alice's a1 and a2, and bob's other device, but not b1.
		await env.DB.prepare("UPDATE sessions SET push_due_at = NULL").run();
		const v = await envelope({ kind: "u", epoch: 1, shareId: id, by: bob, device: b1.device.id });
		await callJson(`/v2/collab/shares/${id}/log`, {
			token: b1.token,
			method: "POST",
			body: v,
			headers: { "content-type": "application/octet-stream" },
			env: fake(),
			now: START + 60,
		});
		expect(await dueDevices()).toEqual([a1.device.id, a2b.device.id, b2.device.id].sort());
	});

	it("an invited member who has not accepted, a refused append and a snapshot wake nobody", async () => {
		await register(a2, apns());
		await register(b1, apns());
		const id = await shareWith(alice, [{ account: bob, role: "view" }], false);
		const send = (token: string, bytes: Uint8Array) =>
			callJson(`/v2/collab/shares/${id}/log`, { token, method: "POST", body: bytes, headers: { "content-type": "application/octet-stream" }, env: fake() });
		expect((await send(a1.token, await envelope({ kind: "u", epoch: 1, shareId: id, by: alice, device: a1.device.id }))).status).toBe(200);
		// Bob is only invited: nothing for him; Alice's own other device is a member's device.
		expect(await dueDevices()).toEqual([a2.device.id]);
		await env.DB.prepare("UPDATE sessions SET push_due_at = NULL").run();
		await call(`/v2/collab/shares/${id}/accept`, { token: b1.token, method: "POST", env: fake() });
		// A view member can't write: refused, nobody woken.
		const refused = await send(b1.token, await envelope({ kind: "u", epoch: 1, shareId: id, by: bob, device: b1.device.id }));
		expect(refused.status).toBe(403);
		expect(await dueDevices()).toEqual([]);
	});

	it("a disabled member account is not woken", async () => {
		await register(b1, apns());
		const id = await shareWith(alice, [{ account: bob, role: "edit" }]);
		await env.DB.prepare("UPDATE accounts SET disabled_at = ?2 WHERE id = ?1").bind(bob.creds.accountId, START).run();
		await callJson(`/v2/collab/shares/${id}/log`, {
			token: a1.token,
			method: "POST",
			body: await envelope({ kind: "u", epoch: 1, shareId: id, by: alice, device: a1.device.id }),
			headers: { "content-type": "application/octet-stream" },
			env: fake(),
		});
		expect(await dueDevices()).toEqual([]);
	});
});

// ------------------------------------------------------------- coalescing

describe("coalescing", () => {
	beforeEach(async () => {
		await register(a1, apns());
		await register(a2, apns());
	});

	it("the first change is due at once; later ones inside the window keep the first due time; the next after a send waits 30 s", async () => {
		await change(a1, START);
		await change(a1, START + 5000);
		expect((await row(alice.creds.accountId, a2.device.id)).push_due_at).toBe(START);
		const first = recorder();
		await deliverDue(fake(), alice.creds.accountId, START, first.send);
		expect((await row(alice.creds.accountId, a2.device.id)).push_last_sent_at).toBe(START);
		await change(a1, START + 6000);
		expect((await row(alice.creds.accountId, a2.device.id)).push_due_at).toBe(START + PUSH_INTERVAL_MS);
		// Too early: nothing is claimed, and the answer says when to come back.
		const early = recorder();
		expect(await deliverDue(fake(), alice.creds.accountId, START + PUSH_INTERVAL_MS - 1, early.send)).toMatchObject({ claimed: 0, next: START + PUSH_INTERVAL_MS });
		expect(early.sent).toEqual([]);
		const on = recorder();
		expect(await deliverDue(fake(), alice.creds.accountId, START + PUSH_INTERVAL_MS, on.send)).toMatchObject({ claimed: 1, next: null });
		expect(on.sent.map((s) => s.deviceId)).toEqual([a2.device.id]);
	});

	it("a change long after the last send is due at once", async () => {
		await change(a1, START);
		await deliverDue(fake(), alice.creds.accountId, START, recorder().send);
		await change(a1, START + DAY);
		expect((await row(alice.creds.accountId, a2.device.id)).push_due_at).toBe(START + DAY);
		expect(scheduled.at(-1)).toEqual({ accountId: alice.creds.accountId, at: START + DAY });
	});

	it("two alarms racing claim each device once", async () => {
		await change(a1, START);
		const one = recorder();
		const two = recorder();
		await Promise.all([deliverDue(fake(), alice.creds.accountId, START, one.send), deliverDue(fake(), alice.creds.accountId, START, two.send)]);
		expect(one.sent.length + two.sent.length).toBe(1);
	});

	it("a send that fails or is unconfigured is logged, not thrown, and not retried; a throwing sender is the caller's bug but a real sender never throws", async () => {
		const log = vi.spyOn(console, "error").mockImplementation(() => {});
		await register(a3, fcm());
		await change(a1, START);
		const outcomes = new Map<string, Outcome>([
			[a2.device.id, "failed"],
			[a3.device.id, "unconfigured"],
		]);
		const sender = recorder((t) => outcomes.get(t.deviceId)!);
		const d = await deliverDue(fake(), alice.creds.accountId, START, sender.send);
		expect(d).toMatchObject({ claimed: 2, failed: 1, unconfigured: 1, sent: 0, next: null });
		expect(log.mock.calls.map((c) => JSON.parse(String(c[0])))).toEqual([{ route: "push", error: "unconfigured", platform: "fcm" }]);
		expect(await dueDevices()).toEqual([]);
		expect((await row(alice.creds.accountId, a2.device.id)).push_last_sent_at).toBe(START);
	});

	it("sends to every claimed device though some are slow, at most four at a time", async () => {
		const extra: Signed[] = [];
		for (let i = 0; i < 6; i++) {
			const s = await another(alice);
			await register(s, apns());
			extra.push(s);
		}
		await change(a1, START);
		let inFlight = 0;
		let peak = 0;
		const send: Send = async () => {
			inFlight += 1;
			peak = Math.max(peak, inFlight);
			await new Promise((r) => setTimeout(r, 5));
			inFlight -= 1;
			return "sent";
		};
		expect(await deliverDue(fake(), alice.creds.accountId, START, send)).toMatchObject({ claimed: 7, sent: 7 });
		expect(peak).toBeLessThanOrEqual(4);
		expect(peak).toBeGreaterThan(1);
	});
});

// ----------------------------------------------------------------- cleanup

describe("a token is cleared when its session ends", () => {
	let tokened: Signed;
	beforeEach(async () => {
		tokened = await another(alice);
		await register(tokened, apns());
		await register(a1, apns());
	});

	it("on sign-out, keeping the last send time", async () => {
		await env.DB.prepare("UPDATE sessions SET push_last_sent_at = 77 WHERE device_id = ?1").bind(tokened.device.id).run();
		await register(a2, apns());
		await change(a1, START);
		expect((await call("/v2/auth/logout", { method: "POST", token: tokened.token, env: fake() })).status).toBe(204);
		expect(await row(alice.creds.accountId, tokened.device.id)).toMatchObject({ push_token: null, push_platform: null, push_environment: null, push_updated_at: null, push_due_at: null, push_last_sent_at: 77 });
		expect(await tokenOf(alice, a1)).not.toBeNull();
	});

	it("on revoke from another device", async () => {
		expect((await call(`/v2/account/devices/${tokened.device.id}`, { method: "DELETE", token: a1.token, env: fake() })).status).toBe(204);
		expect(await tokenOf(alice, tokened)).toBeNull();
	});

	it("on revoke of itself, and a token can't be registered on a revoked session", async () => {
		await call(`/v2/account/devices/${tokened.device.id}`, { method: "DELETE", token: tokened.token, env: fake() });
		expect(await tokenOf(alice, tokened)).toBeNull();
		expect((await register(tokened, apns())).status).toBe(401);
		expect(await tokenOf(alice, tokened)).toBeNull();
	});

	it("on expiry found by the next request", async () => {
		const r = await callJson("/v2/account", { token: tokened.token, now: START + 181 * DAY, env: fake() });
		expect(r.status).toBe(401);
		expect(r.body.reason).toBe("expired");
		expect(await tokenOf(alice, tokened)).toBeNull();
	});

	it("on expiry found by the cron", async () => {
		await env.DB.prepare("UPDATE sessions SET last_seen_at = ?1 WHERE device_id = ?2").bind(START - 181 * DAY, tokened.device.id).run();
		await runCron(testEnv(), START);
		expect(await row(alice.creds.accountId, tokened.device.id)).toMatchObject({ push_token: null, revoked_at: START });
		expect(await tokenOf(alice, a1)).not.toBeNull();
	});

	it("on a password change, for every other session; the changing device keeps its own", async () => {
		const r = await callJson("/v2/account/password", { token: a1.token, json: passwordBody(alice.creds.authKey, newCreds("alice")), env: fake() });
		expect(r.status).toBe(200);
		expect(await tokenOf(alice, tokened)).toBeNull();
		expect(await tokenOf(alice, a1)).not.toBeNull();
	});

	it("on a recovery, for every session", async () => {
		const next = newCreds("alice");
		const d = newDevice();
		const r = await callJson("/v2/auth/recover/finish", {
			env: fake(),
			json: {
				username: "alice",
				recovery_auth_key: alice.creds.recoveryAuthKey,
				kdf: { alg: "argon2id", version: 19, m_kib: 1_048_576, t: 3, p: 4 },
				salt: next.salt,
				auth_key: next.authKey,
				wrapped_key: next.wrappedKey,
				recovery_auth_key_new: next.recoveryAuthKey,
				recovery_wrapped_key_new: next.recoveryWrappedKey,
				device: { id: d.id, sealed_name: d.sealedName, proof: d.proof },
			},
		});
		expect(r.status).toBe(200);
		expect(await tokenOf(alice, tokened)).toBeNull();
		expect(await tokenOf(alice, a1)).toBeNull();
	});

	it("when the same device signs in again (a new session token), including after a sign-out", async () => {
		const login = (device: typeof tokened.device) =>
			callJson("/v2/auth/login", {
				env: fake(),
				json: { username: "alice", auth_key: alice.creds.authKey, device: { id: device.id, proof: device.proof } },
			});
		expect((await login(tokened.device)).status).toBe(200);
		expect(await tokenOf(alice, tokened)).toBeNull();
		// A brand-new token registers again, and a second login clears it again.
		const next = (await login(tokened.device)).body.token as string;
		await register({ token: next, device: tokened.device }, apns());
		expect(await tokenOf(alice, tokened)).not.toBeNull();
		await login(tokened.device);
		expect(await tokenOf(alice, tokened)).toBeNull();
	});

	it("on eviction by the eleventh device", async () => {
		await env.DB.prepare("UPDATE sessions SET last_seen_at = last_seen_at - 1 WHERE device_id = ?1").bind(tokened.device.id).run();
		await env.DB.prepare("DELETE FROM limits").run();
		for (let i = 0; i < 9; i++) await another(alice);
		expect(await tokenOf(alice, tokened)).toBeNull();
		expect((await row(alice.creds.accountId, tokened.device.id)).revoked_at).not.toBeNull();
	});

	it("DELETE on the route, and not by a sealed-name change or a last-seen update", async () => {
		await callJson("/v2/account/device", { method: "PUT", token: tokened.token, json: { sealed_name: "A".repeat(60) }, env: fake() });
		await callJson("/v2/account", { token: tokened.token, now: START + 2 * 3600_000, env: fake() });
		expect(await tokenOf(alice, tokened)).not.toBeNull();
		await unregister(tokened);
		expect(await tokenOf(alice, tokened)).toBeNull();
	});

	it("a revoked device isn't delivered to even if it was already due", async () => {
		await change(a1, START);
		expect(await dueDevices()).toContain(tokened.device.id);
		await call(`/v2/account/devices/${tokened.device.id}`, { method: "DELETE", token: a1.token, env: fake() });
		const sender = recorder();
		await deliverDue(fake(), alice.creds.accountId, START, sender.send);
		expect(sender.sent.map((s) => s.deviceId)).not.toContain(tokened.device.id);
	});
});

describe("a token the provider reports gone is cleared", () => {
	const cfg = async () => ({ ...(await apnsEnv()), FCM_SERVICE_ACCOUNT: ACCOUNT((await testKeys()).rsaPem) });

	it("APNs 410, APNs 400 BadDeviceToken and FCM UNREGISTERED clear exactly that token; other refusals keep it", async () => {
		const log = vi.spyOn(console, "error").mockImplementation(() => {});
		const cases: [string, Registered, Response, boolean][] = [
			["apns 410", apns(), new Response('{"reason":"Unregistered"}', { status: 410 }), true],
			["apns 400 BadDeviceToken", apns("development"), new Response('{"reason":"BadDeviceToken"}', { status: 400 }), true],
			["apns 400 other", apns(), new Response('{"reason":"PayloadEmpty"}', { status: 400 }), false],
			["apns 429", apns(), new Response('{"reason":"TooManyRequests"}', { status: 429 }), false],
			["apns 503", apns(), new Response("", { status: 503 }), false],
			["apns 200", apns(), new Response(null, { status: 200 }), false],
			["fcm unregistered", fcm(), new Response(JSON.stringify({ error: { code: 404, status: "NOT_FOUND", details: [{ "@type": "x", errorCode: "UNREGISTERED" }] } }), { status: 404 }), true],
			["fcm other 404", fcm(), new Response(JSON.stringify({ error: { code: 404, status: "NOT_FOUND", details: [{ errorCode: "SOMETHING" }] } }), { status: 404 }), false],
			["fcm 500", fcm(), new Response("oops", { status: 500 }), false],
			["fcm 200", fcm(), new Response("{}", { status: 200 }), false],
		];
		for (const [name, reg, answer, cleared] of cases) {
			await register(a2, reg);
			await change(a1, START);
			const fetcher = async (url: string) => (url.includes("oauth2") ? new Response('{"access_token":"t","expires_in":3600}') : answer.clone());
			const env2 = await cfg();
			await deliverDue(env2, alice.creds.accountId, START, pushSender(env2, fetcher, newPushCache()));
			expect(await tokenOf(alice, a2), name).toBe(cleared ? null : reg.token);
			await env.DB.prepare("UPDATE sessions SET push_last_sent_at = NULL").run();
		}
		// Neither a token nor a push body ever reaches the log.
		expect(JSON.stringify(log.mock.calls)).not.toMatch(/[0-9a-f]{64}/);
	});

	it("not cleared when the device registered a different token while the push was in flight", async () => {
		await register(a2, apns());
		await change(a1, START);
		const replacement = apns();
		const fetcher = async (): Promise<Response> => {
			expect((await register(a2, replacement)).status).toBe(204);
			return new Response('{"reason":"Unregistered"}', { status: 410 });
		};
		const env2 = await cfg();
		const d = await deliverDue(env2, alice.creds.accountId, START, pushSender(env2, fetcher, newPushCache()));
		expect(d.gone).toBe(1);
		expect(await tokenOf(alice, a2)).toBe(replacement.token);
	});

	it("a sender that throws is logged and counts as failed; the token stays", async () => {
		const log = vi.spyOn(console, "error").mockImplementation(() => {});
		const reg = apns();
		await register(a2, reg);
		await change(a1, START);
		const env2 = await cfg();
		const d = await deliverDue(env2, alice.creds.accountId, START, pushSender(env2, async () => Promise.reject(new TypeError("network")), newPushCache()));
		expect(d).toMatchObject({ failed: 1, gone: 0 });
		expect(await tokenOf(alice, a2)).toBe(reg.token);
		expect(log.mock.calls.map((c) => JSON.parse(String(c[0])))).toContainEqual({ route: "push", error: "send_failed", platform: "apns", kind: "TypeError" });
	});
});

// ----------------------------------------------------------------- payloads

const encoder = new TextEncoder();
const pemOf = async (key: CryptoKey): Promise<string> => {
	const der = new Uint8Array((await crypto.subtle.exportKey("pkcs8", key)) as ArrayBuffer);
	const text = btoa(String.fromCharCode(...der));
	return `-----BEGIN PRIVATE KEY-----\n${text.match(/.{1,64}/g)!.join("\n")}\n-----END PRIVATE KEY-----\n`;
};

let keyCache: Promise<{ ec: CryptoKeyPair; ecPem: string; rsa: CryptoKeyPair; rsaPem: string }> | undefined;
function testKeys() {
	keyCache ??= (async () => {
		const ec = (await crypto.subtle.generateKey({ name: "ECDSA", namedCurve: "P-256" }, true, ["sign", "verify"])) as CryptoKeyPair;
		const rsa = (await crypto.subtle.generateKey(
			{ name: "RSASSA-PKCS1-v1_5", modulusLength: 2048, publicExponent: new Uint8Array([1, 0, 1]), hash: "SHA-256" },
			true,
			["sign", "verify"],
		)) as CryptoKeyPair;
		return { ec, ecPem: await pemOf(ec.privateKey), rsa, rsaPem: await pemOf(rsa.privateKey) };
	})();
	return keyCache;
}

async function apnsEnv(): Promise<Env> {
	return fake({ APNS_KEY_P8: (await testKeys()).ecPem, APNS_KEY_ID: "ABC123DEFG", APNS_TEAM_ID: "TEAM123456", APNS_TOPIC: "app.kanso.Kanso" });
}
const ACCOUNT = (private_key: string, extra: object = {}) =>
	JSON.stringify({ type: "service_account", project_id: "kanso-test", client_email: "push@kanso-test.iam.gserviceaccount.com", private_key, token_uri: "https://oauth2.googleapis.com/token", ...extra });
async function fcmEnv(extra: object = {}): Promise<Env> {
	return fake({ FCM_SERVICE_ACCOUNT: ACCOUNT((await testKeys()).rsaPem, extra) });
}

interface Seen {
	url: string;
	init: RequestInit;
}
/** A fetch that records requests and answers by `answer`. */
function fetching(answer: (url: string, init: RequestInit, n: number) => Response) {
	const seen: Seen[] = [];
	const fetcher = async (url: string, init: RequestInit): Promise<Response> => {
		seen.push({ url, init });
		return answer(url, init, seen.length);
	};
	return { seen, fetcher };
}

const decodeJwt = (jwt: string) => {
	const [h, c, s] = jwt.split(".");
	const json = (part: string) => JSON.parse(new TextDecoder().decode(fromBase64url(part)!));
	return { header: json(h), claims: json(c), signed: encoder.encode(`${h}.${c}`), signature: fromBase64url(s)! };
};

const target = (reg: Registered, deviceId = "d_x"): PushTarget => ({ deviceId, ...reg });

describe("APNs requests", () => {
	it("posts the exact request to the production host, and to the sandbox for a development token", async () => {
		const keys = await testKeys();
		const e = await apnsEnv();
		const { seen, fetcher } = fetching(() => new Response(null, { status: 200 }));
		const send = pushSender(e, fetcher, newPushCache());
		const now = 1_800_100_000_000;
		const prod = apns();
		const dev = apns("development");
		expect(await send(target(prod), now)).toBe("sent");
		expect(await send(target(dev), now + 1500)).toBe("sent");
		expect(seen).toHaveLength(2);
		expect(seen[0].url).toBe(`https://api.push.apple.com/3/device/${prod.token}`);
		expect(seen[1].url).toBe(`https://api.sandbox.push.apple.com/3/device/${dev.token}`);
		for (const [i, { init }] of seen.entries()) {
			expect(init.method).toBe("POST");
			expect(init.body).toBe('{"aps":{"content-available":1}}');
			const headers = { ...(init.headers as Record<string, string>) };
			const authorization = headers.authorization;
			delete headers.authorization;
			expect(headers).toEqual({
				"apns-push-type": "background",
				"apns-priority": "5",
				"apns-topic": "app.kanso.Kanso",
				"apns-expiration": String(Math.floor((now + (i === 0 ? 0 : 1500)) / 1000) + 3600),
				"apns-collapse-id": "sync",
				"content-type": "application/json",
			});
			expect(authorization).toMatch(/^bearer [A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+$/);
			const jwt = decodeJwt(authorization.slice(7));
			expect(jwt.header).toEqual({ alg: "ES256", kid: "ABC123DEFG" });
			expect(jwt.claims).toEqual({ iss: "TEAM123456", iat: Math.floor(now / 1000) });
			expect(jwt.signature).toHaveLength(64);
			expect(await crypto.subtle.verify({ name: "ECDSA", hash: "SHA-256" }, keys.ec.publicKey, jwt.signature, jwt.signed)).toBe(true);
		}
		// Nothing identifying travels in the request, only the token in the path.
		const dump = JSON.stringify(seen);
		expect(dump).not.toContain("a_");
		expect(dump).not.toContain("alice");
	});

	it("caches the provider token for 40 minutes, then mints a new one; ExpiredProviderToken drops it", async () => {
		const e = await apnsEnv();
		const { seen, fetcher } = fetching(() => new Response(null, { status: 200 }));
		const send = pushSender(e, fetcher, newPushCache());
		const bearer = (i: number) => (seen[i].init.headers as Record<string, string>).authorization;
		const t0 = 1_800_100_000_000;
		await send(target(apns()), t0);
		await send(target(apns()), t0 + 39 * 60_000 + 59_000);
		expect(bearer(1)).toBe(bearer(0));
		await send(target(apns()), t0 + 40 * 60_000);
		expect(bearer(2)).not.toBe(bearer(0));
		expect(decodeJwt(bearer(2).slice(7)).claims.iat).toBe(Math.floor((t0 + 40 * 60_000) / 1000));
		await send(target(apns()), t0 + 40 * 60_000 + 1000);
		expect(bearer(3)).toBe(bearer(2));

		const expired = fetching((_u, _i, n) => (n === 1 ? new Response('{"reason":"ExpiredProviderToken"}', { status: 403 }) : new Response(null, { status: 200 })));
		const resend = pushSender(e, expired.fetcher, newPushCache());
		expect(await resend(target(apns()), t0)).toBe("failed");
		expect(await resend(target(apns()), t0 + 1000)).toBe("sent");
		const tokens = expired.seen.map((s) => (s.init.headers as Record<string, string>).authorization);
		expect(tokens[1]).not.toBe(tokens[0]);
		expect(decodeJwt(tokens[1].slice(7)).claims.iat).toBe(Math.floor((t0 + 1000) / 1000));
	});

	it("concurrent sends share one provider token; a changed key mints a new one", async () => {
		const e = await apnsEnv();
		const { seen, fetcher } = fetching(() => new Response(null, { status: 200 }));
		const send = pushSender(e, fetcher, newPushCache());
		await Promise.all([1, 2, 3].map(() => send(target(apns()), 1_800_100_000_000)));
		expect(new Set(seen.map((s) => (s.init.headers as Record<string, string>).authorization)).size).toBe(1);
		const other = await crypto.subtle.generateKey({ name: "ECDSA", namedCurve: "P-256" }, true, ["sign", "verify"]);
		const e2: Env = { ...e, APNS_KEY_P8: await pemOf((other as CryptoKeyPair).privateKey) };
		const cache = newPushCache();
		const f2 = fetching(() => new Response(null, { status: 200 }));
		await pushSender(e, f2.fetcher, cache)(target(apns()), 1_800_100_000_000);
		await pushSender(e2, f2.fetcher, cache)(target(apns()), 1_800_100_000_000);
		expect((f2.seen[1].init.headers as Record<string, string>).authorization).not.toBe((f2.seen[0].init.headers as Record<string, string>).authorization);
	});

	it("is skipped, with no request, when a secret is missing; a malformed key is a failed send, not a throw", async () => {
		const full = await apnsEnv();
		for (const missing of ["APNS_KEY_P8", "APNS_KEY_ID", "APNS_TEAM_ID", "APNS_TOPIC"] as const) {
			const { seen, fetcher } = fetching(() => new Response(null, { status: 200 }));
			expect(await pushSender({ ...full, [missing]: undefined }, fetcher, newPushCache())(target(apns()), 0)).toBe("unconfigured");
			expect(seen).toEqual([]);
		}
		vi.spyOn(console, "error").mockImplementation(() => {});
		const { seen, fetcher } = fetching(() => new Response(null, { status: 200 }));
		expect(await pushSender({ ...full, APNS_KEY_P8: "-----BEGIN PRIVATE KEY-----\nAAAA\n-----END PRIVATE KEY-----" }, fetcher, newPushCache())(target(apns()), 0)).toBe("failed");
		expect(seen).toEqual([]);
	});

	it("accepts the key as one line with literal \\n, as a secret pasted from a shell may be", async () => {
		const e = await apnsEnv();
		const { fetcher } = fetching(() => new Response(null, { status: 200 }));
		const oneLine = { ...e, APNS_KEY_P8: e.APNS_KEY_P8!.trim().replace(/\n/g, "\\n") };
		expect(await pushSender(oneLine, fetcher, newPushCache())(target(apns()), 0)).toBe("sent");
	});
});

describe("FCM requests", () => {
	const answers = (expires = 3600, status = 200) => (url: string) =>
		url.startsWith("https://oauth2.googleapis.com") ? new Response(JSON.stringify({ access_token: `at-${Math.random()}`, expires_in: expires, token_type: "Bearer" }), { status }) : new Response("{}", { status: 200 });

	it("mints an RS256 assertion, exchanges it, and posts the exact message", async () => {
		const keys = await testKeys();
		const e = await fcmEnv();
		const { seen, fetcher } = fetching(answers());
		const reg = fcm();
		const now = 1_800_100_000_000;
		expect(await pushSender(e, fetcher, newPushCache())(target(reg), now)).toBe("sent");
		expect(seen).toHaveLength(2);
		const [oauth, message] = seen;
		expect(oauth.url).toBe("https://oauth2.googleapis.com/token");
		expect(oauth.init.method).toBe("POST");
		expect((oauth.init.headers as Record<string, string>)["content-type"]).toBe("application/x-www-form-urlencoded");
		const form = new URLSearchParams(String(oauth.init.body));
		expect([...form.keys()].sort()).toEqual(["assertion", "grant_type"]);
		expect(form.get("grant_type")).toBe("urn:ietf:params:oauth:grant-type:jwt-bearer");
		const jwt = decodeJwt(form.get("assertion")!);
		expect(jwt.header).toEqual({ alg: "RS256", typ: "JWT" });
		expect(jwt.claims).toEqual({
			iss: "push@kanso-test.iam.gserviceaccount.com",
			scope: "https://www.googleapis.com/auth/firebase.messaging",
			aud: "https://oauth2.googleapis.com/token",
			iat: Math.floor(now / 1000),
			exp: Math.floor(now / 1000) + 3600,
		});
		expect(await crypto.subtle.verify({ name: "RSASSA-PKCS1-v1_5" }, keys.rsa.publicKey, jwt.signature, jwt.signed)).toBe(true);

		expect(message.url).toBe("https://fcm.googleapis.com/v1/projects/kanso-test/messages:send");
		expect(message.init.method).toBe("POST");
		expect(message.init.body).toBe(`{"message":{"token":"${reg.token}","android":{"priority":"HIGH","ttl":"3600s","collapse_key":"sync"}}}`);
		const headers = message.init.headers as Record<string, string>;
		expect(headers.authorization).toMatch(/^Bearer at-/);
		expect(headers["content-type"]).toBe("application/json");
	});

	it("uses the service account's token_uri, and the default when it has none", async () => {
		const custom = fetching(answers());
		await pushSender(await fcmEnv({ token_uri: "https://auth.example.test/token" }), custom.fetcher, newPushCache())(target(fcm()), 0);
		expect(custom.seen[0].url).toBe("https://auth.example.test/token");
		expect(decodeJwt(new URLSearchParams(String(custom.seen[0].init.body)).get("assertion")!).claims.aud).toBe("https://auth.example.test/token");
		const plain = fetching(answers());
		const parsed = JSON.parse(ACCOUNT((await testKeys()).rsaPem));
		delete parsed.token_uri;
		await pushSender(fake({ FCM_SERVICE_ACCOUNT: JSON.stringify(parsed) }), plain.fetcher, newPushCache())(target(fcm()), 0);
		expect(plain.seen[0].url).toBe("https://oauth2.googleapis.com/token");
	});

	it("caches the access token until five minutes before it expires, then refreshes; 401 drops it", async () => {
		const e = await fcmEnv();
		const { seen, fetcher } = fetching(answers(3600));
		const send = pushSender(e, fetcher, newPushCache());
		const oauthCalls = () => seen.filter((s) => s.url.includes("oauth2")).length;
		const bearer = (i: number) => (seen.filter((s) => s.url.includes("fcm"))[i].init.headers as Record<string, string>).authorization;
		const t0 = 1_800_100_000_000;
		await send(target(fcm()), t0);
		await send(target(fcm()), t0 + 3600_000 - 300_000 - 1);
		expect(oauthCalls()).toBe(1);
		expect(bearer(1)).toBe(bearer(0));
		await send(target(fcm()), t0 + 3600_000 - 300_000);
		expect(oauthCalls()).toBe(2);
		expect(bearer(2)).not.toBe(bearer(0));

		const unauthorized = fetching((url, _i, n) =>
			url.includes("oauth2") ? answers()(url) : n === 2 ? new Response('{"error":{"code":401}}', { status: 401 }) : new Response("{}", { status: 200 }),
		);
		const resend = pushSender(e, unauthorized.fetcher, newPushCache());
		expect(await resend(target(fcm()), t0)).toBe("failed");
		expect(await resend(target(fcm()), t0 + 1000)).toBe("sent");
		expect(unauthorized.seen.filter((s) => s.url.includes("oauth2"))).toHaveLength(2);
	});

	it("concurrent sends share one exchange; a failed exchange is a failed send and is tried again next time", async () => {
		const e = await fcmEnv();
		const shared = fetching(answers());
		const send = pushSender(e, shared.fetcher, newPushCache());
		await Promise.all([1, 2, 3].map(() => send(target(fcm()), 0)));
		expect(shared.seen.filter((s) => s.url.includes("oauth2"))).toHaveLength(1);
		vi.spyOn(console, "error").mockImplementation(() => {});
		let ok = false;
		const flaky = fetching((url) => (url.includes("oauth2") && !ok ? new Response("no", { status: 500 }) : answers()(url)));
		const resend = pushSender(e, flaky.fetcher, newPushCache());
		expect(await resend(target(fcm()), 0)).toBe("failed");
		ok = true;
		expect(await resend(target(fcm()), 1)).toBe("sent");
	});

	it("is skipped, with no request, when the service account is missing or incomplete", async () => {
		for (const value of [undefined, "", "not json", "{}", JSON.stringify({ project_id: "p", client_email: "e" })]) {
			const { seen, fetcher } = fetching(answers());
			expect(await pushSender(fake({ FCM_SERVICE_ACCOUNT: value }), fetcher, newPushCache())(target(fcm()), 0)).toBe("unconfigured");
			expect(seen).toEqual([]);
		}
	});

	it("a platform's missing secrets don't stop the other: one log line each run, tokens kept", async () => {
		const log = vi.spyOn(console, "error").mockImplementation(() => {});
		const apnsReg = apns();
		await register(a2, apnsReg);
		await register(a3, fcm());
		await change(a1, START);
		const { seen, fetcher } = fetching(answers());
		const e = await fcmEnv();
		const d = await deliverDue(e, alice.creds.accountId, START, pushSender(e, fetcher, newPushCache()));
		expect(d).toMatchObject({ claimed: 2, sent: 1, unconfigured: 1 });
		expect(log.mock.calls.map((c) => JSON.parse(String(c[0])))).toEqual([{ route: "push", error: "unconfigured", platform: "apns" }]);
		expect(await tokenOf(alice, a2)).toBe(apnsReg.token);
		expect(seen.filter((s) => s.url.includes("fcm.googleapis.com"))).toHaveLength(1);
	});
});

// --------------------------------------------------------------- property

describe("the 30 s rule", () => {
	// Invariant: whatever devices push and whenever alarms run, a device is sent at
	// most once per 30 s; only because another device changed something since its
	// last send; and each change reaches every other tokened device within 30 s of
	// it, so the last change of a burst is never lost.
	it("prop_pushes_are_spaced_justified_and_never_lost", async () => {
		const devices = [a1, a2, a3];
		for (const s of devices) await register(s, apns());
		const account = alice.creds.accountId;
		const op = fc.oneof(
			fc.record({ kind: fc.constant("change" as const), who: fc.nat(2), dt: fc.integer({ min: 0, max: 45_000 }) }),
			fc.record({ kind: fc.constant("alarm" as const), who: fc.constant(0), dt: fc.integer({ min: 0, max: 45_000 }) }),
		);
		await fc.assert(
			fc.asyncProperty(fc.array(op, { minLength: 1, maxLength: 12 }), async (ops) => {
				await env.DB.prepare("UPDATE sessions SET push_due_at = NULL, push_last_sent_at = NULL").run();
				scheduled.length = 0;
				let t = START + 10 * DAY;
				let seq = 0;
				let armed: number | null = null;
				const changes: { seq: number; time: number; who: string }[] = [];
				const sends: { seq: number; time: number; who: string }[] = [];
				const alarm = async (at: number) => {
					const sender = recorder();
					const d = await deliverDue(fake(), account, at, sender.send);
					for (const s of sender.sent) sends.push({ seq: seq++, time: at, who: s.deviceId });
					armed = d.next;
				};
				// The real alarm fires by itself when its time comes.
				const advance = async (to: number) => {
					while (armed !== null && armed <= to) await alarm(Math.max(armed, t));
					t = to;
				};
				const absorb = () => {
					for (const s of scheduled.splice(0)) armed = armed === null || s.at < armed ? s.at : armed;
				};
				for (const o of ops) {
					await advance(t + o.dt);
					if (o.kind === "change") {
						await change(devices[o.who], t);
						changes.push({ seq: seq++, time: t, who: devices[o.who].device.id });
						absorb();
					} else {
						// A spurious alarm: harmless.
						await alarm(t);
					}
				}
				for (let guard = 0; armed !== null && guard < 50; guard++) await alarm(Math.max(armed, t));
				expect(armed).toBeNull();

				const ids = devices.map((s) => s.device.id);
				for (const id of ids) {
					const mine = sends.filter((s) => s.who === id);
					mine.forEach((s, i) => {
						if (i > 0) expect(s.time - mine[i - 1].time).toBeGreaterThanOrEqual(PUSH_INTERVAL_MS);
						// Justified: another device changed something since this device's last send.
						const since = i > 0 ? mine[i - 1].seq : -1;
						expect(changes.some((c) => c.who !== id && c.seq > since && c.seq < s.seq)).toBe(true);
					});
					// Nothing is lost: each change by another device is followed by a send within 30 s.
					for (const c of changes.filter((c) => c.who !== id)) {
						expect(mine.some((s) => s.seq > c.seq && s.time >= c.time && s.time <= c.time + PUSH_INTERVAL_MS)).toBe(true);
					}
				}
				// A sender is never woken by its own change alone.
				for (const s of sends) expect(changes.some((c) => c.who !== s.who)).toBe(true);
			}),
			runs(20),
		);
	});
});

// ------------------------------------------------------------------ alarm

describe("the AccountRoom alarm", () => {
	const room = () => env.ACCOUNT_ROOM.get(env.ACCOUNT_ROOM.idFromName(alice.creds.accountId));
	const alarmAt = () => runInDurableObject(room(), async (_i: AccountRoom, state) => state.storage.getAlarm());

	it("schedule() arms the alarm for the earliest due time and never later", async () => {
		const base = Date.now() + 3_600_000;
		await room().schedule(alice.creds.accountId, base);
		expect(await alarmAt()).toBe(base);
		await room().schedule(alice.creds.accountId, base + 60_000);
		expect(await alarmAt()).toBe(base);
		await room().schedule(alice.creds.accountId, base - 60_000);
		expect(await alarmAt()).toBe(base - 60_000);
		const stored = await runInDurableObject(room(), async (_i: AccountRoom, state) => state.storage.get("account"));
		expect(stored).toBe(alice.creds.accountId);
	});

	it("the alarm sends what is due, clears the claim, and re-arms for what is waiting", async () => {
		const log = vi.spyOn(console, "error").mockImplementation(() => {});
		await register(a2, apns());
		await register(a3, fcm());
		const now = Date.now();
		await change(a1, now - 1000);
		expect(scheduled).toEqual([{ accountId: alice.creds.accountId, at: now - 1000 }]);
		// The real room is told after the fact, with a far time so it does not fire by itself.
		await room().schedule(alice.creds.accountId, now + 3_600_000);
		expect(await runDurableObjectAlarm(room())).toBe(true);
		// This worker has no push secrets: both devices are claimed and logged as unconfigured, tokens kept.
		expect(log.mock.calls.map((c) => JSON.parse(String(c[0])).error)).toEqual(expect.arrayContaining(["unconfigured"]));
		expect(JSON.stringify(log.mock.calls)).not.toContain(alice.creds.accountId);
		for (const s of [a2, a3]) {
			const r = await row(alice.creds.accountId, s.device.id);
			expect(r.push_due_at).toBeNull();
			expect(r.push_last_sent_at).toBeGreaterThanOrEqual(now);
			expect(r.push_token).not.toBeNull();
		}
		expect(await alarmAt()).toBeNull();

		// A change right after waits for the 30 s; the alarm that finds it too early re-arms for it.
		await change(a1, Date.now());
		const due = (await row(alice.creds.accountId, a2.device.id)).push_due_at!;
		expect(due).toBeGreaterThan(Date.now() + 20_000);
		await room().schedule(alice.creds.accountId, due);
		expect(await alarmAt()).toBe(due);
		expect(await runDurableObjectAlarm(room())).toBe(true);
		expect(await alarmAt()).toBe(due);
		expect((await row(alice.creds.accountId, a2.device.id)).push_due_at).toBe(due);
	});

	it("sends through the global fetch with the room's own secrets", async () => {
		const keys = await testKeys();
		const reg = apns("development");
		await register(a2, reg);
		const now = Date.now();
		await change(a1, now - 1000);
		const calls: Seen[] = [];
		vi.spyOn(globalThis, "fetch").mockImplementation((async (url: string, init: RequestInit) => {
			calls.push({ url: String(url), init });
			return new Response(null, { status: 200 });
		}) as typeof fetch);
		const bindings = env as unknown as Record<string, unknown>;
		const saved = { ...bindings };
		Object.assign(env, { APNS_KEY_P8: keys.ecPem, APNS_KEY_ID: "ABC123DEFG", APNS_TEAM_ID: "TEAM123456", APNS_TOPIC: "app.kanso.Kanso" });
		try {
			await room().schedule(alice.creds.accountId, now + 3_600_000);
			expect(await runDurableObjectAlarm(room())).toBe(true);
		} finally {
			for (const key of ["APNS_KEY_P8", "APNS_KEY_ID", "APNS_TEAM_ID", "APNS_TOPIC"]) {
				if (saved[key] === undefined) delete bindings[key];
				else bindings[key] = saved[key];
			}
		}
		expect(calls.map((c) => c.url)).toEqual([`https://api.sandbox.push.apple.com/3/device/${reg.token}`]);
		expect((await row(alice.creds.accountId, a2.device.id)).push_due_at).toBeNull();
	});

	it("an alarm with nothing stored or nothing due does nothing", async () => {
		const stray = env.ACCOUNT_ROOM.get(env.ACCOUNT_ROOM.idFromName("a_nobody"));
		await runInDurableObject(stray, async (instance: AccountRoom) => instance.alarm());
		await room().schedule(alice.creds.accountId, Date.now() + 3_600_000);
		expect(await runDurableObjectAlarm(room())).toBe(true);
		expect(await alarmAt()).toBeNull();
	});
});
