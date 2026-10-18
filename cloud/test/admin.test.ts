// The admin commands (src/accounts.ts) against the local test D1 through an
// AdminIO over Miniflare: each command's effect, refusals before any SQL,
// the overcommit check, the legacy export and the purge-legacy guard with
// the manifest test vector, and b4e2733's injection properties.

import { applyD1Migrations } from "cloudflare:test";
import { env } from "cloudflare:workers";
import fc from "fast-check";
import { beforeEach, describe, expect, it } from "vitest";
import { type AdminIO, type ManifestRow, UsageError, admin, manifestSha256, sqlText } from "../src/accounts";
import { normalizeUsername } from "../src/usernames";
import {
	type Account,
	START,
	accountRow,
	callJson,
	client,
	login,
	recomputed,
	resetServer,
	runAdmin,
	sealed,
	signUp,
	testIO,
	wireId,
	wireRev,
} from "./helpers";

const runs = (n: number) => ({ numRuns: Math.min(n, Number(env.FAST_CHECK_RUNS)) });
const setting = async (name: string) => env.DB.prepare("SELECT value FROM settings WHERE name = ?1").bind(name).first<string>("value");

let alice: Account;
let bob: Account;

beforeEach(async () => {
	await resetServer();
	alice = await signUp("alice");
	bob = await signUp("bob");
});

