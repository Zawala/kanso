// Runs the shared protocol fixtures (test/fixtures/auth.json, conformance.json
// and sign_in.json) against the Worker, through handle(request, env, now).
// The step language is in each file's `about`; sign_in.json adds Worker vars,
// a deterministic Ed25519 authenticator, a fake GitHub and `repeat`.

import { env } from "cloudflare:workers";
import { expect, vi } from "vitest";
import type { Env } from "../src/env";
import { GITHUB_DEVICE_CODE_URL, GITHUB_TOKEN_URL, GITHUB_USER_URL } from "../src/oauth";
import { admin, setQuota } from "../src/accounts";
import { sha256Hex } from "../src/crypto";
import { handle } from "../src/index";
import { ORIGIN, resetServer, testEnv, testIO } from "./helpers";
import { base64url, fromBase64url } from "../src/protocol";
import { type Credential, assertion, newEd25519Credential, registration } from "./webauthn-helpers";

type Json = null | boolean | number | string | Json[] | { [key: string]: Json };
type Step = Record<string, any>;

export interface Fixture {
	start: number;
	values: Record<string, Json>;
	cases: { name: string; runner?: string; env?: Record<string, string | null>; steps: Step[] }[];
	/** sign_in.json: the Worker vars every case starts with (a case's `env` merges over them; null unsets). */
	env?: Record<string, string | null>;
	/** sign_in.json: Ed25519 authenticators, each a 32-byte seed and credential ID (base64url). */
	authenticators?: Record<string, { seed: string; credential_id: string }>;
	/** sign_in.json: what the fake GitHub answers. */
	github?: { id: number; device_code: string; user_code: string; interval: number };
}

const ROUTES: Record<string, { method: string; path: (p: Record<string, string>) => string; body?: "json" | "file" }> = {
	health: { method: "GET", path: () => "/v2/health" },
	prelogin: { method: "POST", path: () => "/v2/auth/prelogin", body: "json" },
	signup: { method: "POST", path: () => "/v2/auth/signup", body: "json" },
	login: { method: "POST", path: () => "/v2/auth/login", body: "json" },
	recover_start: { method: "POST", path: () => "/v2/auth/recover/start", body: "json" },
	recover_finish: { method: "POST", path: () => "/v2/auth/recover/finish", body: "json" },
	logout: { method: "POST", path: () => "/v2/auth/logout" },
	account: { method: "GET", path: () => "/v2/account" },
	password: { method: "POST", path: () => "/v2/account/password", body: "json" },
	recovery_key: { method: "POST", path: () => "/v2/account/recovery-key", body: "json" },
	delete: { method: "POST", path: () => "/v2/account/delete", body: "json" },
	set_device: { method: "PUT", path: () => "/v2/account/device", body: "json" },
	devices: { method: "GET", path: () => "/v2/account/devices" },
	revoke_device: { method: "DELETE", path: (p) => `/v2/account/devices/${p.device_id}` },
	push: { method: "POST", path: () => "/v2/push", body: "json" },
	changes: { method: "GET", path: () => "/v2/changes" },
	put_file: { method: "PUT", path: (p) => `/v2/files/${p.wire_id}`, body: "file" },
	get_file: { method: "GET", path: (p) => `/v2/files/${p.wire_id}` },
	approval_create: { method: "POST", path: () => "/v2/account/sign-in/approvals", body: "json" },
	approval_list: { method: "GET", path: () => "/v2/account/sign-in/approvals" },
	approval_poll: { method: "GET", path: (p) => `/v2/account/sign-in/approvals/${p.id}` },
	approval_challenge: { method: "POST", path: (p) => `/v2/account/sign-in/approvals/${p.id}/challenge`, body: "json" },
	approval_name: { method: "POST", path: (p) => `/v2/account/sign-in/approvals/${p.id}/name`, body: "json" },
	approval_approve: { method: "POST", path: (p) => `/v2/account/sign-in/approvals/${p.id}/approve`, body: "json" },
	approval_cancel: { method: "DELETE", path: (p) => `/v2/account/sign-in/approvals/${p.id}` },
	passkey_start: { method: "POST", path: () => "/v2/auth/passkey/start", body: "json" },
	passkey_finish: { method: "POST", path: () => "/v2/auth/passkey/finish", body: "json" },
	passkey_signup_start: { method: "POST", path: () => "/v2/auth/passkey/signup/start", body: "json" },
	passkey_signup_finish: { method: "POST", path: () => "/v2/auth/passkey/signup/finish", body: "json" },
	github_start: { method: "POST", path: () => "/v2/auth/github/start", body: "json" },
	github_poll: { method: "POST", path: () => "/v2/auth/github/poll", body: "json" },
	social_signup: { method: "POST", path: () => "/v2/auth/social/signup", body: "json" },
	sign_in_methods: { method: "GET", path: () => "/v2/account/sign-in" },
	proof_start: { method: "POST", path: () => "/v2/account/sign-in/proof/start", body: "json" },
	set_password: { method: "POST", path: () => "/v2/account/sign-in/password", body: "json" },
	passkey_register_start: { method: "POST", path: () => "/v2/account/sign-in/passkeys/start", body: "json" },
	passkey_register_finish: { method: "POST", path: () => "/v2/account/sign-in/passkeys/finish", body: "json" },
	passkey_remove: { method: "POST", path: (p) => `/v2/account/sign-in/passkeys/${p.credential_id}/remove`, body: "json" },
	identity_start: { method: "POST", path: (p) => `/v2/account/sign-in/identities/${p.provider}/start`, body: "json" },
	identity_unlink: { method: "POST", path: (p) => `/v2/account/sign-in/identities/${p.provider}/unlink`, body: "json" },
	aasa: { method: "GET", path: () => "/.well-known/apple-app-site-association" },
};

