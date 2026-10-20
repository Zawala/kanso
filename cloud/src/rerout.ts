// Short links for published notes (docs/ACCOUNTS.md › Published notes ›
// Short links), through Rerout's HTTP API (https://api.rerout.co).
//
// A short link's target is `<origin>/p/<id>` with no #fragment: the share's
// key never leaves the device, so Rerout never has it either. rerout.co
// answers a browser with a plain 302 to that target, and browsers carry the
// fragment of the link they opened (`<short url>#<key>`) across the redirect.
// A link-preview bot gets Rerout's page with the note's title, one line about
// Kanso and Kanso's image; every link is noindex.
//
// Everything here is best effort. Without REROUT_API_KEY nothing is called;
// when Rerout fails, the device keeps the long link, and links that should be
// deleted wait in `short_link_purges` for the cron.

import type { Env } from "./env";

export const REROUT_API = "https://api.rerout.co";
/** How long one call to Rerout may take before publishing goes on without it. */
export const REROUT_TIMEOUT_MS = 5_000;
/** Short links one cron run deletes. */
export const SHORT_LINK_PURGE_BATCH = 25;
/** Rerout's limit for `seo_title`, in characters. */
export const MAX_TITLE_CHARS = 90;
export const DEFAULT_TITLE = "A note";
export const PREVIEW_DESCRIPTION = "A note shared from Kanso. It opens in your browser, end to end encrypted.";
/** Kanso's preview image, a static asset (public/og/kanso.png). */
export const PREVIEW_IMAGE_PATH = "/og/kanso.png";

/** A Rerout link as the Worker keeps it. */
export interface ShortLink {
	code: string;
	shortUrl: string;
}

const CODE = /^[A-Za-z0-9_-]{1,64}$/;

export const isShortCode = (code: unknown): code is string => typeof code === "string" && CODE.test(code);

