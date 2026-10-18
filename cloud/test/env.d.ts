declare namespace Cloudflare {
	interface Env {
		DB: D1Database;
		MIGRATION_DB: D1Database;
		FILES: R2Bucket;
		SYNC_TOKEN?: string;
		TEST_MIGRATIONS: import("cloudflare:test").D1Migration[];
		FAST_CHECK_SEED: string;
		FAST_CHECK_RUNS: string;
	}
	interface GlobalProps {
		mainModule: typeof import("../src/index");
	}
}
