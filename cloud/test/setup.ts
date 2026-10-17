import { applyD1Migrations } from "cloudflare:test";
import { env } from "cloudflare:workers";
import fc from "fast-check";

await applyD1Migrations(env.DB, env.TEST_MIGRATIONS);

fc.configureGlobal({ seed: Number(env.FAST_CHECK_SEED), numRuns: Number(env.FAST_CHECK_RUNS) });
