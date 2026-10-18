// Purges of R2 prefixes and the cron's other jobs (docs/ACCOUNTS.md › R2
// layout, purges and the cron).
//
// A purge round is one list of up to 1,000 keys and one delete. A purge row
// goes only when a list comes back empty and the row is more than an hour
// old, so an upload still in flight can't leave an orphan behind.
//
// Account IDs are chosen by the client, so a deleted account's ID can sign up
// again. Purging `accounts/<id>/` while an account owns it would delete that
// account's files, so sign-up clears the purge row for its prefix, and a
// round finding the account alive drops the row instead of listing; objects
// the old account left behind are then orphans, which reconciliation removes.

import type { Env } from "./env";
import { expiredLimitsSql } from "./limits";
import { FILE_ROW_COST, HOUR, MAX_SEALED_FILE, MIN_SEALED_FILE, REVOKED_KEEP_MS, SESSION_IDLE_MS } from "./protocol";

export const PURGE_LIST_LIMIT = 1000;
/** Rounds the cron spends on purges each run. */
export const CRON_PURGE_ROUNDS = 50;
/** Rounds an account deletion runs inline. */
export const DELETE_PURGE_ROUNDS = 20;
export const CRON_PERIOD_MS = 15 * 60_000;
/** Accounts whose prefixes one cron run reconciles, taken in turn. */
export const RECONCILE_ACCOUNTS_PER_RUN = 5;
/** R2 list calls one cron run may spend reconciling. */
export const RECONCILE_LIST_BUDGET = 50;

export const accountPrefix = (accountId: string): string => `accounts/${accountId}/`;
export const fileKey = (accountId: string, wireId: string): string => `${accountPrefix(accountId)}${wireId}`;

type RoundResult = "more" | "empty" | "done";

/** One purge round for `prefix`. */
export async function purgeRound(env: Env, prefix: string, requestedAt: number, now: number): Promise<RoundResult> {
	const owner = /^accounts\/(a_[0-9a-f]{32})\/$/.exec(prefix)?.[1];
	if (owner) {
		const alive = await env.DB.prepare("SELECT 1 AS alive FROM accounts WHERE id = ?1").bind(owner).first();
		if (alive) {
			await env.DB.prepare("DELETE FROM purges WHERE prefix = ?1").bind(prefix).run();
			return "done";
		}
	}
	const listed = await env.FILES.list({ prefix, limit: PURGE_LIST_LIMIT });
	if (listed.objects.length > 0) {
		await env.FILES.delete(listed.objects.map((o) => o.key));
		return "more";
	}
	if (requestedAt <= now - HOUR) {
		await env.DB.prepare("DELETE FROM purges WHERE prefix = ?1 AND requested_at <= ?2").bind(prefix, now - HOUR).run();
		return "done";
	}
	return "empty";
}

/** Up to `rounds` rounds on one prefix; how many it used. */
export async function purgePrefix(env: Env, prefix: string, requestedAt: number, now: number, rounds: number): Promise<number> {
	let used = 0;
	while (used < rounds) {
		used += 1;
		if ((await purgeRound(env, prefix, requestedAt, now)) !== "more") break;
	}
	return used;
}

/** Up to `budget` rounds over every pending purge, oldest first; the rounds used. */
export async function runPurges(env: Env, now: number, budget = CRON_PURGE_ROUNDS): Promise<number> {
	const { results } = await env.DB.prepare("SELECT prefix, requested_at FROM purges ORDER BY requested_at, prefix").all<{
		prefix: string;
		requested_at: number;
	}>();
	let used = 0;
	for (const row of results) {
		if (used >= budget) break;
		used += await purgePrefix(env, row.prefix, row.requested_at, now, budget - used);
	}
	return used;
}

// ------------------------------------------------------------ reconciliation

export interface ReconcileReport {
	accounts: string[];
	orphansDeleted: number;
	sizesFixed: number;
	rowsDropped: number;
}

const ROWS_FOR_IDS = `SELECT id, size, uploaded_at FROM account_files
  WHERE account_id = ?1 AND id IN (SELECT value FROM json_each(?2))`;

// Correcting a size can grow file_bytes; setting recounted_at exempts it from
// the quota triggers, as the admin recount is.
const FIX_SIZE = [
	`UPDATE accounts SET file_bytes = file_bytes - ?3 + ?4, recounted_at = ?5
  WHERE id = ?1 AND EXISTS (SELECT 1 FROM account_files WHERE account_id = ?1 AND id = ?2 AND size = ?3)`,
	"UPDATE account_files SET size = ?4 WHERE account_id = ?1 AND id = ?2 AND size = ?3",
];

const DROP_ROW = [
	`UPDATE accounts SET files = files - 1, file_bytes = file_bytes - (?3 + ${FILE_ROW_COST})
  WHERE id = ?1 AND EXISTS (SELECT 1 FROM account_files WHERE account_id = ?1 AND id = ?2 AND size = ?3 AND uploaded_at <= ?4)`,
	"DELETE FROM account_files WHERE account_id = ?1 AND id = ?2 AND size = ?3 AND uploaded_at <= ?4",
];

