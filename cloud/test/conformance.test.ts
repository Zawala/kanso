// The v2 sync cases shared with MemoryServer (crates/kanso-sync/tests/conformance.rs):
// push, the feed, files, quotas, the daily caps and isolation. The v1 cases
// (conformance-v1.json) only check the Rust rollback test's stand-in for the
// old Worker (crates/kanso-sync/tests/rollback.rs); this Worker serves no /v1.

import { describe, it } from "vitest";
import fixture from "./fixtures/conformance.json";
import { type Fixture, runCase } from "./runner";

describe("conformance.json (v2 sync)", () => {
	for (const testCase of (fixture as unknown as Fixture).cases) {
		it(testCase.name, () => runCase(fixture as unknown as Fixture, testCase.steps));
	}
});
