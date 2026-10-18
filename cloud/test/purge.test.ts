// The cron (docs/ACCOUNTS.md › R2 layout, purges and the cron): purge rounds
// within their budget, purge rows kept an hour after their prefix empties,
// expired limit rows, session expiry and revoked-row cleanup, and
// reconciliation of each account's R2 prefix with its account_files rows.

import { env } from "cloudflare:workers";
import { beforeEach, describe, expect, it } from "vitest";
import { runCron } from "../src/index";
import { RECONCILE_ACCOUNTS_PER_RUN, reconcile, runPurges } from "../src/purge";
import {
	START,
	accountRow,
	callJson,
	client,
	login,
	newDevice,
	r2Keys,
	recomputed,
	resetServer,
	signUp,
	setSetting,
	signupBody,
	testEnv,
	wireId,
} from "./helpers";

const HOUR = 3_600_000;
const DAY = 86_400_000;

async function putMany(prefix: string, n: number): Promise<void> {
	for (let i = 0; i < n; i += 100) {
		await Promise.all(Array.from({ length: Math.min(100, n - i) }, (_, j) => env.FILES.put(`${prefix}${String(i + j).padStart(5, "0")}`, "x")));
	}
}

const purgeRows = async () => (await env.DB.prepare("SELECT prefix, requested_at FROM purges ORDER BY prefix").all()).results;

beforeEach(() => resetServer());

describe("purges", () => {
	it("spend at most the given rounds of 1,000 keys, oldest request first, and finish on later runs", async () => {
		const old = "accounts/a_11111111111111111111111111111111/";
		const young = "accounts/a_22222222222222222222222222222222/";
		await putMany(old, 2100);
		await putMany(young, 10);
		await env.DB.batch([
			env.DB.prepare("INSERT INTO purges (prefix, requested_at) VALUES (?1, ?2)").bind(old, START - 2 * HOUR),
			env.DB.prepare("INSERT INTO purges (prefix, requested_at) VALUES (?1, ?2)").bind(young, START - HOUR + 1),
		]);
		expect(await runPurges(testEnv(), START, 2)).toBe(2);
		expect((await r2Keys()).filter((k) => k.startsWith(old))).toHaveLength(100);
		expect((await r2Keys()).filter((k) => k.startsWith(young))).toHaveLength(10);
		// Next run: one round empties `old`, one finds it empty and drops its row; `young` empties but its row stays.
		expect(await runPurges(testEnv(), START, 50)).toBe(4);
		expect(await r2Keys()).toEqual([]);
		expect(await purgeRows()).toEqual([{ prefix: young, requested_at: START - HOUR + 1 }]);
		// An hour after its request, the empty prefix's row goes.
		await runPurges(testEnv(), START + 1);
		expect(await purgeRows()).toEqual([]);
	});

	it("keep the row for an hour after the request even once the prefix is empty, so an in-flight upload can't orphan an object", async () => {
		const a = await signUp("alice");
		const alice = client(a.token);
		await alice.putFile(wireId("file"), new Uint8Array(64));
		const deleted = await callJson("/v2/account/delete", { token: a.token, json: { auth_key: a.creds.authKey, username: "alice" } });
		expect(deleted.status).toBe(200);
		expect(await r2Keys()).toEqual([]);
		const prefix = `accounts/${a.creds.accountId}/`;
		expect(await purgeRows()).toEqual([{ prefix, requested_at: START }]);
		// An upload that passed its session check before the deletion lands afterwards.
		await env.FILES.put(`${prefix}${wireId("file")}`, new Uint8Array(64));
		await runCron(testEnv(), START + 30 * 60_000);
		expect(await r2Keys()).toEqual([]);
		expect(await purgeRows()).toHaveLength(1);
		await runCron(testEnv(), START + HOUR);
		expect(await purgeRows()).toEqual([]);
	});

	it("drop a live account's purge row without touching its files", async () => {
		const a = await signUp("alice");
		const id = wireId("file");
		await client(a.token).putFile(id, new Uint8Array(64));
		await env.DB.prepare("INSERT INTO purges (prefix, requested_at) VALUES (?1, 0)").bind(`accounts/${a.creds.accountId}/`).run();
		await runCron(testEnv(), START + 2 * HOUR);
		expect(await purgeRows()).toEqual([]);
		expect(await r2Keys()).toEqual([`accounts/${a.creds.accountId}/${id}`]);
	});

	it("purge the legacy files/ prefix", async () => {
		await putMany("files/file:", 30);
		await env.FILES.put("accounts/a_33333333333333333333333333333333/file:keep", "x");
		await env.DB.prepare("INSERT INTO purges (prefix, requested_at) VALUES ('files/', ?1)").bind(START - 2 * HOUR).run();
		await runCron(testEnv(), START);
		expect((await r2Keys()).filter((k) => k.startsWith("files/"))).toEqual([]);
		expect(await purgeRows()).toEqual([]);
	});

	it("leave a re-created account's files alone", async () => {
		const first = await signUp("alice");
		await client(first.token).putFile(wireId("file"), new Uint8Array(64));
		await callJson("/v2/account/delete", { token: first.token, json: { auth_key: first.creds.authKey, username: "alice" } });
		// A leftover of the old account that the inline purge missed.
		const prefix = `accounts/${first.creds.accountId}/`;
		await env.FILES.put(`${prefix}${wireId("file")}`, "left over");
		// The same account ID signs up again within the hour: sign-up clears the queued purge.
		const again = await callJson("/v2/auth/signup", { json: signupBody(first.creds, newDevice()), now: START + 60_000 });
		expect(again.status).toBe(201);
		expect(await purgeRows()).toEqual([]);
		const kept = wireId("file");
		await client(again.body.token, { now: START + 120_000 }).putFile(kept, new Uint8Array(80));
		// Reconciliation removes the leftover (no row) and keeps the new account's file.
		await runCron(testEnv(), START + 3 * HOUR);
		expect(await r2Keys()).toEqual([`${prefix}${kept}`]);
		expect((await client(again.body.token, { now: START + 3 * HOUR }).getFile(kept)).bytes).toHaveLength(80);
	});
});

