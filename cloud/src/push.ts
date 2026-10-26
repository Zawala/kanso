// Push notifications (docs/ACCOUNTS.md › Push notifications): a device
// registers an APNs or FCM token on its session row, and a change made by
// another device wakes it with a silent push. The push is only a nudge: it
// carries no note ID, title, count, username or account ID, and the device
// pulls over HTTPS as it does for any change.
//
// A change marks the account's other tokened devices due (one D1 UPDATE) and
// tells the account's AccountRoom, whose alarm sends. The alarm, not the
// route, does the sending, so a burst of changes costs one push per device
// per 30 s: `push_due_at` holds the first change since the last push, and
// the alarm fires when it and the device's last push are both 30 s old.
// Nothing here ever fails the route that changed something.

import type { Session } from "./auth";
import { readObject, withSession } from "./auth";
import type { Env } from "./env";
import { badRequest, notFound, unauthorized } from "./http";
import { base64url, isDeviceId } from "./protocol";

/** At most one push per device in this many ms. */
export const PUSH_INTERVAL_MS = 30_000;
/** Sends in flight at once: an account has at most ten devices. */
const SEND_PARALLEL = 4;
const REQUEST_TIMEOUT_MS = 10_000;
/** Apple wants a provider token between 20 and 60 minutes old. */
const APNS_JWT_MAX_AGE_MS = 40 * 60_000;
/** An OAuth access token is dropped this long before Google says it expires. */
const FCM_TOKEN_MARGIN_MS = 5 * 60_000;
/** How long a push may wait for an offline device, in seconds. */
const TTL_SECONDS = 3600;
const FCM_DEFAULT_TOKEN_URI = "https://oauth2.googleapis.com/token";
const FCM_SCOPE = "https://www.googleapis.com/auth/firebase.messaging";

/** The body of every APNs push: a silent wake-up and nothing else. */
export const APNS_BODY = '{"aps":{"content-available":1}}';

export type Platform = "apns" | "fcm";
export type Environment = "production" | "development";

const APNS_TOKEN = /^(?:[0-9a-f]{2}){16,100}$/;
const FCM_TOKEN = /^[A-Za-z0-9_:-]{100,4096}$/;

/** Where one push goes. */
export interface PushTarget {
	deviceId: string;
	platform: Platform;
	token: string;
	environment: Environment;
}

/** `sent`: accepted. `gone`: the provider says the token is dead. `failed`: try again with the next change. `unconfigured`: no secrets for the platform. */
export type Outcome = "sent" | "gone" | "failed" | "unconfigured";
export type Send = (target: PushTarget, now: number) => Promise<Outcome>;
export type Fetcher = (url: string, init: RequestInit) => Promise<Response>;

const encoder = new TextEncoder();

// ------------------------------------------------------------- registration

const CLEAR = "push_platform = NULL, push_token = NULL, push_environment = NULL, push_updated_at = NULL, push_due_at = NULL";

/**
 * `PUT /v2/account/devices/<id>/push` `{platform, token, environment}`: this
 * device's push token, for its own session only. 204. A token another session
 * holds moves here in the same batch, so it lives on one session at most.
 * 400 for a bad body or ID; 404 for any device but the caller's, as
 * `revokeDevice` answers for a device that isn't the caller's to touch.
 */
export function registerPush(deviceId: string) {
	return withSession(async (request, env, now, session) => {
		if (!isDeviceId(deviceId)) return badRequest("Not a device ID.");
		if (deviceId !== session.deviceId) return notFound();
		const body = await readObject(request);
		if (!body.ok) return body.response;
		const { platform, token, environment } = body.value;
		if (platform !== "apns" && platform !== "fcm") return badRequest("platform must be apns or fcm.");
		if (environment !== "production" && environment !== "development") return badRequest("environment must be production or development.");
		if (platform === "fcm" && environment !== "production") return badRequest("fcm has only a production environment.");
		if (typeof token !== "string" || !(platform === "apns" ? APNS_TOKEN : FCM_TOKEN).test(token)) {
			return badRequest(platform === "apns" ? "token must be 32–100 bytes of lowercase hex." : "token must be 100–4096 characters of A-Z a-z 0-9 _ : -.");
		}
		const live = "EXISTS (SELECT 1 FROM sessions WHERE account_id = ?2 AND device_id = ?3 AND token_hash = ?4 AND revoked_at IS NULL)";
		const results = await env.DB.batch([
			env.DB.prepare(`UPDATE sessions SET ${CLEAR} WHERE push_token = ?1 AND NOT (account_id = ?2 AND device_id = ?3) AND ${live}`).bind(
				token,
				session.accountId,
				session.deviceId,
				session.tokenHash,
			),
			env.DB.prepare(
				`UPDATE sessions SET push_platform = ?5, push_token = ?1, push_environment = ?6, push_updated_at = ?7
  WHERE account_id = ?2 AND device_id = ?3 AND token_hash = ?4 AND revoked_at IS NULL RETURNING device_id`,
			).bind(token, session.accountId, session.deviceId, session.tokenHash, platform, environment, now),
		]);
		if (results[1].results.length === 0) return unauthorized("revoked");
		return new Response(null, { status: 204 });
	});
}

