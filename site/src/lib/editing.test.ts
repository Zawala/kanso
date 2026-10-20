/**
 * Tests for the editor's key handling: pairing, stepping over closers,
 * Backspace on empty pairs, Return in lists, and Tab/Shift-Tab indentation.
 * Examples pin each behaviour; properties state round trips (open then
 * Backspace, indent then outdent), wrap-selection and "non-list lines are
 * left to the browser".
 */
import fc from "fast-check";
import { describe, expect, it } from "vitest";
import { PAIRS, type TextState, onBackspace, onEnter, onTab, onType } from "./editing";

const at = (value: string, start = value.length, end = start): TextState => ({ value, start, end });
const OPENERS = Object.keys(PAIRS);
const WORD = /[\p{L}\p{N}_]/u;
const ITEM_PREFIX = /^\s*([-*+]|\d{1,9}[.)])\s/;

describe("onType", () => {
  it.each(OPENERS)("pairs %s at the end of the text", (open) => {
    expect(onType(at("hi "), open)).toEqual({ value: `hi ${open}${PAIRS[open]}`, start: 4, end: 4 });
  });

  it("pairs before whitespace or a closer", () => {
    expect(onType(at("a  b", 2), "(")).toEqual({ value: "a () b", start: 3, end: 3 });
    expect(onType(at("(x)", 2), "[")?.value).toBe("(x[])");
  });

  it("does not pair before a word character", () => {
    expect(onType(at("ab", 0), "(")).toBeNull();
  });

  it("does not pair quotes or backticks after a letter, number or underscore", () => {
    expect(onType(at("don"), '"')).toBeNull();
    expect(onType(at("x1"), "`")).toBeNull();
    expect(onType(at("a_"), '"')).toBeNull();
    expect(onType(at("(("), '"')?.value).toBe('((""');
  });

  it("still pairs brackets after a letter", () => {
    expect(onType(at("f"), "(")).toEqual({ value: "f()", start: 2, end: 2 });
  });

  it("steps over a closer that is already there", () => {
    expect(onType(at("(a)", 2), ")")).toEqual({ value: "(a)", start: 3, end: 3 });
    expect(onType(at('"a"', 2), '"')).toEqual({ value: '"a"', start: 3, end: 3 });
    expect(onType(at("`a`", 2), "`")).toEqual({ value: "`a`", start: 3, end: 3 });
  });

  it("types a closer normally when there is none ahead", () => {
    expect(onType(at("a"), ")")).toBeNull();
    expect(onType(at("(a]", 2), ")")).toBeNull();
  });

  it("wraps a selection in the pair and keeps it selected", () => {
    expect(onType(at("say hello now", 4, 9), "[")).toEqual({ value: "say [hello] now", start: 5, end: 10 });
    expect(onType(at("abc", 0, 3), '"')).toEqual({ value: '"abc"', start: 1, end: 4 });
  });

  it("leaves other characters, and a selection typed over with a non-opener, to the browser", () => {
    expect(onType(at("abc"), "x")).toBeNull();
    expect(onType(at("abc", 0, 2), "x")).toBeNull();
    expect(onType(at("abc", 0, 2), ")")).toBeNull();
  });

  const caretContext = fc.record({
    before: fc.string(),
    open: fc.constantFrom(...OPENERS),
    after: fc.oneof(fc.constant(""), fc.string().map((s) => ` ${s}`), fc.string().map((s) => `\n${s}`)),
  });
  const safeBefore = (before: string, open: string) =>
    (open === '"' || open === "`") && before !== "" && WORD.test(Array.from(before).at(-1) as string) ? `${before} ` : before;

  it("typing an opener before whitespace or the end, then Backspace, restores the text and caret", () => {
    fc.assert(
      fc.property(caretContext, ({ before, open, after }) => {
        const prefix = safeBefore(before, open);
        const original = at(prefix + after, prefix.length);
        const typed = onType(original, open);
        if (typed === null) return false;
        const back = onBackspace(typed);
        return (
          typed.value === prefix + open + PAIRS[open] + after &&
          typed.start === prefix.length + 1 &&
          back !== null &&
          back.value === original.value &&
          back.start === original.start &&
          back.end === original.end
        );
      }),
    );
  });

  it("wrapping any selection with an opener keeps the selected text between the pair", () => {
    fc.assert(
      fc.property(
        fc.string(),
        fc.string({ minLength: 1 }),
        fc.string(),
        fc.constantFrom(...OPENERS),
        (head, selected, tail, open) => {
          const state = at(head + selected + tail, head.length, head.length + selected.length);
          const out = onType(state, open);
          if (out === null) return false;
          return (
            out.value === head + open + selected + PAIRS[open] + tail &&
            out.value.slice(out.start, out.end) === selected &&
            out.value[out.start - 1] === open &&
            out.value[out.end] === PAIRS[open]
          );
        },
      ),
    );
  });

  it("only ever inserts at the caret, at most a pair's worth", () => {
    fc.assert(
      fc.property(fc.string(), fc.nat(), fc.string({ minLength: 1, maxLength: 1 }), (value, n, char) => {
        const caret = value.length === 0 ? 0 : n % (value.length + 1);
        const out = onType(at(value, caret), char);
        if (out === null) return true;
        const grew = out.value.length - value.length;
        return (
          (grew === 0 || grew === 2) &&
          out.value.startsWith(value.slice(0, caret)) &&
          out.value.endsWith(value.slice(caret)) &&
          out.start === out.end
        );
      }),
    );
  });
});

