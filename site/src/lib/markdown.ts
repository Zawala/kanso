/**
 * The Markdown the in-browser Kanso understands: enough of the app's dialect
 * to render its own notes (headings, lists, tasks with due dates, quotes,
 * code, tables, links, [[wiki-links]], #tags) and to colour the source the
 * way the app's editor does. Parsing never throws: anything it doesn't
 * recognise is text.
 */

import { isHighlighterReady, languageFor, tokenize } from "./codeHighlight";

export type Inline =
  | { readonly t: "text"; readonly v: string }
  | { readonly t: "code"; readonly v: string }
  | { readonly t: "strong" | "em" | "del"; readonly c: readonly Inline[] }
  | { readonly t: "link"; readonly href: string; readonly c: readonly Inline[] }
  | { readonly t: "wiki"; readonly target: string; readonly label: string }
  | { readonly t: "tag"; readonly v: string }
  | { readonly t: "date"; readonly v: string }
  | { readonly t: "kbd"; readonly v: string };

export interface ListItem {
  readonly depth: number;
  /** Null when the item isn't a task. */
  readonly checked: boolean | null;
  /** The source line the item is on, so ticking it can change the note. */
  readonly line: number;
  readonly c: readonly Inline[];
}

export type Block =
  | { readonly t: "heading"; readonly level: number; readonly id: string; readonly c: readonly Inline[] }
  | { readonly t: "paragraph"; readonly c: readonly Inline[] }
  | { readonly t: "list"; readonly ordered: boolean; readonly items: readonly ListItem[] }
  | { readonly t: "quote"; readonly c: readonly Inline[] }
  | { readonly t: "code"; readonly lang: string; readonly v: string }
  | { readonly t: "table"; readonly head: readonly (readonly Inline[])[]; readonly rows: readonly (readonly (readonly Inline[])[])[] }
  | { readonly t: "hr" };

const HEADING = /^(#{1,6})\s+(.*?)\s*#*\s*$/;
const FENCE = /^\s*(```|~~~)\s*([\w+-]*)\s*$/;
const HR = /^\s*([-*_])(\s*\1){2,}\s*$/;
const QUOTE = /^\s*>\s?(.*)$/;
const ITEM = /^(\s*)([-*+]|\d{1,9}[.)])\s+(?:\[( |x|X)\]\s+)?(.*)$/;
const TABLE_RULE = /^\s*\|?\s*:?-{2,}:?\s*(\|\s*:?-{2,}:?\s*)*\|?\s*$/;

/** A heading's anchor: lower case, words joined by hyphens. */
export function slug(text: string): string {
  return text
    .toLowerCase()
    .replace(/[^\p{L}\p{N}\s-]/gu, "")
    .trim()
    .replace(/\s+/g, "-");
}

/** The plain text of some inlines, as a reader would see it. */
export function plain(inlines: readonly Inline[]): string {
  return inlines
    .map((i) => {
      switch (i.t) {
        case "text":
        case "code":
        case "kbd":
          return i.v;
        case "tag":
          return `#${i.v}`;
        case "date":
          return i.v;
        case "wiki":
          return i.label;
        default:
          return plain(i.c);
      }
    })
    .join("");
}

/** A fence closes a block only when it is the same kind as the opener and carries no language, as in `parse`. */
function closesFence(fence: RegExpExecArray | null, opener: string): boolean {
  return fence !== null && fence[1] === opener && fence[2] === "";
}

function isStart(line: string): boolean {
  return HEADING.test(line) || FENCE.test(line) || HR.test(line) || QUOTE.test(line) || ITEM.test(line);
}

function cells(line: string): string[] {
  const inner = line.trim().replace(/^\|/, "").replace(/\|$/, "");
  return inner.split("|").map((c) => c.trim());
}

