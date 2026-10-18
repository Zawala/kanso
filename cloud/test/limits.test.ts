// Rate limits: the D1 layer against a model of fixed windows, the
// known-device exemption, what the keys hold, and IP grouping.

import { env } from "cloudflare:workers";
import fc from "fast-check";
import { beforeEach, describe, expect, it } from "vitest";
import { ipGroup } from "../src/limits";
import { base64url, fromBase64url } from "../src/protocol";
import {
	type Account,
	KDF,
	START,
	callJson,
	client,
	login,
	newCreds,
	newDevice,
	passwordBody,
	resetServer,
	signUp,
	signupBody,
} from "./helpers";

const runs = (n: number) => ({ numRuns: Math.min(n, Number(env.FAST_CHECK_RUNS)) });

/** Sign-ups unbounded by the account and daily caps, so only signup-ip refuses. */
const ROOMY = { signup: "open", max_accounts: "100000", signups_per_day: "100000" };

/** Test addresses with their groups, worked out by hand. */
const IPS: { ip: string | undefined; g56: string; g48: string }[] = [
	{ ip: "203.0.113.5", g56: "203.0.113.5", g48: "203.0.113.5" },
	{ ip: "::ffff:203.0.113.5", g56: "203.0.113.5", g48: "203.0.113.5" },
	{ ip: "203.0.113.6", g56: "203.0.113.6", g48: "203.0.113.6" },
	{ ip: "2001:db8:1:100::1", g56: "2001:db8:1:100", g48: "2001:db8:1" },
	{ ip: "2001:DB8:1:1ff:ffff::2", g56: "2001:db8:1:100", g48: "2001:db8:1" },
	{ ip: "2001:db8:1:200::3", g56: "2001:db8:1:200", g48: "2001:db8:1" },
	{ ip: "2001:db8:2::4", g56: "2001:db8:2:0", g48: "2001:db8:2" },
	{ ip: undefined, g56: "unknown", g48: "unknown" },
];

const WINDOWS = { "signup-ip": 3_600_000, "login-user": 900_000, "login-ip": 900_000, "recover-ip": 3_600_000, "password-account": 900_000 };
const MAXES = { "signup-ip": 10, "login-user": 10, "login-ip": 30, "recover-ip": 10, "password-account": 5 };
type Rule = keyof typeof WINDOWS;

/** The fixed-window counters as the spec states them. */
class Model {
	private rows = new Map<string, { start: number; count: number }>();

	/** Counts one request; the refusal (rule and retry_after) when over the maximum. */
	count(rule: Rule, subject: string, now: number): { limit: Rule; retry: number } | null {
		const window = WINDOWS[rule];
		const start = Math.floor(now / window) * window;
		const key = `${rule}|${subject}`;
		const row = this.rows.get(key);
		const count = row && row.start === start ? row.count + 1 : 1;
		this.rows.set(key, { start, count });
		return count > MAXES[rule] ? { limit: rule, retry: Math.max(1, Math.ceil((start + window - now) / 1000)) } : null;
	}
}

