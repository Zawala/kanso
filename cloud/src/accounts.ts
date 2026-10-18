// The admin commands (docs/ACCOUNTS.md › Admin script), planned here and run
// by scripts/admin.ts through `wrangler d1 execute` (plus `wrangler d1
// export`, `wrangler r2 object get` and `wrangler secret delete` where a
// command needs them). There is no admin HTTP route.
//
// `wrangler d1 execute` takes SQL text but no bound parameters, so every
// value is validated before it reaches SQL and quoted when it does. Commands
// talk to the outside world only through `AdminIO`, which the tests implement
// over the test database. Node runs this file as it is (type stripping), so
// it imports nothing but ./usernames.ts and uses only erasable syntax.

import { normalizeUsername } from "./usernames.ts";

export const USAGE = `Usage: npm run -s admin -- [--local [--persist-to DIR]] <command>

  accounts                          Every account: usage against quotas, writes today, devices,
                                    last seen and status; the settings, global sums and purges.
  settings                          The settings.
  disable <username>                Refuse the account's sign-ins and sessions.
  enable <username>                 Undo disable.
  delete <username> --confirm <username>
                                    Delete the account's rows and queue its R2 purge.
  signups open | close | only <username>
                                    Open sign-ups, close them, or reserve them for one name.
  max-accounts <n> [--force]        At most n accounts.
  quota <username> [--docs N] [--doc-mib N] [--files N] [--file-mib N] [--force]
  quota <username> --default        Per-account quotas, or back to the defaults.
  default-quota [--docs N] [--doc-mib N] [--files N] [--file-mib N] [--force]
                                    The default quotas.
  writes-per-day <n>                Documents pushed plus files uploaded, per account per UTC day.
  global-writes-per-day <n>         Stored document writes per UTC day, all accounts together.
  global-upload-bytes-per-day <n>   Uploaded file bytes per UTC day, all accounts together.
  recount [<username>]              Recompute usage counters from the tables.
  restored [<username> | --all]     After a Time Travel restore: a new epoch, so Macs re-merge.
  legacy-status                     The single-user tables and the files/ purge.
  legacy-export --dir DIR           manifest.json, legacy.sql and every live legacy file.
  purge-legacy --account <username> --verified R.json
                                    Drop the single-user tables, queue the files/ purge and
                                    delete SYNC_TOKEN, once R.json proves everything arrived.

Runs against the deployed database unless --local is given. max-accounts, quota and
default-quota refuse a change that would overcommit a global cap unless --force is given.`;

export class UsageError extends Error {}

/** One statement's rows, as `wrangler d1 execute --json` and D1's batch() both report them. */
export interface StatementResult {
	results: Record<string, unknown>[];
}

/** Everything a command does outside this file. */
export interface AdminIO {
	/** Whether the commands run against the local database (wrangler's --local). */
	local: boolean;
	/** Runs the statements in order, in one transaction. */
	sql(statements: string[]): Promise<StatementResult[]>;
	/** `wrangler d1 export` of these tables (schema and rows) into `file`. */
	exportTables(tables: string[], file: string): Promise<void>;
	/** Copies an R2 object of the files bucket into `file`; false when there is no such object. */
	r2Get(key: string, file: string): Promise<boolean>;
	makeDir(dir: string): Promise<void>;
	writeFile(file: string, content: string): Promise<void>;
	readFile(file: string): Promise<string>;
	/** `wrangler secret delete <name>`, with stdin attached so wrangler can ask. */
	deleteSecret(name: string): Promise<void>;
}

export interface Report {
	ok: boolean;
	/** For the terminal; never holds a key or a token. */
	message: string;
}

// -------------------------------------------------------------------- values

/** A SQL string literal. Callers validate the value first; this only doubles quotes. */
export function sqlText(value: string): string {
	return `'${value.replaceAll("'", "''")}'`;
}

const MIB = 1_048_576;
const DAY = 86_400_000;
const ROW_COST = 512;
const FILE_COST = 1024;
/** Settings hold at most 15 digits (the migration's CHECK). */
const MAX_SETTING = 999_999_999_999_999;

function username(value: string | undefined): string {
	const normalized = value === undefined ? null : normalizeUsername(value);
	if (normalized === null) {
		throw new UsageError(`"${value ?? ""}" is not a username: 3–32 letters, digits, - or _, starting and ending with a letter or digit.`);
	}
	return normalized;
}