/** The blocks of a note. */
export function parse(source: string): Block[] {
  const lines = source.replace(/\r\n?/g, "\n").split("\n");
  const blocks: Block[] = [];
  let i = 0;
  while (i < lines.length) {
    const line = lines[i] as string;
    if (line.trim() === "") {
      i++;
      continue;
    }
    const fence = FENCE.exec(line);
    if (fence) {
      const close = fence[1] as string;
      const body: string[] = [];
      i++;
      while (i < lines.length && (lines[i] as string).trim() !== close) body.push(lines[i++] as string);
      i++;
      blocks.push({ t: "code", lang: fence[2] ?? "", v: body.join("\n") });
      continue;
    }
    const heading = HEADING.exec(line);
    if (heading) {
      const c = inline(heading[2] ?? "");
      blocks.push({ t: "heading", level: (heading[1] as string).length, id: slug(plain(c)), c });
      i++;
      continue;
    }
    if (HR.test(line)) {
      blocks.push({ t: "hr" });
      i++;
      continue;
    }
    if (QUOTE.test(line)) {
      const body: string[] = [];
      while (i < lines.length) {
        const q = QUOTE.exec(lines[i] as string);
        if (!q) break;
        body.push(q[1] ?? "");
        i++;
      }
      blocks.push({ t: "quote", c: inline(body.join(" ").trim()) });
      continue;
    }
    if (ITEM.test(line)) {
      const items: ListItem[] = [];
      const ordered = /^\s*\d/.test(line);
      while (i < lines.length) {
        const m = ITEM.exec(lines[i] as string);
        if (!m) break;
        const indent = (m[1] ?? "").replace(/\t/g, "  ").length;
        items.push({
          depth: Math.floor(indent / 2),
          checked: m[3] === undefined ? null : m[3] !== " ",
          line: i,
          c: inline(m[4] ?? ""),
        });
        i++;
      }
      blocks.push({ t: "list", ordered, items });
      continue;
    }
    const next = lines[i + 1];
    if (line.includes("|") && next !== undefined && TABLE_RULE.test(next)) {
      const head = cells(line).map(inline);
      const rows: Inline[][][] = [];
      i += 2;
      while (i < lines.length && (lines[i] as string).includes("|") && (lines[i] as string).trim() !== "") {
        rows.push(cells(lines[i] as string).map(inline));
        i++;
      }
      blocks.push({ t: "table", head, rows });
      continue;
    }
    const body: string[] = [line.trim()];
    i++;
    while (i < lines.length && (lines[i] as string).trim() !== "" && !isStart(lines[i] as string)) {
      body.push((lines[i] as string).trim());
      i++;
    }
    blocks.push({ t: "paragraph", c: inline(body.join(" ")) });
  }
  return blocks;
}

type Rule = {
  readonly re: RegExp;
  /** Only where the character before isn't part of a word. */
  readonly boundary?: boolean;
  readonly make: (m: RegExpExecArray) => Inline;
};

