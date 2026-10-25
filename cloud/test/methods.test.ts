// Changing how an account is entered (docs/ACCOUNTS.md › Account identity ›
// "Changing a way in needs a fresh proof", › Sign-in routes): passkey proofs
// and their challenge, adding and removing passkeys, the first password, and
// the recovery-key and deletion routes with a proof. End to end through
// handle(), with a software authenticator (webauthn-helpers.ts). Property tests
// use the pinned fast-check seed.

import { env } from "cloudflare:workers";
import fc from "fast-check";
import { beforeEach, describe, expect, it } from "vitest";
import { sha256Hex } from "../src/crypto";
import { base64url } from "../src/protocol";
import { type Account, type Device, START, callJson, newCreds, newDevice, resetServer, signUp } from "./helpers";
import { type Credential, assertion, newCredential, newEd25519Credential, passkeyConfig, passkeyEnv, registration } from "./webauthn-helpers";

const runs = (n: number) => ({ numRuns: Math.min(n, Number(env.FAST_CHECK_RUNS)) });
const T0 = START + 12 * 3_600_000;

/** Apple's start builds a URL without calling Apple, so a link start is a cheap consumer of a proof. */
const APPLE = {
	APPLE_CLIENT_ID: "app.kanso.Kanso.web",
	APPLE_TEAM_ID: "4973Y692JU",
	APPLE_KEY_ID: "MS6Y3S5ZL7",
	APPLE_REDIRECT_URI: "https://sync.kansoapp.app/v2/auth/apple/callback",
	APPLE_PRIVATE_KEY: "unused by a start",
};
export const menv = () => passkeyEnv(APPLE);

export const wire = (d: Device) => ({ id: d.id, proof: d.proof });
const named = (d: Device) => ({ id: d.id, proof: d.proof, sealed_name: d.sealedName });

export function as(path: string, json: unknown, token: string, now = T0) {
	return callJson(path, { env: menv(), now, json, token });
}

/** A passwordless account made with a passkey: its session, device, credential and recovery key. */
export interface PasskeyAccount {
	accountId: string;
	username: string;
	token: string;
	device: Device;
	credential: Credential;
	recoveryAuthKey: string;
}

export async function passkeyAccount(username: string, credential: Credential = newCredential(), now = T0): Promise<PasskeyAccount> {
	const device = newDevice();
	const creds = newCreds(username);
	const start = await callJson("/v2/auth/passkey/signup/start", {
		json: { username, account_id: creds.accountId, device: wire(device) },
		env: menv(),
		now,
	});
	expect(start.status).toBe(200);
	const finish = await callJson("/v2/auth/passkey/signup/finish", {
		json: {
			flow_id: start.body.flow_id,
			...registration(credential, start.body.challenge),
			transports: ["internal"],
			username,
			account_id: creds.accountId,
			recovery_auth_key: creds.recoveryAuthKey,
			recovery_wrapped_key: creds.recoveryWrappedKey,
			key_id: creds.keyId,
			device: named(device),
		},
		env: menv(),
		now,
	});
	expect(finish.status).toBe(201);
	return { accountId: creds.accountId, username, token: finish.body.token, device, credential, recoveryAuthKey: creds.recoveryAuthKey };
}

/** Adds `credential` to an account's row set directly (for accounts made another way). */
export async function addCredential(accountId: string, credential: Credential, signCount = 0, transports = "internal"): Promise<void> {
	const { cbor } = await import("./webauthn-helpers");
	await env.DB.prepare(
		`INSERT INTO account_passkeys (credential_id, account_id, public_key, alg, sign_count, transports, created_at)
  VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)`,
	)
		.bind(base64url(credential.credentialId), accountId, base64url(new Uint8Array(cbor(credential.cose))), credential.alg, signCount, transports, T0 - 1000)
		.run();
}

export const proofStart = (token: string, body: unknown = {}, now = T0) => as("/v2/account/sign-in/proof/start", body, token, now);

/** A passkey proof: a fresh challenge from this session, asserted by `credential`. */
export async function passkeyProof(token: string, credential: Credential, options: Parameters<typeof assertion>[2] = {}, now = T0) {
	const started = await proofStart(token, {}, now);
	expect(started.status).toBe(200);
	return { kind: "passkey", flow: started.body.flow_id as string, assertion: assertion(credential, started.body.challenge, options) };
}

export const passwordProof = (a: Account) => ({ kind: "password", auth_key: a.creds.authKey });