describe("commands", () => {
	it("accounts shows usage, quotas, writes, devices and status, with the settings, sums and purges, and no secret", async () => {
		await client(alice.token).push([{ id: wireId(), rev: wireRev(), deleted: false, sealed: sealed(100), base_rev: null }]);
		await env.DB.prepare("INSERT INTO purges (prefix, requested_at) VALUES ('files/', ?1)").bind(START).run();
		const r = await runAdmin(["accounts"]);
		expect(r.ok).toBe(true);
		expect(r.message).toMatch(/^USERNAME +ID +CREATED +DOCS +DOC BYTES +FILES +FILE BYTES +WRITES TODAY +DEVICES +LAST SEEN +STATUS/);
		const line = r.message.split("\n").find((l) => l.startsWith("alice"))!;
		expect(line).toContain(alice.creds.accountId);
		expect(line).toMatch(/1\/50000 +612 B\/100 MiB +0\/10000 +0 B\/500 MiB +1 +1 +\d{4}-\d\d-\d\d \d\d:\d\d +active/);
		expect(r.message).toContain("Accounts: 2 of 50.");
		expect(r.message).toContain("Pending purges: files/");
		expect(r.message).toContain("global_doc_bytes");
		const row = await accountRow(alice.creds.accountId);
		for (const secret of [row!.auth_hash, row!.recovery_hash, row!.salt, row!.wrapped_key, alice.token]) expect(r.message).not.toContain(secret);
	});

	it("settings lists the twelve settings", async () => {
		const r = await runAdmin(["settings"]);
		expect(r.message.split("\n")).toHaveLength(13);
		expect(r.message).toContain("writes_per_day");
	});

	it("disable refuses sign-ins and sessions; enable restores them", async () => {
		expect((await runAdmin(["disable", "Alice"])).ok).toBe(true);
		expect((await runAdmin(["disable", "alice"])).ok).toBe(false);
		expect((await client(alice.token).account()).body.error).toBe("account_disabled");
		expect((await login(alice.creds, alice.device)).status).toBe(403);
		expect((await client(bob.token).account()).status).toBe(200);
		expect((await runAdmin(["enable", "alice"])).ok).toBe(true);
		expect((await client(alice.token).account()).status).toBe(200);
		expect((await runAdmin(["enable", "alice"])).ok).toBe(false);
		expect((await runAdmin(["enable", "nobody"])).message).toContain("nobody");
	});

	it("delete needs the name twice, then removes the account's rows and queues its purge", async () => {
		await client(alice.token).push([{ id: wireId(), rev: wireRev(), deleted: false, sealed: sealed(), base_rev: null }]);
		await client(alice.token).putFile(wireId("file"), new Uint8Array(64));
		await login({ ...alice.creds, authKey: bob.creds.authKey }, alice.device);
		await expect(runAdmin(["delete", "alice"])).rejects.toThrow(UsageError);
		await expect(runAdmin(["delete", "alice", "--confirm", "bob"])).rejects.toThrow(UsageError);
		const r = await runAdmin(["delete", "alice", "--confirm", "ALICE"]);
		expect(r.ok, r.message).toBe(true);
		for (const table of ["account_docs", "account_files", "sessions"]) {
			expect(await env.DB.prepare(`SELECT count(*) AS n FROM ${table} WHERE account_id = ?1`).bind(alice.creds.accountId).first("n"), table).toBe(0);
		}
		expect(await accountRow(alice.creds.accountId)).toBeNull();
		expect(await env.DB.prepare("SELECT count(*) AS n FROM limits WHERE key LIKE ?1").bind(`%${alice.creds.accountId}`).first("n")).toBe(0);
		expect((await env.DB.prepare("SELECT prefix FROM purges").all()).results).toEqual([{ prefix: `accounts/${alice.creds.accountId}/` }]);
		expect((await client(bob.token).account()).status).toBe(200);
		expect((await runAdmin(["delete", "alice", "--confirm", "alice"])).ok).toBe(false);
	});

	it("signups open, close and only <name>, saying when the reservation is already spent", async () => {
		expect((await runAdmin(["signups", "close"])).ok).toBe(true);
		expect(await setting("signup")).toBe("closed");
		expect((await callJson("/v2/health")).body.signup).toBe("closed");
		expect((await runAdmin(["signups", "only", "Carol"])).message).toContain("until that account exists");
		expect(await setting("signup")).toBe("only:carol");
		expect((await runAdmin(["signups", "only", "bob"])).message).toContain("spent");
		expect((await runAdmin(["signups", "open"])).ok).toBe(true);
		expect(await setting("signup")).toBe("open");
		await expect(runAdmin(["signups", "maybe"])).rejects.toThrow(UsageError);
		await expect(runAdmin(["signups", "only"])).rejects.toThrow(UsageError);
	});

	it("quota sets overrides in MiB and --default clears them; default-quota and the daily settings set settings", async () => {
		expect((await runAdmin(["quota", "alice", "--docs", "10", "--doc-mib", "2", "--files", "3", "--file-mib", "4"])).ok).toBe(true);
		const row = await accountRow(alice.creds.accountId);
		expect([row!.quota_docs, row!.quota_doc_bytes, row!.quota_files, row!.quota_file_bytes]).toEqual([10, 2 * 1_048_576, 3, 4 * 1_048_576]);
		expect((await client(alice.token).account()).body.quota).toMatchObject({ docs: 10, doc_bytes: 2 * 1_048_576, files: 3, file_bytes: 4 * 1_048_576 });
		expect((await runAdmin(["quota", "alice", "--default"])).ok).toBe(true);
		const cleared = await accountRow(alice.creds.accountId);
		expect([cleared!.quota_docs, cleared!.quota_doc_bytes, cleared!.quota_files, cleared!.quota_file_bytes]).toEqual([null, null, null, null]);
		expect((await runAdmin(["quota", "nobody", "--docs", "1"])).ok).toBe(false);
		expect((await runAdmin(["default-quota", "--docs", "7", "--files", "8"])).ok).toBe(true);
		expect([await setting("quota_docs"), await setting("quota_files")]).toEqual(["7", "8"]);
		for (const [command, name] of [
			["writes-per-day", "writes_per_day"],
			["global-writes-per-day", "global_writes_per_day"],
			["global-upload-bytes-per-day", "global_upload_bytes_per_day"],
		]) {
			expect((await runAdmin([command, "123"])).ok).toBe(true);
			expect(await setting(name)).toBe("123");
		}
		expect((await runAdmin(["max-accounts", "40"])).ok).toBe(true);
		expect(await setting("max_accounts")).toBe("40");
	});

	it("recount recomputes the counters from the tables and always moves recounted_at forward", async () => {
		await client(alice.token).push([{ id: wireId(), rev: wireRev(), deleted: false, sealed: sealed(100), base_rev: null }]);
		await env.DB.prepare("UPDATE accounts SET docs = 99, doc_bytes = 1, recounted_at = ?2 WHERE id = ?1").bind(alice.creds.accountId, START + 5).run();
		const r = await runAdmin(["recount", "alice"], START);
		expect(r.message).toContain("Recounted 1 account; 1 changed.");
		expect(r.message).toContain("docs 99 → 1, doc_bytes 1 → 612");
		const row = await accountRow(alice.creds.accountId);
		expect({ docs: row!.docs, doc_bytes: row!.doc_bytes }).toEqual({ docs: 1, doc_bytes: 612 });
		expect(row!.recounted_at).toBe(START + 6);
		expect((await runAdmin(["recount"], START + 100)).message).toContain("Recounted 2 accounts; 0 changed.");
		expect((await accountRow(alice.creds.accountId))!.recounted_at).toBe(START + 100);
		expect((await accountRow(bob.creds.accountId))!.recounted_at).toBe(START + 100);
		expect(await recomputed(alice.creds.accountId)).toMatchObject({ docs: 1, doc_bytes: 612 });
	});

	it("restored gives one account, or all, a new epoch that the feed then reports", async () => {
		const before = (await client(alice.token).changes()).body.epoch;
		expect((await runAdmin(["restored", "alice"])).ok).toBe(true);
		const after = (await client(alice.token).changes()).body.epoch;
		expect(after).toMatch(/^[0-9a-f]{16}$/);
		expect(after).not.toBe(before);
		expect((await client(bob.token).changes()).body.epoch).toBe(bob.epoch);
		expect((await runAdmin(["restored", "--all"])).message).toContain("alice, bob");
		expect((await client(bob.token).changes()).body.epoch).not.toBe(bob.epoch);
		await expect(runAdmin(["restored"])).rejects.toThrow(UsageError);
		await expect(runAdmin(["restored", "alice", "--all"])).rejects.toThrow(UsageError);
		expect((await runAdmin(["restored", "nobody"])).ok).toBe(false);
	});
});

