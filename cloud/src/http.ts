// Responses, errors and body reading for the v2 routes.

export const JSON_TYPE = "application/json; charset=utf-8";

export function json(value: unknown, status = 200, headers: Record<string, string> = {}): Response {
	return new Response(JSON.stringify(value), { status, headers: { "content-type": JSON_TYPE, ...headers } });
}

/** `{"error": code, "message": ..., ...extras}`. The client maps codes to its own words and never shows `message`. */
export function error(
	status: number,
	code: string,
	message: string,
	extras: Record<string, unknown> = {},
	headers: Record<string, string> = {},
): Response {
	return json({ error: code, message, ...extras }, status, headers);
}

export const badRequest = (message: string): Response => error(400, "bad_request", message);
export const notFound = (): Response => error(404, "not_found", "Not found.");
export const tooLarge = (message: string): Response => error(413, "too_large", message);

export function methodNotAllowed(allow: string): Response {
	return error(405, "method_not_allowed", "Method not allowed.", {}, { allow });
}

export type SignOutReason = "unknown" | "signed_out" | "revoked" | "password_changed" | "recovered" | "evicted" | "expired" | "rekeyed";

export function unauthorized(reason: SignOutReason): Response {
	return error(401, "unauthorized", "Not signed in.", { reason }, { "www-authenticate": 'Bearer realm="kanso-sync"' });
}

export const invalidCredentials = (): Response => error(401, "invalid_credentials", "Wrong credentials.");
export const accountDisabled = (): Response => error(403, "account_disabled", "The account is disabled.");

/** 429 with `retry_after` (seconds, rounded up) in the body and the `Retry-After` header. */
export function rateLimited(limit: string, retryAfterMs: number): Response {
	const retryAfter = Math.max(1, Math.ceil(retryAfterMs / 1000));
	return error(429, "rate_limited", "Too many requests.", { limit, retry_after: retryAfter }, { "retry-after": String(retryAfter) });
}

const errorText = (err: unknown): string =>
	`${(err as Error)?.message ?? ""} ${String((err as { cause?: unknown })?.cause ?? "")}`;

/** A trigger's refusal (`signup_closed:<reason>`, `quota_exceeded:<kind>`, `rate_limited:<cap>`) in a D1 error, if any. */
export function refusal(err: unknown): { kind: string; detail: string } | null {
	const match = /(signup_closed|quota_exceeded|rate_limited):([a-z_-]+)/.exec(errorText(err));
	return match ? { kind: match[1], detail: match[2] } : null;
}

/** Whether a D1 error is a failed UNIQUE or CHECK constraint. */
export const constraint = (err: unknown, kind: "UNIQUE" | "CHECK"): boolean => errorText(err).includes(`${kind} constraint failed`);

/** Milliseconds from `now` to the next UTC midnight. */
export const untilMidnight = (now: number): number => 86_400_000 - (now % 86_400_000);

/** Reads at most `max` bytes of the body; `null` when it is longer. */
export async function readBody(request: Request, max: number): Promise<Uint8Array | null> {
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

const utf8 = new TextDecoder("utf-8", { fatal: true, ignoreBOM: false });

/** The body as JSON, or the error response: 413 over `max` bytes, 400 for anything that isn't UTF-8 JSON. */
export async function readJson(request: Request, max: number): Promise<{ ok: true; value: unknown } | { ok: false; response: Response }> {
	const bytes = await readBody(request, max);
	if (!bytes) return { ok: false, response: tooLarge(`The body is larger than ${max} bytes.`) };
	try {
		return { ok: true, value: JSON.parse(utf8.decode(bytes)) };
	} catch {
		return { ok: false, response: badRequest("The body is not JSON.") };
	}
}

/** Whether a Content-Type is `type`, with at most a `charset=utf-8` parameter when `charset` is allowed. */
export function isContentType(header: string | null, type: string, charset: boolean): boolean {
	if (header === null) return false;
	const [media, ...params] = header.split(";").map((part) => part.trim().toLowerCase());
	if (media !== type) return false;
	if (params.length === 0) return true;
	return charset && params.length === 1 && /^charset="?utf-8"?$/.test(params[0]);
}
