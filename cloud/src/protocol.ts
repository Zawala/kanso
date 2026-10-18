// The sync protocol's limits and validation. Mirrors
// crates/kanso-sync/src/remote.rs; test/fixtures/conformance-v1.json holds the
// cases both sides run.

export const MAX_PUSH_BYTES = 16 * 1024 * 1024;
/** Three D1 statements per document in one batch; an invocation may run 1,000. */
export const MAX_PUSH_DOCS = 200;
/** D1 rows top out at 2 MB. */
export const MAX_FIELDS_BYTES = 1_900_000;
export const MAX_FILE_BYTES = 10 * 1024 * 1024;
export const DEFAULT_CHANGES_LIMIT = 100;
export const MAX_CHANGES_LIMIT = 500;

const DOC_ID = /^(note|book|tag|file):[A-Za-z0-9_.-]{1,120}$/;
const FILE_ID = /^file:[A-Za-z0-9_.-]{1,120}$/;
const REV = /^[0-9]{1,16}-[0-9a-fA-F]{1,64}$/;
const MD5 = /^[0-9a-fA-F]{32}$/;

export const isDocId = (id: string): boolean => DOC_ID.test(id);
export const isFileId = (id: string): boolean => FILE_ID.test(id);
export const isRev = (rev: string): boolean => REV.test(rev);
export const isMd5Hex = (md5: string): boolean => MD5.test(md5);

/** A validated pushed document. `fields` is its JSON text. */
export interface PushDoc {
	id: string;
	rev: string;
	deleted: boolean;
	fields: string;
	baseRev: string | null;
	updatedAt: number | null;
}

export type Checked<T> = { ok: true; value: T } | { ok: false; status: number; error: string };

const fail = (status: number, error: string): Checked<never> => ({ ok: false, status, error });

function isObject(value: unknown): value is Record<string, unknown> {
	return typeof value === "object" && value !== null && !Array.isArray(value);
}

const encoder = new TextEncoder();

/** Validates a `POST /v1/push` body. Any invalid document rejects the whole push. */
export function checkPush(body: unknown): Checked<PushDoc[]> {
	if (!isObject(body) || !Array.isArray(body.docs)) {
		return fail(400, 'The body must be {"docs": [...]}.');
	}
	if (body.docs.length > MAX_PUSH_DOCS) {
		return fail(413, `A push carries at most ${MAX_PUSH_DOCS} documents.`);
	}
	const docs: PushDoc[] = [];
	for (const [i, doc] of body.docs.entries()) {
		if (!isObject(doc)) return fail(400, `docs[${i}] is not an object.`);
		const { id, rev } = doc;
		if (typeof id !== "string" || !isDocId(id)) return fail(400, `docs[${i}].id is not a document ID.`);
		if (typeof rev !== "string" || !isRev(rev)) return fail(400, `docs[${i}].rev is not a revision.`);
		const deleted = doc.deleted === undefined ? false : doc.deleted;
		if (typeof deleted !== "boolean") return fail(400, `docs[${i}].deleted must be a boolean.`);
		const fields = doc.fields === undefined ? {} : doc.fields;
		if (!isObject(fields)) return fail(400, `docs[${i}].fields must be an object.`);
		const baseRev = doc.base_rev === undefined ? null : doc.base_rev;
		if (baseRev !== null && (typeof baseRev !== "string" || !isRev(baseRev))) {
			return fail(400, `docs[${i}].base_rev is not a revision.`);
		}
		const json = JSON.stringify(fields);
		if (encoder.encode(json).length > MAX_FIELDS_BYTES) {
			return fail(413, `docs[${i}].fields is larger than ${MAX_FIELDS_BYTES} bytes.`);
		}
		const updated = fields.updatedAt;
		const updatedAt = typeof updated === "number" && Number.isFinite(updated) ? Math.trunc(updated) : null;
		docs.push({ id, rev, deleted, fields: json, baseRev, updatedAt });
	}
	return { ok: true, value: docs };
}

/** Parses an integer query parameter; `fallback` when absent. */
export function intParam(value: string | null, fallback: number): number | null {
	if (value === null) return fallback;
	if (!/^-?[0-9]{1,16}$/.test(value)) return null;
	return Number(value);
}
