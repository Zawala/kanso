/**
 * Tests for TasksView: which group a task falls in (groupOf, with a
 * property for its ordering rules), collecting task rows from notes,
 * the grouped list, friendly due dates, and the toggle and open callbacks.
 */
import { fireEvent, render, screen, within } from "@testing-library/react";
import fc from "fast-check";
import { describe, expect, it, vi } from "vitest";
import { GROUPS, TasksView, groupOf, taskRows } from "./TasksView";
import { type Note, dayFrom } from "../../lib/notes";

const TODAY = new Date(2026, 9, 20);
const day = (n: number) => dayFrom(TODAY, n);

const notes: Note[] = [
  {
    id: "a",
    notebook: "N",
    edited: 1,
    body: `# Alpha\n\n- [ ] late @${day(-3)}\n- [ ] yesterday @${day(-1)}\n- [ ] now @${day(0)}\n- [ ] soon @${day(1)}\n- [ ] later @${day(5)}\n- [ ] whenever\n- [x] finished @${day(-9)}`,
  },
  { id: "b", notebook: "N", edited: 2, body: "# Beta\n\nno tasks here" },
  { id: "c", notebook: "N", edited: 3, body: "# Gamma\n\n- [ ] gamma task" },
];

describe("groupOf", () => {
  const t = day(0);
  it.each([
    [{ due: null, checked: true }, "Done"],
    [{ due: day(-5), checked: true }, "Done"],
    [{ due: null, checked: false }, "No date"],
    [{ due: day(-1), checked: false }, "Overdue"],
    [{ due: day(0), checked: false }, "Today"],
    [{ due: day(1), checked: false }, "Upcoming"],
  ] as const)("%j is %s", (task, group) => {
    expect(groupOf(task, t)).toBe(group);
  });

  it("covers every group, in the order the view lists them", () => {
    expect(GROUPS).toEqual(["Overdue", "Today", "Upcoming", "No date", "Done"]);
  });

  it("puts a ticked task in Done, and an open dated task by comparing its date with today", () => {
    const iso = fc.integer({ min: -3000, max: 3000 }).map((n) => dayFrom(TODAY, n));
    fc.assert(
      fc.property(iso, iso, fc.boolean(), (due, now, checked) => {
        const g = groupOf({ due, checked }, now);
        if (checked) return g === "Done";
        return due < now ? g === "Overdue" : due === now ? g === "Today" : g === "Upcoming";
      }),
    );
    fc.assert(fc.property(iso, (now) => groupOf({ due: null, checked: false }, now) === "No date"));
  });
});

describe("taskRows", () => {
  it("gathers every task from every note with its note's title and id", () => {
    const rows = taskRows(notes);
    expect(rows).toHaveLength(8);
    expect(rows.slice(0, 2).map((r) => [r.id, r.note, r.line, r.text])).toEqual([
      ["a", "Alpha", 2, "late"],
      ["a", "Alpha", 3, "yesterday"],
    ]);
    expect(rows.at(-1)).toMatchObject({ id: "c", note: "Gamma", text: "gamma task", due: null, checked: false });
  });

  it("is empty for notes without tasks", () => {
    expect(taskRows([notes[1] as Note])).toEqual([]);
    expect(taskRows([])).toEqual([]);
  });
});

describe("TasksView", () => {
  const setup = (list: readonly Note[] = notes) => {
    const onToggle = vi.fn();
    const onOpen = vi.fn();
    render(<TasksView notes={list} today={TODAY} onToggle={onToggle} onOpen={onOpen} />);
    return { onToggle, onOpen };
  };

  it("lists the groups that have tasks, in order, with counts", () => {
    setup();
    expect(screen.getAllByRole("heading", { level: 3 }).map((h) => h.textContent)).toEqual(["Overdue 2", "Today 1", "Upcoming 2", "No date 2", "Done 1"]);
  });

  it("leaves out groups with no tasks", () => {
    setup([notes[2] as Note]);
    expect(screen.getAllByRole("heading", { level: 3 }).map((h) => h.textContent)).toEqual(["No date 1"]);
    expect(screen.queryByText(/Overdue/)).toBeNull();
  });

  it("shows no groups at all when there are no tasks", () => {
    setup([notes[1] as Note]);
    expect(screen.queryAllByRole("heading", { level: 3 })).toHaveLength(0);
    expect(screen.getByRole("heading", { level: 2, name: "Tasks" })).toBeInTheDocument();
  });

  it("says Today, Tomorrow and Yesterday for the nearest days and a short date otherwise", () => {
    setup();
    const chip = { selector: "span.shrink-0" };
    expect(screen.getByText("Today", chip)).toBeInTheDocument();
    expect(screen.getByText("Tomorrow", chip)).toBeInTheDocument();
    expect(screen.getByText("Yesterday", chip)).toBeInTheDocument();
    expect(screen.getByText(/17 Oct/)).toBeInTheDocument(); // three days ago
    expect(screen.getByText(/25 Oct/)).toBeInTheDocument(); // five days on
    expect(screen.getByText(/11 Oct/)).toBeInTheDocument(); // the finished task
  });

  it("shows each task with its note's title and no date chip when it has none", () => {
    setup();
    const row = screen.getByText("whenever").closest("li") as HTMLElement;
    expect(within(row).getByText("Alpha")).toBeInTheDocument();
    expect(row.querySelectorAll("span.shrink-0")).toHaveLength(0);
  });

  it("toggles a task by its note id and source line, whichever state it is in", () => {
    const { onToggle } = setup();
    fireEvent.click(screen.getByRole("checkbox", { name: "Mark as done: late" }));
    expect(onToggle).toHaveBeenLastCalledWith("a", 2);
    fireEvent.click(screen.getByRole("checkbox", { name: "Mark as not done: finished" }));
    expect(onToggle).toHaveBeenLastCalledWith("a", 8);
    fireEvent.click(screen.getByRole("checkbox", { name: "Mark as done: gamma task" }));
    expect(onToggle).toHaveBeenLastCalledWith("c", 2);
  });

  it("opens a task's note when its text is clicked", () => {
    const { onOpen } = setup();
    fireEvent.click(screen.getByText("gamma task"));
    expect(onOpen).toHaveBeenCalledWith("c");
  });

  it("marks done tasks as ticked and struck through", () => {
    setup();
    expect(screen.getByRole("checkbox", { name: "Mark as not done: finished" })).toHaveAttribute("aria-checked", "true");
    expect(screen.getByText("finished")).toHaveClass("line-through");
  });
});