/** The fixture's passkey defaults: its RP ID and origin. */
const FIXTURE_RP_ID = "sync.kansoapp.app";
const FIXTURE_ORIGIN = "https://sync.kansoapp.app";

/**
 * Replaces every object with a "$register" or "$assert" key by the WebAuthn
 * fields it stands for, merged with its other keys (values already resolved).
 */
function expandWebAuthn(value: Json, fixture: Fixture, credentials: Map<string, Credential>): Json {
	if (Array.isArray(value)) return value.map((v) => expandWebAuthn(v, fixture, credentials));
	if (value === null || typeof value !== "object") return value;
	const out: Record<string, Json> = {};
	for (const [k, v] of Object.entries(value)) if (k !== "$register" && k !== "$assert") out[k] = expandWebAuthn(v, fixture, credentials);
	for (const kind of ["$register", "$assert"] as const) {
		const spec = value[kind] as Record<string, Json> | undefined;
		if (spec === undefined) continue;
		const name = String(spec.authenticator);
		let credential = credentials.get(name);
		if (!credential) {
			const a = fixture.authenticators?.[name];
			if (!a) throw new Error(`no authenticator ${name}`);
			credential = newEd25519Credential(fromBase64url(a.seed)!, fromBase64url(a.credential_id)!);
			credentials.set(name, credential);
		}
		const options = {
			origin: (spec.origin as string | undefined) ?? FIXTURE_ORIGIN,
			rpId: (spec.rp_id as string | undefined) ?? FIXTURE_RP_ID,
			flags: spec.flags as number | undefined,
			signCount: spec.sign_count as number | undefined,
		};
		Object.assign(out, kind === "$register" ? registration(credential, String(spec.challenge), options) : assertion(credential, String(spec.challenge), options));
	}
	return out;
}

/** Expands `{repeat: n, steps}` in place. */
function flatten(steps: Step[]): Step[] {
	return steps.flatMap((step) => ("repeat" in step ? Array.from({ length: step.repeat as number }, () => flatten(step.steps)).flat() : [step]));
}

