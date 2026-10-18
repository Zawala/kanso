// Accounts, sessions and devices: the shared auth.json cases, then the
// Worker's own properties and examples.

import { env } from "cloudflare:workers";
import fc from "fast-check";
import { beforeEach, describe, expect, it } from "vitest";
import { sha256Hex } from "../src/crypto";
import { base64url, fromBase64url } from "../src/protocol";
import fixture from "./fixtures/auth.json";
import {
	type Creds,
	type Device,
	KDF,
	START,
	call,
	callJson,
	clock,
	login,
	newCreds,
	newDevice,
	passwordBody,
	r2Keys,
	resetServer,
	sealed,
	signUp,
	signupBody,
	testEnv,
	wireId,
	wireRev,
	client,
} from "./helpers";
import { type Fixture, runCase } from "./runner";

describe("auth.json", () => {
	for (const testCase of (fixture as unknown as Fixture).cases) {
		it(testCase.name, () => runCase(fixture as unknown as Fixture, testCase.steps));
	}
});

const runs = (n: number) => ({ numRuns: Math.min(n, Number(env.FAST_CHECK_RUNS)) });

/** Every route, with a method it answers. */
const ROUTES: [string, string][] = [
	["GET", "/v2/health"],
	["POST", "/v2/auth/prelogin"],
	["POST", "/v2/auth/signup"],
	["POST", "/v2/auth/login"],
	["POST", "/v2/auth/recover/start"],
	["POST", "/v2/auth/recover/finish"],
	["POST", "/v2/auth/logout"],
	["GET", "/v2/account"],
	["POST", "/v2/account/password"],
	["POST", "/v2/account/recovery-key"],
	["POST", "/v2/account/delete"],
	["PUT", "/v2/account/device"],
	["GET", "/v2/account/devices"],
	["DELETE", `/v2/account/devices/d_${"0".repeat(32)}`],
	["POST", "/v2/push"],
	["GET", "/v2/changes"],
	["PUT", `/v2/files/file:${"A".repeat(22)}`],
	["GET", `/v2/files/file:${"A".repeat(22)}`],
];

const JSON_ROUTES = ROUTES.filter(
	([method, path]) => (method === "POST" || method === "PUT") && path !== "/v2/auth/logout" && !path.startsWith("/v2/files/"),
);

