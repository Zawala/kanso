// Fresh proofs for sensitive changes (docs/ACCOUNTS.md › Account identity ›
// "Changing a way in needs a fresh proof"). A session can sync; it can't
// change how the account is entered. Linking or unlinking a provider, adding
// or removing a passkey, setting the first password, replacing the recovery
// key and deleting the account each carry a `proof`:
//
//   {"kind": "password", "auth_key"}
//   {"kind": "passkey", "flow", "assertion": {credential_id, authenticator_data, client_data_json, signature}}
//   {"kind": "provider", "flow"}
//
// A passkey proof's flow comes from `POST /v2/account/sign-in/proof/start`
// (kind `proof`, five minutes, bound to the session's device proof); a
// provider proof's from `identities/<provider>/start` with `intent: "proof"`.
//
// A checked proof is not yet used. It comes back as a guard: the statements
// that consume its flow, which go first in the change's D1 batch, and a
// condition every statement of the change carries. A flow is consumed by
// setting a fresh, globally unique attempt ID, and the change holds only
// where that attempt ID is stored, so of two requests racing one proof only
// one changes anything, and a proof is used in the same transaction as the
// change it authorizes. A password proof has no flow: its guard is the auth
// hash the Worker just checked, as every compare-and-swap in auth.ts.
//
// Which proofs a change takes (the caller says, through `ProofRules`):
//   password or passkey   always, where the account can make one
//   provider              only to unlink that same provider, or, with the
//                         recovery key, to set the first password, replace
//                         the recovery key or delete the account
// A provider takeover therefore can't add a way in, strip passkeys or other
// providers, or take the break-glass path without a secret it doesn't have.

import { type Session, readObject, withSessionKeys } from "./auth";
import { proofMatches, randomBytes, sha256Hex } from "./crypto";
import type { Env } from "./env";
import { badRequest, error, invalidCredentials, json, notFound } from "./http";
import { accountKey, count, countSession } from "./limits";
import { base64url, fromBase64url, isObject, proof32 } from "./protocol";
import {
	type AuthFlowRow,
	type StatementGuard,
	consumeFlow,
	flowById,
	flowExpired,
	flowStatement,
	hexEqual,
	isFlowId,
	passkeyAdvertisement,
} from "./sign_in";
import { type CoseAlgorithm, bytesField, signCountAccepted, storedTransports, verifyAssertion } from "./webauthn";

/** A passkey or proof challenge lives five minutes. */
export const PROOF_FLOW_MS = 5 * 60_000;

export const reauthRequired = (): Response => error(401, "reauth_required", "This change needs a fresh proof.", { reason: "reauth" });
export const unsupportedMethod = (): Response => error(400, "unsupported_method", "This server doesn't offer passkeys.");

/** The account columns a proof is checked against. */
export interface ProofAccount {
	id: string;
	username: string;
	auth_hash: string;
	recovery_hash: string;
	wrapped_key: string;
	recovery_wrapped_key: string;
	has_password: number;
}

export function proofAccount(env: Env, accountId: string): Promise<ProofAccount | null> {
	return env.DB.prepare(
		"SELECT id, username, auth_hash, recovery_hash, wrapped_key, recovery_wrapped_key, has_password FROM accounts WHERE id = ?1",
	)
		.bind(accountId)
		.first<ProofAccount>();
}

/**
 * A used proof, as the change's batch needs it: `before` consumes its flow
 * (empty for a password) and goes first; `sql(account, value)` is the
 * condition every statement of the change carries.
 */
export interface ProofGuard extends StatementGuard {
	before: D1PreparedStatement[];
}

/** Whether the proof's flow was consumed by this batch, from its results; always true for a password. */
export const proofUsed = (guard: ProofGuard, results: D1Result[]): boolean =>
	guard.before.length === 0 || results[0].results.length === 1;

export type ProofKind = "password" | "passkey" | "provider";