describe("the D1 layer", () => {
	// Model-based: sign-ups, sign-ins (right or wrong key, known, mis-proved
	// or new device, real or unknown name), recovery starts and current-
	// password checks from a pool of IPv4, IPv6 and mapped addresses, with the
	// clock moving by seconds to hours. Every answer's status, `limit` and
	// `retry_after` match the model.
	it("prop_limiter_matches_the_model", async () => {
		// Mostly addresses that share a group (0 and 1 are one IPv4 address; 3 and 4 one /56; 3 to 5 one /48).
		const genIp = fc.oneof({ weight: 3, arbitrary: fc.constantFrom(0, 1, 3, 4, 5) }, { weight: 1, arbitrary: fc.integer({ min: 0, max: IPS.length - 1 }) });
		const genOp = fc.oneof(
			{ weight: 6, arbitrary: fc.record({ kind: fc.constant("login" as const), user: fc.integer({ min: 0, max: 3 }), good: fc.boolean(), device: fc.constantFrom("known", "misproved", "new"), ip: genIp }) },
			{ weight: 2, arbitrary: fc.record({ kind: fc.constant("signup" as const), ip: genIp }) },
			{ weight: 2, arbitrary: fc.record({ kind: fc.constant("recover" as const), user: fc.integer({ min: 0, max: 3 }), good: fc.boolean(), ip: genIp }) },
			{ weight: 1, arbitrary: fc.record({ kind: fc.constant("password" as const), user: fc.integer({ min: 0, max: 1 }) }) },
		);
		const genStep = fc.record({
			op: genOp,
			// Bursts of the same request, so every rule gets pushed past its maximum.
			times: fc.integer({ min: 1, max: 14 }),
			wait: fc.oneof({ weight: 12, arbitrary: fc.integer({ min: 0, max: 20_000 }) }, { weight: 1, arbitrary: fc.integer({ min: 0, max: 2 * 3_600_000 }) }),
		});

		let names = 0;
		await fc.assert(
			fc.asyncProperty(fc.array(genStep, { minLength: 1, maxLength: 30 }), async (steps) => {
				await resetServer(ROOMY);
				const model = new Model();
				// Two accounts, signed up from somewhere outside the pool (signup-ip isn't modelled for them).
				const accounts: Account[] = [await signUp("alice", { ip: "192.0.2.200" }), await signUp("bob", { ip: "192.0.2.201" })];
				const unknown = ["nobody", "nemo"];
				let now = START;
				const requests = steps.flatMap((s) => Array.from({ length: s.times }, (_, i) => ({ op: s.op, wait: i === 0 ? s.wait : 250 })));
				for (const { op, wait } of requests) {
					now += wait;
					let r: { status: number; body: any; headers: Headers };
					let refused: { limit: Rule; retry: number } | null;
					let success: number;
					if (op.kind === "login") {
						const account = accounts[op.user] as Account | undefined;
						const username = account?.creds.username ?? unknown[op.user - 2];
						const device = op.device === "known" && account ? account.device : op.device === "misproved" && account ? { ...account.device, proof: base64url(crypto.getRandomValues(new Uint8Array(32))) } : newDevice();
						const authKey = op.good && account ? account.creds.authKey : base64url(crypto.getRandomValues(new Uint8Array(32)));
						r = await callJson("/v2/auth/login", { now, ip: IPS[op.ip].ip, json: { username, auth_key: authKey, device: { id: device.id, proof: device.proof } } });
						refused = model.count("login-ip", IPS[op.ip].g56, now);
						const known = account !== undefined && op.device === "known";
						if (!refused && !known) refused = model.count("login-user", account?.creds.accountId ?? "unknown", now);
						success = op.good && account ? 200 : 401;
						if (r.status === 200 && account) {
							if (op.device === "new") {
								// Signed straight out again, so no sign-in ever evicts the account's own device.
								expect((await client(r.body.token, { now }).logout()).status).toBe(204);
							} else {
								// The device's row now holds this proof and token (a mis-proved sign-in with the password replaces both).
								account.device = device;
								account.token = r.body.token;
							}
						}
					} else if (op.kind === "signup") {
						r = await callJson("/v2/auth/signup", { now, ip: IPS[op.ip].ip, json: signupBody(newCreds(`user${++names}`), newDevice()) });
						refused = model.count("signup-ip", IPS[op.ip].g48, now);
						success = 201;
					} else if (op.kind === "recover") {
						const account = accounts[op.user] as Account | undefined;
						const username = account?.creds.username ?? unknown[op.user - 2];
						const key = op.good && account ? account.creds.recoveryAuthKey : base64url(crypto.getRandomValues(new Uint8Array(32)));
						r = await callJson("/v2/auth/recover/start", { now, ip: IPS[op.ip].ip, json: { username, recovery_auth_key: key } });
						refused = model.count("recover-ip", IPS[op.ip].g56, now);
						success = op.good && account ? 200 : 401;
					} else {
						const account = accounts[op.user];
						const next = newCreds(account.creds.username);
						r = await callJson("/v2/account/password", {
							now,
							token: account.token,
							json: passwordBody(next.authKey, next),
						});
						refused = model.count("password-account", account.creds.accountId, now);
						success = 401;
					}
					const label = `${JSON.stringify(op)} at +${now - START} ms`;
					if (refused) {
						expect(r.status, label).toBe(429);
						expect(r.body, label).toMatchObject({ error: "rate_limited", limit: refused.limit, retry_after: refused.retry });
						expect(r.headers.get("retry-after"), label).toBe(String(refused.retry));
					} else {
						expect(r.status, `${label}: ${JSON.stringify(r.body)}`).toBe(success);
					}
				}
			}),
			runs(40),
		);
	});

	it("a device another device signed out is counted against the name again", async () => {
		await resetServer(ROOMY);
		const a = await signUp("alice");
		const lost = newDevice();
		expect((await login(a.creds, lost, { ip: "198.51.100.1" })).status).toBe(200);
		expect((await client(a.token).revoke(lost.id)).status).toBe(204);
		const now = START + 15 * 60_000;
		const wrong = { ...a.creds, authKey: base64url(new Uint8Array(32).fill(9)) };
		for (let i = 0; i < 10; i += 1) expect((await login(wrong, newDevice(), { now, ip: `198.51.100.${10 + i}` })).status).toBe(401);
		// The lost Mac still holds its proof, but it's a stranger now: refused like any other.
		const r = await login(a.creds, lost, { now, ip: "198.51.100.40" });
		expect(r.status).toBe(429);
		expect(r.body.limit).toBe("login-user");
		// The account's own signed-in device is still exempt.
		expect((await login(a.creds, a.device, { now, ip: "198.51.100.41" })).status).toBe(200);
	});

	it("a known device proving itself isn't counted against the name, even signed out; one revoked over 90 days ago is", async () => {
		await resetServer(ROOMY);
		const a = await signUp("alice");
		const spare = newDevice();
		expect((await login(a.creds, spare, { ip: "198.51.100.1" })).status).toBe(200);
		const now = START + 15 * 60_000;
		const wrong = { ...a.creds, authKey: base64url(new Uint8Array(32).fill(9)) };
		for (let i = 0; i < 10; i += 1) expect((await login(wrong, newDevice(), { now, ip: `198.51.100.${10 + i}` })).status).toBe(401);
		const locked = await login(a.creds, newDevice(), { now, ip: "198.51.100.40" });
		expect(locked.status).toBe(429);
		expect(locked.body.limit).toBe("login-user");
		// The signed-up device, with its proof: not counted, so not refused, and a wrong key is still just 401.
		expect((await login(wrong, a.device, { now, ip: "198.51.100.41" })).status).toBe(401);
		expect((await login(a.creds, a.device, { now, ip: "198.51.100.42" })).status).toBe(200);
		// The same device ID with another proof is a stranger.
		expect((await login(a.creds, { ...a.device, proof: base64url(new Uint8Array(32).fill(1)) }, { now, ip: "198.51.100.43" })).status).toBe(429);
		// Signed out: still known for 90 days.
		const spareToken = (await login(a.creds, spare, { now: START + 1000, ip: "198.51.100.2" })).body.token;
		expect((await client(spareToken, { now: START + 2000 }).logout()).status).toBe(204);
		expect((await login(a.creds, spare, { now, ip: "198.51.100.44" })).status).toBe(200);
		// Revoked 91 days ago: counted again.
		await env.DB.prepare("UPDATE sessions SET revoked_at = ?2, revoked_reason = 'signed_out' WHERE device_id = ?1")
			.bind(spare.id, now - 91 * 86_400_000)
			.run();
		expect((await login(a.creds, spare, { now, ip: "198.51.100.45" })).status).toBe(429);
	});

	it("sign-ins of every unknown name share one login-user row", async () => {
		await resetServer(ROOMY);
		for (let i = 0; i < 10; i += 1) {
			const r = await login(newCreds(`ghost${i}`), newDevice(), { ip: `198.51.100.${i}` });
			expect(r.status).toBe(401);
		}
		const r = await login(newCreds("ghost99"), newDevice(), { ip: "198.51.100.99" });
		expect(r.body).toMatchObject({ error: "rate_limited", limit: "login-user" });
		expect(await env.DB.prepare("SELECT count FROM limits WHERE key = 'login-user:unknown'").first("count")).toBe(11);
	});

	it("a refusal still counts, and a new window starts from the amount", async () => {
		await resetServer(ROOMY);
		const ip = "198.51.100.7";
		for (let i = 0; i < 12; i += 1) await callJson("/v2/auth/recover/start", { ip, json: { username: "nobody", recovery_auth_key: base64url(new Uint8Array(32)) } });
		const rows = await env.DB.prepare("SELECT window_start, count FROM limits WHERE key GLOB 'recover-ip:*'").all();
		expect(rows.results).toEqual([{ window_start: START, count: 12 }]);
		const later = START + 3_600_000 + 5;
		const r = await callJson("/v2/auth/recover/start", { now: later, ip, json: { username: "nobody", recovery_auth_key: base64url(new Uint8Array(32)) } });
		expect(r.status).toBe(401);
		expect((await env.DB.prepare("SELECT window_start, count FROM limits WHERE key GLOB 'recover-ip:*'").all()).results).toEqual([
			{ window_start: START + 3_600_000, count: 1 },
		]);
	});
});

