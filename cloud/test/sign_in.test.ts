// Provider tokens and health advertisement (design: Kanso accounts spec, revision 3).
/// <reference types="node" />

import { Buffer } from "node:buffer";
import { createHash, generateKeyPairSync, sign as nodeSign, verify as nodeVerify } from "node:crypto";
import type { KeyObject } from "node:crypto";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
	APPLE_ID_TOKEN_ALGORITHMS,
	APPLE_ISSUER,
	APPLE_JWKS_URL,
	APPLE_TOKEN_URL,
	GITHUB_TOKEN_URL,
	GITHUB_USER_URL,
	GOOGLE_ID_TOKEN_ALGORITHMS,
	GOOGLE_ISSUERS,
	GOOGLE_JWKS_URL,
	GOOGLE_TOKEN_URL,
	appleClientSecret,
	exchangeAppleCode,
	exchangeGoogleCode,
	githubDeviceToken,
	githubSubject,
	pkceChallenge,
	pkceMatches,
	resetProviderCache,
	verifyIdToken,
} from "../src/oauth";
import { env } from "cloudflare:workers";
import { healthAuth } from "../src/sign_in";
import { callJson, resetServer, signUp, testEnv } from "./helpers";

const NOW = Date.UTC(2026, 9, 24, 12);
const nowSeconds = Math.floor(NOW / 1000);

const rsa = generateKeyPairSync("rsa", { modulusLength: 2048 });
const otherRsa = generateKeyPairSync("rsa", { modulusLength: 2048 });
const ec = generateKeyPairSync("ec", { namedCurve: "prime256v1" });
const rsaJwk = { ...rsa.publicKey.export({ format: "jwk" }), kid: "rsa-key" };
const ecJwk = { ...ec.publicKey.export({ format: "jwk" }), kid: "ec-key" };
const ecPrivatePem = String(ec.privateKey.export({ type: "pkcs8", format: "pem" }));

const encode = (value: string | Uint8Array): string => Buffer.from(value).toString("base64url");

function claims(overrides: Record<string, unknown> = {}): Record<string, unknown> {
	return {
		iss: APPLE_ISSUER,
		aud: "apple-client",
		exp: nowSeconds + 3600,
		iat: nowSeconds,
		sub: "000123.abc",
		nonce: "nonce-value",
		...overrides,
	};
}

function signedToken(
	privateKey: KeyObject,
	signingAlg: "RS256" | "ES256",
	claimValues: Record<string, unknown> = claims(),
	options: { kid?: string; headerAlg?: string; typ?: string; includeTyp?: boolean } = {},
): string {
	const header: Record<string, string> = { alg: options.headerAlg ?? signingAlg, kid: options.kid ?? (signingAlg === "RS256" ? "rsa-key" : "ec-key") };
	if (options.includeTyp !== false) header.typ = options.typ ?? "JWT";
	const signingInput = `${encode(JSON.stringify(header))}.${encode(JSON.stringify(claimValues))}`;
	const signature =
		signingAlg === "RS256"
			? nodeSign("RSA-SHA256", Buffer.from(signingInput, "ascii"), privateKey)
			: nodeSign("sha256", Buffer.from(signingInput, "ascii"), {
					key: String(privateKey.export({ type: "pkcs8", format: "pem" })),
					dsaEncoding: "ieee-p1363",
			  });
	return `${signingInput}.${encode(signature)}`;
}

function jsonResponse(value: unknown, status = 200): Response {
	return new Response(JSON.stringify(value), { status, headers: { "content-type": "application/json" } });
}

function stubFetch(response: (input: Parameters<typeof fetch>[0], init?: RequestInit) => Promise<Response> | Response) {
	const fetchMock = vi.fn<typeof fetch>().mockImplementation(async (input, init) => response(input, init));
	vi.stubGlobal("fetch", fetchMock);
	return fetchMock;
}

const appleOptions = {
	jwks_url: APPLE_JWKS_URL,
	algorithms: APPLE_ID_TOKEN_ALGORITHMS,
	issuers: [APPLE_ISSUER],
	audience: "apple-client",
	now: NOW,
};

beforeEach(() => {
	vi.unstubAllGlobals();
	vi.restoreAllMocks();
	resetProviderCache();
});