describe("limits", () => {
	it("rows go once their window has ended; rows of unknown rules after two days", async () => {
		const rows: [string, number][] = [
			["login-user:unknown", START - 15 * 60_000],
			["login-ip:aa", START - 15 * 60_000 + 1],
			["signup-ip:bb", START - HOUR],
			["recover-ip:cc", START - HOUR + 1],
			["writes-account:a_1", START - DAY],
			["writes-account:a_2", START - DAY + 1],
			["password-account:a_3", START],
			["mystery:x", START - 2 * DAY],
			["mystery:y", START - 2 * DAY + 1],
		];
		await env.DB.batch(rows.map(([key, start]) => env.DB.prepare("INSERT INTO limits (key, window_start, count) VALUES (?1, ?2, 1)").bind(key, start)));
		await runCron(testEnv(), START);
		const left = (await env.DB.prepare("SELECT key FROM limits ORDER BY key").all<{ key: string }>()).results.map((r) => r.key);
		expect(left).toEqual(["login-ip:aa", "mystery:y", "password-account:a_3", "recover-ip:cc", "writes-account:a_2"]);
	});
});

describe("sessions", () => {
	it("expire after 180 days unseen, and revoked rows go after 90 days", async () => {
		const a = await signUp("alice");
		const spare = newDevice();
		const second = await login(a.creds, spare, { now: START + DAY });
		await client(second.body.token, { now: START + 2 * DAY }).logout();

		await runCron(testEnv(), START + 180 * DAY);
		expect(await env.DB.prepare("SELECT revoked_reason FROM sessions WHERE device_id = ?1").bind(a.device.id).first("revoked_reason")).toBeNull();
		await runCron(testEnv(), START + 180 * DAY + 1);
		expect(await env.DB.prepare("SELECT revoked_reason FROM sessions WHERE device_id = ?1").bind(a.device.id).first("revoked_reason")).toBe("expired");
		expect((await client(a.token, { now: START + 180 * DAY + 2 }).account()).body.reason).toBe("expired");

		// The signed-out row (revoked at day 2) lasts until day 92.
		await runCron(testEnv(), START + 92 * DAY);
		expect(await env.DB.prepare("SELECT count(*) AS n FROM sessions WHERE device_id = ?1").bind(spare.id).first("n")).toBe(0);
		// The expired row (revoked at day 180) is still there to explain the 401.
		expect(await env.DB.prepare("SELECT count(*) AS n FROM sessions WHERE device_id = ?1").bind(a.device.id).first("n")).toBe(1);
		await runCron(testEnv(), START + 270 * DAY + 2);
		expect(await env.DB.prepare("SELECT count(*) AS n FROM sessions").first("n")).toBe(0);
		expect((await client(a.token, { now: START + 270 * DAY + 3 }).account()).body.reason).toBe("unknown");
	});

	it("a session used within the window is touched and doesn't expire", async () => {
		const a = await signUp("alice");
		await client(a.token, { now: START + 100 * DAY }).account();
		await runCron(testEnv(), START + 200 * DAY);
		expect((await client(a.token, { now: START + 200 * DAY }).account()).status).toBe(200);
	});
});