describe("browser and broken requests", () => {
	beforeEach(() => resetServer());

	it("any Origin header gets 403 forbidden_origin first, on every route and unknown paths", async () => {
		const { token } = await signUp("alice");
		for (const [method, path] of [...ROUTES, ["GET", "/v1/changes"], ["GET", "/"], ["OPTIONS", "/v2/push"]] as [string, string][]) {
			for (const origin of ["https://example.com", "null", ""]) {
				const r = await callJson(path, {
					method,
					token,
					headers: { origin, "content-type": "application/json" },
					body: method === "GET" || method === "OPTIONS" ? null : "{}",
				});
				expect(r.status, `${method} ${path} origin ${origin}`).toBe(403);
				expect(r.body.error).toBe("forbidden_origin");
			}
		}
	});

	it("a JSON route refuses any other Content-Type with 415, before touching D1", async () => {
		const { token } = await signUp("alice");
		const before = await env.DB.prepare("SELECT count(*) AS n FROM limits").first<number>("n");
		for (const [method, path] of JSON_ROUTES) {
			for (const type of [null, "text/plain", "application/x-www-form-urlencoded", "multipart/form-data", "application/json; charset=latin1", "application/jsonx"]) {
				const headers: Record<string, string> = type === null ? {} : { "content-type": type };
				const r = await callJson(path, { method, token, headers, body: '{"username":"alice"}' });
				expect(r.status, `${method} ${path} ${type}`).toBe(415);
				expect(r.body.error).toBe("unsupported_media_type");
			}
		}
		expect(await env.DB.prepare("SELECT count(*) AS n FROM limits").first<number>("n")).toBe(before);
		for (const type of ["application/json", "application/json; charset=utf-8", "Application/JSON; Charset=UTF-8"]) {
			const r = await callJson("/v2/auth/prelogin", { headers: { "content-type": type }, body: '{"username":"alice"}' });
			expect(r.status, type).toBe(200);
		}
		const upload = await call(`/v2/files/file:${"A".repeat(22)}`, {
			method: "PUT",
			token,
			headers: { "content-type": "application/json", "x-kanso-sha256": "0".repeat(64) },
			body: new Uint8Array(41),
		});
		expect(upload.status).toBe(415);
	});

	it("unknown paths, /v1 and rekey are 404; wrong methods are 405 with Allow", async () => {
		for (const path of ["/", "/v1/health", "/v1/push", "/v1/changes", "/v2", "/v2/", "/v2/account/rekey", "/v2/files/", "/v2/push/x"]) {
			const r = await callJson(path, { method: "POST", headers: { "content-type": "application/json" }, body: "{}" });
			expect(r.status, path).toBe(404);
			expect(r.body.error).toBe("not_found");
		}
		for (const [method, path] of ROUTES) {
			const wrong = "PATCH";
			const r = await call(path, { method: wrong });
			expect(r.status, `${wrong} ${path}`).toBe(405);
			expect(r.headers.get("allow")).toContain(method);
		}
	});

	it("non-canonical or wrong-length base64url proofs are 400 on every route that takes one", async () => {
		const a = await signUp("alice");
		const good = a.creds.authKey;
		const last = good.at(-1)!;
		// The same 32 bytes with one of the two unused bits set: decodes, but doesn't re-encode to itself.
		const bumped = `${good.slice(0, -1)}${"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_"["ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_".indexOf(last) ^ 1]}`;
		expect(fromBase64url(bumped)).toBeNull();
		for (const [round, bad] of [bumped, good.slice(1), `${good}A`, good.replace(/^./, "+"), `${good.slice(0, -1)}=`].entries()) {
			// A fresh 15-minute window per value: password-account counts even a malformed body (it counts first).
			const now = START + round * 15 * 60_000;
			const device = newDevice();
			const cases: [string, unknown, string?][] = [
				["/v2/auth/login", { username: "alice", auth_key: bad, device: { id: device.id, proof: device.proof } }],
				["/v2/auth/login", { username: "alice", auth_key: good, device: { id: device.id, proof: bad } }],
				["/v2/auth/recover/start", { username: "alice", recovery_auth_key: bad }],
				["/v2/auth/signup", { ...signupBody(newCreds("bob"), device), auth_key: bad }],
				["/v2/account/delete", { auth_key: bad, username: "alice" }, a.token],
				["/v2/account/recovery-key", { current_auth_key: bad, recovery_auth_key: good, recovery_wrapped_key: a.creds.wrappedKey }, a.token],
				["/v2/account/password", { ...passwordBody(good, newCreds("alice")), recovery_auth_key: bad }, a.token],
			];
			for (const [path, json, token] of cases) {
				const r = await callJson(path, { json, token, now, ip: `10.9.${Math.floor(Math.random() * 250)}.${Math.floor(Math.random() * 250)}` });
				expect(r.status, `${path} ${bad}`).toBe(400);
				expect(r.body.error).toBe("bad_request");
			}
		}
	});

	it("auth bodies over 8 KiB are 413 too_large; malformed JSON is 400", async () => {
		const r = await callJson("/v2/auth/login", { json: { username: "alice", pad: "x".repeat(8200) } });
		expect(r.status).toBe(413);
		expect(r.body.error).toBe("too_large");
		for (const body of ["{", "[]", "null", '"alice"', "\xff"]) {
			const bad = await callJson("/v2/auth/prelogin", { headers: { "content-type": "application/json" }, body });
			expect(bad.status, body).toBe(400);
		}
	});

	it("without a valid AUTH_PEPPER, health and the pre-session routes answer 503; session routes keep working", async () => {
		const a = await signUp("alice");
		for (const pepper of [undefined, "", "short", `${env.AUTH_PEPPER!.slice(0, -1)}B`, `${env.AUTH_PEPPER}A`]) {
			const broken = testEnv({ AUTH_PEPPER: pepper });
			for (const [path, json] of [
				["/v2/health", undefined],
				["/v2/auth/prelogin", { username: "alice" }],
				["/v2/auth/signup", signupBody(newCreds("bob"), newDevice())],
				["/v2/auth/login", { username: "alice", auth_key: a.creds.authKey, device: { id: a.device.id, proof: a.device.proof } }],
				["/v2/auth/recover/start", { username: "alice", recovery_auth_key: a.creds.recoveryAuthKey }],
				["/v2/auth/recover/finish", { username: "alice" }],
			] as [string, unknown][]) {
				const r = await callJson(path, { json, env: broken });
				expect(r.status, `${path} with ${pepper}`).toBe(503);
				expect(r.body.error).toBe("misconfigured");
			}
			const alice = client(a.token, { env: broken });
			expect((await alice.account()).status).toBe(200);
			expect((await alice.push([])).status).toBe(200);
		}
		expect((await client(a.token).logout()).status).toBe(204);
	});

	it("the AUTH_LIMITER binding refuses with 429 binding before D1, on /v2/auth/* and /v2/health only", async () => {
		const a = await signUp("alice");
		const keys: string[] = [];
		const refusing: RateLimit = {
			async limit({ key }) {
				keys.push(key);
				return { success: false };
			},
		};
		const limited = testEnv({ AUTH_LIMITER: refusing });
		const before = await env.DB.prepare("SELECT * FROM limits ORDER BY key").all();
		for (const [path, json, token] of [
			["/v2/health", undefined],
			["/v2/auth/prelogin", { username: "alice" }],
			["/v2/auth/signup", signupBody(newCreds("bob"), newDevice())],
			["/v2/auth/login", { username: "alice", auth_key: a.creds.authKey, device: { id: a.device.id, proof: a.device.proof } }],
			["/v2/auth/recover/start", { username: "alice", recovery_auth_key: a.creds.recoveryAuthKey }],
			["/v2/auth/recover/finish", {}],
			["/v2/auth/logout", undefined, a.token],
		] as [string, unknown, string?][]) {
			const r = await callJson(path, {
				json,
				token,
				method: path === "/v2/health" ? "GET" : "POST",
				env: limited,
				ip: "2001:db8:abcd:12ff:1:2:3:4",
			});
			expect(r.status, path).toBe(429);
			expect(r.body).toMatchObject({ error: "rate_limited", limit: "binding", retry_after: 60 });
			expect(r.headers.get("retry-after")).toBe("60");
		}
		expect(new Set(keys)).toEqual(new Set(["2001:db8:abcd:0:0:0:0:0/48"]));
		expect((await env.DB.prepare("SELECT * FROM limits ORDER BY key").all()).results).toEqual(before.results);
		// Session routes outside /v2/auth/ don't go through it.
		expect((await client(a.token, { env: limited }).account()).status).toBe(200);
		expect(keys).toHaveLength(7);
	});
});

