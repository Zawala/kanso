// Usernames (docs/ACCOUNTS.md › Usernames). test/fixtures/usernames.json holds
// the cases this file and crates/kanso-sync/src/username.rs both run. No
// imports and only erasable syntax: the admin script runs this on Node too.

/** Refused at sign-up only; every other route takes the normalized name. */
export const RESERVED: readonly string[] = [
	"abuse",
	"admin",
	"administrator",
	"anonymous",
	"api",
	"everyone",
	"guest",
	"help",
	"hostmaster",
	"kanso",
	"me",
	"null",
	"owner",
	"postmaster",
	"root",
	"security",
	"server",
	"support",
	"sync",
	"system",
	"undefined",
	"webmaster",
	"www",
];

const VALID = /^[a-z0-9][a-z0-9_-]{1,30}[a-z0-9]$/;
// U+0009, U+000A, U+000C, U+000D and U+0020: not JS trim(), which strips more.
const TRIMMED = new Set(["\t", "\n", "\f", "\r", " "]);

/**
 * The normalized username, or null when it is invalid: strip the five
 * whitespace characters at both ends, refuse anything left outside
 * U+0021–U+007E (before lowercasing, which would turn U+212A KELVIN SIGN into
 * `k`), ASCII-lowercase, then require the pattern.
 */
export function normalizeUsername(input: string): string | null {
	let start = 0;
	let end = input.length;
	while (start < end && TRIMMED.has(input[start])) start++;
	while (end > start && TRIMMED.has(input[end - 1])) end--;
	const trimmed = input.slice(start, end);
	for (let i = 0; i < trimmed.length; i++) {
		const code = trimmed.charCodeAt(i);
		if (code < 0x21 || code > 0x7e) return null;
	}
	const lowered = trimmed.replace(/[A-Z]/g, (c) => c.toLowerCase());
	return VALID.test(lowered) ? lowered : null;
}

export const isReserved = (normalized: string): boolean => RESERVED.includes(normalized);
