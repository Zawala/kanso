// The v1 protocol cases shared with MemoryRemote (crates/kanso-sync/tests/conformance.rs).

import { beforeEach, describe, expect, it } from "vitest";
import fixture from "./fixtures/conformance-v1.json";
import { base64ToBytes, call, changes, getFile, push, putFile, resetServer } from "./helpers";

type Step = Record<string, any>;

async function runStep(step: Step): Promise<void> {
	const status = step.status ?? 200;
	if ("push" in step) {
		const response = await push(step.push);
		expect(response.status).toBe(status);
		if (status === 200) expect(response.body).toEqual({ results: step.results });
	} else if ("push_body" in step) {
		const response = await call("/v1/push", { method: "POST", body: step.push_body });
		expect(response.status).toBe(status);
	} else if ("changes" in step) {
		const response = await changes(step.changes.since, step.changes.limit);
		expect(response.status).toBe(status);
		if (status === 200) expect(response.body).toEqual(step.page);
	} else if ("changes_query" in step) {
		const response = await call(`/v1/changes?${step.changes_query}`);
		expect(response.status).toBe(status);
	} else if ("put_file" in step) {
		const { id, md5, base64 } = step.put_file;
		const response = await putFile(id, md5, base64ToBytes(base64));
		expect(response.status).toBe(status);
	} else if ("get_file" in step) {
		const file = await getFile(step.get_file);
		expect(file.status).toBe(status);
		if (status === 200) {
			expect(file.md5).toBe(step.md5);
			expect([...file.bytes]).toEqual([...base64ToBytes(step.base64)]);
		}
	} else {
		throw new Error(`unknown step ${JSON.stringify(step)}`);
	}
}

describe("shared protocol cases", () => {
	beforeEach(resetServer);

	for (const testCase of fixture.cases) {
		it(testCase.name, async () => {
			for (const step of testCase.steps as Step[]) await runStep(step);
		});
	}
});
