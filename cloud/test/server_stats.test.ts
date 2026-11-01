// Server metrics (docs/METRICS.md › Server metrics): requests counted in memory and written in
// batches, the cron's one `server_stats` event to PostHog, and that nothing identifying is in
// the rows or the event. Properties on the closed lists (fast-check, seed pinned in
// vitest.config.ts).

import { env } from "cloudflare:workers";
import fc from "fast-check";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import worker, { handle, runCron } from "../src/index";
import { resetKeyChecks } from "../src/metrics";
import {
	CLASSES,
	FLUSH_EVERY_MS,
	GROUPS,
	MAX_WINDOWS_PER_RUN,
	LATENCY_BUCKETS,
	RequestCounter,
	WINDOW_MS,
	counter,
	flush,
	latencyBucket,
	observe,
	percentileBucket,
	routeGroup,
	statsProps,
	statusClass,
	windowOf,
} from "../src/server_stats";
import { ORIGIN, START, resetServer, signUp, testEnv } from "./helpers";

const runs = (n: number) => ({ numRuns: Math.min(n, Number(env.FAST_CHECK_RUNS)) });
const HOST = "https://eu.i.posthog.com";

interface Sent {
	url: string;
	body: any;
}
let sent: Sent[];

beforeEach(async () => {
	await resetServer();
	await env.DB.prepare("DELETE FROM request_counts").run();
	counter.take(0);
	sent = [];
	resetKeyChecks();
	vi.spyOn(globalThis, "fetch").mockImplementation(async (input, init) => {
		if (String(input).includes("/flags/")) return Response.json({ flags: {} });
		sent.push({ url: String(input), body: JSON.parse(String(init?.body)) });
		return Response.json({ status: "Ok" });
	});
});
afterEach(() => vi.restoreAllMocks());

const rows = async () => (await env.DB.prepare("SELECT * FROM request_counts ORDER BY window_start, route_group, status_class, latency_bucket").all()).results;

describe("grouping", () => {
	it("names every router route's group, and leaves only the unknown to other", () => {
		expect(routeGroup("signup")).toBe("auth");
		expect(routeGroup("passkey_signup_finish")).toBe("auth");
		expect(routeGroup("devices")).toBe("account");
		expect(routeGroup("approval_approve")).toBe("account");
		expect(routeGroup("push")).toBe("sync");
		expect(routeGroup("files")).toBe("sync");
		expect(routeGroup("shares")).toBe("publish");
		expect(routeGroup("viewer")).toBe("publish_page");
		expect(routeGroup("public")).toBe("publish_page");
		expect(routeGroup("invite")).toBe("share_page");
		expect(routeGroup("collab_log")).toBe("collab");
		expect(routeGroup("metrics")).toBe("telemetry");
		expect(routeGroup("unknown")).toBe("other");
		expect(routeGroup("aasa")).toBe("other");
	});

	it("prop_every_value_produced_is_in_its_closed_list_and_buckets_are_monotonic", () => {
		fc.assert(
			fc.property(fc.string(), fc.integer({ min: 100, max: 599 }), fc.double({ min: 0, max: 100_000, noNaN: true }), fc.double({ min: 0, max: 100_000, noNaN: true }), (name, status, a, b) => {
				expect(GROUPS).toContain(routeGroup(name));
				expect(CLASSES).toContain(statusClass(status));
				const [lo, hi] = a <= b ? [a, b] : [b, a];
				expect(LATENCY_BUCKETS.indexOf(latencyBucket(lo))).toBeLessThanOrEqual(LATENCY_BUCKETS.indexOf(latencyBucket(hi)));
				expect(statusClass(status) === "5xx").toBe(status >= 500);
			}),
			runs(200),
		);
	});

	it("prop_percentiles_are_ordered_buckets_and_the_event_has_only_named_props", () => {
		const groups = fc.constantFrom(...GROUPS);
		const row = fc.record({ route_group: groups, status_class: fc.constantFrom(...CLASSES), latency_bucket: fc.constantFrom(...LATENCY_BUCKETS), count: fc.integer({ min: 1, max: 1000 }) });
		fc.assert(
			fc.property(fc.array(row, { maxLength: 40 }), fc.nat(50), fc.option(fc.nat(50), { nil: null }), (list, accounts, devices) => {
				const props = statsProps({ rows: list, newAccounts: accounts, devicesSeen: devices });
				const total = list.reduce((n, r) => n + r.count, 0);
				expect(props.requests_total).toBe(total);
				expect(props.errors_5xx).toBe(list.filter((r) => r.status_class === "5xx").reduce((n, r) => n + r.count, 0));
				if (total === 0) {
					expect([props.latency_p50, props.latency_p95]).toEqual(["none", "none"]);
				} else {
					expect(LATENCY_BUCKETS.indexOf(props.latency_p50 as any)).toBeLessThanOrEqual(LATENCY_BUCKETS.indexOf(props.latency_p95 as any));
				}
				for (const [key, value] of Object.entries(props)) {
					expect(key).toMatch(/^[a-z0-9_]+$/);
					if (typeof value === "string") expect([...LATENCY_BUCKETS, "none"]).toContain(value);
					else expect(Number.isInteger(value) && value >= 0).toBe(true);
				}
			}),
			runs(200),
		);
		expect(percentileBucket({ "<50ms": 90, "1000ms+": 10 }, 0.5)).toBe("<50ms");
		expect(percentileBucket({ "<50ms": 90, "1000ms+": 10 }, 0.95)).toBe("1000ms+");
	});
});

