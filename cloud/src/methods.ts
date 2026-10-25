// Changes to how an account is entered that take a fresh `proof` (proofs.ts)
// and touch the account's password or recovery material (docs/ACCOUNTS.md ›
// Sign-in routes, › Changed existing routes): the first password of an
// account made without one, a new recovery key, and deleting the account.
// Each is a compare-and-swap whose statements all carry the proof's guard,
// so the proof is used in the same transaction as the change.

import { signOutSockets } from "./account_room";
import {
	type Session,
	afterDeletion,
	authHashGuard,
	deletionStatements,
	newRecoveryKey,
	readObject,
	respond,
	withSession,
} from "./auth";
import { memberships } from "./collab";
import { proofMatches, sha256Hex } from "./crypto";
import type { Env } from "./env";
import { badRequest, error, invalidCredentials, json } from "./http";
import { accountKey, count } from "./limits";
import { checkKdf, isSalt, isWrappedKey, proof32 } from "./protocol";
import { type ProofAccount, type ProofGuard, proofAccount, proofUsed, proofWithRecovery } from "./proofs";
import { flowExpired } from "./sign_in";
import { normalizeUsername } from "./usernames";

const alreadyHasPassword = (): Response =>
	error(409, "already_has_password", "The account has a password; change it with POST /v2/account/password.");

/**
 * `POST /v2/account/sign-in/password` — `{proof, current_recovery_auth_key?,
 * kdf, salt, auth_key, wrapped_key, recovery_auth_key, recovery_wrapped_key}`.
 * Sets the first password of an account made with a passkey or a provider:
 * the KDF parameters, salt, auth hash and password wrap, `has_password = 1`,
 * and a new recovery key, in one compare-and-swap on `has_password`,
 * `auth_hash` and `recovery_hash`. Every other session is revoked
 * (`password_changed`) and its event socket closed. A provider proof also
 * needs `current_recovery_auth_key`, so a provider takeover can't set the
 * first password. 200 `{signed_out_devices}`; 409 `already_has_password`;
 * 401 `invalid_credentials` when the proof or the recovery key fails. A
 * retry after a lost answer, with the same values, is 200 with 0. Counts
 * `password-account`.
 */
export const setPassword = withSession(async (request, env, now, session) => {
	const limited = await count(env.DB, "password-account", accountKey("password-account", session.accountId), now);
	if (limited) return limited;
	const body = await readObject(request);
	if (!body.ok) return body.response;
	const b = body.value;
	const kdf = checkKdf(b.kdf);
	if (!kdf.ok) return respond(kdf);
	if (typeof b.salt !== "string" || !isSalt(b.salt)) return badRequest("salt must be 22 base64url characters.");
	const authKey = proof32(b.auth_key);
	if (!authKey) return badRequest("auth_key must be 32 bytes of canonical base64url.");
	if (typeof b.wrapped_key !== "string" || !isWrappedKey(b.wrapped_key)) return badRequest("wrapped_key must be 98 base64url characters.");
	const recovery = await newRecoveryKey(b);
	if (!recovery.ok) return recovery.response;
	const authHash = await sha256Hex(authKey);
	const account = await proofAccount(env, session.accountId);
	if (!account) return invalidCredentials();

	// Already set, by this very request (a retry after a lost answer) or before it.
	const done = (row: typeof account) =>
		row.has_password === 1 &&
		row.auth_hash === authHash &&
		row.wrapped_key === b.wrapped_key &&
		row.recovery_hash === recovery.hash &&
		row.recovery_wrapped_key === recovery.wrap;
	if (account.has_password === 1) return done(account) ? json({ signed_out_devices: 0 }) : alreadyHasPassword();

	const checked = await proofWithRecovery(env, now, session, account, b, { passwordCounted: true });
	if (!checked.ok) return checked.response;
	const { guard } = checked;
	// Every statement holds only while the account is still passwordless with the
	// auth and recovery hashes just read, and the proof is this batch's.
	const unchanged = "EXISTS (SELECT 1 FROM accounts WHERE id = ?1 AND has_password = 0 AND auth_hash = ?2 AND recovery_hash = ?3)";
	const results = await env.DB.batch([
		...guard.before,
		env.DB.prepare(
			`UPDATE sessions SET revoked_at = ?5, revoked_reason = 'password_changed'
  WHERE account_id = ?1 AND revoked_at IS NULL AND token_hash != ?6 AND ${unchanged} AND ${guard.sql(1, 4)}
  RETURNING device_id`,
		).bind(account.id, account.auth_hash, account.recovery_hash, guard.value, now, session.tokenHash),
		env.DB.prepare(
			`UPDATE accounts SET kdf_m_kib = ?5, kdf_t = ?6, kdf_p = ?7, salt = ?8, auth_hash = ?9, wrapped_key = ?10,
    recovery_hash = ?11, recovery_wrapped_key = ?12, has_password = 1
  WHERE id = ?1 AND has_password = 0 AND auth_hash = ?2 AND recovery_hash = ?3 AND ${guard.sql(1, 4)} RETURNING id`,
		).bind(
			account.id,
			account.auth_hash,
			account.recovery_hash,
			guard.value,
			kdf.value.m_kib,
			kdf.value.t,
			kdf.value.p,
			b.salt,
			authHash,
			b.wrapped_key,
			recovery.hash,
			recovery.wrap,
		),
	]);
	const n = guard.before.length;
	if (results[n + 1].results.length === 1) {
		const signedOut = (results[n].results as { device_id: string }[]).map((r) => r.device_id);
		if (signedOut.length > 0) await signOutSockets(env, account.id, signedOut);
		return json({ signed_out_devices: signedOut.length });
	}
	if (!proofUsed(guard, results)) return flowExpired();
	const latest = await proofAccount(env, session.accountId);
	if (latest?.has_password === 1) return done(latest) ? json({ signed_out_devices: 0 }) : alreadyHasPassword();
	return invalidCredentials();
});