describe("onBackspace", () => {
  it("removes an empty pair in one go", () => {
    expect(onBackspace(at("a()b", 2))).toEqual({ value: "ab", start: 1, end: 1 });
    expect(onBackspace(at('x""', 2))?.value).toBe("x");
    expect(onBackspace(at("{}", 1))?.value).toBe("");
  });

  it("leaves everything else to the browser", () => {
    expect(onBackspace(at("a()b", 3))).toBeNull(); // after the closer
    expect(onBackspace(at("(a)", 2))).toBeNull(); // pair not empty
    expect(onBackspace(at("(]", 1))).toBeNull(); // mismatched
    expect(onBackspace(at("abc", 0))).toBeNull(); // start of text
    expect(onBackspace(at("()", 0, 2))).toBeNull(); // selection
    expect(onBackspace(at("ab", 1))).toBeNull();
  });
});

describe("onEnter", () => {
  it("carries a bullet list on with the same marker and indent", () => {
    expect(onEnter(at("- one"))).toEqual({ value: "- one\n- ", start: 8, end: 8 });
    expect(onEnter(at("  * nested"))?.value).toBe("  * nested\n  * ");
    expect(onEnter(at("+ x"))?.value).toBe("+ x\n+ ");
  });

  it("increments ordered list numbers and keeps the delimiter", () => {
    expect(onEnter(at("1. one"))?.value).toBe("1. one\n2. ");
    expect(onEnter(at("9) nine"))?.value).toBe("9) nine\n10) ");
    expect(onEnter(at("  41. deep"))?.value).toBe("  41. deep\n  42. ");
  });

  it("continues task items with an empty box, whether the last was ticked or not", () => {
    expect(onEnter(at("- [ ] todo"))?.value).toBe("- [ ] todo\n- [ ] ");
    expect(onEnter(at("- [x] done"))?.value).toBe("- [x] done\n- [ ] ");
    expect(onEnter(at("2. [X] done"))?.value).toBe("2. [X] done\n3. [ ] ");
  });

  it("ends the list on an empty item, leaving an empty line", () => {
    expect(onEnter(at("- a\n- "))).toEqual({ value: "- a\n", start: 4, end: 4 });
    expect(onEnter(at("1. a\n2. "))?.value).toBe("1. a\n");
    expect(onEnter(at("- [ ] a\n- [ ] "))?.value).toBe("- [ ] a\n");
    expect(onEnter(at("- "))).toEqual({ value: "", start: 0, end: 0 });
  });

  it("splits an item when Return is pressed in the middle of it", () => {
    expect(onEnter(at("- ab", 3))).toEqual({ value: "- a\n- b", start: 6, end: 6 });
  });

  it("carries on, rather than ending, when the caret is not at the end of an empty item", () => {
    const out = onEnter(at("-   ", 2));
    expect(out?.value.split("\n")).toHaveLength(2);
    expect(out?.value.startsWith("- \n- ")).toBe(true);
  });

  it("works on a middle line of a longer note", () => {
    const value = "# T\n\n- a\n- b\n\nend";
    const caret = value.indexOf("- b") + 3;
    expect(onEnter(at(value, caret))?.value).toBe("# T\n\n- a\n- b\n- \n\nend");
  });

  it("leaves non-list lines and selections to the browser", () => {
    expect(onEnter(at("plain"))).toBeNull();
    expect(onEnter(at(""))).toBeNull();
    expect(onEnter(at("- a", 0, 3))).toBeNull();
    expect(onEnter(at("-nospace"))).toBeNull();
  });

  it("returns null on every non-list line, wherever the caret is", () => {
    const notItem = fc.string().filter((l) => !ITEM_PREFIX.test(l) && !l.includes("\n"));
    fc.assert(
      fc.property(fc.array(notItem, { minLength: 1, maxLength: 5 }), fc.nat(), (lines, n) => {
        const value = lines.join("\n");
        return onEnter(at(value, n % (value.length + 1))) === null;
      }),
    );
  });

  it("at the end of an ordered item, the next line starts with the next number", () => {
    fc.assert(
      fc.property(
        fc.nat(9998),
        fc.constantFrom(".", ")"),
        fc.stringMatching(/^[a-z][a-z ]{0,10}$/),
        fc.constantFrom("", "  ", "    "),
        (n, delimiter, body, indent) => {
          const out = onEnter(at(`${indent}${n}${delimiter} ${body}`));
          return out !== null && out.value.endsWith(`\n${indent}${n + 1}${delimiter} `) && out.start === out.value.length;
        },
      ),
    );
  });

  it("an empty item always ends the list and removes only that item's text", () => {
    fc.assert(
      fc.property(
        fc.array(fc.stringMatching(/^[a-z]{1,6}$/), { maxLength: 3 }),
        fc.constantFrom("- ", "* ", "1. ", "- [ ] "),
        (previous, marker) => {
          const head = previous.map((p) => `- ${p}\n`).join("");
          const out = onEnter(at(head + marker));
          return out !== null && out.value === head && out.start === head.length;
        },
      ),
    );
  });
});