/** The cheapest change that takes a password or passkey proof: an Apple link's start. */
export const linkStart = (token: string, device: Device, proof: unknown, now = T0) =>
	as("/v2/account/sign-in/identities/apple/start", { intent: "link", proof, device: wire(device) }, token, now);

beforeEach(async () => {
	await resetServer({ signup: "open", signups_per_day: "1000" });
});

// ------------------------------------------------------------- proof/start

describe("POST /v2/account/sign-in/proof/start", () => {
	it("answers a five-minute challenge bound to this account, device and device proof", async () => {
		const p = await passkeyAccount("prover");
		const started = await proofStart(p.token);
		expect(started.status).toBe(200);
		expect(started.body).toMatchObject({
			rp_id: passkeyConfig.PASSKEY_RP_ID,
			allow_credentials: [{ id: base64url(p.credential.credentialId), transports: ["internal"] }],
			user_verification: "required",
			timeout: 300_000,
		});
		expect(started.body.flow_id).toMatch(/^[A-Za-z0-9_-]{43}$/);
		expect(started.body.challenge).toMatch(/^[A-Za-z0-9_-]{43}$/);
		const row = await env.DB.prepare("SELECT * FROM auth_flows WHERE id = ?1").bind(started.body.flow_id).first<Record<string, unknown>>();
		expect(row).toMatchObject({
			kind: "proof",
			intent: "proof",
			account_id: p.accountId,
			device_id: p.device.id,
			device_proof_hash: await sha256Hex(Uint8Array.from(Buffer.from(p.device.proof, "base64url"))),
			challenge: started.body.challenge,
			provider: null,
			subject: null,
			expires_at: T0 + 300_000,
			consumed_at: null,
		});
	});

	it("names one credential when asked: the account's own, else 404; a malformed ID is 400", async () => {
		const p = await passkeyAccount("prover");
		const second = newCredential();
		await addCredential(p.accountId, second, 0, "usb smartcard");
		const all = await proofStart(p.token);
		expect(all.body.allow_credentials).toHaveLength(2);
		const one = await proofStart(p.token, { credential_id: base64url(second.credentialId) });
		expect(one.body.allow_credentials).toEqual([{ id: base64url(second.credentialId), transports: ["usb", "smart-card"] }]);
		expect((await proofStart(p.token, { credential_id: base64url(newCredential().credentialId) })).status).toBe(404);
		expect((await proofStart(p.token, { credential_id: "not base64url!" })).body.error).toBe("bad_request");
		expect((await proofStart(p.token, { credential_id: 7 })).body.error).toBe("bad_request");
	});

	it("is a session route: no token is 401, an Origin is 403, no passkeys configured is unsupported_method", async () => {
		const p = await passkeyAccount("prover");
		expect((await callJson("/v2/account/sign-in/proof/start", { env: menv(), now: T0, json: {} })).body).toMatchObject({ error: "unauthorized" });
		expect(
			(await callJson("/v2/account/sign-in/proof/start", { env: menv(), now: T0, json: {}, token: p.token, headers: { origin: "https://x.example" } })).status,
		).toBe(403);
		const off = await callJson("/v2/account/sign-in/proof/start", { now: T0, json: {}, token: p.token });
		expect(off).toMatchObject({ status: 400, body: { error: "unsupported_method" } });
	});

	it("counts proof-account: twenty an hour, then 429", async () => {
		const p = await passkeyAccount("prover");
		for (let i = 0; i < 20; i++) expect((await proofStart(p.token)).status).toBe(200);
		const limited = await proofStart(p.token);
		expect(limited).toMatchObject({ status: 429, body: { error: "rate_limited", limit: "proof-account" } });
		expect(Number(limited.headers.get("retry-after"))).toBeGreaterThan(0);
		// The next hour's window starts again.
		expect((await proofStart(p.token, {}, T0 + 3_600_000)).status).toBe(200);
	});
});

// ------------------------------------------------------------ passkey proofs

