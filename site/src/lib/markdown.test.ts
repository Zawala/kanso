/**
 * Tests for the in-browser Markdown dialect: block parsing, inline parsing
 * (including the regression where recursive parsing moved shared sticky
 * regex state), editor highlighting, tasks and toggling, headings, titles,
 * word counts, slugs, tags and snippets. Examples pin the behaviour; the
 * fast-check properties state invariants (round trips, totality, counts
 * against a generator's own model) rather than re-deriving the output.
 */
import fc from "fast-check";
import { describe, expect, it } from "vitest";
import {
  headings,
  highlight,
  inline,
  parse,
  plain,
  slug,
  snippet,
  tagsOf,
  tasks,
  title,
  toggleTask,
  wordCount,
} from "./markdown";

const flat = (src: string): string =>
  highlight(src)
    .map((spans) => spans.map((s) => s.text).join(""))
    .join("\n");

/** Strings, including awkward ones, and strings built from Markdown's own punctuation. */
const TOKENS = [
  "*", "**", "_", "`", "```", "~~~", "~~", "[", "]", "(", ")", "[[", "]]", "#", "## ", "> ", "- ", "- [ ] ", "- [x] ", "1. ",
  "|", "| --- |", "---", "@2026-10-20", "<kbd>", "</kbd>", "http://x.y", "\n", "\n\n", " ", "\t", "\r\n", "a", "b c", "é", "日本", "😀",
];
const markdownish = fc.array(fc.constantFrom(...TOKENS), { maxLength: 30 }).map((parts) => parts.join(""));
const anyText = fc.oneof(fc.string(), fc.string({ unit: "binary" }), fc.string({ unit: "grapheme" }), markdownish);

describe("slug", () => {
  it.each([
    ["Hello World", "hello-world"],
    ["  Spaced   out  ", "spaced-out"],
    ["What's new?", "whats-new"],
    ["Déjà vu", "déjà-vu"],
    ["日本語 ノート", "日本語-ノート"],
    ["", ""],
    ["!!!", ""],
  ])("slug(%j) = %j", (input, expected) => {
    expect(slug(input)).toBe(expected);
  });

  it("only ever contains letters, numbers and hyphens", () => {
    fc.assert(fc.property(anyText, (text) => /^[\p{L}\p{N}-]*$/u.test(slug(text))));
  });

  it("is idempotent", () => {
    fc.assert(fc.property(anyText, (text) => slug(slug(text)) === slug(text)));
  });
});