/** `DELETE /v2/account/devices/<id>/push`: this device's pushes off. 204 whether or not a token was set. */
export function unregisterPush(deviceId: string) {
	return withSession(async (_request, env, _now, session) => {
		if (!isDeviceId(deviceId)) return badRequest("Not a device ID.");
		if (deviceId !== session.deviceId) return notFound();
		await env.DB.prepare(`UPDATE sessions SET ${CLEAR} WHERE token_hash = ?1`).bind(session.tokenHash).run();
		return new Response(null, { status: 204 });
	});
}

// -------------------------------------------------------------------- wakes

/**
 * Marks due: a device already due keeps its earlier time; otherwise the time is
 * now, or 30 s after its last push when that is later.
 */
const MARK = `push_due_at = coalesce(push_due_at, max(?3, coalesce(push_last_sent_at + ${PUSH_INTERVAL_MS}, ?3)))`;
const ACTIVE = "account_id IN (SELECT id FROM accounts WHERE disabled_at IS NULL)";

/** Tells each account's room when its first device is due. The route never waits on more than this. */
async function schedule(env: Env, due: Map<string, number>): Promise<void> {
	for (const [accountId, at] of due) {
		try {
			await env.ACCOUNT_ROOM.get(env.ACCOUNT_ROOM.idFromName(accountId)).schedule(accountId, at);
		} catch (err) {
			console.error(JSON.stringify({ route: "push", error: "schedule_failed", kind: (err as Error)?.name ?? typeof err }));
		}
	}
}

/** The earliest due time per account of rows an UPDATE ... RETURNING marked. */
function earliest(rows: { account_id: string; push_due_at: number }[]): Map<string, number> {
	const due = new Map<string, number>();
	for (const row of rows) due.set(row.account_id, Math.min(row.push_due_at, due.get(row.account_id) ?? Infinity));
	return due;
}

/**
 * After a successful `POST /v2/push`: wakes the account's other active devices
 * that have a token, never the sender's, never a disabled account's. A failure
 * is logged, never thrown.
 */
export async function wakeAccount(env: Env, accountId: string, senderDevice: string, now: number): Promise<void> {
	try {
		const { results } = await env.DB.prepare(
			`UPDATE sessions SET ${MARK} WHERE account_id = ?1 AND device_id != ?2 AND push_token IS NOT NULL AND revoked_at IS NULL AND ${ACTIVE}
  RETURNING account_id, push_due_at`,
		)
			.bind(accountId, senderDevice, now)
			.all<{ account_id: string; push_due_at: number }>();
		await schedule(env, earliest(results));
	} catch (err) {
		console.error(JSON.stringify({ route: "push", error: "wake_failed", kind: (err as Error)?.name ?? typeof err }));
	}
}

/**
 * After a shared note's update or comment: wakes the active devices of every
 * member account of the share, except the sending device. A failure is
 * logged, never thrown.
 */
export async function wakeShare(env: Env, shareId: string, sender: Pick<Session, "accountId" | "deviceId">, now: number): Promise<void> {
	try {
		const { results } = await env.DB.prepare(
			`UPDATE sessions SET ${MARK} WHERE push_token IS NOT NULL AND revoked_at IS NULL AND NOT (account_id = ?1 AND device_id = ?2) AND ${ACTIVE}
  AND account_id IN (SELECT account_id FROM collab_members WHERE share_id = ?4 AND status = 'member')
  RETURNING account_id, push_due_at`,
		)
			.bind(sender.accountId, sender.deviceId, now, shareId)
			.all<{ account_id: string; push_due_at: number }>();
		await schedule(env, earliest(results));
	} catch (err) {
		console.error(JSON.stringify({ route: "push", error: "wake_failed", kind: (err as Error)?.name ?? typeof err }));
	}
}