describe("reconciliation", () => {
	it("deletes old orphans, fixes sizes and drops rows whose object is gone, keeping counters exact", async () => {
		const a = await signUp("alice");
		const alice = client(a.token);
		const [sized, lost, fine] = [wireId("file"), wireId("file"), wireId("file")];
		for (const id of [sized, lost, fine]) expect((await alice.putFile(id, new Uint8Array(100))).status).toBe(200);
		const prefix = `accounts/${a.creds.accountId}/`;
		// R2 and D1 disagree: a bigger object than the row says, a missing object, and an object with no row.
		await env.FILES.put(`${prefix}${sized}`, new Uint8Array(300));
		await env.FILES.delete(`${prefix}${lost}`);
		const orphan = `${prefix}${wireId("file")}`;
		await env.FILES.put(orphan, new Uint8Array(50));

		// Within the hour nothing changes but the orphan, whose R2 upload time is real (older than the test clock).
		const early = await runCron(testEnv(), START + 30 * 60_000);
		expect(early.reconcile).toMatchObject({ sizesFixed: 0, rowsDropped: 0, orphansDeleted: 1 });
		const later = await runCron(testEnv(), START + 2 * HOUR);
		expect(later.reconcile).toMatchObject({ accounts: [a.creds.accountId], sizesFixed: 1, rowsDropped: 1, orphansDeleted: 0 });
		expect(await r2Keys()).toEqual([`${prefix}${sized}`, `${prefix}${fine}`].sort());
		const sizes = (await env.DB.prepare("SELECT id, size FROM account_files ORDER BY id").all()).results;
		expect(sizes).toEqual([{ id: sized, size: 300 }, { id: fine, size: 100 }].sort((x, y) => (x.id < y.id ? -1 : 1)));
		expect(await recomputed(a.creds.accountId)).toEqual({ docs: 0, doc_bytes: 0, files: 2, file_bytes: 400 + 2048 });
		const row = await accountRow(a.creds.accountId);
		expect({ files: row!.files, file_bytes: row!.file_bytes }).toEqual({ files: 2, file_bytes: 400 + 2048 });
		// The growth went through the quota triggers, not the recount's exemption.
		expect(row!.recounted_at).toBeNull();
	});

	// Two racing uploads of one ID can leave a small row over a large object
	// (D1 commits the large one last, R2 stores the small one first). The
	// correction is a growth like any upload: past the account's quota or the
	// server's cap, the row (refunded) and the object both go.
	it("deletes an object larger than its row when the quotas refuse its real size", async () => {
		await resetServer({ signup: "open", signups_per_day: "100" });
		const a = await signUp("alice");
		const b = await signUp("bob");
		await env.DB.prepare("UPDATE accounts SET quota_file_bytes = 5000 WHERE id = ?1").bind(a.creds.accountId).run();
		const [over, within, fine] = [wireId("file"), wireId("file"), wireId("file")];
		for (const id of [over, within, fine]) expect((await client(a.token).putFile(id, new Uint8Array(100))).status).toBe(200);
		const prefix = `accounts/${a.creds.accountId}/`;
		await env.FILES.put(`${prefix}${over}`, new Uint8Array(9000));
		await env.FILES.put(`${prefix}${within}`, new Uint8Array(1500));

		// Bob, within his quota, against the server's file cap.
		const bobs = wireId("file");
		expect((await client(b.token).putFile(bobs, new Uint8Array(100))).status).toBe(200);
		await env.FILES.put(`accounts/${b.creds.accountId}/${bobs}`, new Uint8Array(20_000));
		await setSetting("global_file_bytes", "15000");

		const report = await reconcile(testEnv(), START + 2 * HOUR);
		expect(report.accounts).toHaveLength(2);
		const sizes = (await env.DB.prepare("SELECT account_id, id, size FROM account_files ORDER BY size").all()).results;
		expect(sizes).toEqual([
			{ account_id: a.creds.accountId, id: fine, size: 100 },
			{ account_id: a.creds.accountId, id: within, size: 1500 },
		]);
		expect(await r2Keys()).toEqual([`${prefix}${fine}`, `${prefix}${within}`].sort());
		for (const account of [a, b]) {
			const row = await accountRow(account.creds.accountId);
			expect({ files: row!.files, file_bytes: row!.file_bytes }).toEqual({
				files: (await recomputed(account.creds.accountId)).files,
				file_bytes: (await recomputed(account.creds.accountId)).file_bytes,
			});
			expect(row!.recounted_at).toBeNull();
		}
		expect((await accountRow(a.creds.accountId))!.file_bytes).toBe(1600 + 2 * 1024);
		expect((await accountRow(b.creds.accountId))!.files).toBe(0);
		expect(report).toMatchObject({ oversizeDeleted: 2, sizesFixed: 1 });
	});

	it("leaves an orphan younger than an hour", async () => {
		const a = await signUp("alice");
		const key = `accounts/${a.creds.accountId}/${wireId("file")}`;
		await env.FILES.put(key, new Uint8Array(50));
		// The real clock: the object was uploaded moments ago.
		const report = await reconcile(testEnv(), Date.now());
		expect(report.orphansDeleted).toBe(0);
		expect(await r2Keys()).toEqual([key]);
	});

	it("takes five accounts a run, in turn, so every account comes round", async () => {
		await resetServer({ signup: "open", signups_per_day: "100" });
		const ids: string[] = [];
		for (let i = 0; i < 7; i += 1) ids.push((await signUp(`user${i}`)).creds.accountId);
		const seen = new Set<string>();
		for (let run = 0; run < 3; run += 1) {
			const report = await reconcile(testEnv(), START + run * 15 * 60_000);
			expect(report.accounts).toHaveLength(RECONCILE_ACCOUNTS_PER_RUN);
			for (const id of report.accounts) seen.add(id);
		}
		expect([...seen].sort()).toEqual([...ids].sort());
	});
});