describe("inline", () => {
  it("returns nothing for empty text and one text node for plain text", () => {
    expect(inline("")).toEqual([]);
    expect(inline("just words")).toEqual([{ t: "text", v: "just words" }]);
  });

  it("parses strong, text and em in order (regression: used to loop forever)", () => {
    expect(inline("**bold** and *em*")).toEqual([
      { t: "strong", c: [{ t: "text", v: "bold" }] },
      { t: "text", v: " and " },
      { t: "em", c: [{ t: "text", v: "em" }] },
    ]);
  });

  it("parses nested emphasis inside a link, with the rest of the line intact (regression)", () => {
    const out = inline("see [some *emph* and **bold** words](https://x.test/a) now");
    expect(out).toEqual([
      { t: "text", v: "see " },
      {
        t: "link",
        href: "https://x.test/a",
        c: [
          { t: "text", v: "some " },
          { t: "em", c: [{ t: "text", v: "emph" }] },
          { t: "text", v: " and " },
          { t: "strong", c: [{ t: "text", v: "bold" }] },
          { t: "text", v: " words" },
        ],
      },
      { t: "text", v: " now" },
    ]);
  });

  it("parses strong inside a list item and carries on after it (regression)", () => {
    const [list] = parse("- **strong** item with *em* and `code`");
    expect(list).toMatchObject({ t: "list" });
    if (list?.t !== "list") throw new Error("expected a list");
    expect(list.items[0]?.c).toEqual([
      { t: "strong", c: [{ t: "text", v: "strong" }] },
      { t: "text", v: " item with " },
      { t: "em", c: [{ t: "text", v: "em" }] },
      { t: "text", v: " and " },
      { t: "code", v: "code" },
    ]);
  });

  it("nests strong around em around code", () => {
    expect(inline("**a *b* `c`**")).toEqual([
      {
        t: "strong",
        c: [
          { t: "text", v: "a " },
          { t: "em", c: [{ t: "text", v: "b" }] },
          { t: "text", v: " " },
          { t: "code", v: "c" },
        ],
      },
    ]);
  });

  it("recognises every inline kind", () => {
    expect(inline("`x`")).toEqual([{ t: "code", v: "x" }]);
    expect(inline("<kbd>⌘P</kbd>")).toEqual([{ t: "kbd", v: "⌘P" }]);
    expect(inline("~~gone~~")).toEqual([{ t: "del", c: [{ t: "text", v: "gone" }] }]);
    expect(inline("_soft_")).toEqual([{ t: "em", c: [{ t: "text", v: "soft" }] }]);
    expect(inline("[[Tasks]]")).toEqual([{ t: "wiki", target: "Tasks", label: "Tasks" }]);
    expect(inline("[[ Tasks | the list ]]")).toEqual([{ t: "wiki", target: "Tasks", label: "the list" }]);
    expect(inline("#home/garden")).toEqual([{ t: "tag", v: "home/garden" }]);
    expect(inline("@2026-10-20")).toEqual([{ t: "date", v: "2026-10-20" }]);
    expect(inline("@2026-10-20T09:30")).toEqual([{ t: "date", v: "2026-10-20T09:30" }]);
  });

  it("does not start tags, dates or underscore emphasis inside a word", () => {
    expect(plain(inline("a#b"))).toBe("a#b");
    expect(inline("a#b").every((i) => i.t === "text")).toBe(true);
    expect(inline("mail@2026-10-20").every((i) => i.t === "text")).toBe(true);
    expect(inline("snake_case_name").every((i) => i.t === "text")).toBe(true);
  });

  it("leaves unmatched markers as text", () => {
    for (const text of ["**open", "*lonely", "`tick", "[no](", "[[open", "~~half", "* spaced *", "# nothing"]) {
      expect(plain(inline(text))).toBe(text);
    }
  });

  it("never throws, and always terminates, on arbitrary text", () => {
    fc.assert(fc.property(anyText, (text) => Array.isArray(inline(text))));
  });

  it("keeps plain text as one text node", () => {
    fc.assert(
      fc.property(fc.stringMatching(/^[a-z ,.]{0,40}$/), (text) => {
        const out = inline(text);
        return text === "" ? out.length === 0 : out.length === 1 && out[0]?.t === "text";
      }),
    );
  });

  it("wrapping words in ** gives one strong node holding those words", () => {
    fc.assert(
      fc.property(fc.stringMatching(/^[a-z]{1,8}( [a-z]{1,8}){0,3}$/), (words) => {
        const out = inline(`**${words}**`);
        return out.length === 1 && out[0]?.t === "strong" && plain(out[0].c) === words;
      }),
    );
  });
});

describe("plain", () => {
  it("reads inlines as a reader would", () => {
    expect(plain(inline("**a** [b](u) [[c|d]] #e @2026-01-02 `f` <kbd>g</kbd> ~~h~~ *i*"))).toBe("a b d #e 2026-01-02 f g h i");
  });
});