function positive(value: string | undefined, what: string, scale = 1): number {
	if (value === undefined || !/^[0-9]{1,15}$/.test(value) || Number(value) === 0 || Number(value) * scale > MAX_SETTING) {
		throw new UsageError(`${what} must be a whole number from 1 to ${Math.floor(MAX_SETTING / scale)}.`);
	}
	return Number(value) * scale;
}

/** Positional arguments and flags; a flag in `valued` takes the next argument. */
function parse(args: string[], booleans: string[], valued: string[]): { positional: string[]; flags: Map<string, string | true> } {
	const positional: string[] = [];
	const flags = new Map<string, string | true>();
	for (let i = 0; i < args.length; i++) {
		const arg = args[i];
		if (arg.startsWith("--")) {
			if (flags.has(arg)) throw new UsageError(`${arg} is given twice.`);
			if (booleans.includes(arg)) flags.set(arg, true);
			else if (valued.includes(arg)) {
				const value = args[++i];
				if (value === undefined || value.startsWith("--")) throw new UsageError(`${arg} needs a value.`);
				flags.set(arg, value);
			} else throw new UsageError(`Unknown option ${arg}.\n\n${USAGE}`);
		} else {
			positional.push(arg);
		}
	}
	return { positional, flags };
}

function arity(positional: string[], min: number, max: number): void {
	if (positional.length < min || positional.length > max) throw new UsageError(USAGE);
}

const flag = (flags: Map<string, string | true>, name: string): string | undefined => {
	const value = flags.get(name);
	return typeof value === "string" ? value : undefined;
};

// ------------------------------------------------------------------- reports

const date = (ms: unknown): string => (typeof ms === "number" ? new Date(ms).toISOString().slice(0, 10) : "");
const time = (ms: unknown): string => (typeof ms === "number" ? new Date(ms).toISOString().slice(0, 16).replace("T", " ") : "never");

function size(bytes: unknown): string {
	const n = Number(bytes);
	if (n >= 10 * MIB) return `${Math.round(n / MIB)} MiB`;
	if (n >= 1024) return `${(n / 1024).toFixed(1)} KiB`;
	return `${n} B`;
}

export function table(rows: string[][]): string {
	const widths = rows[0].map((_, c) => Math.max(...rows.map((r) => [...r[c]].length)));
	return rows.map((r) => r.map((cell, c) => cell.padEnd(widths[c] + (cell.length - [...cell].length))).join("  ").trimEnd()).join("\n");
}

const ok = (message: string): Report => ({ ok: true, message });
const refused = (message: string): Report => ({ ok: false, message });

// ----------------------------------------------------------------- capacity

export interface Capacity {
	maxAccounts: number;
	defaultDocBytes: number;
	defaultFileBytes: number;
	globalDocBytes: number;
	globalFileBytes: number;
	accounts: { username: string; quotaDocBytes: number | null; quotaFileBytes: number | null }[];
}

/**
 * What the quotas promise: Σ effective quotas of the existing accounts plus
 * (max_accounts − accounts) × the default, for document and file bytes.
 */
export function committed(c: Capacity): { docBytes: number; fileBytes: number } {
	const open = Math.max(0, c.maxAccounts - c.accounts.length);
	const sum = (pick: (a: Capacity["accounts"][number]) => number) => c.accounts.reduce((total, a) => total + pick(a), 0);
	return {
		docBytes: sum((a) => a.quotaDocBytes ?? c.defaultDocBytes) + open * c.defaultDocBytes,
		fileBytes: sum((a) => a.quotaFileBytes ?? c.defaultFileBytes) + open * c.defaultFileBytes,
	};
}

/** The caps a capacity overcommits, in words; empty when it fits. */
export function overcommitted(c: Capacity): string[] {
	const { docBytes, fileBytes } = committed(c);
	const problems: string[] = [];
	if (docBytes > c.globalDocBytes) problems.push(`document quotas would promise ${docBytes} bytes, over global_doc_bytes ${c.globalDocBytes}`);
	if (fileBytes > c.globalFileBytes) problems.push(`file quotas would promise ${fileBytes} bytes, over global_file_bytes ${c.globalFileBytes}`);
	return problems;
}

const CAPACITY_SQL = [
	"SELECT name, value FROM settings",
	"SELECT username, quota_doc_bytes, quota_file_bytes FROM accounts ORDER BY username",
];