// ----------------------------------------------------------------- delivery

export interface Delivery {
	claimed: number;
	sent: number;
	gone: number;
	failed: number;
	unconfigured: number;
	/** When the alarm should next fire for this account, or null when no device is waiting. */
	next: number | null;
}

/** The claim: a row is sent at most once per 30 s, and only while its session and account are live. */
const CLAIM = `UPDATE sessions SET push_last_sent_at = ?2, push_due_at = NULL
  WHERE account_id = ?1 AND push_due_at <= ?2 AND push_token IS NOT NULL AND revoked_at IS NULL
    AND (push_last_sent_at IS NULL OR push_last_sent_at <= ?2 - ${PUSH_INTERVAL_MS}) AND ${ACTIVE}
  RETURNING device_id, push_platform, push_token, push_environment`;

/** The earliest moment a waiting device can be sent: due, and 30 s after its last push. */
const NEXT = `SELECT min(max(push_due_at, coalesce(push_last_sent_at + ${PUSH_INTERVAL_MS}, 0))) AS next FROM sessions
  WHERE account_id = ?1 AND push_due_at IS NOT NULL AND push_token IS NOT NULL AND revoked_at IS NULL AND ${ACTIVE}`;

/** Runs `fn` over `items`, at most `limit` at a time, keeping order. */
async function mapLimit<T, R>(items: T[], limit: number, fn: (item: T) => Promise<R>): Promise<R[]> {
	const out: R[] = new Array(items.length);
	let next = 0;
	const worker = async (): Promise<void> => {
		for (let i = next++; i < items.length; i = next++) out[i] = await fn(items[i]);
	};
	await Promise.all(Array.from({ length: Math.min(limit, items.length) }, worker));
	return out;
}

/**
 * The account's alarm: claims the devices due at `now` (one UPDATE, so no
 * device is sent twice), sends to each, clears a token the provider reports
 * gone (only if the row still holds the token that was sent), and says when to
 * wake next. A send that fails is logged and not retried until the next
 * change. D1 errors propagate: the alarm retries.
 */
export async function deliverDue(env: Env, accountId: string, now: number, send: Send): Promise<Delivery> {
	const { results } = await env.DB.prepare(CLAIM)
		.bind(accountId, now)
		.all<{ device_id: string; push_platform: Platform; push_token: string; push_environment: Environment }>();
	const targets: PushTarget[] = results.map((r) => ({
		deviceId: r.device_id,
		platform: r.push_platform,
		token: r.push_token,
		environment: r.push_environment,
	}));
	const outcomes = await mapLimit(targets, SEND_PARALLEL, (target) => send(target, now));
	const delivery: Delivery = { claimed: targets.length, sent: 0, gone: 0, failed: 0, unconfigured: 0, next: null };
	const gone: D1PreparedStatement[] = [];
	const unconfigured = new Set<Platform>();
	outcomes.forEach((outcome, i) => {
		delivery[outcome] += 1;
		if (outcome === "unconfigured") unconfigured.add(targets[i].platform);
		if (outcome === "gone") {
			gone.push(
				env.DB.prepare(`UPDATE sessions SET ${CLEAR} WHERE account_id = ?1 AND device_id = ?2 AND push_token = ?3`).bind(
					accountId,
					targets[i].deviceId,
					targets[i].token,
				),
			);
		}
	});
	for (const platform of unconfigured) console.error(JSON.stringify({ route: "push", error: "unconfigured", platform }));
	if (gone.length > 0) await env.DB.batch(gone);
	delivery.next = (await env.DB.prepare(NEXT).bind(accountId).first<{ next: number | null }>())?.next ?? null;
	return delivery;
}

// ------------------------------------------------------------------ senders

export interface PushCache {
	apns?: { config: string; at: number; jwt: Promise<string> };
	fcm?: { config: string; expiresAt?: number; token: Promise<string> };
}

const shared: PushCache = {};

/** An empty cache of provider tokens (tests). */
export const newPushCache = (): PushCache => ({});

const defaultFetch: Fetcher = (url, init) => fetch(url, init);