describe("parse", () => {
  it("returns no blocks for empty or blank text", () => {
    expect(parse("")).toEqual([]);
    expect(parse("  \n\t\n")).toEqual([]);
  });

  it("parses headings with levels, ids and trailing hashes", () => {
    const blocks = parse("# One\n## Two *em* ##\n###### Six");
    expect(blocks.map((b) => b.t === "heading" && [b.level, b.id])).toEqual([
      [1, "one"],
      [2, "two-em"],
      [6, "six"],
    ]);
  });

  it("does not take seven hashes or a hash without a space for a heading", () => {
    expect(parse("####### seven")[0]?.t).toBe("paragraph");
    expect(parse("#tag")[0]?.t).toBe("paragraph");
  });

  it("joins the lines of a paragraph with spaces until a blank line or a block start", () => {
    const blocks = parse("one\ntwo\n\nthree\n- item");
    expect(blocks.map((b) => b.t)).toEqual(["paragraph", "paragraph", "list"]);
    expect(blocks[0]).toMatchObject({ c: [{ t: "text", v: "one two" }] });
  });

  it("parses fenced code with a language, with ~~~ fences and with no closing fence", () => {
    expect(parse("```sh\nls -la\necho hi\n```\nafter")).toEqual([
      { t: "code", lang: "sh", v: "ls -la\necho hi" },
      { t: "paragraph", c: [{ t: "text", v: "after" }] },
    ]);
    expect(parse("~~~\n# not a heading\n~~~")).toEqual([{ t: "code", lang: "", v: "# not a heading" }]);
    expect(parse("```\nnever closed\n- [ ] not a task")).toEqual([{ t: "code", lang: "", v: "never closed\n- [ ] not a task" }]);
  });

  it("closes a fence only with the same fence characters", () => {
    expect(parse("```\na\n~~~\nb\n```")).toEqual([{ t: "code", lang: "", v: "a\n~~~\nb" }]);
  });

  it("parses horizontal rules", () => {
    expect(parse("---\n***\n___\n- - -").map((b) => b.t)).toEqual(["hr", "hr", "hr", "hr"]);
  });

  it("parses quotes, joining consecutive lines", () => {
    expect(parse("> one\n> two\n>three")).toEqual([{ t: "quote", c: [{ t: "text", v: "one two three" }] }]);
  });

  it("parses bullet, ordered, nested and task lists with their source lines", () => {
    const block = parse("- a\n  - b\n    - c\n- [ ] open\n- [x] done\n- [X] shouted")[0];
    if (block?.t !== "list") throw new Error("expected a list");
    expect(block.ordered).toBe(false);
    expect(block.items.map((i) => [i.depth, i.checked, i.line])).toEqual([
      [0, null, 0],
      [1, null, 1],
      [2, null, 2],
      [0, false, 3],
      [0, true, 4],
      [0, true, 5],
    ]);
  });

  it("treats a tab as one level of indent and numbers as an ordered list", () => {
    const [block] = parse("1. one\n2) two\n\t3. three");
    if (block?.t !== "list") throw new Error("expected a list");
    expect(block.ordered).toBe(true);
    expect(block.items.map((i) => i.depth)).toEqual([0, 0, 1]);
  });

  it("records original line numbers for items that follow other blocks", () => {
    const blocks = parse("# T\n\ntext\n\n- [ ] a\n- [ ] b");
    const list = blocks.find((b) => b.t === "list");
    if (list?.t !== "list") throw new Error("expected a list");
    expect(list.items.map((i) => i.line)).toEqual([4, 5]);
  });

  it("parses tables with a header, a rule and rows, with or without edge pipes", () => {
    const [table] = parse("| A | B |\n| --- | :--: |\n| 1 | **2** |\nx | y");
    if (table?.t !== "table") throw new Error("expected a table");
    expect(table.head.map(plain)).toEqual(["A", "B"]);
    expect(table.rows.map((r) => r.map(plain))).toEqual([
      ["1", "2"],
      ["x", "y"],
    ]);
  });

  it("does not make a table of a pipe line with no rule under it", () => {
    expect(parse("a | b\nc | d")[0]?.t).toBe("paragraph");
  });

  it("treats CRLF and CR line endings like LF", () => {
    expect(parse("# A\r\n\r\ntext\rmore")).toEqual(parse("# A\n\ntext\nmore"));
  });

  it("never throws on arbitrary text", () => {
    fc.assert(fc.property(anyText, (text) => Array.isArray(parse(text))));
  });

  it("always gives list items the line they are on", () => {
    fc.assert(
      fc.property(fc.array(fc.constantFrom("- a", "  - b", "text", "", "1. n", "> q", "# h"), { maxLength: 15 }), (lines) => {
        const src = lines.join("\n");
        const rows = src.split("\n");
        for (const block of parse(src)) {
          if (block.t !== "list") continue;
          for (const item of block.items) if (!/^\s*([-*+]|\d+[.)])\s/.test(rows[item.line] ?? "")) return false;
        }
        return true;
      }),
    );
  });
});

