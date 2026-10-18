declare namespace Cloudflare {
	interface Env {
		DB: D1Database;
		MIGRATION_DB: D1Database;
		FILES: R2Bucket;
		AUTH_PEPPER?: string;
		AUTH_LIMITER?: RateLimit;
		SESSION_LIMITER?: RateLimit;
		TEST_MIGRATIONS: import("cloudflare:test").D1Migration[];
		FAST_CHECK_SEED: string;
		FAST_CHECK_RUNS: string;
	}
	interface GlobalProps {
		mainModule: typeof import("../src/index");
	}
}
