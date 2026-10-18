// Rate limits (docs/ACCOUNTS.md › Rate limits). Two layers:
//
// 1. The Workers rate-limit binding AUTH_LIMITER, keyed by the caller's IP
//    group, in front of every /v2/auth/* and /v2/health request. It writes
//    nothing to D1; without the binding (unit tests) the layer is skipped.
// 2. Fixed-window counters in D1, the authority. The counting upsert runs
//    before credentials are checked or any work is done, and the request is
//    refused when the count it returns is over the maximum, so parallel
//    requests can't pass together.
//
// Keys never hold an IP or a username: IP subjects go through an HMAC keyed
// by the pepper's `limit` subkey; account subjects use the account ID.

import { type PepperKeys, hex, hmac16 } from "./crypto";
import { rateLimited } from "./http";
import { DAY, HOUR } from "./protocol";

export type Rule = "signup-ip" | "login-user" | "login-ip" | "recover-ip" | "password-account" | "writes-account";

/** Window and maximum of each rule; `writes-account` takes its maximum from `settings.writes_per_day`. */
export const RULES: Record<Rule, { window: number; max: number | null }> = {
	"signup-ip": { window: HOUR, max: 10 },
	"login-user": { window: 15 * 60_000, max: 10 },
	"login-ip": { window: 15 * 60_000, max: 30 },
	"recover-ip": { window: HOUR, max: 10 },
	"password-account": { window: 15 * 60_000, max: 5 },
	"writes-account": { window: DAY, max: null },
};

/** The binding's window, which is also its `retry_after`. */
export const BINDING_PERIOD_MS = 60_000;

export const COUNT_SQL = `INSERT INTO limits (key, window_start, count) VALUES (?1, ?2, ?3)
ON CONFLICT (key) DO UPDATE SET
  count = CASE WHEN window_start = excluded.window_start THEN count + excluded.count ELSE excluded.count END,
  window_start = excluded.window_start
RETURNING count`;

export const windowStart = (now: number, window: number): number => Math.floor(now / window) * window;

/** `rule:` and 16 bytes of HMAC(limit_key, rule || 0x00 || subject), as hex. */
export async function ipKey(keys: PepperKeys, rule: Rule, subject: string): Promise<string> {
	return `${rule}:${hex(await hmac16(keys.limit, rule, subject))}`;
}

/** Account subjects aren't secret: `rule:<account id>`, which the admin can read. */
export const accountKey = (rule: Rule, accountId: string): string => `${rule}:${accountId}`;

/** `login-user` for a name with no account: every such name shares this row. */
export const UNKNOWN_LOGIN_KEY = "login-user:unknown";

/**
 * Counts `amount` against `key` under `rule`; the 429 response when the
 * window's count is now over `max`, else null.
 */
export async function count(db: D1Database, rule: Rule, key: string, now: number, amount = 1, max?: number): Promise<Response | null> {
	const { window } = RULES[rule];
	const start = windowStart(now, window);
	if (rule === "writes-account") {
		const [counted, setting] = await db.batch<{ count?: number; max?: number | null }>([
			db.prepare(COUNT_SQL).bind(key, start, amount),
			db.prepare("SELECT CAST(value AS INTEGER) AS max FROM settings WHERE name = 'writes_per_day'"),
		]);
		const total = counted.results[0]?.count ?? Number.MAX_SAFE_INTEGER;
		// A missing setting refuses, like the triggers.
		const limit = setting.results[0]?.max ?? 0;
		return total > limit ? rateLimited(rule, start + window - now) : null;
	}
	const row = await db.prepare(COUNT_SQL).bind(key, start, amount).first<{ count: number }>();
	const limit = max ?? RULES[rule].max ?? 0;
	return (row?.count ?? Number.MAX_SAFE_INTEGER) > limit ? rateLimited(rule, start + window - now) : null;
}

/** Deletes every row whose window has ended (the cron); rows of unknown rules go after two days. */
export function expiredLimitsSql(): string {
	const rules = Object.entries(RULES).map(([rule, { window }]) => `(key GLOB '${rule}:*' AND window_start + ${window} <= ?1)`);
	return `DELETE FROM limits WHERE ${rules.join(" OR ")} OR window_start + ${2 * DAY} <= ?1`;
}

// ----------------------------------------------------------------- IP groups

/** The eight 16-bit groups of an IPv6 address, or null. */
function parseIpv6(text: string): number[] | null {
	const address = text.split("%")[0].toLowerCase();
	const halves = address.split("::");
	if (halves.length > 2) return null;
	const parse = (part: string): number[] | null => {
		if (part === "") return [];
		const groups: number[] = [];
		const pieces = part.split(":");
		for (const [i, piece] of pieces.entries()) {
			if (i === pieces.length - 1 && piece.includes(".")) {
				const v4 = parseIpv4(piece);
				if (!v4) return null;
				groups.push((v4[0] << 8) | v4[1], (v4[2] << 8) | v4[3]);
			} else if (/^[0-9a-f]{1,4}$/.test(piece)) {
				groups.push(parseInt(piece, 16));
			} else {
				return null;
			}
		}
		return groups;
	};
	const head = parse(halves[0]);
	const tail = halves.length === 2 ? parse(halves[1]) : [];
	if (!head || !tail) return null;
	if (halves.length === 1) return head.length === 8 ? head : null;
	const missing = 8 - head.length - tail.length;
	if (missing < 1) return null;
	return [...head, ...new Array<number>(missing).fill(0), ...tail];
}

function parseIpv4(text: string): number[] | null {
	const parts = text.split(".");
	if (parts.length !== 4 || parts.some((p) => !/^[0-9]{1,3}$/.test(p) || Number(p) > 255)) return null;
	return parts.map(Number);
}

/**
 * The subject a rule counts by: IPv4 per address, IPv6 per `/prefix` (56 in
 * D1, 48 for sign-up and the binding). `CF-Connecting-IP` supplies the
 * address; "unknown" stands in when it's absent (local dev).
 */
export function ipGroup(ip: string | null, prefix: 48 | 56): string {
	if (!ip) return "unknown";
	if (!ip.includes(":")) return parseIpv4(ip) ? ip : ip.toLowerCase();
	const groups = parseIpv6(ip);
	if (!groups) return ip.toLowerCase();
	// An IPv4-mapped address (::ffff:a.b.c.d) is that IPv4 address.
	if (groups.slice(0, 5).every((g) => g === 0) && groups[5] === 0xffff) {
		return [groups[6] >> 8, groups[6] & 255, groups[7] >> 8, groups[7] & 255].join(".");
	}
	const masked = groups.map((g, i) => {
		const bits = Math.max(0, Math.min(16, prefix - 16 * i));
		return bits === 0 ? 0 : g & ((0xffff << (16 - bits)) & 0xffff);
	});
	return `${masked.map((g) => g.toString(16)).join(":")}/${prefix}`;
}

export const clientIp = (request: Request): string | null => request.headers.get("cf-connecting-ip");

/** The binding layer: 429 `binding` when AUTH_LIMITER refuses this IP group, else null. */
export async function bindingFilter(limiter: RateLimit | undefined, request: Request): Promise<Response | null> {
	if (!limiter) return null;
	const { success } = await limiter.limit({ key: ipGroup(clientIp(request), 48) });
	return success ? null : rateLimited("binding", BINDING_PERIOD_MS);
}