export type CheckedProof = { ok: true; kind: ProofKind; guard: ProofGuard } | { ok: false; response: Response };

export interface ProofRules {
	/**
	 * Which provider proofs this change takes: none (null), the provider being
	 * unlinked, or any linked provider ("any") for the changes that also need
	 * the recovery key.
	 */
	provider: string | "any" | null;
	/** Count `proof-account` when a passkey or provider proof is used (a new recovery key, a deletion). */
	countProof?: boolean;
	/** The route counted `password-account` already, so a password proof doesn't count twice. */
	passwordCounted?: boolean;
	/**
	 * SHA-256 hex of a recovery key the caller checked: the proof is used only
	 * while it is still the account's, in the same statement. Required with a
	 * provider proof for the changes that need it; optional otherwise.
	 */
	recoveryHash?: string;
}

const failed = (response: Response): CheckedProof => ({ ok: false, response });

/**
 * Checks `value`, a request's `proof`, for a change to `account` by this
 * session. Nothing is consumed here except a passkey proof whose assertion
 * failed: a challenge is good for one try. The answer is the guard to run the
 * change under, or the response that ends the request:
 *
 * - no proof, an unknown kind, or a kind this change doesn't take: 401 `reauth_required`;
 * - a flow that is unknown, used, past its time, or another session's: 400 `flow_expired`;
 * - a wrong password or a failed assertion: 401 `invalid_credentials`.
 */
export async function checkProof(env: Env, now: number, session: Session, account: ProofAccount, value: unknown, rules: ProofRules): Promise<CheckedProof> {
	if (!isObject(value)) return failed(reauthRequired());
	switch (value.kind) {
		case "password":
			return passwordProof(env, now, session, account, value, rules);
		case "passkey":
			return passkeyProof(env, now, session, value, rules);
		case "provider":
			return providerProof(env, now, session, value, rules);
		default:
			return failed(reauthRequired());
	}
}

async function passwordProof(
	env: Env,
	now: number,
	session: Session,
	account: ProofAccount,
	value: Record<string, unknown>,
	rules: ProofRules,
): Promise<CheckedProof> {
	if (!rules.passwordCounted) {
		const limited = await countSession(env.DB, "password-account", session, now);
		if (limited) return failed(limited);
	}
	const authKey = proof32(value.auth_key);
	if (!authKey) return failed(badRequest("proof.auth_key must be 32 bytes of canonical base64url."));
	// A passwordless account's auth hash is random; nobody holds a key for it.
	if (!(await proofMatches(authKey, account.auth_hash)) || account.has_password !== 1) return failed(invalidCredentials());
	const recovery = rules.recoveryHash;
	return {
		ok: true,
		kind: "password",
		guard: {
			before: [],
			accountId: account.id,
			// With a recovery key the guard is both hashes, joined (hex has no colon).
			value: recovery === undefined ? account.auth_hash : `${account.auth_hash}:${recovery}`,
			sql: (a, v) =>
				recovery === undefined
					? `EXISTS (SELECT 1 FROM accounts WHERE id = ?${a} AND has_password = 1 AND auth_hash = ?${v})`
					: `EXISTS (SELECT 1 FROM accounts WHERE id = ?${a} AND has_password = 1 AND auth_hash || ':' || recovery_hash = ?${v})`,
		},
	};
}

/** The guard of a flow-backed proof: the attempt ID its consumption stores. */
function flowGuard(accountId: string, attemptId: string, before: D1PreparedStatement[]): ProofGuard {
	return {
		before,
		accountId,
		value: attemptId,
		sql: (a, v) => `EXISTS (SELECT 1 FROM auth_flows WHERE completion_attempt_id = ?${v} AND account_id = ?${a})`,
	};
}

/** A flow of this session's account and device that is still unused. */
function liveFlow(flow: AuthFlowRow | null, session: Session, now: number): flow is AuthFlowRow {
	return (
		flow !== null &&
		flow.account_id === session.accountId &&
		flow.consumed_at === null &&
		flow.expires_at > now
	);
}

