// A software authenticator for tests, built on node:crypto: it produces the
// exact WebAuthn messages a platform passkey or a Linux security key would (a
// COSE public key, attestation "none", and signatures over
// `authenticatorData || SHA-256(clientDataJSON)`). No page is loaded and no
// network is reached. An Ed25519 credential made from a seed is
// deterministic, which the shared sign-in fixture relies on: Ed25519
// signatures depend only on the key and the message.
/// <reference types="node" />

import { Buffer } from "node:buffer";
import { type KeyObject, createHash, createPrivateKey, createPublicKey, generateKeyPairSync, randomBytes, sign as nodeSign } from "node:crypto";
import type { Env } from "../src/env";
import { base64url } from "../src/protocol";
import { testEnv } from "./helpers";

export const PASSKEY_ORIGIN = "https://kanso-sync.test";
export const RP_ID = "kanso-sync.test";
export const passkeyConfig = { PASSKEY_RP_ID: RP_ID, PASSKEY_ORIGINS: JSON.stringify([PASSKEY_ORIGIN]) };
export const passkeyEnv = (overrides: Partial<Env> = {}): Env => testEnv({ ...passkeyConfig, ...overrides });

const encoder = new TextEncoder();
export const sha256 = (data: Uint8Array | string): Uint8Array =>
	new Uint8Array(createHash("sha256").update(typeof data === "string" ? Buffer.from(data) : data).digest());

// --------------------------------------------------------------------- CBOR

export type Cbor = number | string | Uint8Array | Cbor[] | Map<number | string, Cbor>;

function head(major: number, length: number): number[] {
	if (length < 24) return [(major << 5) | length];
	if (length <= 0xff) return [(major << 5) | 24, length];
	if (length <= 0xffff) return [(major << 5) | 25, length >> 8, length & 0xff];
	return [(major << 5) | 26, (length >>> 24) & 0xff, (length >>> 16) & 0xff, (length >>> 8) & 0xff, length & 0xff];
}

/** Definite-length CBOR, map entries in insertion order. */
export function cbor(value: Cbor): number[] {
	if (typeof value === "number") return value >= 0 ? head(0, value) : head(1, -1 - value);
	if (typeof value === "string") {
		const bytes = encoder.encode(value);
		return [...head(3, bytes.length), ...bytes];
	}
	if (value instanceof Uint8Array) return [...head(2, value.length), ...value];
	if (Array.isArray(value)) return [...head(4, value.length), ...value.flatMap(cbor)];
	const entries = [...value.entries()];
	return [...head(5, entries.length), ...entries.flatMap(([k, v]) => [...cbor(k), ...cbor(v)])];
}

// ------------------------------------------------------------ credentials

export interface Credential {
	privateKey: KeyObject;
	credentialId: Uint8Array;
	cose: Map<number | string, Cbor>;
	alg: -7 | -8;
}

/** A P-256 (ES256) credential with a random ID. */
export function newCredential(): Credential {
	const { privateKey, publicKey } = generateKeyPairSync("ec", { namedCurve: "prime256v1" });
	const jwk = publicKey.export({ format: "jwk" }) as { x: string; y: string };
	const cose = new Map<number | string, Cbor>([
		[1, 2],
		[3, -7],
		[-1, 1],
		[-2, new Uint8Array(Buffer.from(jwk.x, "base64url"))],
		[-3, new Uint8Array(Buffer.from(jwk.y, "base64url"))],
	]);
	return { privateKey, credentialId: new Uint8Array(randomBytes(32)), cose, alg: -7 };
}

/** PKCS#8 DER prefix of an Ed25519 private key; the 32-byte seed follows. */
const ED25519_PKCS8 = Buffer.from("302e020100300506032b657004220420", "hex");

/**
 * An Ed25519 (EdDSA, -8) credential: random, or from a 32-byte `seed` and a
 * fixed credential ID, which makes every message it signs deterministic.
 */