describe("highlight", () => {
  const tones = (src: string) => highlight(src).map((l) => l.map((s) => `${s.tone}:${s.text}`));

  it("colours headings, rules, quotes and list markers", () => {
    expect(tones("# Hi *x*")).toEqual([["mark:# ", "heading:Hi ", "em:*x*"]]);
    expect(tones("---")).toEqual([["mark:---"]]);
    expect(tones("> said `c`")).toEqual([["mark:> ", "quote:said ", "code:`c`"]]);
    expect(tones("- item")).toEqual([["mark:- ", "plain:item"]]);
    expect(tones("1. item")).toEqual([["mark:1. ", "plain:item"]]);
  });

  it("colours task boxes and dims done tasks", () => {
    expect(tones("- [ ] open @2026-10-20")).toEqual([["mark:- ", "mark:[ ] ", "plain:open ", "date:@2026-10-20"]]);
    expect(tones("- [x] done")).toEqual([["mark:- ", "mark:[x] ", "done:done"]]);
  });

  it("colours every inline kind", () => {
    const kinds = highlight("`c` <kbd>k</kbd> **s** ~~d~~ [l](u) [[w]] #t @2026-01-01 *e*")[0]?.map((s) => s.tone);
    expect(kinds?.filter((t) => t !== "plain")).toEqual(["code", "code", "strong", "del", "link", "wiki", "tag", "date", "em"]);
  });

  it("colours code inside fences as code, the fences as fences, and empty lines as nothing", () => {
    expect(tones("```js\n# not\n\n```\n# yes")).toEqual([["fence:```js"], ["code:# not"], [], ["fence:```"], ["mark:# ", "heading:yes"]]);
  });

  it("colours a different kind of fence inside a block as code, since it does not close it", () => {
    expect(tones("~~~\n```\n~~~")).toEqual([["fence:~~~"], ["code:```"], ["fence:~~~"]]);
  });

  it("does not close a block on a fence that carries a language, as parse does not", () => {
    expect(tones("```\n```js\n```")).toEqual([["fence:```"], ["code:```js"], ["fence:```"]]);
  });

  it("returns one line of spans per line of source", () => {
    expect(highlight("")).toEqual([[]]);
    expect(highlight("a\n\nb")).toHaveLength(3);
  });

  it("round-trips: spans joined per line and lines joined by newlines give back the source", () => {
    fc.assert(fc.property(anyText, (src) => flat(src) === src));
  });

  it("round-trips inside fences and for task, quote and list lines", () => {
    const line = fc.oneof(
      markdownish,
      fc.constantFrom("```", "~~~", "- [ ] t @2026-10-20", "  - [X] x", "> q", "1) n", "# h #", "***"),
    );
    fc.assert(fc.property(fc.array(line, { maxLength: 12 }), (lines) => flat(lines.join("\n")) === lines.join("\n")));
  });

  it("has one line per source line and never an empty span", () => {
    fc.assert(
      fc.property(anyText, (src) => {
        const out = highlight(src);
        return out.length === src.split("\n").length && out.every((l) => l.every((s) => s.text !== ""));
      }),
    );
  });
});