describe("onTab", () => {
  it("indents a list item by two spaces and moves the selection with it", () => {
    expect(onTab(at("- a\n- b", 7), false)).toEqual({ value: "- a\n  - b", start: 9, end: 9 });
  });

  it("outdents by two spaces, one space or one tab, and not below the margin", () => {
    expect(onTab(at("    - a", 7), true)?.value).toBe("  - a");
    expect(onTab(at(" - a", 4), true)?.value).toBe("- a");
    expect(onTab(at("\t- a", 4), true)?.value).toBe("- a");
    const flush = at("- a", 3);
    expect(onTab(flush, true)).toBe(flush);
  });

  it("keeps the caret on its line when outdenting near the margin", () => {
    expect(onTab(at("  - a", 1), true)).toEqual({ value: "- a", start: 0, end: 0 });
  });

  it("handles task and ordered items", () => {
    expect(onTab(at("1. x", 2), false)?.value).toBe("  1. x");
    expect(onTab(at("- [ ] t", 7), false)?.value).toBe("  - [ ] t");
  });

  it("leaves non-list lines, and selections across lines, to the browser", () => {
    expect(onTab(at("text"), false)).toBeNull();
    expect(onTab(at("text"), true)).toBeNull();
    expect(onTab(at("- a\n- b", 0, 7), false)).toBeNull();
  });

  const listLine = fc.record({
    lead: fc.constantFrom("", " ", "  ", "\t", "    "),
    marker: fc.constantFrom("- ", "* ", "+ ", "1. ", "12) ", "- [ ] ", "- [x] "),
    body: fc.stringMatching(/^[a-z ]{0,12}$/),
  });

  it("Tab then Shift-Tab restores a list line and its selection", () => {
    fc.assert(
      fc.property(
        fc.array(fc.stringMatching(/^[a-z ]{0,8}$/), { maxLength: 3 }),
        listLine,
        fc.array(fc.stringMatching(/^[a-z ]{0,8}$/), { maxLength: 3 }),
        fc.nat(),
        fc.nat(),
        (above, { lead, marker, body }, below, p, q) => {
          const line = `${lead}${marker}${body}`;
          const head = above.map((l) => `${l}\n`).join("");
          const value = head + line + below.map((l) => `\n${l}`).join("");
          const [a, b] = [p % (line.length + 1), q % (line.length + 1)].sort((x, y) => x - y) as [number, number];
          const state = { value, start: head.length + a, end: head.length + b };
          const indented = onTab(state, false);
          if (indented === null) return false;
          const back = onTab(indented, true);
          return (
            indented.value.length === value.length + 2 &&
            back !== null &&
            back.value === value &&
            back.start === state.start &&
            back.end === state.end
          );
        },
      ),
    );
  });

  it("is left to the browser on a line that is not a list item", () => {
    const notItem = fc.string().filter((l) => !ITEM_PREFIX.test(l) && !l.includes("\n"));
    fc.assert(fc.property(notItem, fc.boolean(), (line, outdent) => onTab(at(line), outdent) === null));
  });
});