/** The fake GitHub: device codes from the fixture, token answers from the case's queue. */
function installGithub(fixture: Fixture, queue: string[]): void {
	const github = fixture.github!;
	const reply = (value: unknown) => new Response(JSON.stringify(value), { status: 200, headers: { "content-type": "application/json" } });
	vi.stubGlobal("fetch", async (input: RequestInfo | URL) => {
		const url = String(input instanceof Request ? input.url : input);
		if (url === GITHUB_DEVICE_CODE_URL) {
			return reply({
				device_code: github.device_code,
				user_code: github.user_code,
				verification_uri: "https://github.com/login/device",
				expires_in: 900,
				interval: github.interval,
			});
		}
		if (url === GITHUB_TOKEN_URL) {
			const answer = queue.shift();
			if (answer === "authorized") return reply({ access_token: "gho_fixture", token_type: "bearer", scope: "" });
			if (answer === "denied") return reply({ error: "access_denied" });
			if (answer === "slow_down") return reply({ error: "slow_down", interval: github.interval + 5 });
			return reply({ error: "authorization_pending" });
		}
		if (url === GITHUB_USER_URL) return reply({ id: github.id });
		throw new Error(`unexpected fetch ${url}`);
	});
}

/** The case's Worker bindings: the fixture's vars, then the case's (null unsets). */
function caseEnv(fixture: Fixture, overrides: Record<string, string | null> = {}): Env {
	const vars: Record<string, string> = {};
	for (const [k, v] of Object.entries({ ...(fixture.env ?? {}), ...overrides })) if (v !== null) vars[k] = v;
	return testEnv(vars as Partial<Env>);
}

class Values {
	captured = new Map<string, Json>();
	constructor(private readonly fixture: Fixture) {}

	get(name: string): Json {
		if (this.captured.has(name)) return this.captured.get(name)!;
		if (name in this.fixture.values) return this.fixture.values[name];
		throw new Error(`no value $${name}`);
	}

	/** Replaces every string that is exactly "$name". */
	resolve(value: Json): Json {
		if (typeof value === "string" && /^\$[A-Za-z0-9_]+$/.test(value)) return this.get(value.slice(1));
		if (Array.isArray(value)) return value.map((v) => this.resolve(v));
		if (value !== null && typeof value === "object") {
			return Object.fromEntries(Object.entries(value).map(([k, v]) => [k, this.resolve(v)]));
		}
		return value;
	}
}

/** Pattern matching as the fixtures define it; throws with the path of the first difference. */
function match(pattern: Json, actual: unknown, values: Values, path: string): void {
	const where = `${path}: expected ${JSON.stringify(pattern)}, got ${JSON.stringify(actual)}`;
	if (pattern === null) {
		expect(actual === null || actual === undefined, where).toBe(true);
	} else if (pattern === "*") {
		expect(actual !== null && actual !== undefined, where).toBe(true);
	} else if (typeof pattern === "string" && /^!\$[A-Za-z0-9_]+$/.test(pattern)) {
		expect(actual, where).not.toEqual(values.get(pattern.slice(2)));
	} else if (typeof pattern === "string" && /^\$[A-Za-z0-9_]+$/.test(pattern)) {
		expect(actual, where).toEqual(values.get(pattern.slice(1)));
	} else if (Array.isArray(pattern)) {
		expect(Array.isArray(actual), where).toBe(true);
		expect((actual as unknown[]).length, where).toBe(pattern.length);
		pattern.forEach((p, i) => match(p, (actual as unknown[])[i], values, `${path}[${i}]`));
	} else if (typeof pattern === "object") {
		expect(actual !== null && typeof actual === "object", where).toBe(true);
		for (const [key, p] of Object.entries(pattern)) match(p, (actual as Record<string, unknown>)[key], values, `${path}.${key}`);
	} else {
		expect(actual, where).toEqual(pattern);
	}
}

function dig(value: unknown, dotted: string): Json {
	return dotted.split(".").reduce<any>((v, key) => (v === null || v === undefined ? v : v[key]), value);
}

/** Runs one case from an empty server with sign-ups open and the clock at the fixture's start. */
export async function runCase(fixture: Fixture, steps: Step[], overrides?: Record<string, string | null>): Promise<void> {
	await resetServer({ signup: "open" });
	const values = new Values(fixture);
	const workerEnv = caseEnv(fixture, overrides);
	const credentials = new Map<string, Credential>();
	const githubQueue: string[] = [];
	if (fixture.github) installGithub(fixture, githubQueue);
	try {
		await runSteps(fixture, flatten(steps), values, workerEnv, credentials, githubQueue);
	} finally {
		if (fixture.github) vi.unstubAllGlobals();
	}
}

