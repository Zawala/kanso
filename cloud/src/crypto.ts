// Hashes, tokens and the pepper's subkeys (docs/ACCOUNTS.md › Server-side
// hashes and the pepper). The Worker never runs Argon2 and never decrypts
// anything: it hashes 32-byte proofs and compares the digests in constant time.

import { base64url, proof32 } from "./protocol";

const encoder = new TextEncoder();

export const hex = (bytes: Uint8Array): string => [...bytes].map((b) => b.toString(16).padStart(2, "0")).join("");

export function fromHex(text: string): Uint8Array {
	const out = new Uint8Array(text.length / 2);
	for (let i = 0; i < out.length; i++) out[i] = parseInt(text.slice(2 * i, 2 * i + 2), 16);
	return out;
}

export async function sha256(data: Uint8Array | string): Promise<Uint8Array> {
	const bytes = typeof data === "string" ? encoder.encode(data) : data;
	return new Uint8Array(await crypto.subtle.digest("SHA-256", bytes));
}

export const sha256Hex = async (data: Uint8Array | string): Promise<string> => hex(await sha256(data));

export const randomBytes = (n: number): Uint8Array => crypto.getRandomValues(new Uint8Array(n));
export const randomHex = (bytes: number): string => hex(randomBytes(bytes));

/** `kanso_` and 32 random bytes in unpadded base64url. */
export const newToken = (): string => `kanso_${base64url(randomBytes(32))}`;

/** The SHA-256 of a token as 64 lowercase hex digits: the only form of a token the database holds. */
export const hashToken = (token: string): Promise<string> => sha256Hex(token);

const ZERO_DIGEST = new Uint8Array(32);

/**
 * Whether SHA-256 of `proof` equals the stored hex digest, in constant time.
 * With no stored digest (an unknown username) the proof is still hashed and
 * compared, against zeros, so both cases cost the same.
 */
export async function proofMatches(proof: Uint8Array, storedHex: string | null | undefined): Promise<boolean> {
	const digest = await sha256(proof);
	const stored = storedHex && storedHex.length === 64 ? fromHex(storedHex) : ZERO_DIGEST;
	return crypto.subtle.timingSafeEqual(digest, stored) && stored !== ZERO_DIGEST;
}

// -------------------------------------------------------------------- pepper

export interface PepperKeys {
	/** HMAC key for fake prelogin answers. */
	fake: CryptoKey;
	/** HMAC key for IP rate-limit keys. */
	limit: CryptoKey;
	/**
	 * AES-GCM key for short-lived flow secrets. A GitHub device code can mint
	 * an access token, so `auth_flows.device_code` holds it sealed under this
	 * key rather than in the clear (docs/ACCOUNTS.md › Migration 0006).
	 */
	flow: CryptoKey;
}

const peppers = new Map<string, Promise<PepperKeys>>();

async function derive(pepper: Uint8Array, info: string): Promise<Uint8Array> {
	const ikm = await crypto.subtle.importKey("raw", pepper, "HKDF", false, ["deriveBits"]);
	const bits = await crypto.subtle.deriveBits(
		{ name: "HKDF", hash: "SHA-256", salt: new Uint8Array(32), info: encoder.encode(info) },
		ikm,
		256,
	);
	return new Uint8Array(bits);
}

async function subkey(pepper: Uint8Array, info: string): Promise<CryptoKey> {
	return crypto.subtle.importKey("raw", await derive(pepper, info), { name: "HMAC", hash: "SHA-256" }, false, ["sign"]);
}

/** The pepper's HKDF subkeys, or null when AUTH_PEPPER is missing or isn't 32 bytes of canonical base64url. */
export function pepperKeys(pepper: string | undefined): Promise<PepperKeys> | null {
	if (pepper === undefined) return null;
	const bytes = proof32(pepper);
	if (!bytes) return null;
	let keys = peppers.get(pepper);
	if (!keys) {
		keys = Promise.all([
			subkey(bytes, "kanso/v1/fake"),
			subkey(bytes, "kanso/v1/limit"),
			derive(bytes, "kanso/v1/flow").then((bits) => crypto.subtle.importKey("raw", bits, { name: "AES-GCM" }, false, ["encrypt", "decrypt"])),
		]).then(([fake, limit, flow]) => ({ fake, limit, flow }));
		peppers.set(pepper, keys);
	}
	return keys;
}

/** HMAC-SHA256(key, label || 0x00 || subject), first 16 bytes. */
export async function hmac16(key: CryptoKey, label: string, subject: string): Promise<Uint8Array> {
	const input = encoder.encode(`${label}\u0000${subject}`);
	return new Uint8Array(await crypto.subtle.sign("HMAC", key, input)).slice(0, 16);
}