describe("provider ID tokens", () => {
	it("verifies a valid Apple RS256 token and returns its claims", async () => {
		stubFetch(() => jsonResponse({ keys: [rsaJwk] }));
		const result = await verifyIdToken(signedToken(rsa.privateKey, "RS256"), appleOptions);
		expect(result).toEqual({ ok: true, claims: expect.objectContaining({ sub: "000123.abc" }) });
	});

	it("verifies a valid Google RS256 token", async () => {
		stubFetch(() => jsonResponse({ keys: [rsaJwk] }));
		const token = signedToken(rsa.privateKey, "RS256", claims({ iss: "accounts.google.com", aud: ["other", "google-client"] }));
		const result = await verifyIdToken(token, {
			jwks_url: GOOGLE_JWKS_URL,
			algorithms: GOOGLE_ID_TOKEN_ALGORITHMS,
			issuers: GOOGLE_ISSUERS,
			audience: "google-client",
			now: NOW,
		});
		expect(result).toMatchObject({ ok: true, claims: { sub: "000123.abc" } });
	});

	it("rejects a bad signature", async () => {
		stubFetch(() => jsonResponse({ keys: [rsaJwk] }));
		const result = await verifyIdToken(signedToken(otherRsa.privateKey, "RS256"), appleOptions);
		expect(result).toEqual({ ok: false, error: "invalid_credentials" });
	});

	it("refetches an unknown kid exactly once, then refuses a second miss without asking again", async () => {
		const fetchMock = stubFetch(() => jsonResponse({ keys: [rsaJwk] }));
		const token = signedToken(rsa.privateKey, "RS256", claims(), { kid: "rotated-key" });
		expect(await verifyIdToken(token, appleOptions)).toEqual({ ok: false, error: "invalid_credentials" });
		expect(fetchMock).toHaveBeenCalledTimes(2);
		expect(fetchMock.mock.calls.map(([input]) => String(input))).toEqual([APPLE_JWKS_URL, APPLE_JWKS_URL]);
		// The one refetch for this cache window is spent: another unknown kid is
		// refused locally instead of hammering the provider.
		expect(await verifyIdToken(token, appleOptions)).toEqual({ ok: false, error: "invalid_credentials" });
		expect(fetchMock).toHaveBeenCalledTimes(2);
	});

	it.each(["none", "HS256"])("refuses algorithm %s without fetching JWKS", async (alg) => {
		const fetchMock = stubFetch(() => jsonResponse({ keys: [rsaJwk] }));
		const token = signedToken(rsa.privateKey, "RS256", claims(), { headerAlg: alg });
		const result = await verifyIdToken(token, appleOptions);
		expect(result).toEqual({ ok: false, error: "invalid_credentials" });
		expect(fetchMock).not.toHaveBeenCalled();
	});

	it("rejects a wrong audience, issuer, expired token, or future-issued token", async () => {
		stubFetch(() => jsonResponse({ keys: [rsaJwk] }));
		const invalidClaims = [
			claims({ aud: "another-client" }),
			claims({ iss: "https://evil.example" }),
			claims({ exp: nowSeconds - 121 }),
			claims({ iat: nowSeconds + 121 }),
		];
		for (const value of invalidClaims) {
			const result = await verifyIdToken(signedToken(rsa.privateKey, "RS256", value), appleOptions);
			expect(result).toEqual({ ok: false, error: "invalid_credentials" });
		}
	});

	it("reports a nonce mismatch and accepts a matching nonce", async () => {
		stubFetch(() => jsonResponse({ keys: [rsaJwk] }));
		const mismatch = await verifyIdToken(signedToken(rsa.privateKey, "RS256", claims({ nonce: "other" })), {
			...appleOptions,
			nonce: "expected",
		});
		expect(mismatch).toEqual({ ok: false, error: "nonce_reused" });
		const match = await verifyIdToken(signedToken(rsa.privateKey, "RS256", claims({ nonce: "expected" })), {
			...appleOptions,
			nonce: "expected",
		});
		expect(match.ok).toBe(true);
	});

	it("caches a JWKS for consecutive verifications", async () => {
		const fetchMock = stubFetch(() => jsonResponse({ keys: [rsaJwk] }));
		const token = signedToken(rsa.privateKey, "RS256");
		expect((await verifyIdToken(token, appleOptions)).ok).toBe(true);
		expect((await verifyIdToken(token, appleOptions)).ok).toBe(true);
		expect(fetchMock).toHaveBeenCalledTimes(1);
	});

	it("verifies ES256 for a provider that allows it, and refuses a tampered signature", async () => {
		stubFetch(() => jsonResponse({ keys: [ecJwk] }));
		const token = signedToken(ec.privateKey, "ES256");
		const es256Options = { ...appleOptions, algorithms: ["ES256"] as const };
		expect(await verifyIdToken(token, es256Options)).toMatchObject({ ok: true, claims: { sub: "000123.abc" } });
		const parts = token.split(".");
		const signature = Buffer.from(parts[2], "base64url");
		signature[0] ^= 1;
		const tampered = `${parts[0]}.${parts[1]}.${encode(signature)}`;
		expect(await verifyIdToken(tampered, es256Options)).toEqual({ ok: false, error: "invalid_credentials" });
	});

	it("refuses an ES256 ID token when the provider pins RS256", async () => {
		const fetchMock = stubFetch(() => jsonResponse({ keys: [ecJwk] }));
		const token = signedToken(ec.privateKey, "ES256");
		expect(await verifyIdToken(token, appleOptions)).toEqual({ ok: false, error: "invalid_credentials" });
		expect(await verifyIdToken(token, { ...appleOptions, jwks_url: GOOGLE_JWKS_URL, issuers: GOOGLE_ISSUERS, audience: "apple-client", algorithms: GOOGLE_ID_TOKEN_ALGORITHMS })).toEqual({ ok: false, error: "invalid_credentials" });
		expect(fetchMock).not.toHaveBeenCalled();
	});

	it("refuses a token whose payload was swapped for another", async () => {
		stubFetch(() => jsonResponse({ keys: [rsaJwk] }));
		const original = signedToken(rsa.privateKey, "RS256");
		const [header, , signature] = original.split(".");
		const swapped = `${header}.${encode(JSON.stringify(claims({ sub: "attacker" })))}.${signature}`;
		expect(await verifyIdToken(swapped, appleOptions)).toEqual({ ok: false, error: "invalid_credentials" });
	});

	it("allows typ to be omitted but refuses a non-JWT typ", async () => {
		stubFetch(() => jsonResponse({ keys: [rsaJwk] }));
		const noType = await verifyIdToken(signedToken(rsa.privateKey, "RS256", claims(), { includeTyp: false }), appleOptions);
		expect(noType.ok).toBe(true);
		const wrongType = await verifyIdToken(signedToken(rsa.privateKey, "RS256", claims(), { typ: "application/example" }), appleOptions);
		expect(wrongType).toEqual({ ok: false, error: "invalid_credentials" });
	});

	it("refuses malformed and non-canonical JWT segments", async () => {
		const fetchMock = stubFetch(() => jsonResponse({ keys: [rsaJwk] }));
		for (const token of ["a.b", "e30=.e30.sig", "a.!.AA", "W10.W10.AA"]) {
			expect(await verifyIdToken(token, appleOptions)).toEqual({ ok: false, error: "invalid_credentials" });
		}
		expect(fetchMock).not.toHaveBeenCalled();
	});
});

