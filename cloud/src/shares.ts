// Published notes (docs/ACCOUNTS.md › Published notes): the owner's routes
// `PUT` and `DELETE /v2/shares/<id>`, the public `GET /v2/public/<id>` and the
// viewer page `GET /p/<id>`, and the purges that delete a share's R2 object.
//
// The Worker stores what the device uploads, `0x01 || nonce(12) || AES-256-GCM
// ciphertext || tag(16)`, at `shares/<id>` in R2, and one D1 row saying whose
// it is, how big, its SHA-256, and until when it is served. The key is in the
// link's #fragment, which browsers never send: the Worker never receives it.
//
// An ID that stops being published (unpublished, expired, taken down, or its
// account deleted) is retired for good (`retired_shares`, filled by a trigger
// on every delete of a share row): whoever holds the old link knows its key,
// so nobody may publish under that ID again.
//
// A share counts as one file towards its account's quotas (size + 1024 bytes),
// and its upload towards the account's daily writes and upload bytes. The D1
// row is the authority: a share is served only while it has a row, its
// account isn't disabled, and R2's object has the row's checksum.

import { type Session, withSession } from "./auth";
import { sha256Hex } from "./crypto";
import type { Env } from "./env";
import { badRequest, constraint, error, json, notFound, rateLimited, readBody, refusal, tooLarge, unauthorized, untilMidnight } from "./http";
import { BINDING_PERIOD_MS, accountKey, clientIp, count, ipGroup } from "./limits";
import { DAY, FILE_ROW_COST, HOUR, fromBase64url, isSha256Hex } from "./protocol";
import { PUBLIC_HEADERS, viewerPage } from "./viewer";

/** The format byte, the nonce and the tag. */
export const SHARE_OVERHEAD = 1 + 12 + 16;
/** Plaintext up to 10 MiB. */
export const MAX_SHARE_PLAINTEXT = 10 * 1024 * 1024;
export const MIN_SEALED_SHARE = SHARE_OVERHEAD;
export const MAX_SEALED_SHARE = MAX_SHARE_PLAINTEXT + SHARE_OVERHEAD;
export const SHARE_FORMAT_V1 = 0x01;
/** The furthest ahead an expiry may be. */
export const MAX_EXPIRY_MS = 366 * DAY;
/** Share purges one cron run or one account deletion handles. */
export const SHARE_PURGE_BATCH = 500;

export const shareKey = (id: string): string => `shares/${id}`;

/** 16 bytes of canonical unpadded base64url: 22 characters. */
export function isShareId(id: string): boolean {
	if (!/^[A-Za-z0-9_-]{22}$/.test(id)) return false;
	return fromBase64url(id)?.length === 16;
}

// --------------------------------------------------------------------- owner

/** Whether `id` is free for `accountId` to write: not another account's, never retired. */
const WRITABLE = `NOT EXISTS (SELECT 1 FROM shares WHERE id = ?2 AND account_id != ?1)
  AND NOT EXISTS (SELECT 1 FROM retired_shares WHERE id = ?2)`;

const PUT_SHARE = [
	"SELECT size, sha256 FROM shares WHERE id = ?2 AND account_id = ?1",
	`UPDATE accounts SET
  files = files + (NOT EXISTS (SELECT 1 FROM shares WHERE id = ?2 AND account_id = ?1)),
  file_bytes = file_bytes + ?3 + ${FILE_ROW_COST}
       - coalesce((SELECT size + ${FILE_ROW_COST} FROM shares WHERE id = ?2 AND account_id = ?1), 0)
  WHERE id = ?1 AND ${WRITABLE} RETURNING id`,
	`INSERT INTO shares (id, account_id, size, sha256, created_at, updated_at, expires_at)
  SELECT ?2, ?1, ?3, ?6, ?4, ?4, ?5 FROM accounts WHERE id = ?1 AND ${WRITABLE}
  ON CONFLICT (id) DO UPDATE SET size = excluded.size, sha256 = excluded.sha256, updated_at = excluded.updated_at,
    expires_at = excluded.expires_at
  WHERE shares.account_id = excluded.account_id`,
];