describe("keys", () => {
	beforeEach(() => resetServer(ROOMY));

	/** The spec's derivation, computed here independently of src/crypto.ts. */
	async function expectedIpKey(rule: string, subject: string): Promise<string> {
		const pepper = fromBase64url(env.AUTH_PEPPER!)!;
		const ikm = await crypto.subtle.importKey("raw", pepper, "HKDF", false, ["deriveBits"]);
		const bits = await crypto.subtle.deriveBits({ name: "HKDF", hash: "SHA-256", salt: new Uint8Array(0), info: new TextEncoder().encode("kanso/v1/limit") }, ikm, 256);
		const key = await crypto.subtle.importKey("raw", bits, { name: "HMAC", hash: "SHA-256" }, false, ["sign"]);
		const mac = new Uint8Array(await crypto.subtle.sign("HMAC", key, new TextEncoder().encode(`${rule}\u0000${subject}`)));
		return `${rule}:${[...mac.slice(0, 16)].map((b) => b.toString(16).padStart(2, "0")).join("")}`;
	}

	it("hold no IP address, IP group or username: HMACs of the group, or the account ID", async () => {
		const a = await signUp("alice", { ip: "2001:db8:aa:bb00::1" });
		await login(a.creds, newDevice(), { ip: "203.0.113.77" });
		await login(newCreds("carol"), newDevice(), { ip: "203.0.113.77" });
		await callJson("/v2/auth/recover/start", { ip: "2001:db8:aa:bbcc::9", json: { username: "alice", recovery_auth_key: a.creds.recoveryAuthKey } });
		const next = newCreds("alice");
		await callJson("/v2/account/password", {
			token: a.token,
			json: passwordBody(next.authKey, next),
		});
		await client(a.token).push([]);
		const keys = (await env.DB.prepare("SELECT key FROM limits ORDER BY key").all<{ key: string }>()).results.map((r) => r.key);
		expect(keys.sort()).toEqual(
			[
				await expectedIpKey("signup-ip", "2001:db8:aa:0:0:0:0:0/48"),
				await expectedIpKey("login-ip", "203.0.113.77"),
				`login-user:${a.creds.accountId}`,
				"login-user:unknown",
				await expectedIpKey("recover-ip", "2001:db8:aa:bb00:0:0:0:0/56"),
				`password-account:${a.creds.accountId}`,
			].sort(),
		);
		for (const key of keys) {
			for (const leak of ["alice", "carol", "203.0.113", "2001", "db8"]) expect(key).not.toContain(leak);
			expect(key).toMatch(/^(signup-ip|login-ip|recover-ip):[0-9a-f]{32}$|^(login-user|password-account|writes-account):(a_[0-9a-f]{32}|unknown)$/);
		}
	});
});