describe("PKCE", () => {
	it("computes the S256 challenge and matches only its verifier", async () => {
		const verifier = "a-long-random-pkce-verifier";
		const expected = createHash("sha256").update(verifier).digest("base64url");
		expect(await pkceChallenge(verifier)).toBe(expected);
		expect(await pkceMatches(verifier, expected)).toBe(true);
		expect(await pkceMatches("different-verifier", expected)).toBe(false);
		expect(await pkceMatches(verifier, `${expected}=`)).toBe(false);
	});
});

describe("Apple client secret", () => {
	const appleEnv = () =>
		testEnv({
			APPLE_CLIENT_ID: "app.kanso.Kanso.web",
			APPLE_TEAM_ID: "4973Y692JU",
			APPLE_KEY_ID: "MS6Y3S5ZL7",
			APPLE_REDIRECT_URI: "https://sync.kansoapp.app/v2/auth/apple/callback",
			APPLE_PRIVATE_KEY: ecPrivatePem,
		});

	it("mints a correctly formed, verifiable ES256 secret", async () => {
		const secret = await appleClientSecret(appleEnv(), NOW);
		const [encodedHeader, encodedClaims, encodedSignature] = secret.split(".");
		const header = JSON.parse(Buffer.from(encodedHeader, "base64url").toString("utf8")) as Record<string, unknown>;
		const body = JSON.parse(Buffer.from(encodedClaims, "base64url").toString("utf8")) as Record<string, unknown>;
		expect(header).toEqual({ alg: "ES256", kid: "MS6Y3S5ZL7" });
		expect(body).toEqual({
			iss: "4973Y692JU",
			iat: nowSeconds,
			exp: nowSeconds + 86400,
			aud: APPLE_ISSUER,
			sub: "app.kanso.Kanso.web",
		});
		expect(
			nodeVerify(
				"sha256",
				Buffer.from(`${encodedHeader}.${encodedClaims}`, "ascii"),
				{ key: String(ec.publicKey.export({ type: "spki", format: "pem" })), dsaEncoding: "ieee-p1363" },
				Buffer.from(encodedSignature, "base64url"),
			),
		).toBe(true);
		expect(await appleClientSecret(appleEnv(), NOW + 22 * 60 * 60 * 1000)).toBe(secret);
		expect(await appleClientSecret(appleEnv(), NOW + 23 * 60 * 60 * 1000 + 1)).not.toBe(secret);
	});

	it("throws only a generic typed failure for missing or malformed key configuration", async () => {
		await expect(appleClientSecret(testEnv(), NOW)).rejects.toMatchObject({ name: "ProviderUnavailable", message: "Provider unavailable." });
		await expect(appleClientSecret({ ...appleEnv(), APPLE_PRIVATE_KEY: "not a PEM" }, NOW)).rejects.toMatchObject({
			name: "ProviderUnavailable",
			message: "Provider unavailable.",
		});
	});
});

