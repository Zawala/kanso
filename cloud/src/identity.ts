// Identity keys for sharing (docs/ACCOUNTS.md › Collaboration › Identity
// keys): `PUT` and `GET /v2/account/identity`, and `GET /v2/users/<username>`,
// which hands out another account's public keys.
//
// An identity is set once and never replaced: a different one is 409
// `identity_exists`. The server keeps the private keys only sealed under a key
// the account's devices derive from the data key, so it can't use them, and
// clients check every public key it hands out against the binding signature
// and their own pins, so it can't swap one in unnoticed after the first share.

import { withSession } from "./auth";
import type { Env } from "./env";
import { badRequest, error, json, notFound, readJson } from "./http";
import { accountKey, count } from "./limits";
import { fromBase64url, isObject } from "./protocol";
import { normalizeUsername } from "./usernames";

export interface Identity {
	sign_pub: string;
	box_pub: string;
	binding: string;
	sealed: string;
}

/** Exactly `bytes` bytes of canonical unpadded base64url. */
export function isB64(value: unknown, bytes: number): value is string {
	if (typeof value !== "string" || value.length !== Math.ceil((bytes * 4) / 3)) return false;
	return fromBase64url(value)?.length === bytes;
}

const IDENTITY_BODY = 4 * 1024;

/** An identity body: two 32-byte public keys, a 64-byte binding and the 105-byte sealed private keys. */
export function checkIdentity(value: unknown): Identity | null {
	if (!isObject(value)) return null;
	const { sign_pub, box_pub, binding, sealed } = value;
	if (!isB64(sign_pub, 32) || !isB64(box_pub, 32) || !isB64(binding, 64) || !isB64(sealed, 105)) return null;
	return { sign_pub, box_pub, binding, sealed };
}

/** `PUT /v2/account/identity`: 201 `{ok}` when set now, 200 when this identity is already set, 409 `identity_exists` for another. */
export const putIdentity = withSession(async (request, env, now, session) => {
	const body = await readJson(request, IDENTITY_BODY);
	if (!body.ok) return body.response;
	const identity = checkIdentity(body.value);
	if (!identity) return badRequest("sign_pub, box_pub, binding and sealed must be 43, 43, 86 and 140 base64url characters.");
	const inserted = await env.DB.prepare(
		`INSERT INTO identities (account_id, sign_pub, box_pub, binding, sealed, created_at)
  SELECT ?1, ?2, ?3, ?4, ?5, ?6 FROM accounts WHERE id = ?1
  ON CONFLICT (account_id) DO NOTHING RETURNING account_id`,
	)
		.bind(session.accountId, identity.sign_pub, identity.box_pub, identity.binding, identity.sealed, now)
		.first();
	if (inserted) return json({ ok: true }, 201);
	const existing = await identityOf(env, session.accountId);
	if (
		existing &&
		existing.sign_pub === identity.sign_pub &&
		existing.box_pub === identity.box_pub &&
		existing.binding === identity.binding &&
		existing.sealed === identity.sealed
	) {
		return json({ ok: true });
	}
	return error(409, "identity_exists", "This account already has another identity.");
});

/** `GET /v2/account/identity`: the account's identity, or 404 before it has one. */
export const getIdentity = withSession(async (_request, env, _now, session) => {
	const identity = await identityOf(env, session.accountId);
	return identity ? json(identity) : notFound();
});

export function identityOf(env: Env, accountId: string): Promise<Identity | null> {
	return env.DB.prepare("SELECT sign_pub, box_pub, binding, sealed FROM identities WHERE account_id = ?1").bind(accountId).first<Identity>();
}

/**
 * `GET /v2/users/<username>`: an account's public keys, counted by
 * `lookup-account`. 404 for an unknown or disabled account, or one without an
 * identity, alike; 400 `invalid_username` for a name that can't exist.
 */
export function getUser(raw: string) {
	return withSession(async (_request, env, now, session) => {
		const limited = await count(env.DB, "lookup-account", accountKey("lookup-account", session.accountId), now);
		if (limited) return limited;
		const username = normalizeUsername(raw);
		if (username === null) return error(400, "invalid_username", "Not a valid username.");
		const row = await env.DB.prepare(
			`SELECT a.id AS account_id, a.username, i.sign_pub, i.box_pub, i.binding
  FROM accounts a JOIN identities i ON i.account_id = a.id WHERE a.username = ?1 AND a.disabled_at IS NULL`,
		)
			.bind(username)
			.first<{ account_id: string; username: string; sign_pub: string; box_pub: string; binding: string }>();
		return row ? json(row) : notFound();
	});
}