describe("refusals before any SQL", () => {
	/** An AdminIO that fails the test if anything reaches it. */
	const untouchable = (): { io: AdminIO; touched: string[] } => {
		const touched: string[] = [];
		const record =
			(what: string) =>
			async (...args: unknown[]): Promise<never> => {
				touched.push(`${what} ${JSON.stringify(args)}`);
				throw new Error(`${what} reached`);
			};
		return {
			touched,
			io: {
				local: true,
				sql: record("sql"),
				exportTables: record("exportTables"),
				r2Get: record("r2Get"),
				makeDir: record("makeDir"),
				writeFile: record("writeFile"),
				readFile: record("readFile"),
				deleteSecret: record("deleteSecret"),
			},
		};
	};

	it("refuses malformed commands with a UsageError, touching nothing", async () => {
		const cases: string[][] = [
			[],
			["nope"],
			["accounts", "extra"],
			["settings", "x"],
			["disable"],
			["disable", "o'brien"],
			["disable", "a b"],
			["disable", "-alice"],
			["disable", "x".repeat(33)],
			["enable", "alice", "bob"],
			["delete", "alice", "--confirm"],
			["delete", "alice", "--confirm", "alice", "--confirm", "alice"],
			["signups"],
			["signups", "only", "al'ice"],
			["max-accounts"],
			["max-accounts", "0"],
			["max-accounts", "-1"],
			["max-accounts", "1.5"],
			["max-accounts", "1e3"],
			["max-accounts", "10", "--forse"],
			["quota", "alice"],
			["quota", "alice", "--docs"],
			["quota", "alice", "--docs", "0"],
			["quota", "alice", "--doc-mib", "x"],
			["quota", "alice", "--default", "--docs", "1"],
			["quota", "alice", "--doc-mib", "999999999999"],
			["default-quota"],
			["writes-per-day", "1000000000000000"],
			["global-writes-per-day"],
			["recount", "alice", "bob"],
			["recount", "bad name"],
			["legacy-status", "x"],
			["legacy-export"],
			["legacy-export", "--dir"],
			["purge-legacy", "--account", "alice"],
			["purge-legacy", "--verified", "R.json"],
			["purge-legacy", "--account", "'; DROP TABLE accounts; --", "--verified", "R.json"],
		];
		for (const args of cases) {
			const { io, touched } = untouchable();
			await expect(admin(args, io, START), JSON.stringify(args)).rejects.toThrow(UsageError);
			expect(touched, JSON.stringify(args)).toEqual([]);
		}
	});
});

