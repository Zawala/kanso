// Kanso's sync server.
//
//   GET  /v1/health               {ok: true}, no auth
//   POST /v1/push                 {docs: [SyncDoc]} -> {results: [{id, status: "ok", seq} | {id, status: "conflict", current}]}
//   GET  /v1/changes?since&limit  {docs: [SyncDoc & {seq}], last_seq}
//   PUT  /v1/files/<id>           bytes, x-kanso-md5 -> {ok, md5}
//   GET  /v1/files/<id>           bytes with x-kanso-md5
//
// Everything but health needs `Authorization: Bearer <SYNC_TOKEN>`.
//
// D1 holds every document's current revision, tombstones included, and a
// counter that hands each accepted write the next change sequence. A pushed
// document is accepted iff its base_rev is the server's current revision (or
// the server has none and base_rev is null). R2 holds attachment bytes under
// `files/<id>` with their MD5 in custom metadata.

import {
	DEFAULT_CHANGES_LIMIT,
	MAX_CHANGES_LIMIT,
	MAX_FILE_BYTES,
	MAX_PUSH_BYTES,
	checkPush,
	intParam,
	isFileId,
	isMd5Hex,
} from "./protocol";

export interface Env {
	DB: D1Database;
	FILES: R2Bucket;
	SYNC_TOKEN?: string;
}

export default {
	async fetch(request: Request, env: Env): Promise<Response> {
		try {
			return await route(request, env);
		} catch (err) {
			console.error("kanso-sync:", err);
			return error(500, "Internal error.");
		}
	},
} satisfies ExportedHandler<Env>;

async function route(request: Request, env: Env): Promise<Response> {
	const url = new URL(request.url);
	const path = url.pathname;
	if (path === "/v1/health") {
		return request.method === "GET" || request.method === "HEAD"
			? json({ ok: true })
			: methodNotAllowed("GET");
	}
	if (!path.startsWith("/v1/")) return error(404, "Not found.");

	const denied = await authorize(request, env);
	if (denied) return denied;

	if (path === "/v1/push") {
		return request.method === "POST" ? push(request, env) : methodNotAllowed("POST");
	}
	if (path === "/v1/changes") {
		return request.method === "GET" ? changes(url, env) : methodNotAllowed("GET");
	}
	const file = /^\/v1\/files\/([^/]+)$/.exec(path);
	if (file) {
		let id: string;
		try {
			id = decodeURIComponent(file[1]);
		} catch {
			return error(400, "Not a file ID.");
		}
		if (!isFileId(id)) return error(400, "Not a file ID.");
		if (request.method === "PUT") return putFile(request, env, id);
		if (request.method === "GET") return getFile(env, id);
		return methodNotAllowed("GET, PUT");
	}
	return error(404, "Not found.");
}

// ---------------------------------------------------------------- responses

function json(value: unknown, status = 200, headers: Record<string, string> = {}): Response {
	return new Response(JSON.stringify(value), {
		status,
		headers: { "content-type": "application/json; charset=utf-8", ...headers },
	});
}

function error(status: number, message: string, headers: Record<string, string> = {}): Response {
	return json({ error: message }, status, headers);
}

function methodNotAllowed(allow: string): Response {
	return error(405, "Method not allowed.", { allow });
}

// --------------------------------------------------------------------- auth

const encoder = new TextEncoder();

/** Constant-time comparison: both sides are hashed to equal-length digests first. */
async function sameSecret(a: string, b: string): Promise<boolean> {
	const [da, db] = await Promise.all([
		crypto.subtle.digest("SHA-256", encoder.encode(a)),
		crypto.subtle.digest("SHA-256", encoder.encode(b)),
	]);
	return crypto.subtle.timingSafeEqual(da, db);
}