export function newEd25519Credential(seed?: Uint8Array, credentialId?: Uint8Array): Credential {
	const privateKey = seed
		? createPrivateKey({ key: Buffer.concat([ED25519_PKCS8, Buffer.from(seed)]), format: "der", type: "pkcs8" })
		: generateKeyPairSync("ed25519").privateKey;
	const jwk = createPublicKey(privateKey).export({ format: "jwk" }) as { x: string };
	const cose = new Map<number | string, Cbor>([
		[1, 1],
		[3, -8],
		[-1, 6],
		[-2, new Uint8Array(Buffer.from(jwk.x, "base64url"))],
	]);
	return { privateKey, credentialId: credentialId ?? new Uint8Array(randomBytes(32)), cose, alg: -8 };
}

// --------------------------------------------------------------- messages

export function authData(options: { rpId?: string; flags: number; signCount: number; credential?: Credential; aaguid?: Uint8Array }): Uint8Array {
	const count = new Uint8Array(4);
	new DataView(count.buffer).setUint32(0, options.signCount);
	const parts: number[] = [...sha256(options.rpId ?? RP_ID), options.flags, ...count];
	if (options.credential) {
		parts.push(...(options.aaguid ?? new Uint8Array(16).fill(7)));
		parts.push(options.credential.credentialId.length >> 8, options.credential.credentialId.length & 0xff);
		parts.push(...options.credential.credentialId);
		parts.push(...cbor(options.credential.cose));
	}
	return new Uint8Array(parts);
}

export function clientData(type: string, challenge: string, origin = PASSKEY_ORIGIN, crossOrigin: boolean | undefined = false): Uint8Array {
	return encoder.encode(JSON.stringify({ type, challenge, origin, ...(crossOrigin === undefined ? {} : { crossOrigin }) }));
}

export interface RegistrationOptions {
	origin?: string;
	rpId?: string;
	/** UP, UV and AT by default (0x45). */
	flags?: number;
	signCount?: number;
	attStmt?: Cbor | null;
	aaguid?: Uint8Array;
}

/** `{credential_id, attestation_object, client_data_json}` for a registration over `challenge`. */
export function registration(credential: Credential, challenge: string, options: RegistrationOptions = {}) {
	const data = authData({ flags: options.flags ?? 0x45, signCount: options.signCount ?? 0, credential, rpId: options.rpId, aaguid: options.aaguid });
	const attStmt = options.attStmt === undefined ? new Map() : options.attStmt;
	const object = new Map<number | string, Cbor>([["fmt", "none"]]);
	if (attStmt !== null) object.set("attStmt", attStmt);
	object.set("authData", data);
	return {
		credential_id: base64url(credential.credentialId),
		attestation_object: base64url(new Uint8Array(cbor(object))),
		client_data_json: base64url(clientData("webauthn.create", challenge, options.origin)),
	};
}

export interface AssertionOptions {
	origin?: string;
	/** UP and UV by default (0x05). */
	flags?: number;
	signCount?: number;
	challengeOverride?: string;
	rpId?: string;
	signatureBitFlip?: boolean;
}

/** `{credential_id, authenticator_data, client_data_json, signature}` for an assertion over `challenge`. */
export function assertion(credential: Credential, challenge: string, options: AssertionOptions = {}) {
	const data = authData({ flags: options.flags ?? 0x05, signCount: options.signCount ?? 1, rpId: options.rpId });
	const client = clientData("webauthn.get", options.challengeOverride ?? challenge, options.origin);
	const signed = new Uint8Array(data.length + 32);
	signed.set(data);
	signed.set(sha256(client), data.length);
	let signature: Uint8Array;
	if (credential.alg === -8) {
		signature = new Uint8Array(nodeSign(null, Buffer.from(signed), credential.privateKey));
	} else {
		// WebAuthn carries ECDSA signatures in ASN.1 DER, as authenticators do.
		signature = new Uint8Array(nodeSign("sha256", Buffer.from(signed), credential.privateKey));
	}
	if (options.signatureBitFlip) signature[0] ^= 1;
	return {
		credential_id: base64url(credential.credentialId),
		authenticator_data: base64url(data),
		client_data_json: base64url(client),
		signature: base64url(signature),
	};
}