/**
 * The body's proof, or for a password account its auth key in `field` (the
 * routes' older form): the account and the guard to run the change under, or
 * the response. Counts `password-account` for a password (either form) and
 * `proof-account` for a passkey or provider proof. A provider proof also needs
 * `current_recovery_auth_key` (401 `reauth_required` without it).
 */
async function proofOrPassword(
	env: Env,
	now: number,
	session: Session,
	body: Record<string, unknown>,
	field: string,
): Promise<{ ok: true; account: ProofAccount; guard: ProofGuard } | { ok: false; response: Response }> {
	if (body.proof === undefined) {
		const limited = await count(env.DB, "password-account", accountKey("password-account", session.accountId), now);
		if (limited) return { ok: false, response: limited };
		const current = proof32(body[field]);
		if (!current) return { ok: false, response: badRequest(`${field} must be 32 bytes of canonical base64url.`) };
		const account = await proofAccount(env, session.accountId);
		if (!account) return { ok: false, response: invalidCredentials() };
		if (!(await proofMatches(current, account.auth_hash))) return { ok: false, response: invalidCredentials() };
		return { ok: true, account, guard: { before: [], accountId: account.id, ...authHashGuard(account.auth_hash) } };
	}
	const account = await proofAccount(env, session.accountId);
	if (!account) return { ok: false, response: invalidCredentials() };
	const checked = await proofWithRecovery(env, now, session, account, body, { countProof: true });
	if (!checked.ok) return checked;
	return { ok: true, account, guard: checked.guard };
}

/**
 * `POST /v2/account/recovery-key` — `{current_auth_key, recovery_auth_key,
 * recovery_wrapped_key}`, or `{proof, current_recovery_auth_key?, …}` in place
 * of `current_auth_key`, which is how a passwordless account replaces its
 * recovery key. 200 `{}`. Sessions stay. Setting the same values again with
 * the current password (a retry) changes the row just the same; a proof is
 * used once, so a retry with a used one is 400 `flow_expired`.
 */
export const recoveryKey = withSession(async (request, env, now, session) => {
	const body = await readObject(request);
	if (!body.ok) return body.response;
	const b = body.value;
	const checked = await proofOrPassword(env, now, session, b, "current_auth_key");
	if (!checked.ok) return checked.response;
	const recovery = await newRecoveryKey(b);
	if (!recovery.ok) return recovery.response;
	const { account, guard } = checked;
	const results = await env.DB.batch([
		...guard.before,
		env.DB.prepare(`UPDATE accounts SET recovery_hash = ?3, recovery_wrapped_key = ?4 WHERE id = ?1 AND ${guard.sql(1, 2)} RETURNING id`).bind(
			account.id,
			guard.value,
			recovery.hash,
			recovery.wrap,
		),
	]);
	if (results[guard.before.length].results.length === 1) return json({});
	if (!proofUsed(guard, results)) return flowExpired();
	return invalidCredentials();
});

/**
 * `POST /v2/account/delete` — `{auth_key, username}` for a password account,
 * or `{proof, current_recovery_auth_key?, username}`. 200 `{deleted: true}`.
 * 400 `username_mismatch` unless the username normalizes to the account's
 * (checked after the proof, which isn't used then). One batch deletes
 * everything of the account under the proof's guard (auth.ts
 * deletionStatements); then the R2 purge, shared rooms and every event socket.
 */
export const deleteAccount = withSession(async (request, env, now, session) => {
	const body = await readObject(request);
	if (!body.ok) return body.response;
	const b = body.value;
	const checked = await proofOrPassword(env, now, session, b, "auth_key");
	if (!checked.ok) return checked.response;
	const { account, guard } = checked;
	if (typeof b.username !== "string") return badRequest("username must be a string.");
	if (normalizeUsername(b.username) !== account.username) {
		return error(400, "username_mismatch", "The username doesn't match this account.");
	}
	const shared = await memberships(env, account.id);
	const statements = deletionStatements(env.DB, account.id, now, guard);
	const results = await env.DB.batch([...guard.before, ...statements]);
	// The account's deletion is the second to last statement.
	if (results[results.length - 2].results.length !== 1) return proofUsed(guard, results) ? invalidCredentials() : flowExpired();
	await afterDeletion(env, account.id, shared, now);
	return json({ deleted: true });
});