describe("prelogin", () => {
	beforeEach(() => resetServer());

	const genName = fc.stringMatching(/^[a-z0-9][a-z0-9_-]{1,30}[a-z0-9]$/);

	// Unknown names get answers indistinguishable in shape from real ones,
	// stable per (pepper, name), different across names and peppers.
	it("prop_prelogin_is_deterministic_and_shaped_like_a_real_answer", async () => {
		const a = await signUp("alice");
		const real = (await callJson("/v2/auth/prelogin", { json: { username: "alice" } })).body;
		expect(real).toEqual({ kdf: KDF, salt: a.creds.salt, account_id: a.creds.accountId });
		const otherPepper = testEnv({ AUTH_PEPPER: base64url(new Uint8Array(32).fill(7)) });
		await fc.assert(
			fc.asyncProperty(genName, genName, async (name, other) => {
				fc.pre(name !== "alice" && other !== "alice" && name !== other);
				const first = await callJson("/v2/auth/prelogin", { json: { username: name } });
				const again = await callJson("/v2/auth/prelogin", { json: { username: ` ${name.toUpperCase()}\t` } });
				expect(first.status).toBe(200);
				expect(again.body).toEqual(first.body);
				expect(Object.keys(first.body).sort()).toEqual(Object.keys(real).sort());
				expect(first.body.kdf).toEqual(KDF);
				expect(first.body.salt).toMatch(/^[A-Za-z0-9_-]{22}$/);
				expect(fromBase64url(first.body.salt)).toHaveLength(16);
				expect(first.body.account_id).toMatch(/^a_[0-9a-f]{32}$/);
				const theirs = await callJson("/v2/auth/prelogin", { json: { username: other } });
				expect(theirs.body.salt).not.toBe(first.body.salt);
				expect(theirs.body.account_id).not.toBe(first.body.account_id);
				const elsewhere = await callJson("/v2/auth/prelogin", { json: { username: name }, env: otherPepper });
				expect(elsewhere.body.salt).not.toBe(first.body.salt);
			}),
			runs(50),
		);
		// Prelogin writes nothing.
		expect(await env.DB.prepare("SELECT count(*) AS n FROM limits").first<number>("n")).toBe(1);
	});
});