/** The senders for `env`'s secrets. Tests pass their own `fetcher` and `cache`. */
export function pushSender(env: Env, fetcher: Fetcher = defaultFetch, cache: PushCache = shared): Send {
	return async (target, now) => {
		try {
			return target.platform === "apns" ? await sendApns(env, fetcher, cache, target, now) : await sendFcm(env, fetcher, cache, target, now);
		} catch (err) {
			console.error(JSON.stringify({ route: "push", error: "send_failed", platform: target.platform, kind: (err as Error)?.name ?? typeof err }));
			return "failed";
		}
	};
}

/** A PKCS#8 PEM's DER bytes, or null. */
function pemBytes(pem: string): Uint8Array | null {
	const match = /^-----BEGIN PRIVATE KEY-----([\s\S]+)-----END PRIVATE KEY-----$/.exec(pem.replace(/\\n/g, "\n").trim());
	if (!match) return null;
	const encoded = match[1].replace(/\s/g, "");
	if (!/^[A-Za-z0-9+/]+={0,2}$/.test(encoded)) return null;
	try {
		return Uint8Array.from(atob(encoded), (c) => c.charCodeAt(0));
	} catch {
		return null;
	}
}

/** A compact JWS over `header` and `claims`, signed with the PKCS#8 PEM `pem`. */
async function jws(header: object, claims: object, pem: string, alg: "ES256" | "RS256"): Promise<string> {
	const der = pemBytes(pem);
	if (!der) throw new Error("bad key");
	const key =
		alg === "ES256"
			? await crypto.subtle.importKey("pkcs8", der, { name: "ECDSA", namedCurve: "P-256" }, false, ["sign"])
			: await crypto.subtle.importKey("pkcs8", der, { name: "RSASSA-PKCS1-v1_5", hash: "SHA-256" }, false, ["sign"]);
	const sign = alg === "ES256" ? { name: "ECDSA", hash: "SHA-256" } : { name: "RSASSA-PKCS1-v1_5" };
	const signing = `${base64url(encoder.encode(JSON.stringify(header)))}.${base64url(encoder.encode(JSON.stringify(claims)))}`;
	// ECDSA's WebCrypto signature is already the raw r || s a JWS wants.
	return `${signing}.${base64url(new Uint8Array(await crypto.subtle.sign(sign, key, encoder.encode(signing))))}`;
}

/** The text of a response, whatever it is, so the connection is released. */
const bodyText = (response: Response): Promise<string> => response.text().catch(() => "");

// --------------------------------------------------------------------- APNs

function apnsJwt(env: Env, cache: PushCache, now: number): Promise<string> {
	const config = JSON.stringify([env.APNS_KEY_P8, env.APNS_KEY_ID, env.APNS_TEAM_ID]);
	const held = cache.apns;
	if (held && held.config === config && now >= held.at && now - held.at < APNS_JWT_MAX_AGE_MS) return held.jwt;
	const jwt = jws(
		{ alg: "ES256", kid: env.APNS_KEY_ID },
		{ iss: env.APNS_TEAM_ID, iat: Math.floor(now / 1000) },
		env.APNS_KEY_P8 ?? "",
		"ES256",
	);
	const entry = { config, at: now, jwt };
	cache.apns = entry;
	jwt.catch(() => {
		if (cache.apns === entry) cache.apns = undefined;
	});
	return jwt;
}

async function sendApns(env: Env, fetcher: Fetcher, cache: PushCache, target: PushTarget, now: number): Promise<Outcome> {
	if (!env.APNS_KEY_P8 || !env.APNS_KEY_ID || !env.APNS_TEAM_ID || !env.APNS_TOPIC) return "unconfigured";
	const host = target.environment === "development" ? "api.sandbox.push.apple.com" : "api.push.apple.com";
	const response = await fetcher(`https://${host}/3/device/${target.token}`, {
		method: "POST",
		headers: {
			authorization: `bearer ${await apnsJwt(env, cache, now)}`,
			"apns-push-type": "background",
			"apns-priority": "5",
			"apns-topic": env.APNS_TOPIC,
			"apns-expiration": String(Math.floor(now / 1000) + TTL_SECONDS),
			"apns-collapse-id": "sync",
			"content-type": "application/json",
		},
		body: APNS_BODY,
		signal: AbortSignal.timeout(REQUEST_TIMEOUT_MS),
	});
	const text = await bodyText(response);
	if (response.status === 200) return "sent";
	let reason = "";
	try {
		reason = String(JSON.parse(text).reason ?? "");
	} catch {
		// No reason given.
	}
	if (response.status === 410 || (response.status === 400 && reason === "BadDeviceToken")) return "gone";
	if (response.status === 403 && reason === "ExpiredProviderToken") cache.apns = undefined;
	console.error(JSON.stringify({ route: "push", error: "refused", platform: "apns", status: response.status }));
	return "failed";
}