/** Undoes a share's D1 batch after R2 failed, if the row still holds this upload. */
function compensation(
	db: D1Database,
	accountId: string,
	id: string,
	size: number,
	sha: string,
	now: number,
	old: { size: number; sha256: string } | null,
): D1PreparedStatement[] {
	const holds = "EXISTS (SELECT 1 FROM shares WHERE id = ?2 AND account_id = ?1 AND size = ?3 AND updated_at = ?4 AND sha256 = ?5)";
	if (old === null) {
		// The row goes, which retires the ID: the device publishes again under a new one.
		return [
			db
				.prepare(`UPDATE accounts SET files = files - 1, file_bytes = file_bytes - (?3 + ${FILE_ROW_COST}) WHERE id = ?1 AND ${holds}`)
				.bind(accountId, id, size, now, sha),
			db
				.prepare("DELETE FROM shares WHERE id = ?2 AND account_id = ?1 AND size = ?3 AND updated_at = ?4 AND sha256 = ?5")
				.bind(accountId, id, size, now, sha),
		];
	}
	// The old object may be gone (R2 overwrites in place): the share keeps its old size and
	// checksum, and is served only while R2 still holds that object. Growing back is exempt
	// from the quotas, as for files.
	return [
		db
			.prepare(
				`UPDATE accounts SET file_bytes = file_bytes - ?3 + ?6, recounted_at = CASE WHEN ?6 > ?3 THEN ?4 ELSE recounted_at END
  WHERE id = ?1 AND ${holds}`,
			)
			.bind(accountId, id, size, now, sha, old.size),
		db
			.prepare("UPDATE shares SET size = ?6, sha256 = ?7 WHERE id = ?2 AND account_id = ?1 AND size = ?3 AND updated_at = ?4 AND sha256 = ?5")
			.bind(accountId, id, size, now, sha, old.size, old.sha256),
	];
}

/** The usage the triggers refused, as sync.ts reports it for files. */
async function quotaExceeded(env: Env, accountId: string, quota: string): Promise<Response> {
	const row = await env.DB.prepare(
		`SELECT a.files, a.file_bytes, coalesce(a.quota_files, d.files, 0) AS quota_files,
  coalesce(a.quota_file_bytes, d.file_bytes, 0) AS quota_file_bytes, coalesce(d.global_file_bytes, 0) AS global_file_bytes,
  (SELECT coalesce(sum(file_bytes), 0) FROM accounts) AS all_file_bytes
  FROM accounts a, defaults d WHERE a.id = ?1`,
	)
		.bind(accountId)
		.first<Record<string, number>>();
	if (!row) return unauthorized("unknown");
	const figures: Record<string, [number, number]> = {
		files: [row.files, row.quota_files],
		file_bytes: [row.file_bytes, row.quota_file_bytes],
		server: [row.all_file_bytes, row.global_file_bytes],
	};
	const name = quota in figures ? quota : "server";
	const [used, limit] = figures[name];
	return error(413, "quota_exceeded", "Over quota.", { quota: name, used, limit });
}

/** `x-kanso-expires-at`: absent for never, else Unix ms after now and at most 366 days ahead. */
function expiry(request: Request, now: number): { ok: true; value: number | null } | { ok: false } {
	const header = request.headers.get("x-kanso-expires-at");
	if (header === null) return { ok: true, value: null };
	if (!/^[0-9]{1,15}$/.test(header)) return { ok: false };
	const at = Number(header);
	return at > now && at <= now + MAX_EXPIRY_MS ? { ok: true, value: at } : { ok: false };
}

/**
 * `PUT /v2/shares/<id>`: stores the sealed note, new or replacing the
 * account's own share. Another account's ID is 409 `share_id_taken`, a
 * retired one 409 `share_retired`; either way the device publishes under a
 * new ID and key, and the old link stays dead.
 */
