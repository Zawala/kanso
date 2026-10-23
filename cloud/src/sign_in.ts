// Sign-in methods (design: Kanso accounts spec, revision 3): what GET /v2/health
// advertises, and the `auth_flows` rows every ceremony stores.
//
// The advertisement is the deployment's capability list: the app shows a
// button only for a method named here, so a value this Worker can't actually
// use must not appear. A provider is listed only when every value its flow
// needs is present; passkeys only when there is a usable RP ID and at least
// one origin under it. Absence of `auth` means "username and password only",
// which is what an older server sends. The `events` advertisement lands with
// `/v2/account/events`; an absent `events` means the app uses GET fallbacks.

import { type PepperKeys, randomBytes, sha256Hex } from "./crypto";
import type { Env } from "./env";
import { badRequest, error } from "./http";
import { base64url, fromBase64url } from "./protocol";

export const PROVIDERS = ["apple", "google", "github"] as const;
export type Provider = (typeof PROVIDERS)[number];

export interface PasskeyAdvertisement {
	rp_id: string;
	origins: string[];
	security_key: boolean;
}

export interface AuthAdvertisement {
	password: true;
	passkeys: PasskeyAdvertisement | null;
	providers: Provider[];
}

export const isProvider = (value: string): value is Provider => (PROVIDERS as readonly string[]).includes(value);

const present = (value: string | undefined): value is string => typeof value === "string" && value.trim() !== "";

/** Whether every value the provider's flow needs is configured. */
export function providerConfigured(env: Env, provider: Provider): boolean {
	switch (provider) {
		case "apple":
			return (
				present(env.APPLE_CLIENT_ID) &&
				present(env.APPLE_TEAM_ID) &&
				present(env.APPLE_KEY_ID) &&
				present(env.APPLE_REDIRECT_URI) &&
				present(env.APPLE_PRIVATE_KEY)
			);
		case "google":
			return present(env.GOOGLE_CLIENT_ID) && present(env.GOOGLE_CLIENT_SECRET);
		case "github":
			// The device flow needs no client secret; that is the point of choosing it.
			return present(env.GITHUB_CLIENT_ID);
	}
}

/** A host WebAuthn can use as an RP ID: not an IP address, and not a URL. */
function usableRpId(rpId: string): boolean {
	if (!/^[A-Za-z0-9.-]+$/.test(rpId)) return false;
	if (rpId.includes("..") || rpId.startsWith(".") || rpId.endsWith(".") || rpId.startsWith("-")) return false;
	// IPv4, and anything bracketed (an IPv6 literal) is not a valid RP ID.
	return !/^\d{1,3}(\.\d{1,3}){3}$/.test(rpId);
}

/** The https origins (or `http://localhost`) accepted in `clientDataJSON`, each under the RP ID. */
function passkeyOrigins(raw: string | undefined, rpId: string): string[] | null {
	if (!present(raw)) return null;
	let parsed: unknown;
	try {
		parsed = JSON.parse(raw);
	} catch {
		return null;
	}
	if (!Array.isArray(parsed) || parsed.length === 0) return null;
	const origins: string[] = [];
	for (const value of parsed) {
		if (typeof value !== "string") return null;
		let url: URL;
		try {
			url = new URL(value);
		} catch {
			return null;
		}
		// No path, query or fragment: an origin is the scheme, host and port only.
		if (url.origin !== value.replace(/\/$/, "") || value.endsWith("/")) return null;
		const localhost = url.protocol === "http:" && url.hostname === "localhost";
		if (url.protocol !== "https:" && !localhost) return null;
		if (url.hostname !== rpId && !url.hostname.endsWith(`.${rpId}`)) return null;
		origins.push(value);
	}
	return origins;
}

/** The passkey half of the advertisement, or null when this deployment can't run a ceremony. */
export function passkeyAdvertisement(env: Env): PasskeyAdvertisement | null {
	const rpId = env.PASSKEY_RP_ID?.trim();
	if (!present(rpId) || !usableRpId(rpId)) return null;
	const origins = passkeyOrigins(env.PASSKEY_ORIGINS, rpId);
	if (!origins) return null;
	// Linux registers and asserts through CTAP2 directly, so a configured passkey
	// deployment always accepts a security-key ceremony. The app still checks for
	// an attached authenticator itself; health cannot see one.
	return { rp_id: rpId, origins, security_key: true };
}

/** `GET /v2/health`'s `auth` object. Always present on this Worker; an older one omits it. */
export function healthAuth(env: Env): AuthAdvertisement {
	return {
		password: true,
		passkeys: passkeyAdvertisement(env),
		providers: PROVIDERS.filter((provider) => providerConfigured(env, provider)),
	};
}

// ---------------------------------------------------------------- auth flows

/** One ceremony (docs/ACCOUNTS.md › Migration 0006 › `auth_flows`). */
export interface AuthFlowRow {
	id: string;
	kind: string;
	intent: "signin" | "signup" | "link" | "proof" | null;
	account_id: string | null;
	device_id: string | null;
	device_proof_hash: string | null;
	challenge: string | null;
	rp_id: string | null;
	nonce_hash: string | null;
	state_hash: string | null;
	pkce_hash: string | null;
	redirect_uri: string | null;
	provider: string | null;
	subject: string | null;
	device_code: string | null;
	next_poll_at: number | null;
	created_at: number;
	expires_at: number;
	consumed_at: number | null;
}

export type FlowInput = Omit<AuthFlowRow, "id" | "created_at" | "consumed_at"> & { id?: string };

const FLOW_COLUMNS = `id, kind, intent, account_id, device_id, device_proof_hash, challenge, rp_id,
  nonce_hash, state_hash, pkce_hash, redirect_uri, provider, subject, device_code, next_poll_at,
  created_at, expires_at, consumed_at`;