describe("secrets at rest", () => {
	beforeEach(() => resetServer());

	/** Every way a 32-byte proof or a token could appear in a row. */
	const forms = (value: string): string[] => {
		const bytes = fromBase64url(value.replace(/^kanso_/, ""));
		const hex = bytes ? [...bytes].map((b) => b.toString(16).padStart(2, "0")).join("") : "";
		return [value, value.replace(/^kanso_/, ""), hex, btoa(String.fromCharCode(...(bytes ?? []))).replace(/=+$/, "")].filter((f) => f.length >= 16);
	};

	it("nothing in D1 equals a submitted auth key, recovery key, device proof or session token", async () => {
		const a = await signUp("alice");
		const d2 = newDevice();
		const second = await login(a.creds, d2);
		// A password change (which brings a new recovery key), then a recovery with that key.
		const changedTo = newCreds("alice");
		expect((await callJson("/v2/account/password", { token: a.token, json: passwordBody(a.creds.authKey, changedTo) })).status).toBe(200);
		const next = newCreds("alice");
		const d3 = newDevice();
		const recovered = await callJson("/v2/auth/recover/finish", {
			json: {
				username: "alice",
				recovery_auth_key: changedTo.recoveryAuthKey,
				kdf: KDF,
				salt: next.salt,
				auth_key: next.authKey,
				wrapped_key: next.wrappedKey,
				recovery_auth_key_new: next.recoveryAuthKey,
				recovery_wrapped_key_new: next.recoveryWrappedKey,
				device: { id: d3.id, sealed_name: d3.sealedName, proof: d3.proof },
			},
		});
		expect(recovered.status).toBe(200);
		const secrets = [
			a.token,
			second.body.token,
			recovered.body.token,
			a.creds.authKey,
			a.creds.recoveryAuthKey,
			changedTo.authKey,
			changedTo.recoveryAuthKey,
			next.authKey,
			next.recoveryAuthKey,
			a.device.proof,
			d2.proof,
			d3.proof,
		];
		const tables = ["accounts", "sessions", "account_docs", "account_files", "limits", "purges", "settings", "daily"];
		const dump = JSON.stringify(await Promise.all(tables.map((t) => env.DB.prepare(`SELECT * FROM ${t}`).all())));
		for (const secret of secrets) for (const form of forms(secret)) expect(dump, secret).not.toContain(form);
		// What is stored instead: SHA-256 of the proof bytes, and of the token text.
		const row = await env.DB.prepare("SELECT auth_hash, recovery_hash FROM accounts").first<{ auth_hash: string; recovery_hash: string }>();
		expect(row!.auth_hash).toBe(await sha256Hex(fromBase64url(next.authKey)!));
		expect(row!.recovery_hash).toBe(await sha256Hex(fromBase64url(next.recoveryAuthKey)!));
	});
});