async function capacity(io: AdminIO): Promise<{ capacity: Capacity; settings: Map<string, string> }> {
	const [settingsRows, accountRows] = await io.sql(CAPACITY_SQL);
	const settings = new Map(settingsRows.results.map((r) => [String(r.name), String(r.value)]));
	// A missing setting reads as 0, as the triggers read it.
	const n = (name: string) => Number(settings.get(name) ?? 0);
	return {
		settings,
		capacity: {
			maxAccounts: n("max_accounts"),
			defaultDocBytes: n("quota_doc_bytes"),
			defaultFileBytes: n("quota_file_bytes"),
			globalDocBytes: n("global_doc_bytes"),
			globalFileBytes: n("global_file_bytes"),
			accounts: accountRows.results.map((r) => ({
				username: String(r.username),
				quotaDocBytes: r.quota_doc_bytes === null ? null : Number(r.quota_doc_bytes),
				quotaFileBytes: r.quota_file_bytes === null ? null : Number(r.quota_file_bytes),
			})),
		},
	};
}

/** Refuses (unless forced) a change after which the quotas would promise more than a global cap. */
function overcommitRefusal(after: Capacity, force: boolean): Report | null {
	const problems = overcommitted(after);
	if (problems.length === 0 || force) return null;
	return refused(`Refused: ${problems.join("; ")}. Raise the global cap first, or repeat with --force.`);
}

// ------------------------------------------------------------ legacy rows

/** One manifest row: [id, rev, deleted, md5digest]. */
export type ManifestRow = [string, string, boolean, string | null];

/** The legacy rows as `legacy-export` writes them and `purge-legacy` re-reads them. */
export const LEGACY_ROWS_SQL = `SELECT id, rev, deleted,
  CASE WHEN id GLOB 'file:*' AND json_valid(fields) AND json_type(fields, '$.md5digest') = 'text'
       THEN json_extract(fields, '$.md5digest') END AS md5digest
  FROM docs ORDER BY id`;

const toManifestRows = (result: StatementResult): ManifestRow[] =>
	result.results.map((r) => [String(r.id), String(r.rev), Number(r.deleted) !== 0, r.md5digest === null ? null : String(r.md5digest)]);

const encoder = new TextEncoder();

function compareUtf8(a: string, b: string): number {
	const x = encoder.encode(a);
	const y = encoder.encode(b);
	for (let i = 0; i < Math.min(x.length, y.length); i++) if (x[i] !== y[i]) return x[i] - y[i];
	return x.length - y.length;
}

/**
 * The manifest hash (docs/ACCOUNTS.md › Command line): rows sorted by id
 * (bytewise UTF-8), each `id TAB rev TAB 1|0 TAB md5digest-or-nothing LF`,
 * SHA-256 as 64 lowercase hex digits. `exported_at` is not covered.
 */
export async function manifestSha256(rows: ManifestRow[]): Promise<string> {
	const sorted = [...rows].sort((a, b) => compareUtf8(a[0], b[0]));
	const text = sorted.map(([id, rev, deleted, md5]) => `${id}\t${rev}\t${deleted ? "1" : "0"}\t${md5 ?? ""}\n`).join("");
	const digest = new Uint8Array(await crypto.subtle.digest("SHA-256", encoder.encode(text)));
	return [...digest].map((b) => b.toString(16).padStart(2, "0")).join("");
}

async function legacyTables(io: AdminIO): Promise<string[]> {
	const [tables] = await io.sql(["SELECT name FROM sqlite_master WHERE type = 'table' AND name IN ('docs', 'counter') ORDER BY name"]);
	return tables.results.map((r) => String(r.name));
}

const join = (dir: string, name: string): string => `${dir.replace(/\/+$/, "")}/${name}`;

// ----------------------------------------------------------------- commands

const SETTING_COMMANDS: Record<string, string> = {
	"writes-per-day": "writes_per_day",
	"global-writes-per-day": "global_writes_per_day",
	"global-upload-bytes-per-day": "global_upload_bytes_per_day",
};

const QUOTA_FLAGS = ["--docs", "--doc-mib", "--files", "--file-mib"];

interface QuotaChange {
	docs?: number;
	docBytes?: number;
	files?: number;
	fileBytes?: number;
}