describe("provider code exchanges", () => {
	it("posts the Apple authorization code with its fixed redirect URI", async () => {
		const fetchMock = stubFetch(() => jsonResponse({ id_token: "apple-id-token" }));
		const result = await exchangeAppleCode(
			testEnv({
				APPLE_CLIENT_ID: "apple-client",
				APPLE_TEAM_ID: "team",
				APPLE_KEY_ID: "key",
				APPLE_REDIRECT_URI: "https://sync.kansoapp.app/v2/auth/apple/callback",
				APPLE_PRIVATE_KEY: ecPrivatePem,
			}),
			"authorization-code",
			NOW,
		);
		expect(result).toEqual({ ok: true, id_token: "apple-id-token" });
		expect(fetchMock).toHaveBeenCalledTimes(1);
		const [url, init] = fetchMock.mock.calls[0];
		expect(url).toBe(APPLE_TOKEN_URL);
		expect(init?.redirect).toBe("error");
		expect(init?.signal).toBeDefined();
		const form = new URLSearchParams(String(init?.body));
		expect(form.get("client_id")).toBe("apple-client");
		expect(form.get("code")).toBe("authorization-code");
		expect(form.get("grant_type")).toBe("authorization_code");
		expect(form.get("redirect_uri")).toBe("https://sync.kansoapp.app/v2/auth/apple/callback");
		expect(form.get("client_secret")).toContain(".");
	});

	it("posts Google's PKCE verifier and optional loopback redirect URI", async () => {
		const fetchMock = stubFetch(() => jsonResponse({ id_token: "google-id-token" }));
		const result = await exchangeGoogleCode(
			testEnv({ GOOGLE_CLIENT_ID: "google-client", GOOGLE_CLIENT_SECRET: "google-secret" }),
			"authorization-code",
			"pkce-verifier",
			"http://127.0.0.1:49152",
		);
		expect(result).toEqual({ ok: true, id_token: "google-id-token" });
		const [url, init] = fetchMock.mock.calls[0];
		expect(url).toBe(GOOGLE_TOKEN_URL);
		const form = new URLSearchParams(String(init?.body));
		expect(form.get("code")).toBe("authorization-code");
		expect(form.get("client_id")).toBe("google-client");
		expect(form.get("client_secret")).toBe("google-secret");
		expect(form.get("code_verifier")).toBe("pkce-verifier");
		expect(form.get("grant_type")).toBe("authorization_code");
		expect(form.get("redirect_uri")).toBe("http://127.0.0.1:49152");
	});

	it("maps invalid_grant to invalid_credentials and hides network failures", async () => {
		stubFetch(() => jsonResponse({ error: "invalid_grant", error_description: "sensitive provider details" }, 400));
		expect(await exchangeGoogleCode(testEnv({ GOOGLE_CLIENT_ID: "id", GOOGLE_CLIENT_SECRET: "secret" }), "bad-code", "verifier"))
			.toEqual({ ok: false, error: "invalid_credentials" });
		stubFetch(() => {
			throw new Error("authorization-code must not appear in errors");
		});
		expect(await exchangeGoogleCode(testEnv({ GOOGLE_CLIENT_ID: "id", GOOGLE_CLIENT_SECRET: "secret" }), "bad-code", "verifier"))
			.toEqual({ ok: false, error: "provider_unavailable" });
	});
});

