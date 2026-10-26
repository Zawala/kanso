/**
 * Tests for codeHighlight: tokens are `{ text, scope }` spans whose text
 * always joins back to the source, known languages get scopes, and unknown
 * ones stay plain. Round-trip properties run with fast-check's pinned seed
 * (src/test/setup.ts).
 */
import fc from "fast-check";
import { beforeAll, describe, expect, it } from "vitest";
import { languageFor, loadHighlighter, scopeClass, tokenize } from "./codeHighlight";
import { highlight } from "./markdown";

const LANGS = ["js", "ts", "rs", "py", "bash", "sh", "zsh", "shell", "json", "css", "xml", "html", "go", "golang", "swift", "sql", "yaml", "yml", "diff", "markdown", "md", "", "nope", "RUST"];

const join = (source: string, lang: string) =>
  tokenize(source, lang)
    .map((t) => t.text)
    .join("");

beforeAll(async () => {
  await loadHighlighter();
});

describe("tokenize", () => {
  it("gives a Rust snippet keyword and string scopes", () => {
    const tokens = tokenize('fn main() {\n    let s = "hi"; // greet\n}', "rust");
    const scopes = (scope: string) => tokens.filter((t) => t.scope === scope).map((t) => t.text);
    expect(scopes("keyword")).toEqual(expect.arrayContaining(["fn", "let"]));
    expect(scopes("string")).toContain('"hi"');
    expect(scopes("comment")).toContain("// greet");
    expect(tokens.map((t) => t.text).join("")).toBe('fn main() {\n    let s = "hi"; // greet\n}');
  });

  it("maps aliases to their language", () => {
    for (const [alias, language] of [["js", "javascript"], ["ts", "typescript"], ["rs", "rust"], ["py", "python"], ["yml", "yaml"], ["golang", "go"], ["md", "markdown"], ["zsh", "bash"], ["shell", "bash"], ["sh", "bash"], ["html", "xml"]]) {
      expect(languageFor(alias as string)).toBe(language);
    }
    expect(languageFor("klingon")).toBeNull();
    expect(languageFor("")).toBeNull();
    expect(languageFor("constructor")).toBeNull();
  });

  it("leaves an unknown or empty language as one plain span", () => {
    expect(tokenize("let x = 1;", "klingon")).toEqual([{ text: "let x = 1;", scope: null }]);
    expect(tokenize("let x = 1;", "")).toEqual([{ text: "let x = 1;", scope: null }]);
  });

  it("colours every supported language", () => {
    const samples: Record<string, string> = {
      js: "const a = 1;",
      ts: "const a: number = 1;",
      rs: "let a = 1;",
      py: "def f():\n    return 1",
      bash: "echo \"hi\" # x",
      json: '{"a": 1}',
      css: "a { color: red; }",
      xml: '<a href="x">y</a>',
      go: "func main() {}",
      swift: "let a = 1",
      sql: "SELECT 1 FROM t;",
      yaml: "a: 1",
      diff: "+ added\n- removed",
      md: "# Title\n*em*",
    };
    for (const [lang, source] of Object.entries(samples)) {
      const tokens = tokenize(source, lang);
      expect(tokens.some((t) => t.scope !== null), lang).toBe(true);
      expect(tokens.map((t) => t.text).join(""), lang).toBe(source);
    }
  });

  it("maps scopes to colour classes", () => {
    expect(scopeClass("keyword")).toBe("tok-keyword");
    expect(scopeClass("title.function")).toBe("tok-title");
    expect(scopeClass("title.class")).toBe("tok-type");
    expect(scopeClass("comment")).toBe("tok-comment");
    expect(scopeClass(null)).toBeUndefined();
    expect(scopeClass("subst")).toBeUndefined();
  });
});

describe("round trip", () => {
  it("joins back to any string, in any language", () => {
    fc.assert(
      fc.property(fc.oneof(fc.string(), fc.string({ unit: "binary" }), fc.constantFrom("a < b && c > d", "\"unclosed", "/* open", "<!-- x", "`${x}", "&amp; &lt;")), fc.constantFrom(...LANGS), (source, lang) => {
        expect(join(source, lang)).toBe(source);
      }),
    );
  });

  it("joins back for code-looking text", () => {
    const piece = fc.constantFrom("fn ", "let ", "const ", "{", "}", "(", ")", "\"", "'", "//", "/*", "*/", "#", "<", ">", "&", "\n", " ", "1", "x", "=>", "<div>", "</div>", "--", "\\");
    fc.assert(
      fc.property(fc.array(piece, { maxLength: 40 }).map((p) => p.join("")), fc.constantFrom(...LANGS), (source, lang) => {
        expect(join(source, lang)).toBe(source);
      }),
    );
  });

  it("keeps the editor's text identical to the source for documents with fences", () => {
    const line = fc.oneof(
      fc.string({ maxLength: 20 }),
      fc.constantFrom("```", "~~~", "```rust", "```ts", "~~~py", "```nope", "fn main() {}", "let x = \"s\"; // c", "# heading", "- [ ] task", "> quote", ""),
    );
    fc.assert(
      fc.property(fc.array(line, { maxLength: 14 }).map((l) => l.join("\n")), (source) => {
        const text = highlight(source)
          .map((spans) => spans.map((s) => s.text).join(""))
          .join("\n");
        expect(text).toBe(source);
      }),
    );
  });
});

describe("highlight in the editor", () => {
  it("colours lines inside a fence with a known language, and not an unknown one", () => {
    const rust = highlight("```rust\nfn main() {}\n```")[1] as { scope?: string; text: string }[];
    expect(rust.some((s) => s.scope === "keyword" && s.text === "fn")).toBe(true);
    const other = highlight("```nope\nfn main() {}\n```")[1] as { tone: string; text: string }[];
    expect(other).toEqual([{ tone: "code", text: "fn main() {}" }]);
  });

  it("carries a multi-line comment across lines", () => {
    const lines = highlight("```rust\n/* a\n b */\n```");
    expect(lines[1]?.[0]?.scope).toBe("comment");
    expect(lines[2]?.[0]?.scope).toBe("comment");
  });
});