function quotaChange(flags: Map<string, string | true>): QuotaChange {
	const change: QuotaChange = {};
	if (flags.has("--docs")) change.docs = positive(flag(flags, "--docs"), "--docs");
	if (flags.has("--doc-mib")) change.docBytes = positive(flag(flags, "--doc-mib"), "--doc-mib", MIB);
	if (flags.has("--files")) change.files = positive(flag(flags, "--files"), "--files");
	if (flags.has("--file-mib")) change.fileBytes = positive(flag(flags, "--file-mib"), "--file-mib", MIB);
	return change;
}

/**
 * Sets an account's quota overrides in raw units (null: back to the
 * default), refusing to overcommit a global cap unless forced. The fixtures'
 * `{admin: "quota"}` steps run this.
 */
export async function setQuota(
	io: AdminIO,
	name: string,
	change: { docs?: number | null; docBytes?: number | null; files?: number | null; fileBytes?: number | null },
	force: boolean,
): Promise<Report> {
	const columns: [keyof typeof change, string][] = [
		["docs", "quota_docs"],
		["docBytes", "quota_doc_bytes"],
		["files", "quota_files"],
		["fileBytes", "quota_file_bytes"],
	];
	const sets = columns
		.filter(([key]) => change[key] !== undefined)
		.map(([key, column]) => {
			const value = change[key];
			if (value !== null && (!Number.isSafeInteger(value) || (value as number) <= 0)) throw new UsageError(`${column} must be positive.`);
			return `${column} = ${value === null ? "NULL" : String(value)}`;
		});
	if (sets.length === 0) throw new UsageError("Name at least one of --docs, --doc-mib, --files, --file-mib, or --default.");
	const { capacity: before } = await capacity(io);
	const target = before.accounts.find((a) => a.username === name);
	if (!target) return refused(`No account is named ${name}.`);
	const after: Capacity = {
		...before,
		accounts: before.accounts.map((a) =>
			a.username !== name
				? a
				: {
						...a,
						quotaDocBytes: change.docBytes !== undefined ? change.docBytes : a.quotaDocBytes,
						quotaFileBytes: change.fileBytes !== undefined ? change.fileBytes : a.quotaFileBytes,
					},
		),
	};
	const refusal = overcommitRefusal(after, force);
	if (refusal) return refusal;
	const [updated] = await io.sql([`UPDATE accounts SET ${sets.join(", ")} WHERE username = ${sqlText(name)} RETURNING id`]);
	return updated.results.length === 1 ? ok(`Set ${sets.join(", ")} for ${name}.`) : refused(`No account is named ${name}.`);
}

