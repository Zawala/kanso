// The server's own numbers (docs/METRICS.md › Server metrics): requests by route group and status
// class, latency buckets, page views, new accounts and devices seen, sent to PostHog as one
// `server_stats` event per cron run. Aggregates only: nothing here holds or sends an IP, an ID, a
// token or a path. A route group is a name from the router, never the URL.
//
// Counting must not slow a request, so each isolate adds to a map in memory and writes the map to
// D1 in one batch, at most once a minute, after the response (ctx.waitUntil). An isolate that is
// recycled before its flush loses its last minute; these are statistics, not accounts.

import type { Env } from "./env";
import { forward, checkKey } from "./metrics";

/** The window the counts and the event cover. It is the cron's period. */
export const WINDOW_MS = 15 * 60_000;
/** An isolate writes its counts at most this often. */
export const FLUSH_EVERY_MS = 60_000;
export const SENDER = "kanso-sync-server";

export const GROUPS = ["auth", "account", "sync", "publish", "publish_page", "share_page", "collab", "telemetry", "other"] as const;
export type Group = (typeof GROUPS)[number];
export const CLASSES = ["2xx", "3xx", "4xx", "5xx"] as const;
export type StatusClass = (typeof CLASSES)[number];
/** Closed latency buckets, in order. */
export const LATENCY_BUCKETS = ["<50ms", "50-100ms", "100-250ms", "250-500ms", "500-1000ms", "1000ms+"] as const;
export type Latency = (typeof LATENCY_BUCKETS)[number];

const AUTH = /^(health|prelogin|signup|login|logout|recover_.*|passkey_(start|finish|signup_.*)|apple_.*|google_.*|github_.*|social_signup)$/;
const ACCOUNT =
	/^(account|password|recovery_key|delete|set_device|devices|device_push|revoke_device|identity|user|sign_in_methods|set_password|proof_start|passkey_register_.*|passkey_remove|identity_.*|approvals?(_.*)?|account_events)$/;

/** The group of a router route name (`unknown` and `aasa` are `other`). */
export function routeGroup(name: string): Group {
	if (AUTH.test(name)) return "auth";
	if (ACCOUNT.test(name)) return "account";
	if (name === "push" || name === "changes" || name === "files") return "sync";
	if (name === "shares") return "publish";
	if (name === "public" || name === "viewer") return "publish_page";
	if (name === "invite") return "share_page";
	if (name.startsWith("collab_")) return "collab";
	if (name === "metrics" || name === "crash_reports") return "telemetry";
	return "other";
}

/** The class of a status; 1xx and anything odd count as 2xx and 5xx would hide, so they are `2xx`. */
export function statusClass(status: number): StatusClass {
	if (status >= 500) return "5xx";
	if (status >= 400) return "4xx";
	if (status >= 300) return "3xx";
	return "2xx";
}

export function latencyBucket(ms: number): Latency {
	if (ms < 50) return "<50ms";
	if (ms < 100) return "50-100ms";
	if (ms < 250) return "100-250ms";
	if (ms < 500) return "250-500ms";
	if (ms < 1000) return "500-1000ms";
	return "1000ms+";
}

export const windowOf = (ms: number): number => Math.floor(ms / WINDOW_MS) * WINDOW_MS;

export interface CountRow {
	window: number;
	group: Group;
	status: StatusClass;
	latency: Latency;
	count: number;
}

/** The counts one isolate holds between flushes. */
export class RequestCounter {
	private rows = new Map<string, CountRow>();
	private lastFlush = 0;

	record(routeName: string, status: number, ms: number, at: number): void {
		const row: CountRow = { window: windowOf(at), group: routeGroup(routeName), status: statusClass(status), latency: latencyBucket(ms), count: 1 };
		const key = `${row.window}|${row.group}|${row.status}|${row.latency}`;
		const held = this.rows.get(key);
		if (held) held.count += 1;
		else this.rows.set(key, row);
	}

	get size(): number {
		return this.rows.size;
	}

	/** Whether a flush is due: something is held and the last one was a minute ago. */
	due(at: number): boolean {
		return this.rows.size > 0 && at - this.lastFlush >= FLUSH_EVERY_MS;
	}

	/** Hands over what is held, emptying it. */
	take(at: number): CountRow[] {
		this.lastFlush = at;
		const rows = [...this.rows.values()];
		this.rows.clear();
		return rows;
	}
}

export const counter = new RequestCounter();

const UPSERT = `INSERT INTO request_counts (window_start, route_group, status_class, latency_bucket, count) VALUES (?1, ?2, ?3, ?4, ?5)
ON CONFLICT (window_start, route_group, status_class, latency_bucket) DO UPDATE SET count = count + excluded.count`;

/** Adds `rows` to D1 in one batch. A failure loses them (and says so by name only). */
export async function flush(env: Env, rows: CountRow[]): Promise<void> {
	if (rows.length === 0) return;
	try {
		await env.DB.batch(rows.map((r) => env.DB.prepare(UPSERT).bind(r.window, r.group, r.status, r.latency, r.count)));
	} catch (err) {
		console.error(JSON.stringify({ route: "server_stats", outcome: "flush_failed", kind: (err as Error)?.name ?? "error" }));
	}
}

