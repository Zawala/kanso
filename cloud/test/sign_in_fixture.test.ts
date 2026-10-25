// The shared sign-in fixture (test/fixtures/sign_in.json), run against the
// Worker. MemoryServer runs the same file (crates/kanso-sync/tests/conformance.rs);
// the step language is in the file's `about`.

import { describe, it } from "vitest";
import fixture from "./fixtures/sign_in.json";
import { type Fixture, runCase } from "./runner";

describe("sign_in.json", () => {
	const f = fixture as unknown as Fixture;
	for (const testCase of f.cases) {
		it(testCase.name, () => runCase(f, testCase.steps, testCase.env));
	}
});