const recoveryCondition = (accountParam: number, hashParam: number): string =>
	` AND EXISTS (SELECT 1 FROM accounts WHERE id = ?${accountParam} AND recovery_hash = ?${hashParam})`;

async function passkeyProof(env: Env, now: number, session: Session, value: Record<string, unknown>, rules: ProofRules): Promise<CheckedProof> {
	const advertisement = passkeyAdvertisement(env);
	if (!advertisement) return failed(unsupportedMethod());
	const flowId = value.flow;
	if (typeof flowId !== "string" || !isFlowId(flowId)) return failed(reauthRequired());
	const assertion = value.assertion;
	if (!isObject(assertion)) return failed(badRequest("proof.assertion must be an object."));
	const credentialId = bytesField(assertion.credential_id, 16, 1023);
	const authenticatorData = bytesField(assertion.authenticator_data, 37, 8192);
	const clientDataJson = bytesField(assertion.client_data_json, 1, 8192);
	const signature = bytesField(assertion.signature, 1, 1024);
	if (!credentialId || !authenticatorData || !clientDataJson || !signature) return failed(badRequest("proof.assertion is malformed."));
	if (rules.countProof) {
		const limited = await countSession(env.DB, "proof-account", session, now);
		if (limited) return failed(limited);
	}

	const flow = await flowById(env, flowId);
	// Another session's flow is answered as an unknown one.
	if (
		!liveFlow(flow, session, now) ||
		flow.kind !== "proof" ||
		flow.device_id !== session.deviceId ||
		flow.device_proof_hash === null ||
		!hexEqual(flow.device_proof_hash, session.proofHash)
	) {
		return failed(flowExpired());
	}
	const credential = base64url(credentialId);
	// One try per challenge: a failed assertion uses the flow up.
	const burn = async (): Promise<CheckedProof> => {
		await consumeFlow(env, flowId, now);
		return failed(invalidCredentials());
	};
	// A flow started for one credential (`subject` holds SHA-256 of its ID) takes only that one.
	if (flow.subject !== null && !hexEqual(flow.subject, await sha256Hex(credential))) return burn();
	const row = await env.DB.prepare("SELECT public_key, alg, sign_count FROM account_passkeys WHERE credential_id = ?1 AND account_id = ?2")
		.bind(credential, session.accountId)
		.first<{ public_key: string; alg: CoseAlgorithm; sign_count: number }>();
	const publicKey = row ? fromBase64url(row.public_key) : null;
	if (!row || !publicKey) return burn();
	const verified = await verifyAssertion(
		{ authenticatorData, clientDataJson, signature },
		{ publicKey, alg: row.alg },
		{ rpId: advertisement.rp_id, origins: advertisement.origins, challenge: flow.challenge ?? "" },
	);
	if (!verified || !signCountAccepted(row.sign_count, verified.signCount)) return burn();

	const attemptId = base64url(randomBytes(32));
	const recovery = rules.recoveryHash;
	// The flow is consumed only while the count still allows this assertion (a
	// clone that raced it loses) and, when asked, the recovery key is unchanged;
	// the count and last use are then recorded under the same attempt.
	const consume = env.DB.prepare(
		`UPDATE auth_flows SET consumed_at = ?2, completion_attempt_id = ?3
  WHERE id = ?1 AND kind = 'proof' AND account_id = ?4 AND consumed_at IS NULL AND expires_at > ?2
    AND EXISTS (SELECT 1 FROM account_passkeys WHERE credential_id = ?5 AND account_id = ?4 AND (sign_count = 0 OR sign_count < ?6))${
			recovery === undefined ? "" : recoveryCondition(4, 7)
		}
  RETURNING id`,
	).bind(flowId, now, attemptId, session.accountId, credential, verified.signCount, ...(recovery === undefined ? [] : [recovery]));
	const counted = env.DB.prepare(
		`UPDATE account_passkeys SET sign_count = ?3, last_used_at = ?4
  WHERE credential_id = ?1 AND account_id = ?2 AND EXISTS (SELECT 1 FROM auth_flows WHERE id = ?5 AND completion_attempt_id = ?6)`,
	).bind(credential, session.accountId, verified.signCount, now, flowId, attemptId);
	return { ok: true, kind: "passkey", guard: flowGuard(session.accountId, attemptId, [consume, counted]) };
}