/** An https URL the device can append `#<key>` to: no fragment, query or whitespace, at most 256 characters. */
export function isShortUrl(url: unknown): url is string {
	if (typeof url !== "string" || url.length > 256 || !url.startsWith("https://") || /[#?\s]/.test(url)) return false;
	try {
		const parsed = new URL(url);
		return parsed.protocol === "https:" && parsed.hostname !== "" && parsed.username === "" && parsed.password === "";
	} catch {
		return false;
	}
}

/** The custom domain short links are made on, lower-cased; null for Rerout's own. */
export function shortLinkDomain(env: Env): string | null {
	const domain = env.REROUT_DOMAIN?.trim().toLowerCase();
	return domain ? domain : null;
}

/**
 * A title as previews show it: control characters and runs of whitespace
 * become single spaces, the ends are trimmed, and past 90 characters it is cut
 * to 89 and an ellipsis. Empty becomes "A note". Pure.
 */
export function previewTitle(text: string): string {
	const words = text
		.replace(/[\u0000-\u001f\u007f-\u009f\u200b\u200e\u200f\u2028-\u202e\u2060-\u206f\ufeff]/g, " ")
		.split(/\s+/)
		.filter((w) => w !== "")
		.join(" ");
	const chars = Array.from(words);
	if (chars.length === 0) return DEFAULT_TITLE;
	if (chars.length <= MAX_TITLE_CHARS) return words;
	return `${chars
		.slice(0, MAX_TITLE_CHARS - 1)
		.join("")
		.trimEnd()}…`;
}

/** `x-kanso-title`: the note's title as percent-encoded UTF-8. Absent, too long or malformed is "A note". */
export function titleHeader(header: string | null): string {
	if (header === null || header.length > 4096) return DEFAULT_TITLE;
	try {
		return previewTitle(decodeURIComponent(header));
	} catch {
		return DEFAULT_TITLE;
	}
}

/** Whether short links are on: a key, and an https origin Rerout can redirect to. */
export function enabled(env: Env, origin: string): boolean {
	return typeof env.REROUT_API_KEY === "string" && env.REROUT_API_KEY.trim() !== "" && origin.startsWith("https://");
}

function log(op: string, outcome: string | number): void {
	// Operations and statuses only: never codes, titles or URLs.
	console.error(JSON.stringify({ route: "rerout", op, outcome }));
}

async function call(env: Env, method: string, path: string, body?: unknown): Promise<Response | null> {
	try {
		return await fetch(`${REROUT_API}${path}`, {
			method,
			headers: {
				authorization: `Bearer ${env.REROUT_API_KEY?.trim()}`,
				accept: "application/json",
				"user-agent": "kanso-sync",
				...(body === undefined ? {} : { "content-type": "application/json" }),
			},
			body: body === undefined ? undefined : JSON.stringify(body),
			redirect: "manual",
			signal: AbortSignal.timeout(REROUT_TIMEOUT_MS),
		});
	} catch (err) {
		log(method.toLowerCase(), (err as Error)?.name ?? "error");
		return null;
	}
}

/**
 * Creates the short link of share `id`, previewing as `title`. Null when
 * Rerout failed or answered something unusable; a link it made but the
 * Worker can't use is deleted again.
 */
export async function createShortLink(env: Env, origin: string, id: string, title: string): Promise<ShortLink | null> {
	const target = `${origin}/p/${id}`;
	const domain = shortLinkDomain(env);
	const response = await call(env, "POST", "/v1/links", {
		target_url: target,
		seo_title: title,
		seo_description: PREVIEW_DESCRIPTION,
		seo_image_url: `${origin}${PREVIEW_IMAGE_PATH}`,
		// Every preview field is set, so Rerout never fetches the target to fill one in.
		seo_canonical_url: target,
		seo_noindex: true,
		...(domain ? { domain_hostname: domain } : {}),
	});
	if (!response) return null;
	if (response.status !== 201 && response.status !== 200) {
		log("create", response.status);
		await response.body?.cancel();
		return null;
	}
	let link: { code?: unknown; short_url?: unknown; project_id?: unknown; target_url?: unknown };
	try {
		link = await response.json();
	} catch {
		log("create", "unparsable");
		return null;
	}
	if (!isShortCode(link?.code)) {
		log("create", "no_code");
		return null;
	}
	const project = env.REROUT_PROJECT_ID?.trim();
	// With a custom domain, a link on any other host isn't one to hand out: the
	// device keeps the long link instead.
	const usable =
		isShortUrl(link.short_url) &&
		(!domain || new URL(link.short_url).hostname === domain) &&
		link.target_url === target &&
		(!project || link.project_id === project);
	if (!usable) {
		log("create", "unusable");
		await deleteShortLink(env, link.code);
		return null;
	}
	return { code: link.code, shortUrl: link.short_url as string };
}

/** Sets a short link's preview title. False when Rerout failed. */
export async function retitleShortLink(env: Env, code: string, title: string): Promise<boolean> {
	const response = await call(env, "PATCH", `/v1/links/${encodeURIComponent(code)}`, { seo_title: title });
	if (!response) return false;
	await response.body?.cancel();
	if (response.ok) return true;
	log("retitle", response.status);
	return false;
}

/** Deletes a short link. True once Rerout no longer has it (deleted now, or already gone). */
export async function deleteShortLink(env: Env, code: string): Promise<boolean> {
	const response = await call(env, "DELETE", `/v1/links/${encodeURIComponent(code)}`);
	if (!response) return false;
	await response.body?.cancel();
	if (response.ok || response.status === 404) return true;
	log("delete", response.status);
	return false;
}

/** Deletes queued short links, those tried least first, and drops the rows Rerout no longer has. */
export async function runShortLinkPurges(env: Env, limit = SHORT_LINK_PURGE_BATCH): Promise<number> {
	if (typeof env.REROUT_API_KEY !== "string" || env.REROUT_API_KEY.trim() === "") return 0;
	const { results } = await env.DB.prepare("SELECT code FROM short_link_purges ORDER BY attempts, requested_at, code LIMIT ?1")
		.bind(limit)
		.all<{ code: string }>();
	let deleted = 0;
	for (const { code } of results) {
		if (await deleteShortLink(env, code)) {
			await env.DB.prepare("DELETE FROM short_link_purges WHERE code = ?1").bind(code).run();
			deleted += 1;
		} else {
			await env.DB.prepare("UPDATE short_link_purges SET attempts = attempts + 1 WHERE code = ?1").bind(code).run();
		}
	}
	return deleted;
}
