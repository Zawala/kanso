/**
 * Tests for Editor: caretAt (pure, with an inverse property), typing through
 * user-event with pairing, Backspace, Return in lists and Tab, keys that
 * must be left alone (modifiers, IME composition), caret reporting, and the
 * coloured layer mirroring the text exactly.
 */
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import fc from "fast-check";
import { useState } from "react";
import { describe, expect, it, vi } from "vitest";
import { type Caret, Editor, caretAt } from "./Editor";

function Harness({ initial = "", onCaret }: { initial?: string; onCaret?: (c: Caret) => void }) {
  const [value, setValue] = useState(initial);
  return onCaret ? <Editor value={value} onChange={setValue} onCaret={onCaret} label="Note" /> : <Editor value={value} onChange={setValue} label="Note" />;
}
const area = () => screen.getByRole("textbox", { name: "Note" }) as HTMLTextAreaElement;

describe("caretAt", () => {
  it.each([
    ["", 0, { line: 1, column: 1 }],
    ["abc", 0, { line: 1, column: 1 }],
    ["abc", 3, { line: 1, column: 4 }],
    ["ab\ncd", 3, { line: 2, column: 1 }],
    ["ab\ncd", 5, { line: 2, column: 3 }],
    ["ab\n\ncd", 3, { line: 2, column: 1 }],
    ["ab\n", 3, { line: 2, column: 1 }],
  ])("caretAt(%j, %i) = %j", (value, index, caret) => {
    expect(caretAt(value, index)).toEqual(caret);
  });

  it("can be turned back into the index: preceding lines plus newlines plus the column", () => {
    fc.assert(
      fc.property(fc.string(), fc.nat(), (value, n) => {
        const index = n % (value.length + 1);
        const { line, column } = caretAt(value, index);
        const before = value.split("\n").slice(0, line - 1);
        const back = before.reduce((sum, l) => sum + l.length + 1, 0) + (column - 1);
        return line >= 1 && column >= 1 && back === index;
      }),
    );
  });

  it("counts a line more for each newline before the caret", () => {
    fc.assert(
      fc.property(fc.string(), fc.nat(), (value, n) => {
        const index = n % (value.length + 1);
        return caretAt(value, index).line === value.slice(0, index).split("\n").length;
      }),
    );
  });
});

