// Username normalization: the shared fixture (also run by
// crates/kanso-sync/src/username.rs), then properties.

import fc from "fast-check";
import { describe, expect, it } from "vitest";
import { RESERVED, isReserved, normalizeUsername } from "../src/usernames";
import fixture from "./fixtures/usernames.json";

const PATTERN = /^[a-z0-9][a-z0-9_-]{1,30}[a-z0-9]$/;
const TRIMMED = ["\t", "\n", "\f", "\r", " "];

describe("usernames.json", () => {
	it("lists the same reserved names", () => {
		expect([...RESERVED]).toEqual(fixture.reserved);
	});

	for (const c of fixture.cases) {
		it(`${JSON.stringify(c.input)}${c.note ? ` (${c.note})` : ""}`, () => {
			const normalized = normalizeUsername(c.input);
			expect(normalized).toBe(c.normalized);
			const error = normalized === null ? "invalid_username" : isReserved(normalized) ? "reserved_username" : null;
			expect(error).toBe(c.error);
		});
	}
});

describe("properties", () => {
	const nameChars = fc.constantFrom(..."abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-");
	const nameLike = fc.string({ unit: nameChars, minLength: 0, maxLength: 40 });
	const padding = fc.string({ unit: fc.constantFrom(...TRIMMED), maxLength: 4 });
	const anything = fc.oneof(fc.string({ unit: "binary", maxLength: 40 }), nameLike, fc.string({ maxLength: 40 }));

	// Idempotence: a normalized name normalizes to itself.
	it("prop_normalizing_is_idempotent", () => {
		fc.assert(
			fc.property(anything, (input) => {
				const once = normalizeUsername(input);
				if (once !== null) expect(normalizeUsername(once)).toBe(once);
			}),
		);
	});

	// Refinement: whatever is accepted matches the stored pattern (and so D1's CHECK).
	it("prop_accepted_names_match_the_pattern", () => {
		fc.assert(
			fc.property(anything, (input) => {
				const normalized = normalizeUsername(input);
				if (normalized !== null) expect(normalized).toMatch(PATTERN);
			}),
		);
	});

	// Metamorphic: ASCII case and the five trimmed characters at either end don't change the result.
	it("prop_case_and_outer_whitespace_dont_matter", () => {
		fc.assert(
			fc.property(nameLike, padding, padding, (name, before, after) => {
				const upper = name.toUpperCase();
				expect(normalizeUsername(`${before}${upper}${after}`)).toBe(normalizeUsername(name.toLowerCase()));
			}),
		);
	});

	// Any character outside U+0021–U+007E left after trimming refuses the name, wherever it sits.
	it("prop_non_ascii_or_inner_whitespace_is_refused", () => {
		const outside = fc.oneof(
			fc.integer({ min: 0, max: 0x20 }).map((c) => String.fromCharCode(c)),
			fc.integer({ min: 0x7f, max: 0xffff }).map((c) => String.fromCharCode(c)),
			fc.constantFrom("K", " ", " ", "﻿", "\u000b", "é"),
		);
		fc.assert(
			fc.property(nameLike, outside, nameLike, (head, bad, tail) => {
				expect(normalizeUsername(`a${head}${bad}${tail}a`)).toBeNull();
			}),
		);
	});
});