describe("GitHub device flow", () => {
	it.each([
		["authorization_pending", "pending"],
		["slow_down", "slow_down"],
		["expired_token", "flow_expired"],
		["access_denied", "provider_denied"],
	] as const)("maps %s", async (providerError, expected) => {
		const fetchMock = stubFetch(() => jsonResponse({ error: providerError }));
		const result = await githubDeviceToken(testEnv({ GITHUB_CLIENT_ID: "github-client" }), "device-code-value");
		expect(result).toEqual({ ok: false, error: expected });
		const [url, init] = fetchMock.mock.calls[0];
		expect(url).toBe(GITHUB_TOKEN_URL);
		expect(init?.method).toBe("POST");
		expect(init?.redirect).toBe("error");
		expect(init?.signal).toBeDefined();
		expect(new URLSearchParams(String(init?.body))).toEqual(
			new URLSearchParams({
				client_id: "github-client",
				device_code: "device-code-value",
				grant_type: "urn:ietf:params:oauth:grant-type:device_code",
			}),
		);
	});

	it("returns an access token only after a successful response", async () => {
		stubFetch(() => jsonResponse({ access_token: "github-access-token", token_type: "bearer" }));
		expect(await githubDeviceToken(testEnv({ GITHUB_CLIENT_ID: "github-client" }), "device-code"))
			.toEqual({ ok: true, token: "github-access-token" });
	});

	it("maps network, malformed, and unknown provider responses to provider_unavailable", async () => {
		stubFetch(() => {
			throw new TypeError("network error");
		});
		expect(await githubDeviceToken(testEnv({ GITHUB_CLIENT_ID: "id" }), "code")).toEqual({ ok: false, error: "provider_unavailable" });
		stubFetch(() => new Response("not JSON"));
		expect(await githubDeviceToken(testEnv({ GITHUB_CLIENT_ID: "id" }), "code")).toEqual({ ok: false, error: "provider_unavailable" });
		stubFetch(() => jsonResponse({ error: "unexpected" }));
		expect(await githubDeviceToken(testEnv({ GITHUB_CLIENT_ID: "id" }), "code")).toEqual({ ok: false, error: "provider_unavailable" });
	});

	it("reads only GitHub's numeric ID and refuses a nonnumeric ID", async () => {
		const fetchMock = stubFetch(() => jsonResponse({ id: 12345, login: "private-login", email: "private@example.test", avatar_url: "https://example.test/avatar" }));
		expect(await githubSubject("access-token-value")).toEqual({ ok: true, subject: "12345" });
		const [url, init] = fetchMock.mock.calls[0];
		expect(url).toBe(GITHUB_USER_URL);
		const headers = new Headers(init?.headers);
		expect(headers.get("user-agent")).toBe("kanso-sync/0.1.0");
		expect(headers.get("accept")).toBe("application/vnd.github+json");
		expect(headers.get("authorization")).toBe("Bearer access-token-value");
		stubFetch(() => jsonResponse({ id: "not-numeric", login: "ignored" }));
		expect(await githubSubject("another-token")).toEqual({ ok: false, error: "provider_unavailable" });
	});
});