describe("Editor", () => {
  it("shows the given text in a labelled textarea", () => {
    render(<Harness initial="hello" />);
    expect(area()).toHaveValue("hello");
  });

  it("reports typing through onChange and the caret through onCaret", async () => {
    const onCaret = vi.fn();
    const user = userEvent.setup();
    render(<Harness onCaret={onCaret} />);
    await user.click(area());
    await user.keyboard("ab\ncd");
    expect(area()).toHaveValue("ab\ncd");
    expect(onCaret).toHaveBeenLastCalledWith({ line: 2, column: 3 });
  });

  it("works without an onCaret callback", async () => {
    const user = userEvent.setup();
    render(<Harness />);
    await user.click(area());
    await user.keyboard("a(b{Enter}");
    expect(area()).toHaveValue("a(b\n)");
  });

  it("pairs brackets and steps over the closer the user types", async () => {
    const user = userEvent.setup();
    render(<Harness />);
    await user.click(area());
    await user.keyboard("(");
    expect(area()).toHaveValue("()");
    expect(area().selectionStart).toBe(1);
    await user.keyboard("x)");
    expect(area()).toHaveValue("(x)");
    expect(area().selectionStart).toBe(3);
  });

  it("removes an empty pair with one Backspace, and a single character otherwise", async () => {
    const user = userEvent.setup();
    render(<Harness />);
    await user.click(area());
    await user.keyboard("[[{Backspace}");
    expect(area()).toHaveValue("");
    await user.keyboard("ab{Backspace}");
    expect(area()).toHaveValue("a");
  });

  it("does not pair a quote after a letter", async () => {
    const user = userEvent.setup();
    render(<Harness />);
    await user.click(area());
    await user.keyboard('say"');
    expect(area()).toHaveValue('say"');
  });

  it("wraps the selected text when an opener is typed over it", async () => {
    const user = userEvent.setup();
    render(<Harness initial="make this bold" />);
    area().focus();
    area().setSelectionRange(5, 9);
    await user.keyboard("[[");
    expect(area()).toHaveValue("make [this] bold");
    expect([area().selectionStart, area().selectionEnd]).toEqual([6, 10]);
  });

  it("carries a list on with Return, numbers included, and ends it on an empty item", async () => {
    const user = userEvent.setup();
    render(<Harness />);
    await user.click(area());
    await user.keyboard("1. first{Enter}second{Enter}");
    expect(area()).toHaveValue("1. first\n2. second\n3. ");
    await user.keyboard("{Enter}");
    expect(area()).toHaveValue("1. first\n2. second\n");
  });

  it("continues task items with an empty box", async () => {
    const user = userEvent.setup();
    render(<Harness />);
    await user.click(area());
    await user.keyboard("- [[x] done{Enter}");
    expect(area()).toHaveValue("- [x] done\n- [ ] ");
  });

  it("lets Return on a plain line, and Shift-Return in a list, insert a normal newline", async () => {
    const user = userEvent.setup();
    render(<Harness />);
    await user.click(area());
    await user.keyboard("plain{Enter}");
    expect(area()).toHaveValue("plain\n");
    await user.keyboard("- item{Shift>}{Enter}{/Shift}");
    expect(area()).toHaveValue("plain\n- item\n");
  });

  it("indents a list item with Tab and brings it back with Shift-Tab", async () => {
    const user = userEvent.setup();
    render(<Harness />);
    await user.click(area());
    await user.keyboard("- a{Tab}");
    expect(area()).toHaveValue("  - a");
    await user.keyboard("{Shift>}{Tab}{/Shift}");
    expect(area()).toHaveValue("- a");
    await user.keyboard("{Shift>}{Tab}{/Shift}");
    expect(area()).toHaveValue("- a");
  });

  it("leaves Tab to the browser on a line that is not a list item", async () => {
    render(<Harness initial="plain" />);
    const notPrevented = fireEvent.keyDown(area(), { key: "Tab" });
    expect(notPrevented).toBe(true);
    expect(area()).toHaveValue("plain");
  });

  it("prevents the default for keys it handles, and not for ordinary letters", () => {
    render(<Harness />);
    expect(fireEvent.keyDown(area(), { key: "(" })).toBe(false);
    expect(fireEvent.keyDown(area(), { key: "a" })).toBe(true);
    expect(fireEvent.keyDown(area(), { key: "ArrowLeft" })).toBe(true);
  });

  it("ignores keys pressed with Cmd, Ctrl or Alt, and during IME composition", () => {
    render(<Harness />);
    expect(fireEvent.keyDown(area(), { key: "(", metaKey: true })).toBe(true);
    expect(fireEvent.keyDown(area(), { key: "(", ctrlKey: true })).toBe(true);
    expect(fireEvent.keyDown(area(), { key: "(", altKey: true })).toBe(true);
    expect(fireEvent.keyDown(area(), { key: "(", isComposing: true })).toBe(true);
    expect(area()).toHaveValue("");
  });

  it("reports the caret when the selection moves or the box is clicked", () => {
    const onCaret = vi.fn();
    render(<Harness initial={"ab\ncd"} onCaret={onCaret} />);
    area().setSelectionRange(4, 4);
    fireEvent.select(area());
    expect(onCaret).toHaveBeenLastCalledWith({ line: 2, column: 2 });
    area().setSelectionRange(1, 1);
    fireEvent.click(area());
    expect(onCaret).toHaveBeenLastCalledWith({ line: 1, column: 2 });
  });

  it("keeps typing after the caret where it is after stepping over a closer (regression: a stale caret pulled it back)", async () => {
    const user = userEvent.setup();
    render(<Harness />);
    await user.click(area());
    await user.keyboard("(a) and more");
    expect(area()).toHaveValue("(a) and more");
    expect(area().selectionStart).toBe(12);
  });

  it("applies a key's result that leaves the text unchanged (stepping over) by only moving the caret", () => {
    const onChange = vi.fn();
    const onCaret = vi.fn();
    render(<Editor value="(a)" onChange={onChange} onCaret={onCaret} label="Note" />);
    area().setSelectionRange(2, 2);
    fireEvent.keyDown(area(), { key: ")" });
    expect(onChange).not.toHaveBeenCalled();
    expect(area().selectionStart).toBe(3);
    expect(onCaret).toHaveBeenLastCalledWith({ line: 1, column: 4 });
  });

  describe("the coloured layer", () => {
    const layer = (container: HTMLElement) => container.querySelector("pre") as HTMLElement;

    it("is hidden from assistive technology and colours the source by tone", () => {
      const { container } = render(<Harness initial={"# Title\n- [x] done **b** #tag"} />);
      expect(layer(container)).toHaveAttribute("aria-hidden", "true");
      expect(layer(container).querySelector(".tone-heading")).toHaveTextContent("Title");
      expect(layer(container).querySelector(".tone-strong")).toHaveTextContent("**b**");
      expect(layer(container).querySelector(".tone-tag")).toHaveTextContent("#tag");
      expect(layer(container).querySelector(".tone-done")).not.toBeNull();
    });

    it("colours code inside a fence with a known language and still mirrors the text exactly", async () => {
      const value = '# Code\n\n```rust\nfn main() {\n    let s = "hi"; // greet\n}\n```\n\nafter';
      const { container } = render(<Editor value={value} onChange={() => undefined} label="Note" />);
      await waitFor(() => expect(layer(container).querySelector(".tok-keyword")).not.toBeNull());
      expect(layer(container).querySelector(".tok-string")).toHaveTextContent('"hi"');
      expect(layer(container).querySelector(".tok-comment")).toHaveTextContent("// greet");
      expect(layer(container).textContent).toBe(`${value}\n `);
    });

    it("follows the text as it is typed", async () => {
      const user = userEvent.setup();
      const { container } = render(<Harness />);
      await user.click(area());
      await user.keyboard("# Hi");
      expect(layer(container).querySelector(".tone-heading")).toHaveTextContent("Hi");
    });

    it("mirrors the value exactly, plus a closing line so a trailing newline takes space", () => {
      const { container, rerender } = render(<Editor value="" onChange={() => undefined} label="Note" />);
      fc.assert(
        fc.property(fc.oneof(fc.string(), fc.string({ unit: "grapheme" })), (value) => {
          act(() => rerender(<Editor value={value} onChange={() => undefined} label="Note" />));
          return layer(container).textContent === `${value}\n `;
        }),
        { numRuns: 60 },
      );
    });
  });
});