export function putShare(id: string) {
	return withSession(async (request, env, now, session) => {
		if (!isShareId(id)) return badRequest("Not a share ID.");
		const claimed = request.headers.get("x-kanso-sha256");
		if (claimed === null || !isSha256Hex(claimed)) return badRequest("x-kanso-sha256 must be 64 lowercase hex digits.");
		const expires = expiry(request, now);
		if (!expires.ok) return badRequest("x-kanso-expires-at must be a Unix time in ms, later than now and at most 366 days ahead.");
		const limited = await count(env.DB, "writes-account", accountKey("writes-account", session.accountId), now, 1);
		if (limited) return limited;
		const declared = request.headers.get("content-length");
		const counted = declared !== null && /^[0-9]{1,15}$/.test(declared) ? Math.min(Number(declared), MAX_SEALED_SHARE + 1) : MAX_SEALED_SHARE;
		const uploadKey = accountKey("uploads-account", session.accountId);
		const overBytes = await count(env.DB, "uploads-account", uploadKey, now, counted);
		if (overBytes) return overBytes;
		const bytes = await readBody(request, MAX_SEALED_SHARE);
		if (!bytes) return tooLarge(`A published note is at most ${MAX_SEALED_SHARE} bytes.`);
		if (bytes.length > counted) {
			const overRest = await count(env.DB, "uploads-account", uploadKey, now, bytes.length - counted);
			if (overRest) return overRest;
		}
		// Only sealed bytes are stored: a format byte the Worker knows, and room for a nonce and tag.
		if (bytes.length < MIN_SEALED_SHARE || bytes[0] !== SHARE_FORMAT_V1) {
			return badRequest("Not a sealed note (0x01, a 12-byte nonce, the ciphertext and a 16-byte tag).");
		}
		const actual = await sha256Hex(bytes);
		if (actual !== claimed) return error(400, "checksum_mismatch", "The body doesn't match x-kanso-sha256.");

		const size = bytes.length;
		let outcomes: D1Result[];
		try {
			outcomes = await env.DB.batch([
				env.DB.prepare(PUT_SHARE[0]).bind(session.accountId, id),
				env.DB.prepare(PUT_SHARE[1]).bind(session.accountId, id, size),
				env.DB.prepare(PUT_SHARE[2]).bind(session.accountId, id, size, now, expires.value, actual),
			]);
		} catch (err) {
			const r = refusal(err);
			if (r?.kind === "quota_exceeded") return quotaExceeded(env, session.accountId, r.detail);
			if (r?.kind === "rate_limited") return rateLimited(r.detail, untilMidnight(now));
			if (constraint(err, "CHECK")) return badRequest("A value isn't storable.");
			throw err;
		}
		if (outcomes[1].results.length !== 1) {
			const alive = await env.DB.prepare("SELECT 1 AS alive FROM accounts WHERE id = ?1").bind(session.accountId).first();
			if (!alive) return unauthorized("unknown");
			const retired = await env.DB.prepare("SELECT 1 AS retired FROM retired_shares WHERE id = ?1").bind(id).first();
			if (retired) return error(409, "share_retired", "That link was unpublished and can't be used again; publish under a new one.");
			return error(409, "share_id_taken", "That share ID can't be used; publish under a new one.");
		}
		const old = (outcomes[0].results as { size: number; sha256: string }[])[0] ?? null;
		try {
			await env.FILES.put(shareKey(id), bytes, {
				sha256: actual,
				httpMetadata: { contentType: "application/octet-stream" },
			});
		} catch {
			await env.DB.batch(compensation(env.DB, session.accountId, id, size, actual, now, old));
			return error(500, "internal", "Couldn't store the note.");
		}
		return json({ ok: true, id, size, expires_at: expires.value });
	});
}

/**
 * Unpublishes: the row and its quota go, the ID is retired (the
 * shares_retire trigger), and the object is queued for deletion. With `onlyExpired`, only while the share's expiry has passed (?3),
 * so an update that moved it in the meantime keeps the share.
 */
export function unpublishStatements(db: D1Database, accountId: string, id: string, now: number, onlyExpired = false): D1PreparedStatement[] {
	const extra = onlyExpired ? " AND expires_at IS NOT NULL AND expires_at <= ?3" : " AND ?3 IS NOT NULL";
	const owned = `EXISTS (SELECT 1 FROM shares WHERE id = ?2 AND account_id = ?1${extra})`;
	return [
		db
			.prepare(
				`UPDATE accounts SET files = files - 1,
  file_bytes = file_bytes - (SELECT size + ${FILE_ROW_COST} FROM shares WHERE id = ?2 AND account_id = ?1)
  WHERE id = ?1 AND ${owned}`,
			)
			.bind(accountId, id, now),
		db
			.prepare(`INSERT INTO share_purges (id, requested_at) SELECT ?2, ?3 WHERE ${owned} ON CONFLICT (id) DO UPDATE SET requested_at = excluded.requested_at`)
			.bind(accountId, id, now),
		db.prepare(`DELETE FROM shares WHERE id = ?2 AND account_id = ?1${extra} RETURNING id`).bind(accountId, id, now),
	];
}

/** `DELETE /v2/shares/<id>`: 204, or 404 when the account has no such share. */
export function deleteShare(id: string) {
	return withSession(async (_request, env, now, session: Session) => {
		if (!isShareId(id)) return badRequest("Not a share ID.");
		const results = await env.DB.batch(unpublishStatements(env.DB, session.accountId, id, now));
		if (results[2].results.length !== 1) return notFound();
		try {
			await env.FILES.delete(shareKey(id));
		} catch {
			// The purge row stays; the cron deletes the object.
		}
		return new Response(null, { status: 204 });
	});
}

// -------------------------------------------------------------------- public

/**
 * `GET /v2/public/<id>`: the sealed bytes of a live share, 410 `expired` once
 * its expiry has passed, 404 otherwise: unknown, unpublished, of a disabled
 * account, or while R2's object isn't the one the row describes (two racing
 * uploads). Behind the PUBLIC_LIMITER binding only, so a public read never
 * writes to D1, and a malformed ID never reaches D1. No session, no cookies,
 * no CORS: the viewer page fetches it from the same origin.
 */
