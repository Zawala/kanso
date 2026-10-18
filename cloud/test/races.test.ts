// Concurrency through handle(): requests started together with Promise.all,
// so their D1 calls interleave at every await. Each check is a claim the
// spec makes about parallel requests: counting first, compare-and-swap
// credential changes, and quotas checked inside the batch that grows them.

import { env } from "cloudflare:workers";
import { beforeEach, describe, expect, it } from "vitest";
import { runCron } from "../src/index";
import {
	KDF,
	START,
	accountRow,
	callJson,
	client,
	login,
	newCreds,
	newDevice,
	r2Keys,
	recomputed,
	resetServer,
	sealed,
	setSetting,
	signUp,
	signupBody,
	testEnv,
	wireId,
	wireRev,
} from "./helpers";

const statuses = (rs: { status: number }[]) => rs.map((r) => r.status).sort();
const tally = (rs: { status: number }[]) =>
	rs.reduce<Record<number, number>>((t, r) => {
		t[r.status] = (t[r.status] ?? 0) + 1;
		return t;
	}, {});

describe("sign-up", () => {
	it("parallel sign-ups at max_accounts − 1: exactly one gets in", async () => {
		await resetServer({ signup: "open", max_accounts: "3", signups_per_day: "1000" });
		await signUp("alice");
		await signUp("bob");
		const rs = await Promise.all(Array.from({ length: 12 }, (_, i) => callJson("/v2/auth/signup", { json: signupBody(newCreds(`user${i}`), newDevice()), ip: `198.51.100.${i}` })));
		expect(tally(rs)).toEqual({ 201: 1, 403: 11 });
		for (const r of rs.filter((x) => x.status === 403)) expect(r.body).toMatchObject({ error: "signup_closed", reason: "full" });
		expect(await env.DB.prepare("SELECT count(*) AS n FROM accounts").first("n")).toBe(3);
	});

	it("parallel sign-ups past signups_per_day: exactly the remainder gets in", async () => {
		await resetServer({ signup: "open", signups_per_day: "5" });
		for (const name of ["a1x", "a2x", "a3x"]) await signUp(name);
		const rs = await Promise.all(Array.from({ length: 10 }, (_, i) => callJson("/v2/auth/signup", { json: signupBody(newCreds(`user${i}`), newDevice()), ip: `198.51.100.${i}` })));
		expect(tally(rs)).toEqual({ 201: 2, 403: 8 });
		for (const r of rs.filter((x) => x.status === 403)) expect(r.body.reason).toBe("busy");
	});

	it("parallel sign-ups of one name: one account, the rest username_taken", async () => {
		await resetServer();
		const rs = await Promise.all(Array.from({ length: 8 }, (_, i) => callJson("/v2/auth/signup", { json: signupBody(newCreds("alice"), newDevice()), ip: `198.51.100.${i}` })));
		expect(tally(rs)).toEqual({ 201: 1, 409: 7 });
		for (const r of rs.filter((x) => x.status === 409)) expect(r.body.error).toBe("username_taken");
	});

	it("parallel retries of one sign-up all succeed, with one account and one session", async () => {
		await resetServer();
		const creds = newCreds("alice");
		const device = newDevice();
		const rs = await Promise.all(Array.from({ length: 6 }, () => callJson("/v2/auth/signup", { json: signupBody(creds, device) })));
		expect(statuses(rs)).toEqual(Array(6).fill(201));
		expect(new Set(rs.map((r) => r.body.account_id))).toEqual(new Set([creds.accountId]));
		expect(await env.DB.prepare("SELECT count(*) AS n FROM sessions WHERE revoked_at IS NULL").first("n")).toBe(1);
		// Exactly one of the tokens is the device's live session.
		const live = await Promise.all(rs.map((r) => client(r.body.token).account()));
		expect(tally(live)).toEqual({ 200: 1, 401: 5 });
	});

	it("parallel retries of the sign-up that takes the last place all succeed too", async () => {
		await resetServer({ signup: "open", max_accounts: "2", signups_per_day: "1000" });
		await signUp("alice");
		const creds = newCreds("bob");
		const device = newDevice();
		const rs = await Promise.all(Array.from({ length: 6 }, () => callJson("/v2/auth/signup", { json: signupBody(creds, device) })));
		expect(statuses(rs)).toEqual(Array(6).fill(201));
		expect(await env.DB.prepare("SELECT count(*) AS n FROM accounts").first("n")).toBe(2);
	});
});