// ---------------------------------------------------------------------- FCM

interface ServiceAccount {
	project_id: string;
	client_email: string;
	private_key: string;
	token_uri: string;
}

function serviceAccount(raw: string): ServiceAccount | null {
	try {
		const value = JSON.parse(raw);
		const { project_id, client_email, private_key, token_uri } = value;
		if (typeof project_id !== "string" || typeof client_email !== "string" || typeof private_key !== "string") return null;
		return { project_id, client_email, private_key, token_uri: typeof token_uri === "string" ? token_uri : FCM_DEFAULT_TOKEN_URI };
	} catch {
		return null;
	}
}

/** An OAuth access token for FCM, cached until five minutes before it expires. */
function fcmToken(account: ServiceAccount, raw: string, fetcher: Fetcher, cache: PushCache, now: number): Promise<string> {
	const held = cache.fcm;
	if (held && held.config === raw && (held.expiresAt === undefined || now < held.expiresAt)) return held.token;
	const iat = Math.floor(now / 1000);
	const entry: NonNullable<PushCache["fcm"]> = { config: raw, token: Promise.resolve("") };
	entry.token = (async () => {
		const assertion = await jws(
			{ alg: "RS256", typ: "JWT" },
			{ iss: account.client_email, scope: FCM_SCOPE, aud: account.token_uri, iat, exp: iat + 3600 },
			account.private_key,
			"RS256",
		);
		const response = await fetcher(account.token_uri, {
			method: "POST",
			headers: { "content-type": "application/x-www-form-urlencoded" },
			body: new URLSearchParams({ grant_type: "urn:ietf:params:oauth:grant-type:jwt-bearer", assertion }).toString(),
			signal: AbortSignal.timeout(REQUEST_TIMEOUT_MS),
		});
		if (response.status !== 200) {
			await bodyText(response);
			throw new Error("token refused");
		}
		const answer = JSON.parse(await response.text()) as { access_token?: unknown; expires_in?: unknown };
		if (typeof answer.access_token !== "string" || answer.access_token === "") throw new Error("no token");
		const lifetime = typeof answer.expires_in === "number" && answer.expires_in > 0 ? answer.expires_in : 3600;
		entry.expiresAt = now + lifetime * 1000 - FCM_TOKEN_MARGIN_MS;
		return answer.access_token;
	})();
	cache.fcm = entry;
	entry.token.catch(() => {
		if (cache.fcm === entry) cache.fcm = undefined;
	});
	return entry.token;
}

async function sendFcm(env: Env, fetcher: Fetcher, cache: PushCache, target: PushTarget, now: number): Promise<Outcome> {
	const account = env.FCM_SERVICE_ACCOUNT ? serviceAccount(env.FCM_SERVICE_ACCOUNT) : null;
	if (!env.FCM_SERVICE_ACCOUNT || !account) return "unconfigured";
	const accessToken = await fcmToken(account, env.FCM_SERVICE_ACCOUNT, fetcher, cache, now);
	const response = await fetcher(`https://fcm.googleapis.com/v1/projects/${account.project_id}/messages:send`, {
		method: "POST",
		headers: { authorization: `Bearer ${accessToken}`, "content-type": "application/json" },
		body: JSON.stringify({ message: { token: target.token, android: { priority: "HIGH", ttl: `${TTL_SECONDS}s`, collapse_key: "sync" } } }),
		signal: AbortSignal.timeout(REQUEST_TIMEOUT_MS),
	});
	const text = await bodyText(response);
	if (response.status === 200) return "sent";
	if (response.status === 401) cache.fcm = undefined;
	try {
		const details = JSON.parse(text)?.error?.details;
		if (Array.isArray(details) && details.some((d) => d?.errorCode === "UNREGISTERED")) return "gone";
	} catch {
		// Not JSON.
	}
	console.error(JSON.stringify({ route: "push", error: "refused", platform: "fcm", status: response.status }));
	return "failed";
}