const FLOW_LABEL = "kanso-flow\u0001";

/** AAD for a sealed flow secret: the label, then the flow's account (empty for a sign-up flow). */
function flowAad(accountId: string | null): Uint8Array {
	const id = new TextEncoder().encode(accountId ?? "");
	const label = new TextEncoder().encode(FLOW_LABEL);
	const aad = new Uint8Array(label.length + 2 + id.length);
	aad.set(label, 0);
	aad[label.length] = id.length >> 8;
	aad[label.length + 1] = id.length & 0xff;
	aad.set(id, label.length + 2);
	return aad;
}

/**
 * Seals a short-lived flow secret (today, GitHub's device code) under the
 * pepper's flow key: `0x01 || nonce(12) || AES-256-GCM(ciphertext + tag)`.
 * The pepper is not in D1, so a database copy or Time Travel row alone can't
 * mint a provider token from it.
 */
export async function sealFlowSecret(keys: PepperKeys, accountId: string | null, value: string): Promise<string> {
	const nonce = randomBytes(12);
	const ciphertext = new Uint8Array(
		await crypto.subtle.encrypt(
			{ name: "AES-GCM", iv: nonce, additionalData: flowAad(accountId), tagLength: 128 },
			keys.flow,
			new TextEncoder().encode(value),
		),
	);
	const out = new Uint8Array(1 + nonce.length + ciphertext.length);
	out[0] = 0x01;
	out.set(nonce, 1);
	out.set(ciphertext, 1 + nonce.length);
	return base64url(out);
}

/** Opens a sealed flow secret, or null for a wrong key, wrong account or a malformed value. */
export async function openFlowSecret(keys: PepperKeys, accountId: string | null, sealed: string): Promise<string | null> {
	const bytes = fromBase64url(sealed);
	if (!bytes || bytes.length < 1 + 12 + 16 + 1 || bytes[0] !== 0x01) return null;
	try {
		const plain = await crypto.subtle.decrypt(
			{ name: "AES-GCM", iv: bytes.subarray(1, 13), additionalData: flowAad(accountId), tagLength: 128 },
			keys.flow,
			bytes.subarray(13),
		);
		return new TextDecoder().decode(plain);
	} catch {
		return null;
	}
}

/** The flow's GitHub device code, opened for the one exchange that may use it. */
export function openDeviceCode(keys: PepperKeys, flow: AuthFlowRow): Promise<string | null> {
	return flow.device_code === null ? Promise.resolve(null) : openFlowSecret(keys, flow.account_id, flow.device_code);
}

/** Stores a ceremony; the caller owns the ID (a 32-byte flow ID) or gets a random one back. */
export async function newFlow(env: Env, keys: PepperKeys, flow: FlowInput, now: number): Promise<string> {
	const id = flow.id ?? base64url(randomBytes(32));
	// The column holds no plaintext device code; only the exchange route opens it.
	const deviceCode = flow.device_code === null ? null : await sealFlowSecret(keys, flow.account_id, flow.device_code);
	await env.DB.prepare(
		`INSERT INTO auth_flows (${FLOW_COLUMNS}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, NULL)`,
	)
		.bind(
			id,
			flow.kind,
			flow.intent,
			flow.account_id,
			flow.device_id,
			flow.device_proof_hash,
			flow.challenge,
			flow.rp_id,
			flow.nonce_hash,
			flow.state_hash,
			flow.pkce_hash,
			flow.redirect_uri,
			flow.provider,
			flow.subject,
			deviceCode,
			flow.next_poll_at,
			now,
			flow.expires_at,
		)
		.run();
	return id;
}

export function flowById(env: Env, id: string): Promise<AuthFlowRow | null> {
	return env.DB.prepare(`SELECT ${FLOW_COLUMNS} FROM auth_flows WHERE id = ?1`).bind(id).first<AuthFlowRow>();
}

/**
 * The flow this device started, or the response that ends the request.
 * A flow ID captured from a callback URL grants nothing on another device:
 * the hash of its proof must match (`bad_request`, never which part failed).
 * An expired or unknown flow is `flow_expired`.
 *
 * A consumed flow is returned on purpose: a finish or poll that lost its
 * answer must be able to replay the same result, never start a new ceremony.
 * Callers check `consumed_at` when the two paths differ (for example, to
 * return the account an earlier attempt created instead of creating another).
 */
export async function flowForDevice(
	env: Env,
	id: string,
	device: { id: string; proof: string },
	now: number,
): Promise<AuthFlowRow | Response> {
	if (!isFlowId(id)) return badRequest("flow_id must be 32 bytes of canonical base64url.");
	const flow = await flowById(env, id);
	if (!flow || flow.expires_at <= now) return flowExpired();
	if (flow.device_id !== device.id || flow.device_proof_hash !== (await sha256Hex(device.proof))) {
		return badRequest("This flow belongs to another device.");
	}
	return flow;
}

/**
 * Marks a live flow used, once. `false` when another request already consumed
 * it or it has expired; a route treats the first as its idempotent retry path
 * and the second as `flow_expired`, never as a new ceremony.
 */
export async function consumeFlow(env: Env, id: string, now: number): Promise<boolean> {
	const row = await env.DB.prepare(
		"UPDATE auth_flows SET consumed_at = ?2 WHERE id = ?1 AND consumed_at IS NULL AND expires_at > ?2 RETURNING id",
	)
		.bind(id, now)
		.first<{ id: string }>();
	return row !== null;
}

export const flowExpired = (): Response => error(400, "flow_expired", "This sign-in flow has expired.");

const FLOW_ID = /^[A-Za-z0-9_-]{43}$/;
export const isFlowId = (value: string): boolean => FLOW_ID.test(value);