/** Runs one admin command. Throws `UsageError` for anything malformed, before any SQL runs. */
export async function admin(args: string[], io: AdminIO, now: number = Date.now()): Promise<Report> {
	const [name, ...rest] = args;
	switch (name) {
		case "accounts":
			return accounts(rest, io, now);
		case "settings": {
			arity(parse(rest, [], []).positional, 0, 0);
			const [rows] = await io.sql(["SELECT name, value FROM settings ORDER BY name"]);
			return ok(table([["SETTING", "VALUE"], ...rows.results.map((r) => [String(r.name), String(r.value)])]));
		}
		case "disable":
		case "enable": {
			const { positional } = parse(rest, [], []);
			arity(positional, 1, 1);
			const user = username(positional[0]);
			const disable = name === "disable";
			const [result] = await io.sql([
				disable
					? `UPDATE accounts SET disabled_at = ${now} WHERE username = ${sqlText(user)} AND disabled_at IS NULL RETURNING id`
					: `UPDATE accounts SET disabled_at = NULL WHERE username = ${sqlText(user)} AND disabled_at IS NOT NULL RETURNING id`,
			]);
			return result.results.length === 1
				? ok(`${disable ? "Disabled" : "Enabled"} ${user}.`)
				: refused(`No ${disable ? "enabled" : "disabled"} account is named ${user}.`);
		}
		case "delete": {
			const { positional, flags } = parse(rest, [], ["--confirm"]);
			arity(positional, 1, 1);
			const user = username(positional[0]);
			const confirm = flag(flags, "--confirm");
			if (confirm === undefined || normalizeUsername(confirm) !== user) {
				throw new UsageError(`Repeat the username to confirm: delete ${user} --confirm ${user}`);
			}
			const id = `(SELECT id FROM accounts WHERE username = ${sqlText(user)})`;
			const results = await io.sql([
				`DELETE FROM account_docs WHERE account_id = ${id}`,
				`DELETE FROM account_files WHERE account_id = ${id}`,
				`DELETE FROM sessions WHERE account_id = ${id}`,
				`DELETE FROM limits WHERE key IN ('login-user:' || ${id}, 'password-account:' || ${id}, 'writes-account:' || ${id})`,
				`INSERT INTO purges (prefix, requested_at) SELECT 'accounts/' || id || '/', ${now} FROM accounts WHERE username = ${sqlText(user)}
  ON CONFLICT (prefix) DO UPDATE SET requested_at = excluded.requested_at`,
				`DELETE FROM accounts WHERE username = ${sqlText(user)} RETURNING id`,
			]);
			const deleted = results[5].results[0];
			return deleted
				? ok(`Deleted ${user} (${String(deleted.id)}). The cron empties accounts/${String(deleted.id)}/ in R2 within 15 minutes.`)
				: refused(`No account is named ${user}.`);
		}
		case "signups": {
			const { positional } = parse(rest, [], []);
			const mode = positional[0];
			if (mode === "open" || mode === "close" || mode === "closed") {
				arity(positional, 1, 1);
				const value = mode === "open" ? "open" : "closed";
				await io.sql([`UPDATE settings SET value = ${sqlText(value)} WHERE name = 'signup'`]);
				return ok(`Sign-ups are ${value}.`);
			}
			if (mode === "only") {
				arity(positional, 2, 2);
				const user = username(positional[1]);
				const [, existing] = await io.sql([
					`UPDATE settings SET value = ${sqlText(`only:${user}`)} WHERE name = 'signup'`,
					`SELECT id FROM accounts WHERE username = ${sqlText(user)}`,
				]);
				return existing.results.length > 0
					? ok(`Sign-ups are reserved for ${user}, but ${user} already has an account: the reservation is spent. Run \`signups close\` or \`signups open\`.`)
					: ok(`Sign-ups are reserved for ${user} until that account exists.`);
			}
			throw new UsageError(USAGE);
		}
		case "max-accounts": {
			const { positional, flags } = parse(rest, ["--force"], []);
			arity(positional, 1, 1);
			const n = positive(positional[0], "max-accounts");
			const { capacity: before } = await capacity(io);
			const refusal = overcommitRefusal({ ...before, maxAccounts: n }, flags.has("--force"));
			if (refusal) return refusal;
			await io.sql([`UPDATE settings SET value = ${sqlText(String(n))} WHERE name = 'max_accounts'`]);
			return ok(`max_accounts = ${n}.`);
		}
		case "quota": {
			const { positional, flags } = parse(rest, ["--default", "--force"], QUOTA_FLAGS);
			arity(positional, 1, 1);
			const user = username(positional[0]);
			if (flags.has("--default")) {
				if (QUOTA_FLAGS.some((f) => flags.has(f))) throw new UsageError("--default goes alone.");
				return setQuota(io, user, { docs: null, docBytes: null, files: null, fileBytes: null }, flags.has("--force"));
			}
			return setQuota(io, user, quotaChange(flags), flags.has("--force"));
		}
		case "default-quota": {
			const { positional, flags } = parse(rest, ["--force"], QUOTA_FLAGS);
			arity(positional, 0, 0);
			const change = quotaChange(flags);
			const settings: [string, number | undefined][] = [
				["quota_docs", change.docs],
				["quota_doc_bytes", change.docBytes],
				["quota_files", change.files],
				["quota_file_bytes", change.fileBytes],
			];
			const chosen = settings.filter((s): s is [string, number] => s[1] !== undefined);
			if (chosen.length === 0) throw new UsageError("Name at least one of --docs, --doc-mib, --files, --file-mib.");
			const { capacity: before } = await capacity(io);
			const after = {
				...before,
				defaultDocBytes: change.docBytes ?? before.defaultDocBytes,
				defaultFileBytes: change.fileBytes ?? before.defaultFileBytes,
			};
			const refusal = overcommitRefusal(after, flags.has("--force"));
			if (refusal) return refusal;
			await io.sql(chosen.map(([setting, value]) => `UPDATE settings SET value = ${sqlText(String(value))} WHERE name = ${sqlText(setting)}`));
			return ok(`Defaults: ${chosen.map(([setting, value]) => `${setting} = ${value}`).join(", ")}.`);
		}
		case "writes-per-day":
		case "global-writes-per-day":
		case "global-upload-bytes-per-day": {
			const { positional } = parse(rest, [], []);
			arity(positional, 1, 1);
			const setting = SETTING_COMMANDS[name];
			const n = positive(positional[0], name);
			await io.sql([`UPDATE settings SET value = ${sqlText(String(n))} WHERE name = ${sqlText(setting)}`]);
			return ok(`${setting} = ${n}.`);
		}
		case "recount":
			return recount(rest, io, now);
		case "restored": {
			const { positional, flags } = parse(rest, ["--all"], []);
			arity(positional, 0, 1);
			const all = flags.has("--all");
			if (all === (positional.length === 1)) throw new UsageError("Name one account, or give --all.");
			const where = all ? "" : ` WHERE username = ${sqlText(username(positional[0]))}`;
			const [result] = await io.sql([`UPDATE accounts SET epoch = lower(hex(randomblob(8)))${where} RETURNING username`]);
			if (result.results.length === 0) return refused(all ? "There are no accounts." : `No account is named ${positional[0]}.`);
			return ok(`New epoch for ${result.results.map((r) => String(r.username)).join(", ")}: each Mac re-merges on its next sync.`);
		}
		case "legacy-status":
			arity(parse(rest, [], []).positional, 0, 0);
			return legacyStatus(io);
		case "legacy-export": {
			const { positional, flags } = parse(rest, [], ["--dir"]);
			arity(positional, 0, 0);
			const dir = flag(flags, "--dir");
			if (!dir) throw new UsageError("legacy-export needs --dir DIR.");
			return legacyExport(io, dir, now);
		}
		case "purge-legacy": {
			const { positional, flags } = parse(rest, [], ["--account", "--verified"]);
			arity(positional, 0, 0);
			const user = username(flag(flags, "--account"));
			const verified = flag(flags, "--verified");
			if (!verified) throw new UsageError("purge-legacy needs --verified R.json (from kanso-sync verify --legacy).");
			return purgeLegacy(io, user, verified, now);
		}
		default:
			throw new UsageError(USAGE);
	}
}

