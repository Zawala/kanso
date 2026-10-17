// Example tests: auth, limits, routing and headers.

import { env } from "cloudflare:workers";
import { beforeEach, describe, expect, it } from "vitest";
import worker from "../src/index";
import { MAX_CHANGES_LIMIT, MAX_FIELDS_BYTES, MAX_FILE_BYTES, MAX_PUSH_DOCS } from "../src/protocol";
import { call, getFile, md5Hex, push, putFile, resetServer } from "./helpers";

beforeEach(resetServer);

describe("auth", () => {
	it("health needs no token", async () => {
		const response = await call("/v1/health", { token: null });
		expect(response.status).toBe(200);
		expect(await response.json()).toEqual({ ok: true });
	});

	it("every other route needs the bearer token", async () => {
		for (const [path, method] of [
			["/v1/changes", "GET"],
			["/v1/push", "POST"],
			["/v1/files/file%3Aa", "GET"],
			["/v1/files/file%3Aa", "PUT"],
			["/v1/nope", "GET"],
		]) {
			const missing = await call(path, { method, token: null });
			expect(missing.status, `${method} ${path}`).toBe(401);
			expect(missing.headers.get("www-authenticate")).toContain("Bearer");
			const wrong = await call(path, { method, token: "test-tokem" });
			expect(wrong.status, `${method} ${path}`).toBe(401);
		}
		const basic = await call("/v1/changes", { token: null, headers: { authorization: "Basic dGVzdC10b2tlbg==" } });
		expect(basic.status).toBe(401);
		expect((await call("/v1/changes")).status).toBe(200);
	});

	it("fails closed without a configured secret", async () => {
		const request = new Request("https://x.test/v1/changes", { headers: { authorization: "Bearer " } });
		const response = await worker.fetch(request, { ...env, SYNC_TOKEN: undefined });
		expect(response.status).toBe(503);
		const empty = await worker.fetch(request, { ...env, SYNC_TOKEN: "" });
		expect(empty.status).toBe(503);
	});
});

describe("routing", () => {
	it("answers unknown paths and wrong methods", async () => {
		expect((await call("/")).status).toBe(404);
		expect((await call("/v1/unknown")).status).toBe(404);
		expect((await call("/v1/push")).status).toBe(405);
		expect((await call("/v1/changes", { method: "POST" })).status).toBe(405);
		expect((await call("/v1/files/file%3Aa", { method: "DELETE" })).status).toBe(405);
		expect((await call("/v1/health", { method: "POST" })).status).toBe(405);
	});

	it("accepts file IDs with or without an encoded colon", async () => {
		const bytes = new TextEncoder().encode("png");
		const md5 = await md5Hex(bytes);
		const response = await call("/v1/files/file:raw1", {
			method: "PUT",
			body: bytes,
			headers: { "x-kanso-md5": md5 },
		});
		expect(response.status).toBe(200);
		expect((await getFile("file:raw1")).md5).toBe(md5);
		expect((await call("/v1/files/%E0%A4%A")).status).toBe(400);
		expect((await call("/v1/files/note%3Aa")).status).toBe(400);
	});
});

describe("limits", () => {
	it("refuses more documents than one push may carry", async () => {
		const docs = Array.from({ length: MAX_PUSH_DOCS + 1 }, (_, i) => ({ id: `note:n${i}`, rev: "1-aa" }));
		expect((await push(docs)).status).toBe(413);
		const full = await push(docs.slice(0, MAX_PUSH_DOCS));
		expect(full.status).toBe(200);
		expect(full.body.results.map((r: any) => r.seq)).toEqual(docs.slice(0, MAX_PUSH_DOCS).map((_, i) => i + 1));
	});

	it("refuses a document whose fields D1 could not store", async () => {
		const big = "x".repeat(MAX_FIELDS_BYTES);
		expect((await push([{ id: "note:big", rev: "1-aa", fields: { body: big } }])).status).toBe(413);
		const fits = "x".repeat(MAX_FIELDS_BYTES - 20);
		const ok = await push([{ id: "note:big", rev: "1-aa", fields: { body: fits } }]);
		expect(ok.status).toBe(200);
		const page = await call("/v1/changes?since=0&limit=1");
		expect((await page.json<any>()).docs[0].fields.body.length).toBe(fits.length);
	});

	it("refuses a file larger than the store allows", async () => {
		const bytes = new Uint8Array(MAX_FILE_BYTES + 1);
		const response = await putFile("file:huge", await md5Hex(bytes), bytes);
		expect(response.status).toBe(413);
		expect((await getFile("file:huge")).status).toBe(404);
		const max = new Uint8Array(MAX_FILE_BYTES);
		max[7] = 7;
		expect((await putFile("file:max", await md5Hex(max), max)).status).toBe(200);
		expect((await getFile("file:max")).bytes.length).toBe(MAX_FILE_BYTES);
	});

	it("caps the change feed page", async () => {
		expect((await call(`/v1/changes?limit=${MAX_CHANGES_LIMIT}`)).status).toBe(200);
		expect((await call(`/v1/changes?limit=${MAX_CHANGES_LIMIT + 1}`)).status).toBe(400);
	});
});

describe("files", () => {
	it("returns the stored content type and MD5", async () => {
		const bytes = new Uint8Array([0x89, 0x50, 0x4e, 0x47]);
		const md5 = await md5Hex(bytes);
		const response = await call("/v1/files/file%3Aimg", {
			method: "PUT",
			body: bytes,
			headers: { "x-kanso-md5": md5.toUpperCase(), "content-type": "image/png" },
		});
		expect(response.status).toBe(200);
		expect(await response.json()).toEqual({ ok: true, md5 });
		const got = await call("/v1/files/file%3Aimg");
		expect(got.headers.get("content-type")).toBe("image/png");
		expect(got.headers.get("x-kanso-md5")).toBe(md5);
		expect(got.headers.get("content-length")).toBe("4");
		expect([...new Uint8Array(await got.arrayBuffer())]).toEqual([...bytes]);
	});

	it("requires x-kanso-md5", async () => {
		const response = await call("/v1/files/file%3Ax", { method: "PUT", body: new Uint8Array([1]) });
		expect(response.status).toBe(400);
	});
});