/** Whether a flow's subject may be used: always, except an Apple flow the app hasn't claimed with its secret. */
export const appClaimed = (flow: AuthFlowRow): boolean => flow.kind !== "apple" || flow.app_claimed_at != null;

async function providerProof(env: Env, now: number, session: Session, value: Record<string, unknown>, rules: ProofRules): Promise<CheckedProof> {
	if (rules.provider === null) return failed(reauthRequired());
	const flowId = value.flow;
	if (typeof flowId !== "string" || !isFlowId(flowId)) return failed(reauthRequired());
	if (rules.countProof) {
		const limited = await countSession(env.DB, "proof-account", session, now);
		if (limited) return failed(limited);
	}
	const flow = await flowById(env, flowId);
	// Bound to this session's device and its proof, as a passkey proof's flow is; another session's is unknown.
	if (
		!liveFlow(flow, session, now) ||
		flow.intent !== "proof" ||
		flow.provider === null ||
		flow.device_id !== session.deviceId ||
		flow.device_proof_hash === null ||
		!hexEqual(flow.device_proof_hash, session.proofHash)
	) {
		return failed(flowExpired());
	}
	// A provider proof acts only on its own provider, unless the change takes any.
	if (rules.provider !== "any" && flow.kind !== rules.provider) return failed(reauthRequired());
	// Not finished at the provider yet (or an Apple flow its app never claimed).
	if (flow.subject === null || !appClaimed(flow)) return failed(reauthRequired());
	const linked = await env.DB.prepare("SELECT 1 AS linked FROM account_identities WHERE account_id = ?1 AND provider = ?2 AND subject = ?3")
		.bind(session.accountId, flow.kind, flow.subject)
		.first();
	if (!linked) return failed(reauthRequired());

	const attemptId = base64url(randomBytes(32));
	const recovery = rules.recoveryHash;
	const consume = env.DB.prepare(
		`UPDATE auth_flows SET consumed_at = ?2, completion_attempt_id = ?3
  WHERE id = ?1 AND account_id = ?4 AND intent = 'proof' AND consumed_at IS NULL AND expires_at > ?2
    AND device_id = ?5 AND device_proof_hash = ?6
    AND EXISTS (SELECT 1 FROM sessions WHERE token_hash = ?7 AND device_id = ?5 AND proof_hash = ?6 AND revoked_at IS NULL)
    AND EXISTS (SELECT 1 FROM account_identities i WHERE i.account_id = ?4 AND i.provider = auth_flows.kind AND i.subject = auth_flows.subject)${
			recovery === undefined ? "" : recoveryCondition(4, 8)
		}
  RETURNING id`,
	).bind(flowId, now, attemptId, session.accountId, session.deviceId, session.proofHash, session.tokenHash, ...(recovery === undefined ? [] : [recovery]));
	return { ok: true, kind: "provider", guard: flowGuard(session.accountId, attemptId, [consume]) };
}

/**
 * The optional `current_recovery_auth_key` (or, for unlinking, `recovery_auth_key`)
 * of a body: its hash when it is the account's, undefined when absent, or the
 * response that ends the request (400 malformed, 401 wrong). Compared in
 * constant time; the caller puts the hash in its statements' guard.
 */
