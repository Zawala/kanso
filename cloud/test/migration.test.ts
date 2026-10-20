// Migration 0002_accounts.sql (docs/ACCOUNTS.md › Storage): it applies through
// applyD1Migrations on top of a v1 database, leaves the legacy rows alone, seeds
// sign-ups closed, and its triggers fail closed. The step-1 minimum; the Worker
// builder extends it.

import { applyD1Migrations } from "cloudflare:test";
import { env } from "cloudflare:workers";
import fc from "fast-check";
import { describe, expect, it } from "vitest";
import { SEEDED_SETTINGS } from "./helpers";

const db = env.MIGRATION_DB;
const v1 = () => env.TEST_MIGRATIONS.filter((m) => m.name.startsWith("0001"));

/** Drops every table and view, d1_migrations included, so 0001 can apply again. */
async function emptyDatabase(): Promise<void> {
	const { results } = await db
		.prepare("SELECT type, name FROM sqlite_master WHERE type IN ('table', 'view') AND name NOT LIKE 'sqlite_%' AND name NOT LIKE '_cf_%'")
		.all<{ type: string; name: string }>();
	for (const { type, name } of results.filter((r) => r.type === "view")) await db.exec(`DROP VIEW "${name}"`);
	// Children before parents, so foreign keys never point at a dropped table.
	const order = ["sessions", "account_docs", "account_files", "shares"];
	const tables = results.filter((r) => r.type === "table").map((r) => r.name);
	tables.sort((a, b) => Number(order.includes(b)) - Number(order.includes(a)));
	for (const name of tables) await db.exec(`DROP TABLE "${name}"`);
}

async function legacyState(): Promise<unknown> {
	const docs = await db.prepare("SELECT id, rev, seq, deleted, fields, updated_at FROM docs ORDER BY id").all();
	const counter = await db.prepare("SELECT name, value FROM counter ORDER BY name").all();
	return { docs: docs.results, counter: counter.results };
}

async function refusal(sql: string, ...params: unknown[]): Promise<string> {
	try {
		await db.prepare(sql).bind(...params).run();
	} catch (e) {
		return String((e as Error).message);
	}
	return "accepted";
}

const genLegacyDoc = fc.record({
	id: fc.stringMatching(/^(note|book|tag|file):[A-Za-z0-9]{1,10}$/),
	rev: fc.stringMatching(/^[1-9]-[0-9a-f]{32}$/),
	deleted: fc.boolean(),
	fields: fc.json(),
	updated_at: fc.option(fc.integer({ min: 0, max: 2 ** 41 }), { nil: null }),
});

describe("0002_accounts.sql", () => {
	it("applies after 0001 and seeds sign-ups closed", async () => {
		await emptyDatabase();
		await applyD1Migrations(db, env.TEST_MIGRATIONS);
		const settings = await db.prepare("SELECT name, value FROM settings").all<{ name: string; value: string }>();
		expect(Object.fromEntries(settings.results.map((r) => [r.name, r.value]))).toMatchObject({ signup: "closed", max_accounts: "50" });
		expect(Object.fromEntries(settings.results.map((r) => [r.name, r.value]))).toEqual(SEEDED_SETTINGS);
		const daily = await db.prepare("SELECT name, day, count FROM daily ORDER BY name").all();
		expect(daily.results).toEqual([
			{ name: "signups", day: 0, count: 0 },
			{ name: "upload_bytes", day: 0, count: 0 },
			{ name: "writes", day: 0, count: 0 },
		]);
	});

	it("fails closed: sign-ups refused while closed, settings rows can't be deleted, a missing setting refuses", async () => {
		await emptyDatabase();
		await applyD1Migrations(db, env.TEST_MIGRATIONS);
		const account = [
			`INSERT INTO accounts (id, username, kdf_m_kib, kdf_t, kdf_p, salt, auth_hash, wrapped_key, recovery_hash,
			   recovery_wrapped_key, key_id, epoch, created_at)
			 VALUES (?1, ?2, 1048576, 3, 4, ?3, ?4, ?5, ?4, ?5, '0123456789abcdef', '0123456789abcdef', 1800057600000)`,
			`a_${"0".repeat(32)}`,
			"alice",
			"A".repeat(22),
			"0".repeat(64),
			"A".repeat(98),
		] as const;
		expect(await refusal(...account)).toContain("signup_closed:closed");
		expect(await refusal("DELETE FROM settings WHERE name = 'max_accounts'")).toContain("settings_required");
		expect(await refusal("DELETE FROM daily WHERE name = 'signups'")).toContain("daily_required");
		await db.exec("UPDATE settings SET value = 'open' WHERE name = 'signup'");
		// Drop the guard to show what a lost row would do: max_accounts reads as 0.
		await db.exec("DROP TRIGGER settings_keep");
		await db.exec("DELETE FROM settings WHERE name = 'max_accounts'");
		expect(await refusal(...account)).toContain("signup_closed:full");
	});

	// Invariant: whatever a v1 database holds, applying 0002 leaves its docs and
	// counter rows byte for byte as they were.
	it("prop_0002_leaves_legacy_rows_untouched", async () => {
		await fc.assert(
			fc.asyncProperty(
				fc.uniqueArray(genLegacyDoc, { selector: (d) => d.id, maxLength: 12 }),
				fc.nat({ max: 10_000 }),
				async (docs, extra) => {
					await emptyDatabase();
					await applyD1Migrations(db, v1());
					const insert = db.prepare("INSERT INTO docs (id, rev, seq, deleted, fields, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)");
					const statements = docs.map((d, i) => insert.bind(d.id, d.rev, i + 1, d.deleted ? 1 : 0, JSON.stringify(d.fields), d.updated_at));
					statements.push(db.prepare("UPDATE counter SET value = ?1 WHERE name = 'seq'").bind(docs.length + extra));
					await db.batch(statements);
					const before = await legacyState();
					await applyD1Migrations(db, env.TEST_MIGRATIONS);
					expect(await legacyState()).toEqual(before);
				},
			),
			{ numRuns: 25 },
		);
	});
});