/**
 * Reconciles one account's R2 prefix with its `account_files` rows: deletes
 * objects that have no row and are more than an hour old, sets a row's size
 * to its object's where they differ, and (when the whole prefix was listed)
 * drops rows over an hour old whose object is gone. Returns the list budget left.
 */
async function reconcileAccount(env: Env, accountId: string, now: number, budget: number, report: ReconcileReport): Promise<number> {
	const prefix = accountPrefix(accountId);
	const seen = new Set<string>();
	let cursor: string | undefined;
	let complete = false;
	while (budget > 0) {
		budget -= 1;
		const listed = await env.FILES.list({ prefix, limit: PURGE_LIST_LIMIT, cursor });
		const ids = listed.objects.map((o) => o.key.slice(prefix.length));
		const { results: rows } = await env.DB.prepare(ROWS_FOR_IDS)
			.bind(accountId, JSON.stringify(ids))
			.all<{ id: string; size: number; uploaded_at: number }>();
		const byId = new Map(rows.map((r) => [r.id, r]));
		const orphans: string[] = [];
		const fixes: D1PreparedStatement[] = [];
		for (const object of listed.objects) {
			const id = object.key.slice(prefix.length);
			seen.add(id);
			const row = byId.get(id);
			if (!row) {
				if (object.uploaded.getTime() <= now - HOUR) orphans.push(object.key);
			} else if (
				row.size !== object.size &&
				row.uploaded_at <= now - HOUR &&
				object.size >= MIN_SEALED_FILE &&
				object.size <= MAX_SEALED_FILE
			) {
				fixes.push(
					env.DB.prepare(FIX_SIZE[0]).bind(accountId, id, row.size, object.size, now),
					env.DB.prepare(FIX_SIZE[1]).bind(accountId, id, row.size, object.size),
				);
				report.sizesFixed += 1;
			}
		}
		if (orphans.length > 0) {
			await env.FILES.delete(orphans);
			report.orphansDeleted += orphans.length;
		}
		if (fixes.length > 0) await env.DB.batch(fixes);
		if (!listed.truncated) {
			complete = true;
			break;
		}
		cursor = listed.cursor;
	}
	if (complete) {
		const { results } = await env.DB.prepare("SELECT id, size FROM account_files WHERE account_id = ?1 AND uploaded_at <= ?2")
			.bind(accountId, now - HOUR)
			.all<{ id: string; size: number }>();
		const drops = results
			.filter((r) => !seen.has(r.id))
			.flatMap((r) => DROP_ROW.map((sql) => env.DB.prepare(sql).bind(accountId, r.id, r.size, now - HOUR)));
		if (drops.length > 0) {
			await env.DB.batch(drops);
			report.rowsDropped += drops.length / 2;
		}
	}
	return budget;
}

/** Reconciles a bounded number of accounts' prefixes, taking accounts in turn from one run to the next. */
export async function reconcile(env: Env, now: number): Promise<ReconcileReport> {
	const report: ReconcileReport = { accounts: [], orphansDeleted: 0, sizesFixed: 0, rowsDropped: 0 };
	const { results } = await env.DB.prepare("SELECT id FROM accounts ORDER BY id").all<{ id: string }>();
	if (results.length === 0) return report;
	const count = Math.min(RECONCILE_ACCOUNTS_PER_RUN, results.length);
	const start = (Math.floor(now / CRON_PERIOD_MS) * RECONCILE_ACCOUNTS_PER_RUN) % results.length;
	let budget = RECONCILE_LIST_BUDGET;
	for (let i = 0; i < count && budget > 0; i++) {
		const { id } = results[(start + i) % results.length];
		report.accounts.push(id);
		budget = await reconcileAccount(env, id, now, budget, report);
	}
	return report;
}

// ---------------------------------------------------------------------- cron

export interface CronReport {
	purgeRounds: number;
	reconcile: ReconcileReport;
}

/** The scheduled jobs, every 15 minutes. */
export async function runCron(env: Env, now: number): Promise<CronReport> {
	const purgeRounds = await runPurges(env, now);
	await env.DB.prepare(expiredLimitsSql()).bind(now).run();
	const reconciled = await reconcile(env, now);
	await env.DB.batch([
		env.DB.prepare(
			"UPDATE sessions SET revoked_at = ?1, revoked_reason = 'expired' WHERE revoked_at IS NULL AND last_seen_at < ?1 - ?2",
		).bind(now, SESSION_IDLE_MS),
		env.DB.prepare("DELETE FROM sessions WHERE revoked_at IS NOT NULL AND revoked_at < ?1 - ?2").bind(now, REVOKED_KEEP_MS),
	]);
	return { purgeRounds, reconcile: reconciled };
}
