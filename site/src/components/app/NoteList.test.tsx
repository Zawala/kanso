/**
 * Tests for NoteList: a row per note with title, snippet, time and task
 * progress, the selected row, search and new-note callbacks, the empty state,
 * and server view, where titles and text are the server's sealed forms, the
 * progress chip is hidden and new notes are disabled.
 */
import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { NoteList, type NoteListProps } from "./NoteList";
import type { Note } from "../../lib/notes";

const notes: Note[] = [
  { id: "a", notebook: "N", edited: 2, pinned: true, body: "# Alpha\n\nFirst words here.\n\n- [x] done\n- [ ] open\n- [ ] open too" },
  { id: "b", notebook: "N", edited: 120, body: "# Beta\n\nOnly prose." },
];

const setup = (over: Partial<NoteListProps> = {}) => {
  const props: NoteListProps = {
    heading: "All Notes",
    notes,
    selected: "a",
    query: "",
    server: false,
    sealed: (t) => `S[${t}]`,
    onSelect: vi.fn(),
    onQuery: vi.fn(),
    onNew: vi.fn(),
    ...over,
  };
  render(<NoteList {...props} />);
  return props;
};
const rows = () => within(screen.getByRole("list", { name: "Notes" }));

describe("NoteList", () => {
  it("shows the heading with the note count", () => {
    setup();
    expect(screen.getByRole("heading", { name: "All Notes 2" })).toBeInTheDocument();
  });

  it("shows each note's title, snippet and edited time", () => {
    setup();
    expect(rows().getByText("Alpha")).toBeInTheDocument();
    expect(rows().getByText(/First words here\./)).toBeInTheDocument();
    expect(rows().getByText("2 min ago")).toBeInTheDocument();
    expect(rows().getByText("2 h ago")).toBeInTheDocument();
  });

  it("shows task progress only for notes that have tasks", () => {
    setup();
    expect(rows().getByText("1/3")).toBeInTheDocument();
    const beta = rows().getByText("Beta").closest("button") as HTMLElement;
    expect(beta.textContent).not.toMatch(/\d\/\d/);
  });

  it("marks the selected row as current", () => {
    setup({ selected: "b" });
    const buttons = rows().getAllByRole("button");
    expect(buttons[0]).not.toHaveAttribute("aria-current");
    expect(buttons[1]).toHaveAttribute("aria-current", "true");
  });

  it("selects a note by id when its row is clicked", () => {
    const props = setup();
    fireEvent.click(rows().getByText("Beta"));
    expect(props.onSelect).toHaveBeenCalledWith("b");
  });

  it("reports the search as it is typed, showing the given query", () => {
    const props = setup({ query: "al" });
    const input = screen.getByRole("textbox", { name: "Search notes" });
    expect(input).toHaveValue("al");
    fireEvent.change(input, { target: { value: "alp" } });
    expect(props.onQuery).toHaveBeenCalledWith("alp");
  });

  it("asks for a new note", () => {
    const props = setup();
    fireEvent.click(screen.getByRole("button", { name: "New note" }));
    expect(props.onNew).toHaveBeenCalledTimes(1);
  });

  it("says so when there are no notes", () => {
    setup({ notes: [] });
    expect(screen.getByText("No notes here.")).toBeInTheDocument();
    expect(rows().queryAllByRole("button")).toHaveLength(0);
  });

  describe("in server view", () => {
    it("shows sealed titles and text instead of the real ones", () => {
      setup({ server: true });
      expect(rows().queryByText("Alpha")).toBeNull();
      expect(rows().getByText("S[Alpha]")).toBeInTheDocument();
      expect(rows().getByText(/^S\[First words here\./)).toBeInTheDocument();
    });

    it("hides the task progress chip and disables new notes", () => {
      setup({ server: true });
      expect(rows().queryByText("1/3")).toBeNull();
      expect(screen.getByRole("button", { name: "New note" })).toBeDisabled();
    });

    it("still shows when each note was edited", () => {
      setup({ server: true });
      expect(rows().getByText("2 min ago")).toBeInTheDocument();
    });
  });
});