describe("counting", () => {
	it("holds counts in memory and writes them at most once a minute, after the response", async () => {
		const waits: Promise<unknown>[] = [];
		const ctx = { waitUntil: (p: Promise<unknown>) => void waits.push(p) };
		const e = testEnv();
		// The first request is due at once; the next hundred inside the minute only add to memory.
		observe(e, ctx, "push", 200, 12, START);
		expect(waits).toHaveLength(1);
		for (let i = 0; i < 100; i++) observe(e, ctx, "push", 200, 12, START + 1000 + i);
		observe(e, ctx, "changes", 500, 700, START + 2000);
		expect(waits).toHaveLength(1);
		expect(counter.size).toBe(2);
		await Promise.all(waits);
		expect(await rows()).toEqual([{ window_start: START, route_group: "sync", status_class: "2xx", latency_bucket: "<50ms", count: 1 }]);
		// A minute later one batch writes the rest, adding to the rows that are there.
		observe(e, ctx, "push", 200, 12, START + FLUSH_EVERY_MS);
		expect(waits).toHaveLength(2);
		await Promise.all(waits);
		expect(await rows()).toEqual([
			{ window_start: START, route_group: "sync", status_class: "2xx", latency_bucket: "<50ms", count: 102 },
			{ window_start: START, route_group: "sync", status_class: "5xx", latency_bucket: "500-1000ms", count: 1 },
		]);
	});

	it("counts a minute from the first counted request, not from the cold start", () => {
		const c = new RequestCounter();
		expect(c.due(START)).toBe(false);
		c.record("push", 200, 1, START);
		expect(c.due(START)).toBe(false);
		expect(c.due(START + FLUSH_EVERY_MS - 1)).toBe(false);
		expect(c.due(START + FLUSH_EVERY_MS)).toBe(true);
		c.take(START + FLUSH_EVERY_MS);
		c.record("push", 200, 1, START + FLUSH_EVERY_MS + 1);
		expect(c.due(START + FLUSH_EVERY_MS + 2)).toBe(false);
	});

	it("matches the route once: handle reports the name it matched", async () => {
		const named = async (path: string) => {
			const matched = { name: "?" };
			await handle(new Request(`${ORIGIN}${path}`), testEnv(), START, matched);
			return matched.name;
		};
		expect(await named("/p/abc")).toBe("viewer");
		expect(await named("/j/abc")).toBe("invite");
		expect(await named("/nowhere")).toBe("unknown");
	});

	it("keeps requests of different windows apart, and does nothing without a context", async () => {
		const c = new RequestCounter();
		c.record("push", 200, 1, START);
		c.record("push", 200, 1, START + WINDOW_MS);
		expect(c.size).toBe(2);
		await flush(testEnv(), c.take(START));
		expect((await rows()).map((r) => r.window_start)).toEqual([START, START + WINDOW_MS]);
		observe(testEnv(), undefined, "push", 200, 1, START + 5 * FLUSH_EVERY_MS);
		expect(counter.size).toBe(1);
	});

	it("records the router's name, never the path: no token, ID or address reaches D1", async () => {
		const waits: Promise<unknown>[] = [];
		const ctx = { waitUntil: (p: Promise<unknown>) => void waits.push(p) } as unknown as ExecutionContext;
		const secret = "SECRETSHARETOKEN";
		for (const path of [`/p/${secret}`, `/j/${secret}`, `/v2/public/${secret}`, `/nowhere/${secret}`]) {
			await worker.fetch(new Request(`${ORIGIN}${path}`, { headers: { "cf-connecting-ip": "198.51.100.77" } }), testEnv(), ctx);
		}
		await Promise.all(waits);
		await flush(testEnv(), counter.take(Date.now()));
		const stored = JSON.stringify(await rows());
		for (const leak of [secret, "198.51.100.77", "/p/", "/j/", "nowhere"]) expect(stored).not.toContain(leak);
		const groups = new Set((await rows()).map((r) => r.route_group));
		expect([...groups].every((g) => (GROUPS as readonly string[]).includes(g as string))).toBe(true);
		expect(groups.has("other")).toBe(true);
	});
});