describe("sign-in", () => {
	beforeEach(() => resetServer());

	it("parallel wrong sign-ins: exactly ten are checked, the rest refused, and every one counted", async () => {
		const a = await signUp("alice");
		const wrong = { ...a.creds, authKey: a.creds.recoveryAuthKey };
		const rs = await Promise.all(Array.from({ length: 30 }, (_, i) => login(wrong, newDevice(), { ip: `198.51.100.${i}` })));
		expect(tally(rs)).toEqual({ 401: 10, 429: 20 });
		for (const r of rs.filter((x) => x.status === 429)) expect(r.body.limit).toBe("login-user");
		expect(await env.DB.prepare("SELECT count FROM limits WHERE key = ?1").bind(`login-user:${a.creds.accountId}`).first("count")).toBe(30);
		// Even the right key is refused now, from an unknown device.
		expect((await login(a.creds, newDevice(), { ip: "198.51.100.99" })).status).toBe(429);
	});

	it("parallel sign-ins of 14 devices leave at most 10 active", async () => {
		const a = await signUp("alice");
		// Each in its own login-user window, so only the device cap is in play.
		const rs = await Promise.all(Array.from({ length: 14 }, (_, i) => login(a.creds, newDevice(), { now: START + i * 900_000, ip: `198.51.100.${i}` })));
		expect(statuses(rs)).toEqual(Array(14).fill(200));
		expect(await env.DB.prepare("SELECT count(*) AS n FROM sessions WHERE revoked_at IS NULL").first("n")).toBeLessThanOrEqual(10);
		const live = await Promise.all(rs.map((r) => client(r.body.token).account()));
		expect(live.filter((r) => r.status === 200).length).toBeLessThanOrEqual(10);
	});

	it("a sign-in racing a password change never ends with a live session under the old password", async () => {
		for (let round = 0; round < 15; round += 1) {
			await resetServer();
			const a = await signUp("alice");
			const next = newCreds("alice");
			const other = newDevice();
			const [changed, signedIn] = await Promise.all([
				callJson("/v2/account/password", {
					token: a.token,
					json: { current_auth_key: a.creds.authKey, kdf: KDF, salt: next.salt, auth_key: next.authKey, wrapped_key: next.wrappedKey },
				}),
				login(a.creds, other, { ip: "198.51.100.1" }),
			]);
			expect(changed.status).toBe(200);
			expect([200, 401]).toContain(signedIn.status);
			if (signedIn.status === 200) {
				const r = await client(signedIn.body.token).account();
				expect(r.status, `round ${round}`).toBe(401);
				expect(r.body.reason).toBe("password_changed");
				expect(changed.body.signed_out_devices).toBe(1);
			} else {
				expect(signedIn.body.error).toBe("invalid_credentials");
			}
			expect((await client(a.token).account()).status).toBe(200);
		}
	});

	it("parallel recoveries with one recovery key: one wins, the rest are refused, and the key has rotated", async () => {
		const a = await signUp("alice");
		const attempts = Array.from({ length: 6 }, () => ({ creds: newCreds("alice"), device: newDevice() }));
		const rs = await Promise.all(
			attempts.map(({ creds, device }, i) =>
				callJson("/v2/auth/recover/finish", {
					ip: `198.51.100.${i}`,
					json: {
						username: "alice",
						recovery_auth_key: a.creds.recoveryAuthKey,
						kdf: KDF,
						salt: creds.salt,
						auth_key: creds.authKey,
						wrapped_key: creds.wrappedKey,
						recovery_auth_key_new: creds.recoveryAuthKey,
						recovery_wrapped_key_new: creds.recoveryWrappedKey,
						device: { id: device.id, sealed_name: null, proof: device.proof },
					},
				}),
			),
		);
		expect(tally(rs)).toEqual({ 200: 1, 401: 5 });
		const winner = attempts[rs.findIndex((r) => r.status === 200)];
		const row = await accountRow(a.creds.accountId);
		expect(row!.wrapped_key).toBe(winner.creds.wrappedKey);
		expect((await login(winner.creds, newDevice(), { ip: "198.51.100.50" })).status).toBe(200);
		expect((await client(a.token).account()).body.reason).toBe("recovered");
		expect(await env.DB.prepare("SELECT count(*) AS n FROM sessions WHERE revoked_at IS NULL").first("n")).toBe(2);
	});
});