export function getPublic(id: string) {
	return async (request: Request, env: Env, now: number): Promise<Response> => {
		const limited = await publicLimit(env, request);
		if (limited) return withPublicHeaders(limited);
		if (!isShareId(id)) return withPublicHeaders(notFound());
		const row = await env.DB.prepare(
			`SELECT s.expires_at, s.sha256 FROM shares s JOIN accounts a ON a.id = s.account_id
  WHERE s.id = ?1 AND a.disabled_at IS NULL`,
		)
			.bind(id)
			.first<{ expires_at: number | null; sha256: string }>();
		if (!row) return withPublicHeaders(notFound());
		if (row.expires_at !== null && row.expires_at <= now) return withPublicHeaders(error(410, "expired", "This link has expired."));
		const object = await env.FILES.get(shareKey(id));
		if (!object) return withPublicHeaders(notFound());
		if (checksumHex(object) !== row.sha256) {
			await object.body.cancel();
			return withPublicHeaders(notFound());
		}
		return new Response(object.body, {
			headers: {
				"content-type": "application/octet-stream",
				"content-length": String(object.size),
				"cache-control": "no-store",
				...PUBLIC_HEADERS,
			},
		});
	};
}

/** The SHA-256 R2 checked when the object was stored, as hex; null when it has none. */
export function checksumHex(object: R2Object): string | null {
	const sha = object.checksums.sha256;
	if (!sha) return null;
	return [...new Uint8Array(sha)].map((b) => b.toString(16).padStart(2, "0")).join("");
}

/** The PUBLIC_LIMITER binding, per IP group. It writes nothing to D1. */
async function publicLimit(env: Env, request: Request): Promise<Response | null> {
	if (!env.PUBLIC_LIMITER) return null;
	const { success } = await env.PUBLIC_LIMITER.limit({ key: ipGroup(clientIp(request)) });
	return success ? null : rateLimited("binding", BINDING_PERIOD_MS);
}

function withPublicHeaders(response: Response): Response {
	const headers = new Headers(response.headers);
	for (const [name, value] of Object.entries(PUBLIC_HEADERS)) headers.set(name, value);
	headers.set("cache-control", "no-store");
	return new Response(response.body, { status: response.status, headers });
}

/** `GET /p/<id>`: the viewer page, behind the PUBLIC_LIMITER binding only (it writes nothing). */
export function getViewer(id: string) {
	return async (request: Request, env: Env): Promise<Response> => {
		if (!isShareId(id)) return withPublicHeaders(notFound());
		const limited = await publicLimit(env, request);
		if (limited) return withPublicHeaders(limited);
		return viewerPage();
	};
}

// -------------------------------------------------------------------- purges

/**
 * The statements that unpublish every share of `accountId` (an account
 * deletion), guarded by `guard` (bound as ?1 account, ?2 guard value, ?3 now).
 * The quota counters go with the account, so nothing is refunded.
 */
export function accountShareStatements(db: D1Database, accountId: string, guardSql: string, guardValue: string, now: number): D1PreparedStatement[] {
	return [
		db
			.prepare(
				`INSERT INTO share_purges (id, requested_at) SELECT id, ?3 FROM shares WHERE account_id = ?1 AND ${guardSql}
  ON CONFLICT (id) DO UPDATE SET requested_at = excluded.requested_at`,
			)
			.bind(accountId, guardValue, now),
		db.prepare(`DELETE FROM shares WHERE account_id = ?1 AND ${guardSql}`).bind(accountId, guardValue),
	];
}

/** Expired shares become purges, and their quota goes back to the account. */
export async function expireShares(env: Env, now: number): Promise<number> {
	const { results } = await env.DB.prepare("SELECT id, account_id FROM shares WHERE expires_at IS NOT NULL AND expires_at <= ?1 LIMIT ?2")
		.bind(now, SHARE_PURGE_BATCH)
		.all<{ id: string; account_id: string }>();
	let expired = 0;
	for (const { id, account_id } of results) {
		const done = await env.DB.batch(unpublishStatements(env.DB, account_id, id, now, true));
		if (done[2].results.length === 1) expired += 1;
	}
	return expired;
}

/**
 * Deletes the R2 objects of pending share purges, oldest first, and drops the
 * rows more than an hour old (an upload begun before the purge can't then
 * leave an object behind). Returns how many objects were deleted.
 */
export async function runSharePurges(env: Env, now: number, limit = SHARE_PURGE_BATCH): Promise<number> {
	const { results } = await env.DB.prepare("SELECT id, requested_at FROM share_purges ORDER BY requested_at, id LIMIT ?1")
		.bind(limit)
		.all<{ id: string; requested_at: number }>();
	if (results.length === 0) return 0;
	await env.FILES.delete(results.map((r) => shareKey(r.id)));
	const old = results.filter((r) => r.requested_at <= now - HOUR).map((r) => r.id);
	if (old.length > 0) {
		await env.DB.prepare("DELETE FROM share_purges WHERE id IN (SELECT value FROM json_each(?1)) AND requested_at <= ?2")
			.bind(JSON.stringify(old), now - HOUR)
			.run();
	}
	return results.length;
}
