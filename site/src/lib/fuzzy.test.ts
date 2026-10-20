/**
 * Tests for quick open's fuzzy matching. The properties compare score and
 * rank with a plain subsequence check (case-insensitive, ignoring spaces in
 * the query) and check ordering invariants: matches are never dropped or
 * invented, non-matches never appear, and an empty query changes nothing.
 */
import fc from "fast-check";
import { describe, expect, it } from "vitest";
import { rank, score } from "./fuzzy";

/** Whether the query's letters (spaces ignored) occur in order in the candidate, ignoring case. */
function isSubsequence(query: string, candidate: string): boolean {
  const wanted = [...query.trim().toLowerCase()].filter((c) => c !== " ");
  const haystack = [...candidate.toLowerCase()];
  let at = 0;
  for (const c of wanted) {
    const found = haystack.indexOf(c, at);
    if (found === -1) return false;
    at = found + 1;
  }
  return true;
}

const texts = fc.oneof(fc.string({ maxLength: 12 }), fc.stringMatching(/^[A-Za-z ]{0,14}$/), fc.string({ unit: "grapheme", maxLength: 8 }));
const letters = fc.stringMatching(/^[a-zA-Z ]{0,5}$/);

describe("score", () => {
  it("is zero for an empty or blank query", () => {
    expect(score("", "anything")).toBe(0);
    expect(score("   ", "anything")).toBe(0);
  });

  it("is null when the letters are not all there, in order", () => {
    expect(score("xyz", "Morning pages")).toBeNull();
    expect(score("gm", "Morning pages")).toBeNull();
    expect(score("a", "")).toBeNull();
  });

  it("ignores case on both sides and spaces in the query", () => {
    expect(score("MP", "morning pages")).not.toBeNull();
    expect(score("m p", "Morning pages")).toBe(score("mp", "Morning pages"));
  });

  it("scores word starts and consecutive runs higher", () => {
    expect(score("mp", "Morning pages") as number).toBeGreaterThan(score("mp", "Sample") as number);
    expect(score("abc", "abc xyz") as number).toBeGreaterThan(score("abc", "xaxbxc") as number);
  });

  it("prefers the shorter of two otherwise equal candidates", () => {
    expect(score("note", "note") as number).toBeGreaterThan(score("note", "note and more") as number);
  });

  it("is non-null exactly when the query is a subsequence of the candidate", () => {
    fc.assert(fc.property(letters, texts, (q, c) => (score(q, c) !== null) === isSubsequence(q, c)));
    fc.assert(fc.property(texts, texts, (q, c) => (score(q, c) !== null) === isSubsequence(q, c)));
  });

  it("matches a candidate against any subsequence of itself", () => {
    fc.assert(
      fc.property(fc.stringMatching(/^[a-zA-Z]{1,12}$/), fc.array(fc.boolean(), { minLength: 12, maxLength: 12 }), (candidate, keep) => {
        const query = [...candidate].filter((_, i) => keep[i]).join("");
        return score(query, candidate) !== null;
      }),
    );
  });
});

describe("rank", () => {
  const names = ["Sample", "Morning pages", "Garden plan", "Sunday", "Tasks", "Encrypted sync"];

  it("puts the best match first and drops non-matches", () => {
    expect(rank("mp", names, (n) => n)).toEqual(["Morning pages", "Sample"]);
    expect(rank("zzz", names, (n) => n)).toEqual([]);
  });

  it("keeps the original order among equal scores", () => {
    const items = [{ n: "same", id: 1 }, { n: "same", id: 2 }, { n: "same", id: 3 }];
    expect(rank("sa", items, (i) => i.n).map((i) => i.id)).toEqual([1, 2, 3]);
  });

  it("uses the text accessor and returns the items themselves", () => {
    const items = [{ label: "Alpha" }, { label: "Beta" }];
    const [first] = rank("bet", items, (i) => i.label);
    expect(first).toBe(items[1]);
  });

  it("returns everything, in order, for an empty query", () => {
    fc.assert(
      fc.property(fc.array(texts, { maxLength: 15 }), fc.constantFrom("", "  ", " "), (items, q) => {
        const out = rank(q, items, (s) => s);
        return out.length === items.length && out.every((x, i) => x === items[i]);
      }),
    );
  });

  it("returns exactly the matching items, none lost and none invented", () => {
    fc.assert(
      fc.property(fc.array(texts, { maxLength: 15 }), letters, (items, q) => {
        const out = rank(q, items, (s) => s);
        const expected = items.filter((s) => isSubsequence(q, s));
        return out.length === expected.length && [...out].sort().join("\u0000") === [...expected].sort().join("\u0000");
      }),
    );
  });

  it("orders by score, best first", () => {
    fc.assert(
      fc.property(fc.array(texts, { maxLength: 15 }), letters, (items, q) => {
        const scores = rank(q, items, (s) => s).map((s) => score(q, s) as number);
        return scores.every((s, i) => i === 0 || (scores[i - 1] as number) >= s);
      }),
    );
  });

  it("does not modify its input", () => {
    const items = ["b", "a", "ab"];
    rank("a", items, (s) => s);
    expect(items).toEqual(["b", "a", "ab"]);
  });
});