describe("quotas", () => {
	beforeEach(() => resetServer());

	it("parallel pushes one document under the docs quota: exactly one is stored", async () => {
		const a = await signUp("alice");
		await env.DB.prepare("UPDATE accounts SET quota_docs = 5 WHERE id = ?1").bind(a.creds.accountId).run();
		const alice = client(a.token);
		expect((await alice.push(Array.from({ length: 4 }, () => ({ id: wireId(), rev: wireRev(), deleted: false, sealed: sealed(), base_rev: null })))).status).toBe(200);
		const rs = await Promise.all(Array.from({ length: 10 }, () => alice.push([{ id: wireId(), rev: wireRev(), deleted: false, sealed: sealed(), base_rev: null }])));
		expect(tally(rs)).toEqual({ 200: 1, 413: 9 });
		expect(await recomputed(a.creds.accountId)).toMatchObject({ docs: 5 });
		expect((await accountRow(a.creds.accountId))!.docs).toBe(5);
	});

	it("parallel pushes past the server's document bytes from two accounts: the sum stays under the cap", async () => {
		await setSetting("global_doc_bytes", "10000");
		const a = client((await signUp("alice")).token);
		const b = client((await signUp("bob")).token);
		const rs = await Promise.all(
			Array.from({ length: 16 }, (_, i) => (i % 2 ? a : b).push([{ id: wireId(), rev: wireRev(), deleted: false, sealed: sealed(1000), base_rev: null }])),
		);
		// 1,512 bytes each: 6 fit in 10,000.
		expect(tally(rs)).toEqual({ 200: 6, 413: 10 });
		expect(await env.DB.prepare("SELECT sum(doc_bytes) AS s FROM accounts").first("s")).toBe(6 * 1512);
	});

	it("parallel edits of one document with one base: one is accepted, the rest conflict with it", async () => {
		const alice = client((await signUp("alice")).token);
		const id = wireId();
		const first = { id, rev: wireRev(), deleted: false, sealed: sealed(), base_rev: null };
		await alice.push([first]);
		const edits = Array.from({ length: 8 }, () => ({ id, rev: wireRev(), deleted: false, sealed: sealed(), base_rev: first.rev }));
		const rs = await Promise.all(edits.map((e) => alice.push([e])));
		const results = rs.map((r) => r.body.results[0]);
		expect(results.filter((x) => x.status === "ok")).toHaveLength(1);
		const winner = edits[results.findIndex((x) => x.status === "ok")];
		for (const x of results.filter((r) => r.status === "conflict")) expect(x.current).toMatchObject({ rev: winner.rev, sealed: winner.sealed });
	});

	it("parallel uploads of new files under the files quota: exactly one is stored, in D1 and in R2", async () => {
		const a = await signUp("alice");
		await env.DB.prepare("UPDATE accounts SET quota_files = 3 WHERE id = ?1").bind(a.creds.accountId).run();
		const alice = client(a.token);
		for (let i = 0; i < 2; i += 1) expect((await alice.putFile(wireId("file"), new Uint8Array(100))).status).toBe(200);
		const rs = await Promise.all(Array.from({ length: 8 }, () => alice.putFile(wireId("file"), new Uint8Array(200).fill(3))));
		expect(tally(rs)).toEqual({ 200: 1, 413: 7 });
		expect(await r2Keys()).toHaveLength(3);
		expect(await recomputed(a.creds.accountId)).toEqual({ docs: 0, doc_bytes: 0, files: 3, file_bytes: 400 + 3 * 1024 });
		expect((await accountRow(a.creds.accountId))!.file_bytes).toBe(400 + 3 * 1024);
	});

	it("parallel uploads to one file ID keep the counters exact, and the cron then matches the row to R2", async () => {
		const a = await signUp("alice");
		const alice = client(a.token);
		const id = wireId("file");
		const sizes = [41, 500, 1200, 77, 3000, 999, 64, 2048];
		const rs = await Promise.all(sizes.map((n, i) => alice.putFile(id, new Uint8Array(n).fill(i + 1))));
		expect(statuses(rs)).toEqual(Array(sizes.length).fill(200));
		const row = await env.DB.prepare("SELECT size FROM account_files WHERE id = ?1").bind(id).first<{ size: number }>();
		expect(sizes).toContain(row!.size);
		expect(await recomputed(a.creds.accountId)).toEqual({ docs: 0, doc_bytes: 0, files: 1, file_bytes: row!.size + 1024 });
		expect((await accountRow(a.creds.accountId))!.file_bytes).toBe(row!.size + 1024);
		// D1's last writer and R2's may differ; an hour on, reconciliation takes R2's size.
		const object = await env.FILES.head(`accounts/${a.creds.accountId}/${id}`);
		await runCron(testEnv(), START + 2 * 3_600_000);
		const fixed = await env.DB.prepare("SELECT size FROM account_files WHERE id = ?1").bind(id).first<{ size: number }>();
		expect(fixed!.size).toBe(object!.size);
		expect((await accountRow(a.creds.accountId))!.file_bytes).toBe(object!.size + 1024);
		// The bytes served are the object's, and the account's counters still add up.
		expect((await alice.getFile(id)).bytes.length).toBe(object!.size);
		expect(await recomputed(a.creds.accountId)).toMatchObject({ file_bytes: object!.size + 1024 });
	});

	it("parallel deletions of an account and uploads to it leave neither rows nor objects", async () => {
		const a = await signUp("alice");
		const alice = client(a.token);
		const uploads = Array.from({ length: 6 }, () => alice.putFile(wireId("file"), new Uint8Array(100)));
		const deletion = callJson("/v2/account/delete", { token: a.token, json: { auth_key: a.creds.authKey, username: "alice" } });
		const [deleted] = await Promise.all([deletion, ...uploads]);
		expect(deleted.status).toBe(200);
		expect(await env.DB.prepare("SELECT count(*) AS n FROM account_files").first("n")).toBe(0);
		// Objects an upload put after the inline purge go with the queued purge, an hour on.
		await runCron(testEnv(), START + 2 * 3_600_000);
		expect(await r2Keys()).toEqual([]);
		expect(await env.DB.prepare("SELECT count(*) AS n FROM purges").first("n")).toBe(0);
	});
});