async function authorize(request: Request, env: Env): Promise<Response | null> {
	const secret = env.SYNC_TOKEN;
	if (!secret) return error(503, "The server has no SYNC_TOKEN configured.");
	const given = request.headers.get("authorization") ?? "";
	if (await sameSecret(given, `Bearer ${secret}`)) return null;
	return error(401, "Unauthorized.", { "www-authenticate": 'Bearer realm="kanso-sync"' });
}

// --------------------------------------------------------------- body limit

/** Reads at most `max` bytes of the body; `null` when it is longer. */
async function readBody(request: Request, max: number): Promise<Uint8Array | null> {
	const declared = request.headers.get("content-length");
	if (declared !== null && Number(declared) > max) return null;
	if (!request.body) return new Uint8Array();
	const chunks: Uint8Array[] = [];
	let total = 0;
	const reader = request.body.getReader();
	for (;;) {
		const { done, value } = await reader.read();
		if (done) break;
		total += value.byteLength;
		if (total > max) {
			await reader.cancel();
			return null;
		}
		chunks.push(value);
	}
	const out = new Uint8Array(total);
	let offset = 0;
	for (const chunk of chunks) {
		out.set(chunk, offset);
		offset += chunk.byteLength;
	}
	return out;
}

// --------------------------------------------------------------------- push

// Whether a document with (?1 = id, ?2 = base_rev) may be written now.
const ACCEPT = `((?2 IS NULL AND NOT EXISTS (SELECT 1 FROM docs WHERE id = ?1))
  OR EXISTS (SELECT 1 FROM docs WHERE id = ?1 AND rev = ?2))`;
// 1. Take the next seq, only when the document is accepted.
const BUMP = `UPDATE counter SET value = value + 1 WHERE name = 'seq' AND ${ACCEPT} RETURNING value`;
// 2. On a conflict, the server's version at that point.
const CURRENT = `SELECT id, rev, deleted, fields FROM docs WHERE id = ?1 AND NOT ${ACCEPT}`;
// 3. Write the document under the new seq, only when accepted.
const WRITE = `INSERT INTO docs (id, rev, seq, deleted, fields, updated_at)
  SELECT ?1, ?3, value, ?4, ?5, ?6 FROM counter WHERE name = 'seq' AND ${ACCEPT}
  ON CONFLICT (id) DO UPDATE SET rev = excluded.rev, seq = excluded.seq,
    deleted = excluded.deleted, fields = excluded.fields, updated_at = excluded.updated_at`;

interface DocRow {
	id: string;
	rev: string;
	seq: number;
	deleted: number;
	fields: string;
}

type PushResult =
	| { id: string; status: "ok"; seq: number }
	| { id: string; status: "conflict"; current: { id: string; rev: string; deleted: boolean; fields: unknown } | null };

async function push(request: Request, env: Env): Promise<Response> {
	const bytes = await readBody(request, MAX_PUSH_BYTES);
	if (!bytes) return error(413, "The push is too large.");
	let body: unknown;
	try {
		body = JSON.parse(new TextDecoder().decode(bytes));
	} catch {
		return error(400, "The body is not JSON.");
	}
	const checked = checkPush(body);
	if (!checked.ok) return error(checked.status, checked.error);
	const docs = checked.value;
	if (docs.length === 0) return json({ results: [] });

	// One batch is one transaction; its statements run in order, so each
	// document sees the ones before it.
	const bump = env.DB.prepare(BUMP);
	const current = env.DB.prepare(CURRENT);
	const write = env.DB.prepare(WRITE);
	const statements: D1PreparedStatement[] = [];
	for (const doc of docs) {
		statements.push(
			bump.bind(doc.id, doc.baseRev),
			current.bind(doc.id, doc.baseRev),
			write.bind(doc.id, doc.baseRev, doc.rev, doc.deleted ? 1 : 0, doc.fields, doc.updatedAt),
		);
	}
	const outcomes = await env.DB.batch(statements);

	const results: PushResult[] = docs.map((doc, i) => {
		const seq = (outcomes[3 * i].results as { value: number }[])[0]?.value;
		if (seq !== undefined) return { id: doc.id, status: "ok", seq };
		const row = (outcomes[3 * i + 1].results as DocRow[])[0];
		return {
			id: doc.id,
			status: "conflict",
			current: row
				? { id: row.id, rev: row.rev, deleted: row.deleted !== 0, fields: JSON.parse(row.fields) }
				: null,
		};
	});

	// A deleted file's bytes are no longer needed. A device whose edit beats
	// the deletion uploads them again before it pushes the file.
	const gone = docs
		.filter((doc, i) => doc.deleted && isFileId(doc.id) && results[i].status === "ok")
		.map((doc) => fileKey(doc.id));
	if (gone.length > 0) await env.FILES.delete(gone);

	return json({ results });
}

