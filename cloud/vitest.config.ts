import { fileURLToPath } from "node:url";
import { cloudflareTest, readD1Migrations } from "@cloudflare/vitest-plugin";
import { defineConfig } from "vitest/config";

// Property tests run with a pinned seed; override with FAST_CHECK_SEED to explore.
const seed = process.env.FAST_CHECK_SEED ?? "20261017";
const runs = process.env.FAST_CHECK_RUNS ?? "100";

export default defineConfig({
	plugins: [
		cloudflareTest(async () => ({
			wrangler: { configPath: "./wrangler.jsonc" },
			miniflare: {
				// A second, empty database the migration tests build up step by step.
				d1Databases: ["MIGRATION_DB"],
				bindings: {
					SYNC_TOKEN: "test-token",
					FAST_CHECK_SEED: seed,
					FAST_CHECK_RUNS: runs,
					TEST_MIGRATIONS: await readD1Migrations(fileURLToPath(new URL("./migrations", import.meta.url))),
				},
			},
		})),
	],
	test: {
		setupFiles: ["./test/setup.ts"],
		testTimeout: 180_000,
	},
});
