/**
 * Tests for Preview: Markdown rendered as the app shows it. Headings carry
 * ids for quick open, checkboxes call onToggleTask with their source line,
 * wiki-links call onOpenWiki, code blocks copy through the clipboard, and
 * tables, quotes, lists, tags, dates, kbd and links render as their elements.
 */
import { act, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { Inlines, Preview } from "./Preview";
import { inline } from "../../lib/markdown";

const setup = (source: string) => {
  const onToggleTask = vi.fn();
  const onOpenWiki = vi.fn();
  const view = render(<Preview source={source} onToggleTask={onToggleTask} onOpenWiki={onOpenWiki} />);
  return { onToggleTask, onOpenWiki, ...view };
};

afterEach(() => {
  vi.useRealTimers();
  vi.unstubAllGlobals();
});

describe("Preview blocks", () => {
  it("renders every heading level with an id quick open can scroll to", () => {
    setup("# One\n## Two Words\n### Three\n#### Four\n##### Five\n###### Six");
    expect(screen.getAllByRole("heading").map((h) => [h.tagName, h.id])).toEqual([
      ["H1", "note-one"],
      ["H2", "note-two-words"],
      ["H3", "note-three"],
      ["H4", "note-four"],
      ["H5", "note-five"],
      ["H6", "note-six"],
    ]);
  });

  it("renders paragraphs, quotes and rules", () => {
    const { container } = setup("Plain text\n\n> Quoted *words*\n\n---");
    expect(screen.getByText("Plain text").tagName).toBe("P");
    expect(container.querySelector("blockquote")).toHaveTextContent("Quoted words");
    expect(container.querySelector("hr")).not.toBeNull();
  });

  it("renders nothing for an empty note", () => {
    setup("");
    expect(screen.getByTestId("preview")).toBeEmptyDOMElement();
  });

  it("renders a table with its header and rows", () => {
    setup("| Mode | Shows |\n| --- | --- |\n| Edit | **Markdown** |\n| Split | Both |");
    expect(screen.getAllByRole("columnheader").map((c) => c.textContent)).toEqual(["Mode", "Shows"]);
    const rows = screen.getAllByRole("row");
    expect(rows).toHaveLength(3);
    expect(within(rows[1] as HTMLElement).getByText("Markdown").tagName).toBe("STRONG");
  });

  it("renders bullet lists with indentation and ordered lists as numbers", () => {
    const { container } = setup("- a\n  - b\n\n1. one\n2. two");
    const [bullets, numbers] = [container.querySelector("ul"), container.querySelector("ol")];
    expect(bullets?.querySelectorAll("li")).toHaveLength(2);
    expect((bullets?.querySelectorAll("li")[1] as HTMLElement).style.marginLeft).toBe("1.4rem");
    expect(numbers?.querySelectorAll("li")).toHaveLength(2);
    expect(numbers?.querySelector("span[aria-hidden]")).toBeNull();
  });
});

describe("Preview tasks", () => {
  const source = "# T\n\n- [ ] open one\n- [x] done one\n- plain";

  it("shows state through role=checkbox and aria-checked", () => {
    setup(source);
    expect(screen.getByRole("checkbox", { name: "Mark as done" })).toHaveAttribute("aria-checked", "false");
    expect(screen.getByRole("checkbox", { name: "Mark as not done" })).toHaveAttribute("aria-checked", "true");
    expect(screen.getAllByRole("checkbox")).toHaveLength(2);
  });

  it("calls onToggleTask with the source line of the box that was clicked", () => {
    const { onToggleTask } = setup(source);
    fireEvent.click(screen.getByRole("checkbox", { name: "Mark as done" }));
    expect(onToggleTask).toHaveBeenLastCalledWith(2);
    fireEvent.click(screen.getByRole("checkbox", { name: "Mark as not done" }));
    expect(onToggleTask).toHaveBeenLastCalledWith(3);
    expect(onToggleTask).toHaveBeenCalledTimes(2);
  });

  it("strikes through done tasks only", () => {
    setup(source);
    expect(screen.getByText("done one")).toHaveClass("line-through");
    expect(screen.getByText("open one")).not.toHaveClass("line-through");
  });
});

describe("Preview inlines", () => {
  it("opens a wiki-link through onOpenWiki with its target, showing its label", () => {
    const { onOpenWiki } = setup("See [[Tasks|the task list]] and [[Writing]].");
    fireEvent.click(screen.getByRole("button", { name: "the task list" }));
    expect(onOpenWiki).toHaveBeenLastCalledWith("Tasks");
    fireEvent.click(screen.getByRole("button", { name: "Writing" }));
    expect(onOpenWiki).toHaveBeenLastCalledWith("Writing");
  });

  it("opens external links in a new tab and keeps relative links in the page", () => {
    setup("[out](https://example.com/x) [mail](http://a.test) [in](/download)");
    expect(screen.getByRole("link", { name: "out" })).toHaveAttribute("target", "_blank");
    expect(screen.getByRole("link", { name: "out" })).toHaveAttribute("rel", "noreferrer");
    expect(screen.getByRole("link", { name: "mail" })).toHaveAttribute("target", "_blank");
    expect(screen.getByRole("link", { name: "in" })).not.toHaveAttribute("target");
    expect(screen.getByRole("link", { name: "in" })).toHaveAttribute("href", "/download");
  });

  it("renders strong, em, del, code, kbd, tags and dates", () => {
    const { container } = setup("**s** *e* ~~d~~ `c` <kbd>⌘P</kbd> #home/garden @2026-10-20");
    expect(container.querySelector("strong")).toHaveTextContent("s");
    expect(container.querySelector("em")).toHaveTextContent("e");
    expect(container.querySelector("del")).toHaveTextContent("d");
    expect(container.querySelector("p code")).toHaveTextContent("c");
    expect(container.querySelector("kbd")).toHaveTextContent("⌘P");
    expect(screen.getByText("#home/garden")).toBeInTheDocument();
    expect(screen.getByText("2026-10-20")).toBeInTheDocument();
  });

  it("nests inlines inside links and strong", () => {
    const { container } = setup("[a **b** *c*](https://x.test)");
    const link = screen.getByRole("link");
    expect(link.querySelector("strong")).toHaveTextContent("b");
    expect(link.querySelector("em")).toHaveTextContent("c");
    expect(container.querySelectorAll("a")).toHaveLength(1);
  });

  it("Inlines renders a list of inlines on its own", () => {
    render(<p><Inlines c={inline("x **y**")} onOpenWiki={() => undefined} /></p>);
    expect(screen.getByText("y").tagName).toBe("STRONG");
  });
});

describe("Preview code blocks", () => {
  const source = "```sh\nls -la\n```\n\n```\nplain\n```";

  it("shows the code and its language", () => {
    const { container } = setup(source);
    expect(container.querySelectorAll("pre code")[0]).toHaveTextContent("ls -la");
    expect(screen.getByText("sh")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Copy the sh" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Copy the code" })).toBeInTheDocument();
  });

  it("copies the code to the clipboard, says Copied, then goes back", async () => {
    vi.useFakeTimers();
    const writeText = vi.fn().mockResolvedValue(undefined);
    vi.stubGlobal("navigator", { ...navigator, clipboard: { writeText } });
    setup(source);
    fireEvent.click(screen.getByRole("button", { name: "Copy the sh" }));
    await act(async () => {});
    expect(writeText).toHaveBeenCalledWith("ls -la");
    expect(screen.getByRole("button", { name: "Copied" })).toBeInTheDocument();
    act(() => void vi.advanceTimersByTime(1600));
    expect(screen.getByRole("button", { name: "Copy the sh" })).toBeInTheDocument();
  });

  it("stays quiet when the clipboard refuses", async () => {
    const writeText = vi.fn().mockRejectedValue(new Error("denied"));
    vi.stubGlobal("navigator", { ...navigator, clipboard: { writeText } });
    setup(source);
    fireEvent.click(screen.getByRole("button", { name: "Copy the sh" }));
    await act(async () => {});
    expect(writeText).toHaveBeenCalled();
    expect(screen.getByRole("button", { name: "Copy the sh" })).toBeInTheDocument();
  });

  it("does nothing, without throwing, when there is no clipboard", () => {
    vi.stubGlobal("navigator", { ...navigator, clipboard: undefined });
    setup(source);
    expect(() => fireEvent.click(screen.getByRole("button", { name: "Copy the sh" }))).not.toThrow();
  });
});

describe("Preview code colours", () => {
  it("renders a known language's tokens as coloured spans and keeps the text whole", async () => {
    const code = 'fn main() {\n    let s = "hi"; // greet\n}';
    const { container } = setup("```rust\n" + code + "\n```");
    await waitFor(() => expect(container.querySelector("code .tok-keyword")).not.toBeNull());
    const el = container.querySelector("code") as HTMLElement;
    expect(el.textContent).toBe(code);
    expect(within(el).getByText("fn")).toHaveClass("tok-keyword");
    expect(within(el).getByText('"hi"')).toHaveClass("tok-string");
    expect(within(el).getByText("// greet")).toHaveClass("tok-comment");
  });

  it("leaves an unknown language uncoloured", () => {
    const { container } = setup("```klingon\nfn main() {}\n```");
    expect(container.querySelector("code [class^='tok-']")).toBeNull();
    expect(container.querySelector("code")).toHaveTextContent("fn main() {}");
  });
});