describe("sign-in", () => {
	beforeEach(() => resetServer());

	// Only the registered auth key signs in: any other 32 bytes, for this name
	// or an unknown one, get the same 401.
	it("prop_only_the_registered_auth_key_signs_in", async () => {
		const a = await signUp("alice");
		let t = START;
		await fc.assert(
			fc.asyncProperty(fc.uint8Array({ minLength: 32, maxLength: 32 }), fc.boolean(), fc.boolean(), async (key, mine, known) => {
				// A fresh window for every attempt keeps login-user out of the way.
				t += 15 * 60_000;
				const authKey = mine ? a.creds.authKey : base64url(key);
				fc.pre(mine || authKey !== a.creds.authKey);
				const device = known ? a.device : newDevice();
				const r = await login({ ...a.creds, authKey }, device, { now: t, ip: "192.0.2.1" });
				if (mine) {
					expect(r.status).toBe(200);
					expect(r.body.wrapped_key).toBe(a.creds.wrappedKey);
				} else {
					expect(r.status).toBe(401);
					expect(r.body.error).toBe("invalid_credentials");
					const unknown = await login({ ...a.creds, username: "nobody", authKey }, device, { now: t, ip: "192.0.2.2" });
					expect(unknown.status).toBe(401);
					expect(unknown.body).toEqual({ ...r.body, message: unknown.body.message });
				}
			}),
			runs(60),
		);
	});

	// Model-based: one account and twelve Macs. Sign-ins, sign-outs,
	// revocations, password changes, recoveries and time passing; after every
	// step each token ever issued is still answered exactly as the model
	// says (200, or 401 with its reason), and the device list matches.
	it("prop_sessions_follow_the_model", async () => {
		const DEVICES = 12;
		const genDevice = fc.integer({ min: 0, max: DEVICES - 1 });
		const genOp = fc.oneof(
			{ weight: 6, arbitrary: fc.record({ kind: fc.constant("login" as const), device: genDevice }) },
			// Most of the Macs signing in one after another, so the 11th-device eviction comes up.
			{
				weight: 1,
				arbitrary: fc.record({ kind: fc.constant("crowd" as const), devices: fc.shuffledSubarray([...Array(DEVICES).keys()], { minLength: 9, maxLength: DEVICES }) }),
			},
			{ weight: 2, arbitrary: fc.record({ kind: fc.constant("logout" as const), device: genDevice }) },
			{ weight: 2, arbitrary: fc.record({ kind: fc.constant("revoke" as const), device: genDevice, target: genDevice }) },
			{ weight: 1, arbitrary: fc.record({ kind: fc.constant("password" as const), device: genDevice }) },
			{ weight: 1, arbitrary: fc.record({ kind: fc.constant("recover" as const), device: genDevice }) },
			{
				weight: 2,
				arbitrary: fc.record({
					kind: fc.constant("wait" as const),
					ms: fc.constantFrom(30 * 60_000, 2 * 3_600_000, 100 * 86_400_000, 181 * 86_400_000),
				}),
			},
		);
		type Op = typeof genOp extends fc.Arbitrary<infer T> ? T : never;

		await fc.assert(
			fc.asyncProperty(fc.array(genOp, { minLength: 1, maxLength: 30 }), async (generated: Op[]) => {
				const ops = generated.flatMap((op) => (op.kind === "crowd" ? op.devices.map((device) => ({ kind: "login" as const, device })) : [op]));
				await resetServer();
				clock.now = START;
				let ip = 0;
				const nextIp = () => `198.51.${Math.floor(++ip / 250)}.${ip % 250}`;
				const devices: Device[] = Array.from({ length: DEVICES }, () => newDevice());
				let creds: Creds = newCreds("alice");
				const signup = await callJson("/v2/auth/signup", { json: signupBody(creds, devices[0]), ip: nextIp() });
				expect(signup.status).toBe(201);

				interface Row {
					token: string;
					revoked: string | null;
					lastSeen: number;
					createdAt: number;
				}
				const rows = new Map<number, Row>([[0, { token: signup.body.token, revoked: null, lastSeen: clock.now, createdAt: clock.now }]]);
				const issued: { token: string; device: number }[] = [{ token: signup.body.token, device: 0 }];

				/** What a session request with `token` gets now, applying its side effects to the model. */
				const expected = (token: string, device: number): { status: number; reason?: string } => {
					const row = rows.get(device);
					if (!row || row.token !== token) return { status: 401, reason: "unknown" };
					if (row.revoked) return { status: 401, reason: row.revoked };
					if (clock.now - row.lastSeen > 180 * 86_400_000) {
						row.revoked = "expired";
						return { status: 401, reason: "expired" };
					}
					if (clock.now - row.lastSeen > 3_600_000) row.lastSeen = clock.now;
					return { status: 200 };
				};
				const activeToken = (device: number): string | null => {
					const row = rows.get(device);
					return row && !row.revoked ? row.token : null;
				};

				for (const op of ops) {
					// Three minutes between steps: at most five per 15-minute window, under every per-account rule.
					clock.now += 3 * 60_000;
					if (op.kind === "wait") {
						clock.now += op.ms;
					} else if (op.kind === "login") {
						const r = await login(creds, devices[op.device], { ip: nextIp() });
						expect(r.status, JSON.stringify(r.body)).toBe(200);
						const others = [...rows.entries()].filter(([d, row]) => d !== op.device && !row.revoked);
						let evicted: number | null = null;
						if (others.length >= 10) {
							others.sort(([da, a], [db, b]) => a.lastSeen - b.lastSeen || a.createdAt - b.createdAt || devices[da].id.localeCompare(devices[db].id));
							evicted = others[0][0];
							rows.get(evicted)!.revoked = "evicted";
						}
						expect(r.body.evicted_device?.id ?? null).toBe(evicted === null ? null : devices[evicted].id);
						rows.set(op.device, { token: r.body.token, revoked: null, lastSeen: clock.now, createdAt: clock.now });
						issued.push({ token: r.body.token, device: op.device });
					} else if (op.kind === "logout") {
						const token = rows.get(op.device)?.token;
						if (!token) continue;
						const want = expected(token, op.device);
						const r = await client(token).logout();
						expect(r.status).toBe(want.status === 200 ? 204 : 401);
						if (want.status === 200) rows.get(op.device)!.revoked = "signed_out";
						else expect(r.body.reason).toBe(want.reason);
					} else if (op.kind === "revoke") {
						const token = activeToken(op.device);
						if (!token) continue;
						const want = expected(token, op.device);
						const r = await client(token).revoke(devices[op.target].id);
						if (want.status !== 200) {
							expect(r.body.reason).toBe(want.reason);
							continue;
						}
						const target = rows.get(op.target);
						if (op.target === op.device) {
							expect(r.status).toBe(204);
							target!.revoked = "signed_out";
						} else if (target) {
							// Active or already signed out (for any reason): either way it ends up revoked.
							expect(r.status).toBe(204);
							target.revoked = "revoked";
						} else {
							expect(r.status).toBe(404);
						}
					} else if (op.kind === "password") {
						const token = activeToken(op.device);
						if (!token) continue;
						const want = expected(token, op.device);
						const next = newCreds("alice");
						const r = await callJson("/v2/account/password", { token, json: passwordBody(creds.authKey, next) });
						if (want.status !== 200) {
							expect(r.body.reason).toBe(want.reason);
							continue;
						}
						let signedOut = 0;
						for (const [d, row] of rows) {
							if (d !== op.device && !row.revoked) {
								row.revoked = "password_changed";
								signedOut += 1;
							}
						}
						expect(r.body).toEqual({ signed_out_devices: signedOut });
						// The password change replaced the recovery key too; the recover step uses the new one.
						creds = { ...next, accountId: creds.accountId, keyId: creds.keyId };
					} else {
						const next = newCreds("alice");
						const device = devices[op.device];
						const r = await callJson("/v2/auth/recover/finish", {
							ip: nextIp(),
							json: {
								username: "alice",
								recovery_auth_key: creds.recoveryAuthKey,
								kdf: KDF,
								salt: next.salt,
								auth_key: next.authKey,
								wrapped_key: next.wrappedKey,
								recovery_auth_key_new: next.recoveryAuthKey,
								recovery_wrapped_key_new: next.recoveryWrappedKey,
								device: { id: device.id, sealed_name: null, proof: device.proof },
							},
						});
						expect(r.status).toBe(200);
						for (const row of rows.values()) if (!row.revoked) row.revoked = "recovered";
						rows.set(op.device, { token: r.body.token, revoked: null, lastSeen: clock.now, createdAt: clock.now });
						issued.push({ token: r.body.token, device: op.device });
						creds = { ...next, accountId: creds.accountId, keyId: creds.keyId };
					}

					for (const { token, device } of issued) {
						const want = expected(token, device);
						const r = await client(token).account();
						expect(r.status, `${op.kind}: token of device ${device}`).toBe(want.status);
						if (want.status === 401) expect(r.body.reason, `${op.kind}: token of device ${device}`).toBe(want.reason);
					}
					const viewer = [...rows.entries()].find(([d, row]) => !row.revoked && expected(row.token, d).status === 200);
					if (viewer) {
						const listed = await client(viewer[1].token).devices();
						const active = [...rows.entries()]
							.filter(([, row]) => !row.revoked)
							.sort(([da, a], [db, b]) => b.lastSeen - a.lastSeen || b.createdAt - a.createdAt || devices[da].id.localeCompare(devices[db].id))
							.map(([d]) => devices[d].id);
						expect(listed.body.devices.map((x: { id: string }) => x.id)).toEqual(active);
						expect(active.length).toBeLessThanOrEqual(10);
					}
				}
			}),
			runs(40),
		);
	});
});