/**
 * Counts one answered request and, when a minute has passed, writes the isolate's counts after the
 * response. `ctx` is absent in tests that call `handle` directly.
 */
export function observe(env: Env, ctx: Pick<ExecutionContext, "waitUntil"> | undefined, routeName: string, status: number, ms: number, at: number): void {
	counter.record(routeName, status, ms, at);
	if (ctx && counter.due(at)) ctx.waitUntil(flush(env, counter.take(at)));
}

// ------------------------------------------------------------------- the event

export interface WindowCounts {
	rows: { route_group: string; status_class: string; latency_bucket: string; count: number }[];
	newAccounts: number;
	devicesSynced: number;
}

/** The bucket holding the `p`th percentile (0 to 1) of the counts, in bucket order. */
export function percentileBucket(perBucket: Record<string, number>, p: number): Latency | "none" {
	const total = LATENCY_BUCKETS.reduce((n, b) => n + (perBucket[b] ?? 0), 0);
	if (total === 0) return "none";
	const rank = Math.max(1, Math.ceil(total * p));
	let seen = 0;
	for (const bucket of LATENCY_BUCKETS) {
		seen += perBucket[bucket] ?? 0;
		if (seen >= rank) return bucket;
	}
	return "1000ms+";
}

/** The event's props: every one a count or a bucket label, named from closed lists. */
export function statsProps(w: WindowCounts): Record<string, string | number> {
	const props: Record<string, string | number> = { window_minutes: WINDOW_MS / 60_000 };
	const groups = new Map<string, number>();
	const classes = new Map<string, number>();
	const buckets: Record<string, number> = {};
	for (const g of GROUPS) for (const c of CLASSES) groups.set(`${g}_${c}`, 0);
	for (const c of CLASSES) classes.set(c, 0);
	for (const r of w.rows) {
		const key = `${r.route_group}_${r.status_class}`;
		if (!groups.has(key)) continue;
		groups.set(key, (groups.get(key) ?? 0) + r.count);
		classes.set(r.status_class, (classes.get(r.status_class) ?? 0) + r.count);
		buckets[r.latency_bucket] = (buckets[r.latency_bucket] ?? 0) + r.count;
	}
	props.requests_total = [...classes.values()].reduce((a, b) => a + b, 0);
	for (const c of CLASSES) props[`requests_${c}`] = classes.get(c) ?? 0;
	for (const [key, n] of groups) props[`req_${key}`] = n;
	props.errors_5xx = classes.get("5xx") ?? 0;
	props.latency_p50 = percentileBucket(buckets, 0.5);
	props.latency_p95 = percentileBucket(buckets, 0.95);
	props.new_accounts = w.newAccounts;
	props.devices_synced = w.devicesSynced;
	// A page view is a page that was served.
	props.share_views = groups.get("share_page_2xx") ?? 0;
	props.publish_views = groups.get("publish_page_2xx") ?? 0;
	return props;
}

/** PostHog's batch body for one `server_stats` event (no country, no person profile). */
export function statsBody(key: string, props: Record<string, string | number>, now: number): unknown {
	return {
		api_key: key,
		batch: [
			{
				event: "server_stats",
				distinct_id: SENDER,
				timestamp: new Date(now).toISOString(),
				properties: { ...props, $process_person_profile: false, $geoip_disable: true, $lib: "kanso-worker" },
			},
		],
	};
}

/**
 * The cron's part: reads the window that ended at least one window ago (so every isolate's last
 * flush has landed), sends it as one `server_stats` event, then deletes it and everything older.
 * Without POSTHOG_KEY and POSTHOG_HOST it only does the deleting.
 */
export async function runServerStats(env: Env, now: number): Promise<void> {
	const start = windowOf(now) - 2 * WINDOW_MS;
	const end = start + WINDOW_MS;
	const key = env.POSTHOG_KEY?.trim();
	if (key && env.POSTHOG_HOST) {
		const [rows, accounts, devices] = await Promise.all([
			env.DB.prepare("SELECT route_group, status_class, latency_bucket, count FROM request_counts WHERE window_start = ?1").bind(start).all(),
			env.DB.prepare("SELECT count(*) AS n FROM accounts WHERE created_at >= ?1 AND created_at < ?2").bind(start, end).first<{ n: number }>(),
			// Sessions are stamped when a device is seen, at most once an hour, so a device counts
			// in the window it was refreshed in. Only the number leaves the database.
			env.DB.prepare("SELECT count(*) AS n FROM sessions WHERE last_seen_at >= ?1 AND last_seen_at < ?2").bind(start, end).first<{ n: number }>(),
		]);
		const props = statsProps({
			rows: rows.results as WindowCounts["rows"],
			newAccounts: accounts?.n ?? 0,
			devicesSynced: devices?.n ?? 0,
		});
		await Promise.all([checkKey(env.POSTHOG_HOST, key), forward(env.POSTHOG_HOST, statsBody(key, props, now))]);
	}
	await env.DB.prepare("DELETE FROM request_counts WHERE window_start <= ?1").bind(start).run();
}