describe("health sign-in advertisement", () => {
	it("advertises password only when nothing else is configured", () => {
		expect(healthAuth(testEnv())).toEqual({ password: true, passkeys: null, providers: [] });
	});

	it("advertises passkeys with a valid RP ID and origin", () => {
		expect(
			healthAuth(testEnv({ PASSKEY_RP_ID: "sync.kansoapp.app", PASSKEY_ORIGINS: '["https://sync.kansoapp.app"]' })).passkeys,
		).toEqual({ rp_id: "sync.kansoapp.app", origins: ["https://sync.kansoapp.app"], security_key: true });
	});

	it.each([
		["an IP RP ID", "127.0.0.1", '["https://127.0.0.1"]'],
		["a plain HTTP non-localhost origin", "sync.kansoapp.app", '["http://sync.kansoapp.app"]'],
		["an origin outside the RP ID", "sync.kansoapp.app", '["https://example.com"]'],
		["malformed origin JSON", "sync.kansoapp.app", "{"],
	] as const)("does not advertise passkeys for %s", (_case, rpId, origins) => {
		expect(healthAuth(testEnv({ PASSKEY_RP_ID: rpId, PASSKEY_ORIGINS: origins })).passkeys).toBeNull();
	});

	it("advertises each provider only when all of its required values are present", () => {
		const appleWithoutPrivateKey = {
			APPLE_CLIENT_ID: "apple-client",
			APPLE_TEAM_ID: "team",
			APPLE_KEY_ID: "key",
			APPLE_REDIRECT_URI: "https://sync.kansoapp.app/v2/auth/apple/callback",
		};
		expect(healthAuth(testEnv(appleWithoutPrivateKey)).providers).toEqual([]);
		expect(healthAuth(testEnv({ ...appleWithoutPrivateKey, APPLE_PRIVATE_KEY: ecPrivatePem })).providers).toEqual(["apple"]);
		expect(healthAuth(testEnv({ GOOGLE_CLIENT_ID: "google-client" })).providers).toEqual([]);
		expect(healthAuth(testEnv({ GOOGLE_CLIENT_ID: "google-client", GOOGLE_CLIENT_SECRET: "secret" })).providers).toEqual(["google"]);
		expect(healthAuth(testEnv({ GITHUB_CLIENT_SECRET: "optional-only" })).providers).toEqual([]);
		expect(healthAuth(testEnv({ GITHUB_CLIENT_ID: "github-client" })).providers).toEqual(["github"]);
	});
});

describe("the 0006 schema", () => {
	it("defaults has_password to 1 for an account created with a username", async () => {
		await resetServer({ signup: "open" });
		const account = await signUp("alice");
		const row = await env.DB.prepare("SELECT has_password FROM accounts WHERE id = ?1")
			.bind(account.creds.accountId)
			.first<{ has_password: number }>();
		expect(row?.has_password).toBe(1);
	});

	it("deletes every sign-in row before the account, with foreign keys on", async () => {
		await resetServer({ signup: "open" });
		const account = await signUp("bob");
		const id = account.creds.accountId;
		await env.DB.batch([
			env.DB.prepare("INSERT INTO account_identities (account_id, provider, subject, created_at) VALUES (?1, 'github', '123', ?2)").bind(id, NOW),
			env.DB.prepare(
				"INSERT INTO account_passkeys (credential_id, account_id, public_key, alg, sign_count, created_at) VALUES (?1, ?2, ?3, -7, 0, ?4)",
			).bind("A".repeat(22), id, "B".repeat(40), NOW),
			env.DB.prepare(
				"INSERT INTO device_approvals (id, account_id, device_id, commit_hash, created_at, expires_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
			).bind("C".repeat(43), id, `d_${"0".repeat(32)}`, "D".repeat(43), NOW, NOW + 300_000),
			env.DB.prepare("INSERT INTO auth_flows (id, kind, account_id, created_at, expires_at) VALUES (?1, 'passkey_signin', ?2, ?3, ?4)").bind(
				"E".repeat(43),
				id,
				NOW,
				NOW + 300_000,
			),
		]);
		const deleted = await callJson("/v2/account/delete", {
			token: account.token,
			json: { username: "bob", auth_key: account.creds.authKey },
		});
		expect(deleted.status).toBe(200);
		const counts = await env.DB.prepare(
			`SELECT (SELECT count(*) FROM account_identities) AS identities,
  (SELECT count(*) FROM account_passkeys) AS passkeys,
  (SELECT count(*) FROM device_approvals) AS approvals,
  (SELECT count(*) FROM auth_flows) AS flows`,
		).first();
		expect(counts).toEqual({ identities: 0, passkeys: 0, approvals: 0, flows: 0 });
	});
});