async function accounts(rest: string[], io: AdminIO, now: number): Promise<Report> {
	arity(parse(rest, [], []).positional, 0, 0);
	const today = Math.floor(now / DAY) * DAY;
	const [rows, settings, sums, daily, purges] = await io.sql([
		`SELECT a.username, a.id, a.created_at, a.disabled_at, a.docs, a.doc_bytes, a.files, a.file_bytes,
  coalesce(a.quota_docs, d.docs, 0) AS quota_docs, coalesce(a.quota_doc_bytes, d.doc_bytes, 0) AS quota_doc_bytes,
  coalesce(a.quota_files, d.files, 0) AS quota_files, coalesce(a.quota_file_bytes, d.file_bytes, 0) AS quota_file_bytes,
  coalesce((SELECT CASE WHEN window_start = ${today} THEN count ELSE 0 END FROM limits WHERE key = 'writes-account:' || a.id), 0) AS writes_today,
  (SELECT count(*) FROM sessions s WHERE s.account_id = a.id AND s.revoked_at IS NULL) AS devices,
  (SELECT max(last_seen_at) FROM sessions s WHERE s.account_id = a.id) AS last_seen
  FROM accounts a, defaults d ORDER BY a.created_at, a.username`,
		"SELECT name, value FROM settings ORDER BY name",
		"SELECT count(*) AS accounts, coalesce(sum(doc_bytes), 0) AS doc_bytes, coalesce(sum(file_bytes), 0) AS file_bytes FROM accounts",
		"SELECT name, day, count FROM daily ORDER BY name",
		"SELECT prefix, requested_at FROM purges ORDER BY requested_at, prefix",
	]);
	const accountRows = rows.results.map((a) => [
		String(a.username),
		String(a.id),
		date(a.created_at),
		`${a.docs}/${a.quota_docs}`,
		`${size(a.doc_bytes)}/${size(a.quota_doc_bytes)}`,
		`${a.files}/${a.quota_files}`,
		`${size(a.file_bytes)}/${size(a.quota_file_bytes)}`,
		String(a.writes_today),
		String(a.devices),
		time(a.last_seen),
		a.disabled_at === null ? "active" : `disabled ${date(a.disabled_at)}`,
	]);
	const s = sums.results[0] ?? {};
	const setting = new Map(settings.results.map((r) => [String(r.name), String(r.value)]));
	const day = Math.floor(now / DAY);
	const dailyText = daily.results.map((r) => `${r.name} ${Number(r.day) >= day ? r.count : 0}`).join(", ");
	return ok(
		[
			accountRows.length > 0
				? table([["USERNAME", "ID", "CREATED", "DOCS", "DOC BYTES", "FILES", "FILE BYTES", "WRITES TODAY", "DEVICES", "LAST SEEN", "STATUS"], ...accountRows])
				: "No accounts.",
			"",
			table([["SETTING", "VALUE"], ...settings.results.map((r) => [String(r.name), String(r.value)])]),
			"",
			`Accounts: ${s.accounts} of ${setting.get("max_accounts")}. Documents: ${size(s.doc_bytes)} of ${size(Number(setting.get("global_doc_bytes")))}. Files: ${size(s.file_bytes)} of ${size(Number(setting.get("global_file_bytes")))}.`,
			`Today (UTC): ${dailyText}.`,
			purges.results.length > 0
				? `Pending purges: ${purges.results.map((p) => `${p.prefix} (since ${time(p.requested_at)})`).join(", ")}.`
				: "No pending purges.",
		].join("\n"),
	);
}