describe("IP groups", () => {
	it("IPv4 per address; IPv6 per /56, or /48; mapped IPv4 as IPv4; absent as unknown", () => {
		expect(ipGroup("203.0.113.5", 56)).toBe("203.0.113.5");
		expect(ipGroup("203.0.113.5", 48)).toBe("203.0.113.5");
		expect(ipGroup("::ffff:203.0.113.5", 56)).toBe("203.0.113.5");
		expect(ipGroup("::FFFF:cb00:7105", 48)).toBe("203.0.113.5");
		expect(ipGroup("2001:db8:1:1ff:ffff::2", 56)).toBe("2001:db8:1:100:0:0:0:0/56");
		expect(ipGroup("2001:db8:1:100::1", 56)).toBe("2001:db8:1:100:0:0:0:0/56");
		expect(ipGroup("2001:db8:1:200::1", 56)).toBe("2001:db8:1:200:0:0:0:0/56");
		expect(ipGroup("2001:db8:1:200::1", 48)).toBe("2001:db8:1:0:0:0:0:0/48");
		expect(ipGroup("2001:0db8:0001:02ff:0000:0000:0000:0001", 48)).toBe("2001:db8:1:0:0:0:0:0/48");
		expect(ipGroup("::1", 56)).toBe("0:0:0:0:0:0:0:0/56");
		expect(ipGroup("fe80::1%en0", 48)).toBe("fe80:0:0:0:0:0:0:0/48");
		expect(ipGroup(null, 56)).toBe("unknown");
		expect(ipGroup("", 48)).toBe("unknown");
	});

	// Every spelling of an address in one /56 lands in the same group, and
	// addresses differing above bit 56 never do.
	it("prop_ipv6_groups_follow_the_prefix", () => {
		const genAddress = fc.array(fc.integer({ min: 0, max: 0xffff }), { minLength: 8, maxLength: 8 }).filter((g) => !(g.slice(0, 5).every((x) => x === 0) && g[5] === 0xffff));
		const spell = (g: number[], upper: boolean) => {
			const text = g.map((x) => x.toString(16)).join(":");
			return upper ? text.toUpperCase() : text;
		};
		fc.assert(
			fc.property(genAddress, fc.integer({ min: 0, max: 0xff }), fc.array(fc.integer({ min: 0, max: 0xffff }), { minLength: 4, maxLength: 4 }), fc.boolean(), (g, low, rest, upper) => {
				const sibling = [...g.slice(0, 3), (g[3] & 0xff00) | low, ...rest];
				expect(ipGroup(spell(sibling, upper), 56)).toBe(ipGroup(spell(g, false), 56));
				expect(ipGroup(spell(sibling, upper), 48)).toBe(ipGroup(spell(g, false), 48));
				const far = [...g];
				far[3] ^= 0x100;
				expect(ipGroup(spell(far, false), 56)).not.toBe(ipGroup(spell(g, false), 56));
				expect(ipGroup(spell(far, false), 48)).toBe(ipGroup(spell(g, false), 48));
			}),
		);
	});
});