async function runSteps(
	fixture: Fixture,
	steps: Step[],
	values: Values,
	workerEnv: Env,
	credentials: Map<string, Credential>,
	githubQueue: string[],
): Promise<void> {
	let t = 0;
	for (const [index, step] of steps.entries()) {
		const label = `step ${index} ${JSON.stringify(step).slice(0, 160)}`;
		if ("github" in step) {
			githubQueue.push(step.github as string);
			continue;
		}
		if ("settings" in step) {
			for (const [name, value] of Object.entries(step.settings as Record<string, string>)) {
				await env.DB.prepare("UPDATE settings SET value = ?2 WHERE name = ?1").bind(name, value).run();
			}
			continue;
		}
		if ("admin" in step) {
			const { io } = testIO();
			if (step.admin === "quota") {
				const q = step.quota as Record<string, number | null | undefined>;
				const report = await setQuota(
					io,
					step.username,
					{ docs: q.docs, docBytes: q.doc_bytes, files: q.files, fileBytes: q.file_bytes, writesPerDay: q.writes_per_day },
					false,
				);
				expect(report.ok, `${label}: ${report.message}`).toBe(true);
			} else {
				const report = await admin([step.admin, step.username], io, fixture.start + t);
				expect(report.ok, `${label}: ${report.message}`).toBe(true);
			}
			continue;
		}
		if (step.t !== undefined) {
			expect(step.t, `${label}: t never decreases`).toBeGreaterThanOrEqual(t);
			t = step.t;
		}
		const headers = new Headers();
		if (step.as !== undefined) headers.set("authorization", `Bearer ${values.get(step.as)}`);
		else if (step.token !== undefined) headers.set("authorization", `Bearer ${values.resolve(step.token)}`);
		let request: Request;
		let route = "raw";
		if ("raw" in step) {
			const raw = step.raw as { method: string; path: string; headers?: Record<string, string>; body?: string };
			for (const [k, v] of Object.entries(raw.headers ?? {})) headers.set(k, v);
			const path = raw.path.replace(/\$[A-Za-z0-9_]+/g, (name) => String(values.get(name.slice(1))));
			let body: BodyInit | null = null;
			if (raw.body !== undefined) {
				body = /^\$[A-Za-z0-9_]+$/.test(raw.body) ? fromBase64url(String(values.get(raw.body.slice(1))))! : raw.body;
			}
			request = new Request(`${ORIGIN}${path}`, { method: raw.method, headers, body });
		} else {
			route = step.request;
			const spec = ROUTES[route];
			if (!spec) throw new Error(`${label}: unknown route`);
			const params = values.resolve(step.path ?? {}) as Record<string, string>;
			let path = spec.path(params);
			if (route === "changes" && step.query) {
				const query = new URLSearchParams();
				for (const [k, v] of Object.entries(step.query as Record<string, number>)) query.set(k, String(v));
				path += `?${query}`;
			}
			let body: BodyInit | null = null;
			if (spec.body === "json" && step.body !== undefined) {
				headers.set("content-type", "application/json");
				body = JSON.stringify(expandWebAuthn(values.resolve(step.body), fixture, credentials));
			} else if (spec.body === "file") {
				const bytes = fromBase64url(String(values.resolve(step.file)))!;
				headers.set("content-type", "application/octet-stream");
				headers.set("x-kanso-sha256", await sha256Hex(bytes));
				headers.set("content-length", String(bytes.length));
				body = bytes;
			}
			request = new Request(`${ORIGIN}${path}`, { method: spec.method, headers, body });
		}

		const response = await handle(request, workerEnv, fixture.start + t);
		const text = route === "get_file" && response.status === 200 ? null : await response.text();
		expect(response.status, `${label}: status, body ${text}`).toBe(step.status);
		let body: unknown = null;
		if (text === null) body = { bytes: base64url(new Uint8Array(await response.arrayBuffer())) };
		else if (text !== "") body = JSON.parse(text);
		if (step.response !== undefined) match(step.response, body, values, `${label} response`);
		for (const [name, value] of Object.entries((step.response_headers ?? {}) as Record<string, string>)) {
			expect(response.headers.get(name), `${label} header ${name}`).toBe(value);
		}
		for (const [name, dotted] of Object.entries((step.capture ?? {}) as Record<string, string>)) {
			values.captured.set(name, dig(body, dotted));
		}
	}
}