describe("password changes and the recovery key", () => {
	beforeEach(() => resetServer());

	const recoverStart = (recoveryAuthKey: string, ip: string, now = clock.now) =>
		callJson("/v2/auth/recover/start", { now, ip, json: { username: "alice", recovery_auth_key: recoveryAuthKey } });

	// Security review 2, finding 1: someone who once knew the password plants a
	// recovery key; the owner's password change must end it.
	it("a recovery key planted by someone who knew the password stops working after the owner changes it", async () => {
		const a = await signUp("alice");
		const attackerMac = newDevice();
		const attacker = await login(a.creds, attackerMac, { ip: "203.0.113.66" });
		expect(attacker.status).toBe(200);
		const planted = newCreds("alice");
		const plant = await callJson("/v2/account/recovery-key", {
			token: attacker.body.token,
			json: { current_auth_key: a.creds.authKey, recovery_auth_key: planted.recoveryAuthKey, recovery_wrapped_key: planted.recoveryWrappedKey },
		});
		expect(plant.status).toBe(200);

		// The owner notices the strange Mac and changes the password; the app sends a fresh recovery key with it.
		const owner = newCreds("alice");
		const changed = await callJson("/v2/account/password", { token: a.token, json: passwordBody(a.creds.authKey, owner) });
		expect(changed.body).toEqual({ signed_out_devices: 1 });
		expect((await client(attacker.body.token).account()).body).toMatchObject({ error: "unauthorized", reason: "password_changed" });

		// The planted key is now worthless: recovery can't start or finish with it.
		const start = await recoverStart(planted.recoveryAuthKey, "203.0.113.67");
		expect(start.status).toBe(401);
		expect(start.body.error).toBe("invalid_credentials");
		const takeover = newCreds("alice");
		const finish = await callJson("/v2/auth/recover/finish", {
			ip: "203.0.113.68",
			json: {
				username: "alice",
				recovery_auth_key: planted.recoveryAuthKey,
				kdf: KDF,
				salt: takeover.salt,
				auth_key: takeover.authKey,
				wrapped_key: takeover.wrappedKey,
				recovery_auth_key_new: takeover.recoveryAuthKey,
				recovery_wrapped_key_new: takeover.recoveryWrappedKey,
				device: { id: attackerMac.id, sealed_name: null, proof: attackerMac.proof },
			},
		});
		expect(finish.status).toBe(401);
		expect(finish.body.error).toBe("invalid_credentials");

		// The owner keeps the account: the session stays, and only the new recovery key works.
		expect((await client(a.token).account()).status).toBe(200);
		expect((await recoverStart(a.creds.recoveryAuthKey, "203.0.113.69")).status).toBe(401);
		const mine = await recoverStart(owner.recoveryAuthKey, "203.0.113.70");
		expect(mine.status).toBe(200);
		expect(mine.body.recovery_wrapped_key).toBe(owner.recoveryWrappedKey);
		const row = await env.DB.prepare("SELECT auth_hash, recovery_hash, recovery_wrapped_key FROM accounts").first<Record<string, string>>();
		expect(row).toEqual({
			auth_hash: await sha256Hex(fromBase64url(owner.authKey)!),
			recovery_hash: await sha256Hex(fromBase64url(owner.recoveryAuthKey)!),
			recovery_wrapped_key: owner.recoveryWrappedKey,
		});
	});

	// Model-based: recovery keys replaced from any session (including a
	// stranger's that knew the password), password changes, retried password
	// changes and recoveries, in any order. After every step exactly one
	// recovery key works, the last one set; so every password change ends
	// every recovery key from before it.
	it("prop_a_password_change_always_ends_every_older_recovery_key", async () => {
		const genOp = fc.constantFrom("plant" as const, "password" as const, "retry" as const, "stale_retry" as const, "recover" as const);
		await fc.assert(
			fc.asyncProperty(fc.array(genOp, { minLength: 1, maxLength: 8 }), async (ops) => {
				await resetServer();
				clock.now = START;
				let ip = 0;
				const nextIp = () => `198.18.${Math.floor(++ip / 250)}.${ip % 250}`;
				const a = await signUp("alice");
				let creds: Creds = a.creds;
				let token = a.token;
				const device = a.device;
				const stranger = await login(a.creds, newDevice(), { ip: nextIp() });
				expect(stranger.status).toBe(200);
				let strangerToken: string | null = stranger.body.token;
				const everSet: string[] = [creds.recoveryAuthKey];
				let lastChange: { current: string; next: Creds } | null = null;

				for (const op of ops) {
					// Four minutes apart: at most four password-account checks per 15-minute window.
					clock.now += 4 * 60_000;
					const label = `${op} after ${JSON.stringify(ops)}`;
					if (op === "plant") {
						const planted = newCreds("alice");
						const r = await callJson("/v2/account/recovery-key", {
							token: strangerToken ?? token,
							json: { current_auth_key: creds.authKey, recovery_auth_key: planted.recoveryAuthKey, recovery_wrapped_key: planted.recoveryWrappedKey },
						});
						expect(r.status, label).toBe(200);
						creds = { ...creds, recoveryAuthKey: planted.recoveryAuthKey, recoveryWrappedKey: planted.recoveryWrappedKey };
						everSet.push(planted.recoveryAuthKey);
					} else if (op === "password") {
						const next = newCreds("alice");
						const r = await callJson("/v2/account/password", { token, json: passwordBody(creds.authKey, next) });
						expect(r.status, label).toBe(200);
						expect(r.body.signed_out_devices, label).toBe(strangerToken ? 1 : 0);
						strangerToken = null;
						lastChange = { current: creds.authKey, next };
						creds = { ...next, accountId: creds.accountId, keyId: creds.keyId };
						everSet.push(next.recoveryAuthKey);
					} else if (op === "retry" || op === "stale_retry") {
						if (!lastChange) continue;
						// The same password change again after a lost answer: the old key fails now.
						const stale = op === "stale_retry" ? newCreds("alice") : null;
						const body = passwordBody(lastChange.current, lastChange.next);
						if (stale) Object.assign(body, { recovery_auth_key: stale.recoveryAuthKey, recovery_wrapped_key: stale.recoveryWrappedKey });
						const r = await callJson("/v2/account/password", { token, json: body });
						const stillStored = creds.authKey === lastChange.next.authKey && creds.recoveryAuthKey === lastChange.next.recoveryAuthKey;
						if (stillStored && !stale) {
							expect(r.status, label).toBe(200);
							expect(r.body, label).toEqual({ signed_out_devices: 0 });
						} else {
							expect(r.status, label).toBe(401);
							expect(r.body.error, label).toBe("invalid_credentials");
						}
					} else {
						const next = newCreds("alice");
						const r = await callJson("/v2/auth/recover/finish", {
							ip: nextIp(),
							json: {
								username: "alice",
								recovery_auth_key: creds.recoveryAuthKey,
								kdf: KDF,
								salt: next.salt,
								auth_key: next.authKey,
								wrapped_key: next.wrappedKey,
								recovery_auth_key_new: next.recoveryAuthKey,
								recovery_wrapped_key_new: next.recoveryWrappedKey,
								device: { id: device.id, sealed_name: null, proof: device.proof },
							},
						});
						expect(r.status, label).toBe(200);
						token = r.body.token;
						strangerToken = null;
						lastChange = null;
						creds = { ...next, accountId: creds.accountId, keyId: creds.keyId };
						everSet.push(next.recoveryAuthKey);
					}

					// Exactly one recovery key works: the last one set.
					for (const key of everSet) {
						const r = await recoverStart(key, nextIp());
						if (key === creds.recoveryAuthKey) {
							expect(r.status, label).toBe(200);
							expect(r.body.recovery_wrapped_key, label).toBe(creds.recoveryWrappedKey);
						} else {
							expect(r.status, `${label}: an older recovery key`).toBe(401);
						}
					}
				}
			}),
			runs(40),
		);
	});
});

