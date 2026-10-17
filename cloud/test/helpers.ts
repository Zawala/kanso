import { env, exports } from "cloudflare:workers";

export const TOKEN = "test-token";
const ORIGIN = "https://kanso-sync.test";

export interface Call {
	method?: string;
	body?: BodyInit | null;
	headers?: Record<string, string>;
	/** `null` sends no Authorization header. */
	token?: string | null;
}

export function call(path: string, init: Call = {}): Promise<Response> {
	const headers = new Headers(init.headers);
	const token = init.token === undefined ? TOKEN : init.token;
	if (token !== null) headers.set("authorization", `Bearer ${token}`);
	return exports.default.fetch(
		new Request(`${ORIGIN}${path}`, { method: init.method ?? "GET", body: init.body ?? null, headers }),
	);
}

export async function push(docs: unknown): Promise<{ status: number; body: any }> {
	const response = await call("/v1/push", {
		method: "POST",
		body: JSON.stringify({ docs }),
		headers: { "content-type": "application/json" },
	});
	return { status: response.status, body: await response.json() };
}

export async function changes(since: number, limit: number): Promise<{ status: number; body: any }> {
	const response = await call(`/v1/changes?since=${since}&limit=${limit}`);
	return { status: response.status, body: await response.json() };
}

export async function putFile(id: string, md5: string, bytes: Uint8Array): Promise<Response> {
	return call(`/v1/files/${encodeURIComponent(id)}`, {
		method: "PUT",
		body: bytes,
		headers: { "x-kanso-md5": md5, "content-type": "application/octet-stream" },
	});
}

export async function getFile(id: string): Promise<{ status: number; md5: string | null; bytes: Uint8Array }> {
	const response = await call(`/v1/files/${encodeURIComponent(id)}`);
	return {
		status: response.status,
		md5: response.headers.get("x-kanso-md5"),
		bytes: new Uint8Array(await response.arrayBuffer()),
	};
}

export async function md5Hex(bytes: Uint8Array): Promise<string> {
	const digest = await crypto.subtle.digest("MD5", bytes);
	return [...new Uint8Array(digest)].map((b) => b.toString(16).padStart(2, "0")).join("");
}

/** Empties the database and the bucket (storage is shared within a test file). */
export async function resetServer(): Promise<void> {
	await env.DB.batch([
		env.DB.prepare("DELETE FROM docs"),
		env.DB.prepare("UPDATE counter SET value = 0 WHERE name = 'seq'"),
	]);
	for (;;) {
		const listed = await env.FILES.list({ limit: 1000 });
		if (listed.objects.length === 0) break;
		await env.FILES.delete(listed.objects.map((o) => o.key));
	}
}

export function base64ToBytes(value: string): Uint8Array {
	return Uint8Array.from(atob(value), (c) => c.charCodeAt(0));
}