export async function recoveryProof(
	body: Record<string, unknown>,
	field: string,
	account: ProofAccount,
): Promise<{ ok: true; hash: string | undefined } | { ok: false; response: Response }> {
	if (body[field] === undefined) return { ok: true, hash: undefined };
	const key = proof32(body[field]);
	if (!key) return { ok: false, response: badRequest(`${field} must be 32 bytes of canonical base64url.`) };
	if (!(await proofMatches(key, account.recovery_hash))) return { ok: false, response: invalidCredentials() };
	return { ok: true, hash: account.recovery_hash };
}

/**
 * Checks a change's proof together with the recovery key it may carry: a
 * provider proof here also needs `current_recovery_auth_key` (401
 * `reauth_required` without it), and a given recovery key is checked and held
 * in the guard whatever the proof.
 */
export async function proofWithRecovery(
	env: Env,
	now: number,
	session: Session,
	account: ProofAccount,
	body: Record<string, unknown>,
	rules: Omit<ProofRules, "recoveryHash" | "provider">,
): Promise<CheckedProof> {
	const recovery = await recoveryProof(body, "current_recovery_auth_key", account);
	if (!recovery.ok) return failed(recovery.response);
	const proof = body.proof;
	if (isObject(proof) && proof.kind === "provider" && recovery.hash === undefined) return failed(reauthRequired());
	return checkProof(env, now, session, account, proof, { ...rules, provider: "any", recoveryHash: recovery.hash });
}

// ----------------------------------------------------------------- proof/start

/**
 * `POST /v2/account/sign-in/proof/start` — `{credential_id?}`. A challenge
 * for a passkey proof: an `auth_flows` row of kind `proof` for this account,
 * bound to this session's device and its proof, alive five minutes and used
 * once. With `credential_id` the assertion must come from that credential
 * (404 when it isn't one of the account's); without it, from any of them.
 * Counts `proof-account`. 400 `unsupported_method` without passkeys.
 */
export const proofStart = withSessionKeys(async (request, env, now, keys, session) => {
	const advertisement = passkeyAdvertisement(env);
	if (!advertisement) return unsupportedMethod();
	const limited = await countSession(env.DB, "proof-account", session, now);
	if (limited) return limited;
	const body = await readObject(request);
	if (!body.ok) return body.response;
	let only: string | null = null;
	if (body.value.credential_id !== undefined) {
		const raw = bytesField(body.value.credential_id, 16, 1023);
		if (!raw) return badRequest("credential_id must be 16–1023 bytes of canonical base64url.");
		only = base64url(raw);
	}
	const { results } = await env.DB.prepare(
		"SELECT credential_id, transports FROM account_passkeys WHERE account_id = ?1 ORDER BY created_at, credential_id",
	)
		.bind(session.accountId)
		.all<{ credential_id: string; transports: string }>();
	const listed = only === null ? results : results.filter((row) => row.credential_id === only);
	if (only !== null && listed.length === 0) return notFound();

	const challenge = base64url(randomBytes(32));
	const { id, statement } = await flowStatement(
		env,
		keys,
		{
			kind: "proof",
			intent: "proof",
			account_id: session.accountId,
			device_id: session.deviceId,
			device_proof_hash: session.proofHash,
			challenge,
			rp_id: advertisement.rp_id,
			nonce_hash: null,
			state_hash: null,
			pkce_hash: null,
			redirect_uri: null,
			provider: null,
			// The one credential this proof may come from, as SHA-256 of its ID (the column is short).
			subject: only === null ? null : await sha256Hex(only),
			device_code: null,
			next_poll_at: null,
			expires_at: now + PROOF_FLOW_MS,
		},
		now,
	);
	await statement.run();
	return json({
		flow_id: id,
		challenge,
		rp_id: advertisement.rp_id,
		allow_credentials: listed.map((row) => ({ id: row.credential_id, transports: storedTransports(row.transports) })),
		user_verification: "required",
		timeout: PROOF_FLOW_MS,
	});
});