const RULES: readonly Rule[] = [
  { re: /`([^`\n]+)`/y, make: (m) => ({ t: "code", v: m[1] as string }) },
  { re: /<kbd>([^<\n]+)<\/kbd>/y, make: (m) => ({ t: "kbd", v: m[1] as string }) },
  {
    re: /\[\[([^\]|\n]+)(?:\|([^\]\n]+))?\]\]/y,
    make: (m) => ({ t: "wiki", target: (m[1] as string).trim(), label: (m[2] ?? (m[1] as string)).trim() }),
  },
  { re: /\[([^\]\n]+)\]\(([^)\s]+)\)/y, make: (m) => ({ t: "link", href: m[2] as string, c: inline(m[1] as string) }) },
  { re: /\*\*(?=\S)([^\n]*?\S)\*\*/y, make: (m) => ({ t: "strong", c: inline(m[1] as string) }) },
  { re: /~~(?=\S)([^\n]*?\S)~~/y, make: (m) => ({ t: "del", c: inline(m[1] as string) }) },
  { re: /\*(?=[^\s*])([^*\n]*?[^\s*])\*/y, make: (m) => ({ t: "em", c: inline(m[1] as string) }) },
  { re: /_(?=[^\s_])([^_\n]*?[^\s_])_(?![\p{L}\p{N}])/uy, boundary: true, make: (m) => ({ t: "em", c: inline(m[1] as string) }) },
  { re: /#([A-Za-z][\w/-]*)/y, boundary: true, make: (m) => ({ t: "tag", v: m[1] as string }) },
  { re: /@(\d{4}-\d{2}-\d{2}(?:T\d{2}:\d{2})?)/y, boundary: true, make: (m) => ({ t: "date", v: m[1] as string }) },
];

const WORD = /[\p{L}\p{N}_]/u;

/** The inlines of one line or paragraph of text. */
export function inline(text: string): Inline[] {
  const out: Inline[] = [];
  let buffer = "";
  let i = 0;
  outer: while (i < text.length) {
    for (const rule of RULES) {
      if (rule.boundary && i > 0 && WORD.test(text[i - 1] as string)) continue;
      rule.re.lastIndex = i;
      const m = rule.re.exec(text);
      if (m) {
        // Read the end first: make() parses the inside with these same
        // regexes, which moves their lastIndex.
        const end = rule.re.lastIndex;
        if (buffer) out.push({ t: "text", v: buffer });
        buffer = "";
        out.push(rule.make(m));
        i = end;
        continue outer;
      }
    }
    buffer += text[i];
    i++;
  }
  if (buffer) out.push({ t: "text", v: buffer });
  return out;
}

/** How a stretch of source text is coloured in the editor. */
export type Tone =
  | "plain"
  | "mark"
  | "heading"
  | "code"
  | "fence"
  | "strong"
  | "em"
  | "del"
  | "link"
  | "wiki"
  | "tag"
  | "date"
  | "quote"
  | "done";

export interface Span {
  readonly tone: Tone;
  readonly text: string;
  /** Inside a fence with a known language: the code token's scope (see codeHighlight.ts). */
  readonly scope?: string;
}

const SOURCE_TONES: Readonly<Record<Inline["t"], Tone>> = {
  text: "plain",
  code: "code",
  kbd: "code",
  strong: "strong",
  em: "em",
  del: "del",
  link: "link",
  wiki: "wiki",
  tag: "tag",
  date: "date",
};

function inlineSpans(text: string, base: Tone): Span[] {
  const spans: Span[] = [];
  let buffer = "";
  let i = 0;
  outer: while (i < text.length) {
    for (const rule of RULES) {
      if (rule.boundary && i > 0 && WORD.test(text[i - 1] as string)) continue;
      rule.re.lastIndex = i;
      const m = rule.re.exec(text);
      if (m) {
        const end = rule.re.lastIndex;
        if (buffer) spans.push({ tone: base, text: buffer });
        buffer = "";
        spans.push({ tone: SOURCE_TONES[rule.make(m).t], text: m[0] });
        i = end;
        continue outer;
      }
    }
    buffer += text[i];
    i++;
  }
  if (buffer) spans.push({ tone: base, text: buffer });
  return spans;
}

/** The lines of a fence's body as spans coloured as code; null when its language isn't one we colour (or isn't loaded yet). */
function codeLines(body: string, lang: string): Span[][] | null {
  if (languageFor(lang) === null || !isHighlighterReady()) return null;
  const out: Span[][] = [[]];
  for (const token of tokenize(body, lang)) {
    token.text.split("\n").forEach((piece, n) => {
      if (n > 0) out.push([]);
      if (piece === "") return;
      (out[out.length - 1] as Span[]).push(token.scope ? { tone: "plain", text: piece, scope: token.scope } : { tone: "plain", text: piece });
    });
  }
  return out;
}

/**
 * The source, line by line, as coloured spans. Joining every span's text,
 * and the lines with newlines, gives back the source exactly.
 */
export function highlight(source: string): Span[][] {
  const lines = source.split("\n");
  let fenced: string | null = null;
  let body: Span[][] | null = null;
  let bodyStart = 0;
  return lines.map((line, index) => {
    const fence = FENCE.exec(line);
    if (fenced !== null) {
      const closes = closesFence(fence, fenced);
      if (closes) fenced = null;
      else if (body?.[index - bodyStart]) return body[index - bodyStart] as Span[];
      return line ? [{ tone: closes ? "fence" : "code", text: line }] : [];
    }
    if (fence) {
      fenced = fence[1] as string;
      let end = index + 1;
      while (end < lines.length && !closesFence(FENCE.exec(lines[end] as string), fenced)) end++;
      bodyStart = index + 1;
      body = codeLines(lines.slice(bodyStart, end).join("\n"), fence[2] ?? "");
      return [{ tone: "fence", text: line }];
    }
    if (line === "") return [];
    const heading = /^(#{1,6}\s+)(.*)$/.exec(line);
    if (heading) return [{ tone: "mark", text: heading[1] as string }, ...inlineSpans(heading[2] as string, "heading")];
    if (HR.test(line)) return [{ tone: "mark", text: line }];
    const quote = /^(\s*>\s?)(.*)$/.exec(line);
    if (quote) return [{ tone: "mark", text: quote[1] as string }, ...inlineSpans(quote[2] as string, "quote")];
    const item = /^(\s*(?:[-*+]|\d{1,9}[.)])\s+)(\[[ xX]\]\s+)?(.*)$/.exec(line);
    if (item) {
      const box = item[2];
      const done = box !== undefined && box[1] !== " ";
      const spans: Span[] = [{ tone: "mark", text: item[1] as string }];
      if (box !== undefined) spans.push({ tone: "mark", text: box });
      return [...spans, ...inlineSpans(item[3] as string, done ? "done" : "plain")];
    }
    return inlineSpans(line, "plain");
  });
}

export interface Task {
  readonly line: number;
  readonly checked: boolean;
  readonly text: string;
  /** The task's due date, `YYYY-MM-DD`, when it has one. */
  readonly due: string | null;
}

const TASK = /^(\s*(?:[-*+]|\d{1,9}[.)])\s+\[)( |x|X)(\]\s+)(.*)$/;

/** Every checkbox in a note, in order, outside code blocks. */
export function tasks(source: string): Task[] {
  const out: Task[] = [];
  let fenced: string | null = null;
  source.split("\n").forEach((line, index) => {
    const fence = FENCE.exec(line);
    if (fenced !== null) {
      if (closesFence(fence, fenced)) fenced = null;
      return;
    }
    if (fence) {
      fenced = fence[1] as string;
      return;
    }
    const m = TASK.exec(line);
    if (!m) return;
    const body = m[4] as string;
    const due = /(?:^|\s)@(\d{4}-\d{2}-\d{2})/.exec(body);
    out.push({
      line: index,
      checked: m[2] !== " ",
      text: plain(inline(body.replace(/(?:^|\s)@\d{4}-\d{2}-\d{2}(?:T\d{2}:\d{2})?/g, "").trim())),
      due: due ? (due[1] as string) : null,
    });
  });
  return out;
}

/** The note with the checkbox on `line` ticked or unticked; unchanged when that line has none. */
export function toggleTask(source: string, line: number): string {
  const lines = source.split("\n");
  const target = lines[line];
  if (target === undefined) return source;
  const m = TASK.exec(target);
  if (!m) return source;
  lines[line] = `${m[1]}${m[2] === " " ? "x" : " "}${m[3]}${m[4]}`;
  return lines.join("\n");
}

/** The note's headings, for quick open and the outline. */
export function headings(source: string): { level: number; text: string; id: string }[] {
  return parse(source).flatMap((b) => (b.t === "heading" ? [{ level: b.level, text: plain(b.c), id: b.id }] : []));
}

/** The note's title: its first heading, or its first paragraph. */
export function title(source: string): string {
  const first = parse(source)[0];
  if (first === undefined) return "Untitled";
  if (first.t === "heading" || first.t === "paragraph" || first.t === "quote") return plain(first.c).trim() || "Untitled";
  return "Untitled";
}

/** Words, the way the app's footer counts them. */
export function wordCount(source: string): number {
  return source.split(/\s+/).filter((w) => /[\p{L}\p{N}]/u.test(w)).length;
}

function walkTags(inlines: readonly Inline[], out: Set<string>): void {
  for (const i of inlines) {
    if (i.t === "tag") out.add(i.v);
    else if (i.t === "strong" || i.t === "em" || i.t === "del" || i.t === "link") walkTags(i.c, out);
  }
}

/** The note's #tags, in the order they first appear. */
export function tagsOf(source: string): string[] {
  const out = new Set<string>();
  for (const block of parse(source)) {
    if (block.t === "heading" || block.t === "paragraph" || block.t === "quote") walkTags(block.c, out);
    else if (block.t === "list") for (const item of block.items) walkTags(item.c, out);
    else if (block.t === "table") for (const row of [block.head, ...block.rows]) for (const cell of row) walkTags(cell, out);
  }
  return [...out];
}

/** A line or two of the note after its title, for the note list. */
export function snippet(source: string): string {
  const blocks = parse(source);
  const rest = blocks[0]?.t === "heading" ? blocks.slice(1) : blocks;
  const text = rest
    .map((b) => {
      switch (b.t) {
        case "heading":
        case "paragraph":
        case "quote":
          return plain(b.c);
        case "list":
          return b.items.map((i) => plain(i.c)).join(" · ");
        case "code":
          return b.v;
        case "table":
          return b.head.map(plain).join(" · ");
        default:
          return "";
      }
    })
    .filter(Boolean)
    .join(" ");
  return text.replace(/\s+/g, " ").trim().slice(0, 160);
}