describe("sign-up retries", () => {
	beforeEach(() => resetServer());

	// Only the same request again is a retry. The right auth key with anything
	// else is answered exactly like a wrong one, so sign-up can't be used to
	// test passwords outside the sign-in limits, even with sign-ups closed.
	it("the right auth key in a different request gets what a wrong key gets, never a session", async () => {
		const a = await signUp("alice");
		const other = newCreds("alice");
		const body = signupBody(a.creds, newDevice());
		const variants: Record<string, unknown> = {
			account_id: other.accountId,
			salt: other.salt,
			wrapped_key: other.wrappedKey,
			recovery_auth_key: other.recoveryAuthKey,
			recovery_wrapped_key: other.recoveryWrappedKey,
			key_id: other.keyId,
			kdf: { ...KDF, t: 4 },
		};
		let ip = 0;
		for (const signup of ["open", "closed"]) {
			await env.DB.prepare("UPDATE settings SET value = ?1 WHERE name = 'signup'").bind(signup).run();
			const send = (json: unknown) => callJson("/v2/auth/signup", { json, ip: `198.51.100.${++ip}` });
			for (const [field, value] of Object.entries(variants)) {
				// The same request with a wrong auth key, then with the right one: one answer.
				const wrong = await send({ ...body, [field]: value, auth_key: other.authKey });
				const right = await send({ ...body, [field]: value });
				expect(wrong.status, `${signup}: ${field}`).toBe(signup === "open" ? 409 : 403);
				expect(right.status, `${signup}: ${field}`).toBe(wrong.status);
				expect(right.body, `${signup}: ${field}`).toEqual(wrong.body);
			}
			// The same request again is still a retry, whatever the settings.
			const retry = await send(body);
			expect(retry.status).toBe(201);
			expect(retry.body.account_id).toBe(a.creds.accountId);
		}
		expect(await env.DB.prepare("SELECT count(*) AS n FROM sessions WHERE revoked_at IS NULL").first("n")).toBe(2);
	});
});