describe("tasks", () => {
  it("lists checkboxes with line, state, text and due date", () => {
    const src = "# T\n- [ ] Send draft @2026-10-19\n- [x] Back up\n  1. [X] nested @2026-01-01T09:30 later\n- plain";
    expect(tasks(src)).toEqual([
      { line: 1, checked: false, text: "Send draft", due: "2026-10-19" },
      { line: 2, checked: true, text: "Back up", due: null },
      { line: 3, checked: true, text: "nested later", due: "2026-01-01" },
    ]);
  });

  it("strips markup from the text", () => {
    expect(tasks("- [ ] Call **Mum** about [[Trip]]")[0]?.text).toBe("Call Mum about Trip");
  });

  it("skips checkboxes inside code fences, then resumes", () => {
    expect(tasks("```\n- [ ] no\n```\n- [ ] yes\n~~~\n- [ ] no\n~~~").map((t) => t.line)).toEqual([3]);
  });

  it("needs a space after the closing bracket", () => {
    expect(tasks("- [ ]")).toEqual([]);
    expect(tasks("- [] x")).toEqual([]);
  });

  /** A generated note and the lines the generator itself made into real checkboxes. */
  const noteModel = fc
    .array(
      fc.oneof(
        fc.record({
          kind: fc.constant("task" as const),
          mark: fc.constantFrom(" ", "x", "X"),
          bullet: fc.constantFrom("-", "*", "+", "1.", "2)"),
          indent: fc.constantFrom("", "  ", "    "),
          body: fc.stringMatching(/^[a-z ]{0,12}$/),
        }),
        fc.record({ kind: fc.constant("item" as const), body: fc.stringMatching(/^[a-z ]{0,12}$/) }),
        fc.record({ kind: fc.constant("text" as const), body: fc.stringMatching(/^[a-z ]{0,12}$/) }),
        fc.record({
          kind: fc.constant("fence" as const),
          fence: fc.constantFrom("```", "~~~"),
          inside: fc.array(fc.constantFrom("- [ ] hidden", "- [x] hidden", "text", ""), { maxLength: 4 }),
        }),
      ),
      { maxLength: 20 },
    )
    .map((entries) => {
      const lines: string[] = [];
      const taskLines: { line: number; checked: boolean }[] = [];
      for (const e of entries) {
        if (e.kind === "task") {
          taskLines.push({ line: lines.length, checked: e.mark !== " " });
          lines.push(`${e.indent}${e.bullet} [${e.mark}] ${e.body}`);
        } else if (e.kind === "item") lines.push(`- ${e.body}`);
        else if (e.kind === "text") lines.push(e.body);
        else lines.push(e.fence, ...e.inside, e.fence);
      }
      return { src: lines.join("\n"), taskLines };
    });

  it("finds exactly the checkbox lines outside fences, in order, with their state", () => {
    fc.assert(
      fc.property(noteModel, ({ src, taskLines }) => {
        const found = tasks(src);
        return (
          found.length === taskLines.length &&
          found.every((t, i) => t.line === taskLines[i]?.line && t.checked === taskLines[i]?.checked)
        );
      }),
    );
  });

  it("agrees with parse about which lines are checkbox items, fences included", () => {
    const line = fc.constantFrom("```", "```js", "~~~", "~~~sh", "- [ ] a", "- [x] b", "  1. [ ] c", "- plain", "text", "", "# h", "> q");
    fc.assert(
      fc.property(fc.array(line, { maxLength: 14 }), (lines) => {
        const src = lines.join("\n");
        const fromParse = parse(src).flatMap((b) => (b.t === "list" ? b.items.filter((i) => i.checked !== null).map((i) => i.line) : []));
        return JSON.stringify(tasks(src).map((t) => t.line)) === JSON.stringify(fromParse);
      }),
    );
  });

  it("never throws on arbitrary text, and every task points at a line holding a checkbox", () => {
    fc.assert(
      fc.property(anyText, (src) => {
        const rows = src.split("\n");
        return tasks(src).every((t) => /\[[ xX]\]/.test(rows[t.line] ?? ""));
      }),
    );
  });
});