describe("the cron's server_stats event", () => {
	const at = START + 2 * WINDOW_MS;
	const posthog = () => testEnv({ POSTHOG_KEY: "phc_test_key", POSTHOG_HOST: HOST });

	async function seed() {
		const alice = await signUp("alice");
		await signUp("bob");
		const c = new RequestCounter();
		for (let i = 0; i < 8; i++) c.record("push", 200, 20, START + 5);
		c.record("changes", 200, 400, START + 5);
		c.record("login", 401, 80, START + 5);
		c.record("viewer", 200, 30, START + 5);
		c.record("viewer", 200, 30, START + 6);
		c.record("invite", 200, 30, START + 5);
		c.record("push", 503, 1500, START + 5);
		// Another window is not part of this event.
		c.record("push", 200, 20, START + WINDOW_MS + 5);
		await flush(testEnv(), c.take(START));
		return alice;
	}

	it("sends one event with aggregates only, then deletes the window and everything older", async () => {
		const alice = await seed();
		await runCron(posthog(), at);
		expect(sent).toHaveLength(1);
		expect(sent[0].url).toBe(`${HOST}/batch/`);
		expect(sent[0].body.batch).toHaveLength(1);
		const [event] = sent[0].body.batch;
		expect(event).toMatchObject({ event: "server_stats", distinct_id: "kanso-sync-server" });
		// Stamped where its window ended.
		expect(event.timestamp).toBe(new Date(START + WINDOW_MS).toISOString());
		const p = event.properties;
		expect(p).toMatchObject({
			window_minutes: 15,
			requests_total: 14,
			requests_2xx: 12,
			requests_4xx: 1,
			requests_5xx: 1,
			errors_5xx: 1,
			req_sync_2xx: 9,
			req_sync_5xx: 1,
			req_auth_4xx: 1,
			latency_p50: "<50ms",
			latency_p95: "1000ms+",
			new_accounts: 2,
			share_views: 1,
			publish_views: 2,
			$process_person_profile: false,
			$geoip_disable: true,
			$lib: "kanso-worker",
		});
		expect(p).not.toHaveProperty("country");
		// The window doesn't end on the hour, so it doesn't report devices.
		expect(p).not.toHaveProperty("devices_seen");
		const wire = JSON.stringify(sent[0].body);
		for (const leak of [alice.token, alice.device.id, alice.creds.username, "198.51"]) expect(wire).not.toContain(leak);
		expect(wire).not.toMatch(/[ad]_[0-9a-f]{32}/);
		// The later window stays for its own turn.
		expect((await rows()).map((r) => r.window_start)).toEqual([START + WINDOW_MS]);
	});

	it("sends nothing without a key, but still clears the window", async () => {
		await seed();
		await runCron(testEnv(), at);
		expect(sent).toEqual([]);
		expect((await rows()).map((r) => r.window_start)).toEqual([START + WINDOW_MS]);
	});

	it("reports a quiet window as zeros", async () => {
		await runCron(posthog(), at);
		expect(sent[0].body.batch[0].properties).toMatchObject({ requests_total: 0, errors_5xx: 0, latency_p50: "none", latency_p95: "none", new_accounts: 0 });
	});

	it("catches up: every unsent window goes oldest first, capped, and none is dropped", async () => {
		const c = new RequestCounter();
		for (let i = 0; i < 12; i++) c.record("push", 200, 20, START + i * WINDOW_MS + 5);
		await flush(testEnv(), c.take(START));
		const later = START + 14 * WINDOW_MS;
		await runCron(posthog(), later);
		const stamps = sent.map((s) => s.body.batch[0].timestamp);
		expect(stamps).toHaveLength(MAX_WINDOWS_PER_RUN);
		expect(stamps).toEqual([...stamps].sort());
		expect(stamps[0]).toBe(new Date(START + WINDOW_MS).toISOString());
		const left = (await rows()).map((r) => r.window_start);
		expect(left).toEqual(Array.from({ length: 12 - (MAX_WINDOWS_PER_RUN - 1) }, (_, i) => START + (MAX_WINDOWS_PER_RUN - 1 + i) * WINDOW_MS));
		// The next run sends the rest: nothing is lost between them.
		sent.length = 0;
		await runCron(posthog(), later + WINDOW_MS);
		expect(sent.length).toBeGreaterThan(0);
		expect((await rows()).map((r) => r.window_start)).toEqual([START + 11 * WINDOW_MS].filter((w) => w > windowOf(later + WINDOW_MS) - 2 * WINDOW_MS));
	});

	it("keeps a window PostHog refused, and sends it on the next run", async () => {
		await seed();
		vi.spyOn(globalThis, "fetch").mockImplementation(async (input) => (String(input).includes("/flags/") ? Response.json({ flags: {} }) : new Response("no", { status: 503 })));
		await runCron(posthog(), at);
		expect([...new Set((await rows()).map((r) => r.window_start))]).toEqual([START, START + WINDOW_MS]);
		vi.restoreAllMocks();
		vi.spyOn(globalThis, "fetch").mockImplementation(async (input, init) => {
			if (String(input).includes("/flags/")) return Response.json({ flags: {} });
			sent.push({ url: String(input), body: JSON.parse(String(init?.body)) });
			return Response.json({ status: "Ok" });
		});
		await runCron(posthog(), at + WINDOW_MS);
		expect(sent.map((s) => s.body.batch[0].timestamp)).toEqual([new Date(START + WINDOW_MS).toISOString(), new Date(START + 2 * WINDOW_MS).toISOString()]);
		expect(await rows()).toEqual([]);
	});

	it("drops a window that stayed unsent for a day", async () => {
		await seed();
		await runCron(posthog(), START + 2 * 24 * 60 * 60_000);
		expect(sent.map((s) => s.body.batch[0].timestamp)).toEqual([new Date(START + 2 * 24 * 60 * 60_000 - 2 * WINDOW_MS + WINDOW_MS).toISOString()]);
		expect(await rows()).toEqual([]);
	});

	it("reports devices_seen over the hour, only on the window that ends on it", async () => {
		await signUp("alice");
		await signUp("bob");
		const hourEnd = START + 3_600_000;
		// The newest finished window is the one ending on the hour.
		await runCron(posthog(), hourEnd + 2 * WINDOW_MS - WINDOW_MS);
		expect(sent).toHaveLength(1);
		expect(sent[0].body.batch[0].properties.devices_seen).toBe(2);
		sent.length = 0;
		await runCron(posthog(), hourEnd + 2 * WINDOW_MS);
		expect(sent[0].body.batch[0].properties).not.toHaveProperty("devices_seen");
	});

	it("the window it reads is a whole one that ended a window ago", () => {
		expect(windowOf(at)).toBe(at);
		expect(START % WINDOW_MS).toBe(0);
	});
});