// ------------------------------------------------------------------ changes

async function changes(url: URL, env: Env): Promise<Response> {
	const since = intParam(url.searchParams.get("since"), 0);
	if (since === null || since < 0) return error(400, "since must be a non-negative integer.");
	const limit = intParam(url.searchParams.get("limit"), DEFAULT_CHANGES_LIMIT);
	if (limit === null || limit < 1 || limit > MAX_CHANGES_LIMIT) {
		return error(400, `limit must be between 1 and ${MAX_CHANGES_LIMIT}.`);
	}
	const { results } = await env.DB.prepare(
		"SELECT id, rev, seq, deleted, fields FROM docs WHERE seq > ?1 ORDER BY seq LIMIT ?2",
	)
		.bind(since, limit)
		.all<DocRow>();
	// `fields` is stored as JSON text; splice it in rather than re-encoding it.
	const docs = results.map(
		(row) =>
			`{"id":${JSON.stringify(row.id)},"rev":${JSON.stringify(row.rev)},"deleted":${row.deleted !== 0},"fields":${row.fields},"seq":${row.seq}}`,
	);
	const lastSeq = results.length > 0 ? results[results.length - 1].seq : since;
	return new Response(`{"docs":[${docs.join(",")}],"last_seq":${lastSeq}}`, {
		headers: { "content-type": "application/json; charset=utf-8" },
	});
}

// -------------------------------------------------------------------- files

const fileKey = (id: string): string => `files/${id}`;

function hex(buffer: ArrayBuffer): string {
	return [...new Uint8Array(buffer)].map((b) => b.toString(16).padStart(2, "0")).join("");
}

async function putFile(request: Request, env: Env, id: string): Promise<Response> {
	const claimed = (request.headers.get("x-kanso-md5") ?? "").toLowerCase();
	if (!isMd5Hex(claimed)) return error(400, "x-kanso-md5 must be 32 hex digits.");
	const bytes = await readBody(request, MAX_FILE_BYTES);
	if (!bytes) return error(413, `The file is larger than ${MAX_FILE_BYTES} bytes.`);
	const actual = hex(await crypto.subtle.digest("MD5", bytes));
	if (actual !== claimed) return error(400, "The bytes do not match x-kanso-md5.");
	const contentType = request.headers.get("content-type") || "application/octet-stream";
	await env.FILES.put(fileKey(id), bytes, {
		md5: actual,
		httpMetadata: { contentType },
		customMetadata: { md5: actual },
	});
	return json({ ok: true, md5: actual });
}

async function getFile(env: Env, id: string): Promise<Response> {
	const object = await env.FILES.get(fileKey(id));
	if (!object) return error(404, "No such file.");
	const headers = new Headers();
	object.writeHttpMetadata(headers);
	const md5 = object.customMetadata?.md5 ?? (object.checksums.md5 ? hex(object.checksums.md5) : "");
	headers.set("x-kanso-md5", md5);
	headers.set("content-length", String(object.size));
	headers.set("etag", object.httpEtag);
	return new Response(object.body, { headers });
}