describe("account deletion", () => {
	beforeEach(() => resetServer());

	it("removes every row and R2 object of the account and nothing of anyone else's", async () => {
		const a = await signUp("alice");
		const b = await signUp("bob");
		const alice = client(a.token);
		const bob = client(b.token);
		const files = [wireId("file"), wireId("file")];
		for (const c of [alice, bob]) {
			for (const id of files) expect((await c.putFile(id, new Uint8Array(100).fill(1))).status).toBe(200);
			expect((await c.push([{ id: wireId(), rev: wireRev(), deleted: false, sealed: sealed(), base_rev: null }])).status).toBe(200);
		}
		const r = await callJson("/v2/account/delete", { token: a.token, json: { auth_key: a.creds.authKey, username: "alice" } });
		expect(r.body).toEqual({ deleted: true });
		expect(await r2Keys()).toEqual(files.map((id) => `accounts/${b.creds.accountId}/${id}`).sort());
		for (const table of ["accounts", "sessions", "account_docs", "account_files"]) {
			const column = table === "accounts" ? "id" : "account_id";
			const n = await env.DB.prepare(`SELECT count(*) AS n FROM ${table} WHERE ${column} = ?1`).bind(a.creds.accountId).first<number>("n");
			expect(n, table).toBe(0);
		}
		expect(await env.DB.prepare("SELECT count(*) AS n FROM limits WHERE key LIKE ?1").bind(`%${a.creds.accountId}`).first<number>("n")).toBe(0);
		expect(await env.DB.prepare("SELECT prefix FROM purges").all()).toMatchObject({ results: [{ prefix: `accounts/${a.creds.accountId}/` }] });
		expect((await bob.account()).body.usage).toMatchObject({ docs: 1, files: 2 });
	});
});