describe("toggleTask", () => {
  it("ticks and unticks a checkbox, keeping everything else", () => {
    const src = "# T\n  - [ ] a @2026-10-20 **b**\n- [x] c";
    const ticked = toggleTask(src, 1);
    expect(ticked).toBe("# T\n  - [x] a @2026-10-20 **b**\n- [x] c");
    expect(toggleTask(ticked, 2)).toBe("# T\n  - [x] a @2026-10-20 **b**\n- [ ] c");
  });

  it("returns the source unchanged for a line that is not a task, or does not exist", () => {
    const src = "# T\n- plain\n- [] odd";
    expect(toggleTask(src, 0)).toBe(src);
    expect(toggleTask(src, 1)).toBe(src);
    expect(toggleTask(src, 2)).toBe(src);
    expect(toggleTask(src, 99)).toBe(src);
    expect(toggleTask(src, -1)).toBe(src);
  });

  it("writes a ticked box as lower case x (an upper case X is normalised, not preserved)", () => {
    expect(toggleTask("- [X] a", 0)).toBe("- [ ] a");
    expect(toggleTask(toggleTask("- [X] a", 0), 0)).toBe("- [x] a");
  });

  const taskLine = fc
    .record({
      indent: fc.constantFrom("", " ", "  ", "\t"),
      bullet: fc.constantFrom("-", "*", "+", "7.", "12)"),
      mark: fc.constantFrom(" ", "x"),
      body: fc.stringMatching(/^[a-zA-Z0-9 @#*_\[\]-]{0,16}$/),
    })
    .map(({ indent, bullet, mark, body }) => `${indent}${bullet} [${mark}] ${body}`);
  const surrounded = fc.record({
    before: fc.array(fc.string(), { maxLength: 3 }),
    task: taskLine,
    after: fc.array(fc.string(), { maxLength: 3 }),
  });

  it("twice is the identity", () => {
    fc.assert(
      fc.property(surrounded, ({ before, task, after }) => {
        const src = [...before.map((l) => l.replace(/\n/g, " ")), task, ...after.map((l) => l.replace(/\n/g, " "))].join("\n");
        const line = before.length;
        return toggleTask(toggleTask(src, line), line) === src;
      }),
    );
  });

  it("changes one character, flips that task only, and keeps the others", () => {
    fc.assert(
      fc.property(surrounded, ({ before, task, after }) => {
        const clean = (l: string) => l.replace(/\n/g, " ");
        const lines = [...before.map(clean), task, ...after.map(clean)];
        const src = lines.join("\n");
        const line = before.length;
        const out = toggleTask(src, line);
        if (out.length !== src.length) return false;
        const diffs = [...src].filter((ch, i) => ch !== out[i]).length;
        const was = tasks(src);
        const now = tasks(out);
        const index = was.findIndex((t) => t.line === line);
        return (
          diffs === 1 &&
          was.length === now.length &&
          now.every((t, i) => t.checked === (i === index ? !was[i]?.checked : was[i]?.checked))
        );
      }),
    );
  });

  it("returns the source unchanged for any line without a checkbox", () => {
    fc.assert(
      fc.property(
        fc.array(fc.string().filter((s) => !s.includes("[")), { minLength: 1, maxLength: 6 }),
        fc.nat(8),
        (lines, line) => {
          const src = lines.join("\n");
          return toggleTask(src, line) === src;
        },
      ),
    );
  });
});

describe("headings", () => {
  it("lists heading levels, text and ids, skipping other blocks", () => {
    expect(headings("# One\ntext\n## Two *x*\n```\n# code\n```\n### Three")).toEqual([
      { level: 1, text: "One", id: "one" },
      { level: 2, text: "Two x", id: "two-x" },
      { level: 3, text: "Three", id: "three" },
    ]);
  });

  it("gives no headings for a note without any", () => {
    expect(headings("just text")).toEqual([]);
  });
});

describe("title", () => {
  it.each([
    ["# Welcome to Kanso\nbody", "Welcome to Kanso"],
    ["  First paragraph\nruns on", "First paragraph runs on"],
    ["> A quote\nthen", "A quote"],
    ["", "Untitled"],
    ["   \n  ", "Untitled"],
    ["# ", "Untitled"],
    ["- a list first", "Untitled"],
    ["```\ncode\n```", "Untitled"],
    ["---", "Untitled"],
    ["## **Bold** [[Link]] title", "Bold Link title"],
  ])("title(%j) = %j", (src, expected) => {
    expect(title(src)).toBe(expected);
  });

  it("is never empty", () => {
    fc.assert(fc.property(anyText, (src) => title(src).length > 0));
  });
});

describe("wordCount", () => {
  it.each([
    ["", 0],
    ["   \n\t", 0],
    ["one", 1],
    ["one two\nthree", 3],
    ["- [ ] a", 1],
    ["--- ***", 0],
    ["café 日本 42", 3],
  ])("wordCount(%j) = %i", (src, expected) => {
    expect(wordCount(src)).toBe(expected);
  });

  it("adds up across a whitespace join", () => {
    fc.assert(fc.property(anyText, anyText, (a, b) => wordCount(`${a} ${b}`) === wordCount(a) + wordCount(b)));
  });

  it("is never negative", () => {
    fc.assert(fc.property(anyText, (a) => wordCount(a) >= 0));
  });
});

describe("tagsOf", () => {
  it("lists tags once, in order of first appearance, from every block kind", () => {
    const src = [
      "# Title #first",
      "",
      "para #second and **#third** and [#fourth](u) #first",
      "",
      "- item #fifth/nested",
      "> quote #sixth",
      "",
      "| A #seventh | B |",
      "| --- | --- |",
      "| #eighth | c |",
    ].join("\n");
    expect(tagsOf(src)).toEqual(["first", "second", "third", "fourth", "fifth/nested", "sixth", "seventh", "eighth"]);
  });

  it("ignores hashes in code and mid-word", () => {
    expect(tagsOf("`#no` and a#no and\n```\n#no\n```")).toEqual([]);
  });

  it("returns no duplicates for any text", () => {
    fc.assert(
      fc.property(anyText, (src) => {
        const tags = tagsOf(src);
        return new Set(tags).size === tags.length;
      }),
    );
  });

  it("finds a tag written into a paragraph", () => {
    fc.assert(
      fc.property(fc.stringMatching(/^[A-Za-z][A-Za-z0-9_]{0,10}$/), (name) => tagsOf(`some words #${name} more`).includes(name)));
  });
});

describe("snippet", () => {
  it("is the text after the title heading", () => {
    expect(snippet("# Title\n\nFirst **line**.\n\nSecond.")).toBe("First line. Second.");
  });

  it("keeps the first block when it is not a heading", () => {
    expect(snippet("Plain first\n\nmore")).toBe("Plain first more");
  });

  it("summarises lists, code, tables, quotes and later headings; rules give nothing", () => {
    expect(snippet("# T\n- a\n- b\n\n```\nlet x\n```\n\n| H1 | H2 |\n| --- | --- |\n| c | d |\n\n> q\n\n## Sub\n---")).toBe(
      "a · b let x H1 · H2 q Sub",
    );
  });

  it("is empty for a lone heading or nothing", () => {
    expect(snippet("# Only")).toBe("");
    expect(snippet("")).toBe("");
  });

  it("collapses whitespace and stops at 160 characters", () => {
    const long = `# T\n\n${"word ".repeat(100)}`;
    expect(snippet(long)).toHaveLength(160);
    expect(snippet("# T\n\na   b\nc")).toBe("a b c");
  });

  it("never exceeds 160 characters or has doubled or edge whitespace", () => {
    fc.assert(
      fc.property(anyText, (src) => {
        const s = snippet(src);
        return s.length <= 160 && !/\s\s/.test(s) && s === s.trim();
      }),
    );
  });
});
