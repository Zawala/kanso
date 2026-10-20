/**
 * Quick open's matching: a query matches when its letters appear in order in
 * the candidate, ignoring case. Matches at the start of words and runs of
 * consecutive letters score higher, so "mp" finds "Morning pages" before
 * "Sample".
 */

/** The match's score, higher is better, or null when it doesn't match. */
export function score(query: string, candidate: string): number | null {
  const q = query.trim().toLowerCase();
  if (q === "") return 0;
  const c = candidate.toLowerCase();
  let total = 0;
  let from = 0;
  let previous = -2;
  for (const char of q) {
    if (char === " ") continue;
    const at = c.indexOf(char, from);
    if (at === -1) return null;
    total += 1;
    if (at === previous + 1) total += 3;
    if (at === 0 || /[\s/_-]/.test(c[at - 1] as string)) total += 5;
    previous = at;
    from = at + 1;
  }
  return total - c.length / 100;
}

/** The candidates that match, best first; ties keep their order. */
export function rank<T>(query: string, items: readonly T[], text: (item: T) => string): T[] {
  return items
    .map((item, index) => ({ item, index, s: score(query, text(item)) }))
    .filter((r): r is { item: T; index: number; s: number } => r.s !== null)
    .sort((a, b) => b.s - a.s || a.index - b.index)
    .map((r) => r.item);
}