async function recount(rest: string[], io: AdminIO, now: number): Promise<Report> {
	const { positional } = parse(rest, [], []);
	arity(positional, 0, 1);
	const where = positional.length === 1 ? ` WHERE username = ${sqlText(username(positional[0]))}` : "";
	const columns = "username, docs, doc_bytes, files, file_bytes";
	// A recount sets recounted_at, which the quota triggers exempt: a
	// correction must never be refused. It always changes, even within a millisecond.
	const [before, after] = await io.sql([
		`SELECT ${columns} FROM accounts${where} ORDER BY username`,
		`UPDATE accounts SET
  docs = (SELECT count(*) FROM account_docs d WHERE d.account_id = accounts.id AND d.deleted = 0),
  doc_bytes = (SELECT coalesce(sum(length(d.sealed) + ${ROW_COST}), 0) FROM account_docs d WHERE d.account_id = accounts.id),
  files = (SELECT count(*) FROM account_files f WHERE f.account_id = accounts.id),
  file_bytes = (SELECT coalesce(sum(f.size + ${FILE_COST}), 0) FROM account_files f WHERE f.account_id = accounts.id),
  recounted_at = max(${now}, coalesce(recounted_at, 0) + 1)${where}
  RETURNING ${columns}`,
	]);
	if (after.results.length === 0) return refused(positional.length === 1 ? `No account is named ${positional[0]}.` : "There are no accounts.");
	const old = new Map(before.results.map((r) => [String(r.username), r]));
	const changed = after.results.filter((r) => {
		const was = old.get(String(r.username));
		return !was || ["docs", "doc_bytes", "files", "file_bytes"].some((k) => Number(was[k]) !== Number(r[k]));
	});
	const lines = changed.map((r) => {
		const was = old.get(String(r.username)) ?? {};
		return `${r.username}: docs ${was.docs} → ${r.docs}, doc_bytes ${was.doc_bytes} → ${r.doc_bytes}, files ${was.files} → ${r.files}, file_bytes ${was.file_bytes} → ${r.file_bytes}`;
	});
	return ok(
		`Recounted ${after.results.length} account${after.results.length === 1 ? "" : "s"}; ${changed.length} changed.${lines.length > 0 ? `\n${lines.join("\n")}` : ""}`,
	);
}

async function legacyStatus(io: AdminIO): Promise<Report> {
	const tables = await legacyTables(io);
	const [purge] = await io.sql(["SELECT requested_at FROM purges WHERE prefix = 'files/'"]);
	const purgeText =
		purge.results.length > 0
			? `The files/ purge is pending (queued ${time(purge.results[0].requested_at)}).`
			: "No files/ purge is pending.";
	if (tables.length === 0) return ok(`The single-user tables are gone. ${purgeText}`);
	const statements: string[] = [];
	if (tables.includes("docs")) {
		statements.push(`SELECT count(*) AS rows, coalesce(sum(deleted = 0), 0) AS live,
  coalesce(sum(deleted = 0 AND id GLOB 'file:*'), 0) AS live_files,
  coalesce(sum(length(id) + length(rev) + length(fields) + 64), 0) AS bytes FROM docs`);
	}
	if (tables.includes("counter")) statements.push("SELECT value FROM counter WHERE name = 'seq'");
	const results = await io.sql(statements);
	const lines = [`Single-user tables: ${tables.join(", ")}.`];
	if (tables.includes("docs")) {
		const d = results[0].results[0];
		lines.push(`docs: ${d.rows} rows, ${d.live} live documents (${d.live_files} files), about ${size(d.bytes)}.`);
	}
	if (tables.includes("counter")) lines.push(`counter: seq ${results[results.length - 1].results[0]?.value ?? "missing"}.`);
	lines.push(purgeText);
	return ok(lines.join("\n"));
}