describe("a passkey proof", () => {
	it("authorizes a change once, records the sign count and last use, and is used in the change's batch", async () => {
		const p = await passkeyAccount("prover");
		const proof = await passkeyProof(p.token, p.credential, { signCount: 9 });
		const linked = await linkStart(p.token, p.device, proof);
		expect(linked.status).toBe(200);
		const flow = await env.DB.prepare("SELECT consumed_at, completion_attempt_id FROM auth_flows WHERE id = ?1")
			.bind(proof.flow)
			.first<{ consumed_at: number; completion_attempt_id: string }>();
		expect(flow?.consumed_at).toBe(T0);
		expect(flow?.completion_attempt_id).toMatch(/^[A-Za-z0-9_-]{43}$/);
		const key = await env.DB.prepare("SELECT sign_count, last_used_at FROM account_passkeys WHERE credential_id = ?1")
			.bind(base64url(p.credential.credentialId))
			.first();
		expect(key).toEqual({ sign_count: 9, last_used_at: T0 });
		// The same proof again is a used flow.
		expect(await linkStart(p.token, p.device, proof)).toMatchObject({ status: 400, body: { error: "flow_expired" } });
	});

	it("expires after five minutes", async () => {
		const p = await passkeyAccount("prover");
		const proof = await passkeyProof(p.token, p.credential);
		expect(await linkStart(p.token, p.device, proof, T0 + 300_000)).toMatchObject({ status: 400, body: { error: "flow_expired" } });
		const fresh = await passkeyProof(p.token, p.credential, {}, T0 + 300_000);
		expect((await linkStart(p.token, p.device, fresh, T0 + 599_999)).status).toBe(200);
	});

	it("belongs to the session that started it: another device of the account, or another account, can't use it", async () => {
		const p = await passkeyAccount("prover");
		const proof = await passkeyProof(p.token, p.credential);
		// A second device of the same account, signed in with the same passkey.
		const other = newDevice();
		const begin = await callJson("/v2/auth/passkey/start", { json: { device: wire(other) }, env: menv(), now: T0 });
		const signedIn = await callJson("/v2/auth/passkey/finish", {
			json: { flow_id: begin.body.flow_id, ...assertion(p.credential, begin.body.challenge, { signCount: 0 }), device: wire(other) },
			env: menv(),
			now: T0,
		});
		expect(signedIn.status).toBe(200);
		expect(await linkStart(signedIn.body.token, other, proof)).toMatchObject({ status: 400, body: { error: "flow_expired" } });
		const bob = await passkeyAccount("bob");
		expect(await linkStart(bob.token, bob.device, proof)).toMatchObject({ status: 400, body: { error: "flow_expired" } });
		// Still unused: its own session can.
		expect((await linkStart(p.token, p.device, proof)).status).toBe(200);
	});

	it("a failed assertion is invalid_credentials and uses the challenge up", async () => {
		const p = await passkeyAccount("prover");
		const started = await proofStart(p.token);
		const flow = started.body.flow_id;
		const bad = { kind: "passkey", flow, assertion: assertion(p.credential, started.body.challenge, { signatureBitFlip: true }) };
		expect(await linkStart(p.token, p.device, bad)).toMatchObject({ status: 401, body: { error: "invalid_credentials" } });
		const good = { kind: "passkey", flow, assertion: assertion(p.credential, started.body.challenge) };
		expect(await linkStart(p.token, p.device, good)).toMatchObject({ status: 400, body: { error: "flow_expired" } });
	});

	it("refuses a missing UV flag, the wrong origin, RP ID or challenge, another account's credential and a credential the start didn't name", async () => {
		const p = await passkeyAccount("prover");
		const bob = await passkeyAccount("bob");
		const named = newCredential();
		await addCredential(p.accountId, named);
		const tries: ((challenge: string) => object)[] = [
			(c) => assertion(p.credential, c, { flags: 0x01 }),
			(c) => assertion(p.credential, c, { origin: "https://evil.example" }),
			(c) => assertion(p.credential, c, { rpId: "evil.example" }),
			() => assertion(p.credential, base64url(new Uint8Array(32))),
			(c) => assertion(bob.credential, c),
		];
		for (const make of tries) {
			const started = await proofStart(p.token);
			const proof = { kind: "passkey", flow: started.body.flow_id, assertion: make(started.body.challenge) };
			expect(await linkStart(p.token, p.device, proof)).toMatchObject({ status: 401, body: { error: "invalid_credentials" } });
		}
		const pinned = await proofStart(p.token, { credential_id: base64url(named.credentialId) });
		const wrongKey = { kind: "passkey", flow: pinned.body.flow_id, assertion: assertion(p.credential, pinned.body.challenge) };
		expect((await linkStart(p.token, p.device, wrongKey)).body.error).toBe("invalid_credentials");
		const pinnedAgain = await proofStart(p.token, { credential_id: base64url(named.credentialId) });
		const rightKey = { kind: "passkey", flow: pinnedAgain.body.flow_id, assertion: assertion(named, pinnedAgain.body.challenge) };
		expect((await linkStart(p.token, p.device, rightKey)).status).toBe(200);
	});

	it("refuses a sign count that doesn't grow; a stored 0 is never enforced", async () => {
		const p = await passkeyAccount("prover");
		expect((await linkStart(p.token, p.device, await passkeyProof(p.token, p.credential, { signCount: 0 }))).status).toBe(200);
		expect((await linkStart(p.token, p.device, await passkeyProof(p.token, p.credential, { signCount: 5 }))).status).toBe(200);
		const clone = await passkeyProof(p.token, p.credential, { signCount: 5 });
		expect((await linkStart(p.token, p.device, clone)).body.error).toBe("invalid_credentials");
		expect((await linkStart(p.token, p.device, await passkeyProof(p.token, p.credential, { signCount: 6 }))).status).toBe(200);
	});

	it("malformed proofs: no proof or an unknown kind is reauth_required; a broken assertion is bad_request", async () => {
		const p = await passkeyAccount("prover");
		const started = await proofStart(p.token);
		expect((await linkStart(p.token, p.device, undefined)).body).toMatchObject({ error: "reauth_required", reason: "reauth" });
		expect((await linkStart(p.token, p.device, { kind: "magic" })).body.error).toBe("reauth_required");
		expect((await linkStart(p.token, p.device, { kind: "passkey", flow: "short" })).body.error).toBe("reauth_required");
		expect((await linkStart(p.token, p.device, { kind: "passkey", flow: started.body.flow_id })).body.error).toBe("bad_request");
		expect(
			(await linkStart(p.token, p.device, { kind: "passkey", flow: started.body.flow_id, assertion: { credential_id: "x" } })).body.error,
		).toBe("bad_request");
		// An unknown flow is a used one.
		const unknown = { kind: "passkey", flow: base64url(new Uint8Array(32)), assertion: assertion(p.credential, started.body.challenge) };
		expect(await linkStart(p.token, p.device, unknown)).toMatchObject({ status: 400, body: { error: "flow_expired" } });
	});

	it("is single use under a race: parallel changes with one proof, exactly one passes", async () => {
		const p = await passkeyAccount("prover");
		const proof = await passkeyProof(p.token, p.credential);
		const answers = await Promise.all(Array.from({ length: 6 }, () => linkStart(p.token, p.device, proof)));
		expect(answers.filter((a) => a.status === 200)).toHaveLength(1);
		expect(answers.filter((a) => a.status === 400 && a.body.error === "flow_expired")).toHaveLength(5);
		const flows = await env.DB.prepare("SELECT count(*) AS n FROM auth_flows WHERE kind = 'apple'").first<{ n: number }>();
		expect(flows?.n).toBe(1);
	});

	it("an EdDSA credential proves too", async () => {
		const p = await passkeyAccount("eddsa", newEd25519Credential());
		expect((await linkStart(p.token, p.device, await passkeyProof(p.token, p.credential))).status).toBe(200);
	});

	it("property: any change to a signed byte is refused", async () => {
		const p = await passkeyAccount("prover", newEd25519Credential());
		await fc.assert(
			fc.asyncProperty(fc.constantFrom("authenticator_data", "client_data_json", "signature"), fc.nat(), fc.integer({ min: 1, max: 255 }), async (field, at, mask) => {
				const started = await proofStart(p.token);
				const signed = assertion(p.credential, started.body.challenge) as Record<string, string>;
				const bytes = Uint8Array.from(Buffer.from(signed[field], "base64url"));
				bytes[at % bytes.length] ^= mask;
				const proof = { kind: "passkey", flow: started.body.flow_id, assertion: { ...signed, [field]: base64url(bytes) } };
				const answer = await linkStart(p.token, p.device, proof);
				expect(answer.status === 401 || (answer.status === 400 && answer.body.error === "bad_request")).toBe(true);
				await env.DB.prepare("DELETE FROM limits").run();
			}),
			runs(25),
		);
	});

	it("a provider proof can't add a way in", async () => {
		const alice = await signUp("alice", { env: menv(), now: T0 });
		const answer = await linkStart(alice.token, alice.device, { kind: "provider", flow: base64url(new Uint8Array(32)) });
		expect(answer).toMatchObject({ status: 401, body: { error: "reauth_required" } });
	});

	it("a password proof still links, and counts password-account", async () => {
		const alice = await signUp("alice", { env: menv(), now: T0 });
		expect((await linkStart(alice.token, alice.device, passwordProof(alice))).status).toBe(200);
		const wrong = { kind: "password", auth_key: newCreds("x").authKey };
		for (let i = 0; i < 4; i++) expect((await linkStart(alice.token, alice.device, wrong)).body.error).toBe("invalid_credentials");
		expect((await linkStart(alice.token, alice.device, wrong)).body).toMatchObject({ error: "rate_limited", limit: "password-account" });
	});
});
