/**
 * Syntax highlighting for fenced code. `tokenize` turns source into
 * `{ text, scope }` spans, never HTML, and the spans' text always joins back
 * to the source exactly, so the editor's coloured copy can't drift from its
 * textarea. highlight.js is fetched on first use; until it arrives (or for a
 * language it doesn't know) the source is one plain span.
 */

export interface Token {
  readonly text: string;
  /** highlight.js's scope for the text, such as `keyword` or `title.function`; null when plain. */
  readonly scope: string | null;
}

type Hljs = (typeof import("./highlightRuntime"))["hljs"];

/** Fence languages, as people write them, to the highlight.js language. */
const LANGUAGES: Readonly<Record<string, string>> = {
  javascript: "javascript",
  js: "javascript",
  jsx: "javascript",
  mjs: "javascript",
  cjs: "javascript",
  typescript: "typescript",
  ts: "typescript",
  tsx: "typescript",
  rust: "rust",
  rs: "rust",
  python: "python",
  py: "python",
  bash: "bash",
  sh: "bash",
  zsh: "bash",
  shell: "bash",
  json: "json",
  css: "css",
  xml: "xml",
  html: "xml",
  go: "go",
  golang: "go",
  swift: "swift",
  sql: "sql",
  yaml: "yaml",
  yml: "yaml",
  diff: "diff",
  markdown: "markdown",
  md: "markdown",
};

/** Past this, colouring each keystroke isn't worth it. */
const MAX_LENGTH = 40_000;

/** The highlight.js language a fence's language names, or null when it isn't one we colour. */
export function languageFor(lang: string): string | null {
  const key = lang.trim().toLowerCase();
  return Object.hasOwn(LANGUAGES, key) ? (LANGUAGES[key] as string) : null;
}

let runtime: Hljs | null = null;
let loading: Promise<void> | null = null;
const listeners = new Set<() => void>();

/** Fetches highlight.js once; resolves when `tokenize` can colour. */
export function loadHighlighter(): Promise<void> {
  loading ??= import("./highlightRuntime").then(
    (m) => {
      runtime = m.hljs;
      for (const listener of listeners) listener();
    },
    () => undefined,
  );
  return loading;
}

export function isHighlighterReady(): boolean {
  return runtime !== null;
}

export function subscribeHighlighter(listener: () => void): () => void {
  listeners.add(listener);
  return () => void listeners.delete(listener);
}

const ENTITIES: Readonly<Record<string, string>> = { amp: "&", lt: "<", gt: ">", quot: '"', "#x27": "'", "#39": "'" };
const MARKUP = /<span class="([^"]*)">|<\/span>|&(amp|lt|gt|quot|#x27|#39);|[^<&]+|[<&]/g;

/** highlight.js's classes (`hljs-title function_`) as one scope (`title.function`). */
function scopeOf(classes: string): string | null {
  const parts = classes
    .split(/\s+/)
    .filter(Boolean)
    .map((c) => (c.startsWith("hljs-") ? c.slice(5) : c.replace(/_$/, "")));
  return parts.length === 0 ? null : parts.join(".");
}

/** Reads highlight.js's markup (spans and entities only) into tokens. */
function readMarkup(markup: string): Token[] {
  const out: Token[] = [];
  const stack: (string | null)[] = [];
  const push = (text: string) => {
    const scope = stack.length === 0 ? null : (stack[stack.length - 1] ?? null);
    const last = out[out.length - 1];
    if (last && last.scope === scope) out[out.length - 1] = { text: last.text + text, scope };
    else out.push({ text, scope });
  };
  for (const m of markup.matchAll(MARKUP)) {
    const piece = m[0];
    if (m[1] !== undefined) stack.push(scopeOf(m[1]) ?? stack[stack.length - 1] ?? null);
    else if (piece === "</span>") stack.pop();
    else if (m[2] !== undefined) push(ENTITIES[m[2]] as string);
    else push(piece);
  }
  return out;
}

/** The source as tokens whose text joins back to it exactly. */
export function tokenize(source: string, lang: string): Token[] {
  const plain: Token[] = [{ text: source, scope: null }];
  const language = languageFor(lang);
  if (runtime === null || language === null || source === "" || source.length > MAX_LENGTH) return plain;
  try {
    const tokens = readMarkup(runtime.highlight(source, { language, ignoreIllegals: true }).value);
    return tokens.map((t) => t.text).join("") === source ? tokens : plain;
  } catch {
    return plain;
  }
}

const KEYWORD = new Set(["keyword", "literal", "selector-tag", "name", "bullet"]);
const STRING = new Set(["string", "regexp", "char", "symbol", "code", "quote"]);
const ATTR = new Set(["attr", "attribute", "property", "selector-class", "selector-id", "selector-attr", "selector-pseudo", "link"]);
const BUILTIN = new Set(["built_in", "meta", "template-variable", "doctag"]);

/** The colour class for a scope, or undefined to leave the text as it is. */
export function scopeClass(scope: string | null | undefined): string | undefined {
  if (!scope) return undefined;
  const head = scope.split(".")[0] as string;
  if (scope.startsWith("title.class") || head === "type" || head === "class") return "tok-type";
  if (head === "title" || head === "section") return "tok-title";
  if (head === "comment") return "tok-comment";
  if (head === "number") return "tok-number";
  if (head === "deletion") return "tok-del";
  if (head === "addition") return "tok-add";
  if (STRING.has(head) || (head === "meta" && scope.includes("string"))) return "tok-string";
  if (KEYWORD.has(head)) return "tok-keyword";
  if (ATTR.has(head)) return "tok-attr";
  if (BUILTIN.has(head) || scope === "variable.language" || scope === "variable.constant") return "tok-builtin";
  if (head === "punctuation" || head === "operator" || head === "tag") return "tok-punct";
  return undefined;
}