async function legacyExport(io: AdminIO, dir: string, now: number): Promise<Report> {
	const tables = await legacyTables(io);
	if (!tables.includes("docs") || !tables.includes("counter")) return refused("The single-user tables are gone: nothing to export.");
	const [result] = await io.sql([LEGACY_ROWS_SQL]);
	const rows = toManifestRows(result);
	await io.makeDir(join(dir, "files"));
	// The SQL dump first: if wrangler can't make it, no manifest suggests the export is complete.
	await io.exportTables(["docs", "counter"], join(dir, "legacy.sql"));
	await io.writeFile(join(dir, "manifest.json"), `${JSON.stringify({ exported_at: now, rows }, null, 1)}\n`);
	const files = rows.filter(([id, , deleted]) => id.startsWith("file:") && !deleted);
	const missing: string[] = [];
	for (const [id] of files) {
		if (!(await io.r2Get(`files/${id}`, join(dir, `files/${encodeURIComponent(id)}`)))) missing.push(id);
	}
	const hash = await manifestSha256(rows);
	return {
		ok: missing.length === 0,
		message: [
			`Exported ${rows.length} legacy rows (${rows.filter((r) => !r[2]).length} live) to ${join(dir, "manifest.json")} and ${join(dir, "legacy.sql")}.`,
			`Saved ${files.length - missing.length} of ${files.length} live files in ${join(dir, "files")}.`,
			missing.length > 0 ? `Missing from R2: ${missing.join(", ")}.` : "",
			`Manifest SHA-256: ${hash}`,
		]
			.filter(Boolean)
			.join("\n"),
	};
}

async function purgeLegacy(io: AdminIO, user: string, verifiedFile: string, now: number): Promise<Report> {
	let verified: Record<string, unknown>;
	try {
		verified = JSON.parse(await io.readFile(verifiedFile));
	} catch (err) {
		return refused(`Refused: ${verifiedFile} isn't a readable JSON report (${(err as Error).message}).`);
	}
	if (typeof verified !== "object" || verified === null || Array.isArray(verified)) return refused(`Refused: ${verifiedFile} isn't a report.`);
	const { manifest_sha256, account_id } = verified;
	if (typeof manifest_sha256 !== "string" || !/^[0-9a-f]{64}$/.test(manifest_sha256) || typeof account_id !== "string") {
		return refused(`Refused: ${verifiedFile} has no manifest_sha256 or account_id.`);
	}
	if (verified.ok !== true) return refused(`Refused: ${verifiedFile} isn't ok; run kanso-sync verify --legacy again until it is.`);
	const [account] = await io.sql([`SELECT id FROM accounts WHERE username = ${sqlText(user)}`]);
	const id = account.results[0]?.id;
	if (id === undefined) return refused(`Refused: no account is named ${user}.`);
	if (id !== account_id) return refused(`Refused: ${verifiedFile} verifies account ${account_id}, not ${user} (${String(id)}).`);
	const tables = await legacyTables(io);
	if (!tables.includes("docs") || !tables.includes("counter")) return refused("Refused: the single-user tables are already gone.");
	const [rows] = await io.sql([LEGACY_ROWS_SQL]);
	const fresh = await manifestSha256(toManifestRows(rows));
	if (fresh !== manifest_sha256) {
		return refused(`Refused: the legacy rows changed since the verified export (now ${fresh}, verified ${manifest_sha256}). Export and verify again.`);
	}
	await io.sql([
		"DROP TABLE docs",
		"DROP TABLE counter",
		`INSERT INTO purges (prefix, requested_at) VALUES ('files/', ${now}) ON CONFLICT (prefix) DO NOTHING`,
	]);
	const lines = ["Dropped docs and counter, and queued the files/ purge (the cron runs it within 15 minutes)."];
	if (io.local) {
		lines.push("Local database: no SYNC_TOKEN secret to delete.");
	} else {
		await io.deleteSecret("SYNC_TOKEN");
		lines.push("Deleted the SYNC_TOKEN secret.");
	}
	lines.push("D1 Time Travel keeps the dropped rows for 30 days; the export directory keeps them for good.");
	return ok(lines.join("\n"));
}
