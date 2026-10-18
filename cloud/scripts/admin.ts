// Administers the sync Worker's accounts: `npm run -s admin -- [--local] <command>`
// (commands in USAGE, src/accounts.ts). The commands are planned in
// src/accounts.ts; this runner gives them the outside world through
// wrangler: `d1 execute` for SQL, `d1 export`, `r2 object get`, `secret
// delete` and `d1 time-travel restore`. It works on the deployed database
// unless --local is given, and wrangler then needs CLOUDFLARE_API_TOKEN (see
// README.md); with --local it only touches Miniflare's state under .wrangler/
// (or --persist-to DIR).

import { execFileSync, spawnSync } from "node:child_process";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { type AdminIO, D1_DATABASE, type StatementResult, USAGE, UsageError, admin, restoreArgs } from "../src/accounts.ts";

const DATABASE = D1_DATABASE;
const BUCKET = "kanso-files";
const CLOUD = fileURLToPath(new URL("..", import.meta.url));
const WRANGLER = fileURLToPath(new URL("../node_modules/wrangler/bin/wrangler.js", import.meta.url));

class WranglerError extends Error {
	output: string;
	constructor(message: string, output: string) {
		super(message);
		this.output = output;
	}
}

function wrangler(args: string[]): string {
	try {
		return execFileSync(process.execPath, [WRANGLER, ...args], {
			cwd: CLOUD,
			encoding: "utf8",
			stdio: ["ignore", "pipe", "pipe"],
			maxBuffer: 256 * 1024 * 1024,
		});
	} catch (err) {
		const { stdout, stderr, message } = err as { stdout?: string; stderr?: string; message: string };
		throw new WranglerError(`wrangler ${args.slice(0, 3).join(" ")} failed`, [stdout, stderr].filter(Boolean).join("\n") || message);
	}
}

interface WranglerResult extends StatementResult {
	success: boolean;
}

function io(local: boolean, persistTo: string | undefined): AdminIO {
	const target = local ? ["--local", ...(persistTo ? ["--persist-to", persistTo] : [])] : ["--remote"];
	return {
		local,
		async sql(statements) {
			const output = wrangler(["d1", "execute", DATABASE, ...target, "--json", "--command", statements.join(";\n")]);
			const results = JSON.parse(output) as WranglerResult[];
			if (!Array.isArray(results) || results.length !== statements.length || results.some((r) => !r.success)) {
				throw new WranglerError("unexpected answer from wrangler d1 execute", output.slice(0, 4000));
			}
			return results;
		},
		async exportTables(tables, file) {
			// `d1 export` has no --persist-to: locally it only reads the default state directory,
			// which would export a different database than the one the other steps read.
			if (persistTo) throw new WranglerError("wrangler d1 export can't read --persist-to state", "Run legacy-export with --local alone.");
			wrangler(["d1", "export", DATABASE, local ? "--local" : "--remote", "--output", file, ...tables.flatMap((t) => ["--table", t])]);
		},
		async r2Get(key, file) {
			try {
				wrangler(["r2", "object", "get", `${BUCKET}/${key}`, ...target, "--file", file]);
				return true;
			} catch (err) {
				if (err instanceof WranglerError && /does not exist|not found|NoSuchKey/i.test(err.output)) return false;
				throw err;
			}
		},
		async makeDir(dir) {
			await mkdir(dir, { recursive: true });
		},
		async writeFile(file, content) {
			// The export holds the old plaintext: readable by the owner only, and never overwritten.
			await writeFile(file, content, { mode: 0o600, flag: "wx" });
		},
		async readFile(file) {
			return readFile(file, "utf8");
		},
		async deleteSecret(name) {
			const result = spawnSync(process.execPath, [WRANGLER, "secret", "delete", name], { cwd: CLOUD, stdio: "inherit" });
			if (result.status !== 0) throw new WranglerError(`wrangler secret delete ${name} failed`, "");
		},
		async timeTravelRestore(target) {
			// Time Travel exists only on the deployed database; the planner refuses --local first.
			if (local) throw new WranglerError("wrangler d1 time-travel has no local database", "");
			const output = wrangler(restoreArgs(target));
			let answer: { bookmark?: unknown; previous_bookmark?: unknown } = {};
			try {
				answer = JSON.parse(output);
			} catch {
				// Reported below.
			}
			if (typeof answer.bookmark !== "string" || typeof answer.previous_bookmark !== "string") {
				throw new WranglerError("unexpected answer from wrangler d1 time-travel restore", output.slice(0, 4000));
			}
			return { bookmark: answer.bookmark, previousBookmark: answer.previous_bookmark };
		},
	};
}

async function main(argv: string[]): Promise<number> {
	const args: string[] = [];
	let local = false;
	let persistTo: string | undefined;
	for (let i = 0; i < argv.length; i++) {
		if (argv[i] === "--local") local = true;
		else if (argv[i] === "--persist-to") persistTo = argv[++i];
		else args.push(argv[i]);
	}
	if (args.length === 0 || args[0] === "help" || args[0] === "--help") {
		console.error(USAGE);
		return args.length === 0 ? 2 : 0;
	}
	if (persistTo !== undefined && (!local || persistTo === "")) {
		console.error("--persist-to DIR goes with --local.");
		return 2;
	}
	try {
		const report = await admin(args, io(local, persistTo));
		console.log(report.message);
		return report.ok ? 0 : 1;
	} catch (err) {
		if (err instanceof UsageError) {
			console.error(err.message);
			return 2;
		}
		if (err instanceof WranglerError) {
			console.error(`${err.message}:\n${err.output}`);
			return 1;
		}
		throw err;
	}
}

process.exitCode = await main(process.argv.slice(2));