describe("legacy commands", () => {
	/** A fresh database with the v1 tables beside the account tables, as the deployed one has. */
	async function legacyDb(rows: { id: string; rev: string; deleted: boolean; md5?: string }[]): Promise<D1Database> {
		const db = env.MIGRATION_DB;
		const { results } = await db
			.prepare("SELECT type, name FROM sqlite_master WHERE type IN ('table', 'view') AND name NOT LIKE 'sqlite_%' AND name NOT LIKE '_cf_%'")
			.all<{ type: string; name: string }>();
		for (const { name } of results.filter((r) => r.type === "view")) await db.exec(`DROP VIEW "${name}"`);
		const order = ["sessions", "account_docs", "account_files"];
		const tables = results.filter((r) => r.type === "table").map((r) => r.name);
		tables.sort((x, y) => Number(order.includes(y)) - Number(order.includes(x)));
		for (const name of tables) await db.exec(`DROP TABLE "${name}"`);
		await applyD1Migrations(db, env.TEST_MIGRATIONS);
		await db.batch([
			db.prepare("UPDATE settings SET value = 'open' WHERE name = 'signup'"),
			db
				.prepare(
					`INSERT INTO accounts (id, username, kdf_m_kib, kdf_t, kdf_p, salt, auth_hash, wrapped_key, recovery_hash,
  recovery_wrapped_key, key_id, epoch, created_at) VALUES (?1, 'owner', 1048576, 3, 4, ?2, ?3, ?4, ?3, ?4, ?5, ?5, 0)`,
				)
				.bind(OWNER, "A".repeat(22), "0".repeat(64), "A".repeat(98), "0".repeat(16)),
			...rows.map((r, i) =>
				db
					.prepare("INSERT INTO docs (id, rev, seq, deleted, fields, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, 0)")
					.bind(r.id, r.rev, i + 1, r.deleted ? 1 : 0, JSON.stringify(r.md5 ? { md5digest: r.md5, title: "x" } : { title: "x" })),
			),
			db.prepare("UPDATE counter SET value = ?1 WHERE name = 'seq'").bind(rows.length),
		]);
		return db;
	}

	const OWNER = `a_${"0f".repeat(16)}`;
	const ROWS = [
		{ id: "note:b", rev: "2-bb", deleted: false },
		{ id: "file:a", rev: "1-aa", deleted: false, md5: "d41d8cd98f00b204e9800998ecf8427e" },
		{ id: "tag:c", rev: "3-cc", deleted: true },
		{ id: "file:gone", rev: "4-dd", deleted: true, md5: "00000000000000000000000000000000" },
		{ id: "file:lost", rev: "5-ee", deleted: false, md5: "11111111111111111111111111111111" },
	];

	it("the manifest hash matches the spec's test vector, ignores row order, and covers every field", async () => {
		const vector: ManifestRow[] = [
			["note:b", "2-bb", false, null],
			["file:a", "1-aa", false, "d41d8cd98f00b204e9800998ecf8427e"],
			["tag:c", "3-cc", true, null],
		];
		expect(await manifestSha256(vector)).toBe("a0dbdfeedc42d0d264500b4faad3718aa823528946c653f34e62b2be42575b92");
		expect(await manifestSha256([])).toBe("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
		await fc.assert(
			fc.asyncProperty(fc.shuffledSubarray(vector, { minLength: 3, maxLength: 3 }), fc.integer({ min: 0, max: 2 }), fc.integer({ min: 0, max: 3 }), async (shuffled, row, field) => {
				expect(await manifestSha256(shuffled)).toBe(await manifestSha256(vector));
				const changed = vector.map((r) => [...r] as ManifestRow);
				const target = changed[row];
				if (field === 0) target[0] = `${target[0]}x`;
				else if (field === 1) target[1] = `${target[1]}x`;
				else if (field === 2) target[2] = !target[2];
				else target[3] = target[3] === null ? "" : null;
				const original = await manifestSha256(vector);
				// An md5digest of "" and of null write the same line; everything else differs.
				if (!(field === 3 && vector[row][3] === null)) expect(await manifestSha256(changed)).not.toBe(original);
			}),
			runs(40),
		);
	});

	it("legacy-status, legacy-export, then purge-legacy with a verified report", async () => {
		const db = await legacyDb(ROWS);
		await env.FILES.put("files/file:a", new Uint8Array([1, 2, 3]));
		const status = await admin(["legacy-status"], testIO({ db }).io, START);
		expect(status.message).toContain("Single-user tables: counter, docs.");
		expect(status.message).toContain("docs: 5 rows, 3 live documents (2 files)");
		expect(status.message).toContain("counter: seq 5.");

		const exporter = testIO({ db });
		const exported = await admin(["legacy-export", "--dir", "out/"], exporter.io, START);
		// file:lost has no object in R2: the export says so and isn't ok.
		expect(exported.ok).toBe(false);
		expect(exported.message).toContain("Missing from R2: file:lost.");
		const manifest = JSON.parse(exporter.files.get("out/manifest.json") as string);
		expect(manifest.exported_at).toBe(START);
		expect(manifest.rows).toEqual([
			["file:a", "1-aa", false, "d41d8cd98f00b204e9800998ecf8427e"],
			["file:gone", "4-dd", true, "00000000000000000000000000000000"],
			["file:lost", "5-ee", false, "11111111111111111111111111111111"],
			["note:b", "2-bb", false, null],
			["tag:c", "3-cc", true, null],
		]);
		expect(exporter.exports).toEqual([{ tables: ["docs", "counter"], file: "out/legacy.sql" }]);
		expect(exporter.files.get(`out/files/${encodeURIComponent("file:a")}`)).toEqual(new Uint8Array([1, 2, 3]));
		const hash = await manifestSha256(manifest.rows);
		expect(exported.message).toContain(`Manifest SHA-256: ${hash}`);
		// Exporting again into the same directory never overwrites.
		await expect(admin(["legacy-export", "--dir", "out"], exporter.io, START)).rejects.toThrow(/EEXIST/);

		const report = (r: Record<string, unknown>) => {
			const io = testIO({ db, local: false });
			io.files.set("R.json", JSON.stringify(r));
			return io;
		};
		const good = { manifest_sha256: hash, account_id: OWNER, ok: true, rows: 5, failed: [] };
		const refusals: [Record<string, unknown> | string, RegExp][] = [
			["not json", /isn't a readable JSON report/],
			[{ ...good, ok: false }, /isn't ok/],
			[{ ...good, manifest_sha256: "abc" }, /no manifest_sha256/],
			[{ ...good, account_id: `a_${"1".repeat(32)}` }, /verifies account/],
			[{ ...good, manifest_sha256: "f".repeat(64) }, /legacy rows changed/],
		];
		for (const [r, why] of refusals) {
			const io = typeof r === "string" ? testIO({ db, local: false }) : report(r);
			if (typeof r === "string") io.files.set("R.json", r);
			const refused = await admin(["purge-legacy", "--account", "owner", "--verified", "R.json"], io.io, START);
			expect(refused.ok, JSON.stringify(r)).toBe(false);
			expect(refused.message).toMatch(why);
			expect(io.secretsDeleted).toEqual([]);
		}
		expect((await admin(["purge-legacy", "--account", "nobody", "--verified", "R.json"], report(good).io, START)).message).toContain("no account is named nobody");
		// A legacy row changing after the export (nothing should write them, but if it did) refuses the purge.
		await db.prepare("UPDATE docs SET rev = '2-bc' WHERE id = 'note:b'").run();
		expect((await admin(["purge-legacy", "--account", "owner", "--verified", "R.json"], report(good).io, START)).message).toMatch(/legacy rows changed/);
		await db.prepare("UPDATE docs SET rev = '2-bb' WHERE id = 'note:b'").run();

		const remote = report(good);
		const purged = await admin(["purge-legacy", "--account", "owner", "--verified", "R.json"], remote.io, START);
		expect(purged.ok, purged.message).toBe(true);
		expect(remote.secretsDeleted).toEqual(["SYNC_TOKEN"]);
		const tables = (await db.prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name IN ('docs', 'counter')").all()).results;
		expect(tables).toEqual([]);
		expect((await db.prepare("SELECT prefix FROM purges").all()).results).toEqual([{ prefix: "files/" }]);
		expect((await admin(["legacy-status"], testIO({ db }).io, START)).message).toContain("The single-user tables are gone. The files/ purge is pending");
		expect((await admin(["purge-legacy", "--account", "owner", "--verified", "R.json"], report(good).io, START)).message).toContain("already gone");
		expect((await admin(["legacy-export", "--dir", "again"], testIO({ db }).io, START)).ok).toBe(false);
	});

	it("purge-legacy against the local database deletes no secret", async () => {
		const db = await legacyDb(ROWS.slice(0, 3));
		const hash = await manifestSha256(ROWS.slice(0, 3).map((r) => [r.id, r.rev, r.deleted, r.md5 ?? null]));
		const local = testIO({ db, local: true });
		local.files.set("R.json", JSON.stringify({ manifest_sha256: hash, account_id: OWNER, ok: true }));
		const r = await admin(["purge-legacy", "--account", "owner", "--verified", "R.json"], local.io, START);
		expect(r.ok, r.message).toBe(true);
		expect(r.message).toContain("no SYNC_TOKEN secret to delete");
		expect(local.secretsDeleted).toEqual([]);
	});
});

describe("properties", () => {
	const hostile = fc.oneof(
		fc.constantFrom("'", "''", "alice'--", "a'; DROP TABLE accounts; --", "bob\\", "carol\u0000", "x\"y", "%", "_", "*", "ALICE", " alice ", "alice\n"),
		fc.string({ unit: "binary", maxLength: 40 }),
		fc.stringMatching(/^[a-z0-9][a-z0-9_'";\\ -]{1,30}[a-z0-9]$/),
	);

	// Round trip: any string without NUL survives sqlText into SQLite and back exactly.
	it("prop_sql_text_round_trips", async () => {
		await fc.assert(
			fc.asyncProperty(fc.string({ unit: "binary", maxLength: 60 }).filter((s) => !s.includes("\u0000")), async (s) => {
				const row = await env.DB.prepare(`SELECT ${sqlText(s)} AS v`).first<{ v: string }>();
				expect(row!.v).toBe(s);
			}),
			runs(100),
		);
	});

	// Round trip: whatever name `signups only` accepts is stored exactly as its normalized form.
	it("prop_any_accepted_name_is_stored_exactly", async () => {
		await fc.assert(
			fc.asyncProperty(hostile, async (name) => {
				const normalized = normalizeUsername(name);
				const planned = runAdmin(["signups", "only", name]);
				if (normalized === null) {
					await expect(planned).rejects.toThrow(UsageError);
					return;
				}
				expect((await planned).ok).toBe(true);
				expect(await setting("signup")).toBe(`only:${normalized}`);
			}),
			runs(100),
		);
	});

	// Invariant: whatever the arguments, a command is refused up front or
	// changes nothing beyond the one account it names: the schema, the
	// settings' names and every other account and its documents stay as they were.
	it("prop_no_argument_reaches_beyond_its_command", async () => {
		const schema = async () => (await env.DB.prepare("SELECT type, name, sql FROM sqlite_master ORDER BY name").all()).results;
		const before = await schema();
		await fc.assert(
			fc.asyncProperty(
				fc.constantFrom("disable", "enable", "delete", "quota", "recount", "restored", "signups only"),
				fc.oneof(fc.constantFrom("alice", "bob", "carol"), hostile),
				fc.option(hostile, { nil: undefined }),
				async (name, first, second) => {
					await resetServer();
					await signUp("alice");
					const carol = await signUp("carol");
					await client(carol.token).push([{ id: wireId(), rev: wireRev(), deleted: false, sealed: sealed(), base_rev: null }]);
					const snapshot = async () => ({
						docs: (await env.DB.prepare("SELECT * FROM account_docs ORDER BY account_id, id").all()).results,
						carol: await accountRow(carol.creds.accountId),
						settings: (await env.DB.prepare("SELECT name FROM settings ORDER BY name").all()).results,
						accounts: (await env.DB.prepare("SELECT count(*) AS n FROM accounts").first("n")) as number,
					});
					const untouched = await snapshot();
					const args = {
						disable: ["disable", first],
						enable: ["enable", first],
						delete: ["delete", first, "--confirm", second ?? first],
						quota: ["quota", first, "--docs", second ?? "5", "--force"],
						recount: ["recount", first],
						restored: ["restored", first],
						"signups only": ["signups", "only", first],
					}[name];
					let report;
					try {
						report = await runAdmin(args);
					} catch (err) {
						expect(err).toBeInstanceOf(UsageError);
						return;
					}
					expect(normalizeUsername(first)).not.toBeNull();
					expect(await schema()).toEqual(before);
					const after = await snapshot();
					expect(after.settings).toEqual(untouched.settings);
					if (normalizeUsername(first) !== "carol") {
						expect(after.carol).toEqual(untouched.carol);
						expect(after.docs).toEqual(untouched.docs);
						expect(after.accounts).toBeLessThanOrEqual(untouched.accounts);
					}
					expect(typeof report.message).toBe("string");
				},
			),
			runs(80),
		);
	});
});
