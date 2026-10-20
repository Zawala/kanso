/**
 * Tests for Sidebar and TrafficLights: the views with counts, notebooks,
 * tags with counts, the active row, filter callbacks, the server toggle,
 * and sealed notebook and tag names in server view.
 */
import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { Sidebar, TrafficLights } from "./Sidebar";
import type { Filter } from "../../lib/library";
import type { Note } from "../../lib/notes";

const notes: Note[] = [
  { id: "a", notebook: "Start here", edited: 1, pinned: true, body: "# A\n\n- [ ] one\n- [x] two #work" },
  { id: "b", notebook: "Privacy", edited: 2, body: "# B\n\n- [ ] three #work/sub" },
  { id: "c", notebook: "Privacy", edited: 3, body: "# C #home" },
];

const setup = (over: { filter?: Filter; server?: boolean } = {}) => {
  const onFilter = vi.fn();
  const onServer = vi.fn();
  render(
    <Sidebar
      notes={notes}
      filter={over.filter ?? { kind: "all" }}
      server={over.server ?? false}
      sealed={(t) => `k${t}-0123456789abcdef`}
      onFilter={onFilter}
      onServer={onServer}
    />,
  );
  return { onFilter, onServer, nav: within(screen.getByRole("navigation", { name: "Library" })) };
};

describe("Sidebar", () => {
  it("shows the views with counts: all notes, pinned, and open tasks", () => {
    const { nav } = setup();
    expect(nav.getByRole("button", { name: "All Notes3" })).toBeInTheDocument();
    expect(nav.getByRole("button", { name: "Pinned1" })).toBeInTheDocument();
    expect(nav.getByRole("button", { name: "Tasks2" })).toBeInTheDocument();
  });

  it("lists every notebook with how many notes it holds", () => {
    const { nav } = setup();
    expect(nav.getByRole("button", { name: "Start here1" })).toBeInTheDocument();
    expect(nav.getByRole("button", { name: "Privacy2" })).toBeInTheDocument();
    expect(nav.getByRole("button", { name: "Personal0" })).toBeInTheDocument();
  });

  it("lists tags with their note counts", () => {
    const { nav } = setup();
    expect(nav.getByRole("button", { name: "#home 1" })).toBeInTheDocument();
    expect(nav.getByRole("button", { name: "#work 1" })).toBeInTheDocument();
    expect(nav.getByRole("button", { name: "#work/sub 1" })).toBeInTheDocument();
  });

  it("asks for the filter of whatever is clicked", () => {
    const { nav, onFilter } = setup();
    fireEvent.click(nav.getByRole("button", { name: "All Notes3" }));
    expect(onFilter).toHaveBeenLastCalledWith({ kind: "all" });
    fireEvent.click(nav.getByRole("button", { name: "Pinned1" }));
    expect(onFilter).toHaveBeenLastCalledWith({ kind: "pinned" });
    fireEvent.click(nav.getByRole("button", { name: "Tasks2" }));
    expect(onFilter).toHaveBeenLastCalledWith({ kind: "tasks" });
    fireEvent.click(nav.getByRole("button", { name: "Privacy2" }));
    expect(onFilter).toHaveBeenLastCalledWith({ kind: "notebook", name: "Privacy" });
    fireEvent.click(nav.getByRole("button", { name: "#work 1" }));
    expect(onFilter).toHaveBeenLastCalledWith({ kind: "tag", name: "work" });
  });

  it.each<[Filter, string]>([
    [{ kind: "all" }, "All Notes3"],
    [{ kind: "pinned" }, "Pinned1"],
    [{ kind: "tasks" }, "Tasks2"],
    [{ kind: "notebook", name: "Privacy" }, "Privacy2"],
  ])("marks only the row of %j as current", (filter, label) => {
    const { nav } = setup({ filter });
    const current = nav.getAllByRole("button").filter((b) => b.getAttribute("aria-current") === "true");
    expect(current.map((b) => b.getAttribute("aria-label") ?? b.textContent?.replace(/\s+/g, " ").trim())).toEqual([label]);
  });

  it("does not mark a different notebook as current when another is", () => {
    const { nav } = setup({ filter: { kind: "notebook", name: "Privacy" } });
    expect(nav.getByRole("button", { name: "Start here1" })).not.toHaveAttribute("aria-current");
  });

  it("presses the current tag only", () => {
    const { nav } = setup({ filter: { kind: "tag", name: "work" } });
    expect(nav.getByRole("button", { name: "#work 1" })).toHaveAttribute("aria-pressed", "true");
    expect(nav.getByRole("button", { name: "#home 1" })).toHaveAttribute("aria-pressed", "false");
    expect(nav.getByRole("button", { name: "All Notes3" })).not.toHaveAttribute("aria-current");
  });

  it("toggles the server view from the sync status", () => {
    const { nav, onServer } = setup();
    fireEvent.click(nav.getByRole("button", { name: /This browser/ }));
    expect(onServer).toHaveBeenLastCalledWith(true);
  });

  it("turns the server view off when it is on", () => {
    const { nav, onServer } = setup({ server: true });
    fireEvent.click(nav.getByRole("button", { name: /This browser/ }));
    expect(onServer).toHaveBeenLastCalledWith(false);
  });

  it("shows notebook and tag names as the server holds them in server view, not the real ones", () => {
    const { nav } = setup({ server: true });
    expect(nav.queryByText("Privacy")).toBeNull();
    expect(nav.getByText("kPrivacy-0123456789abcdef")).toBeInTheDocument();
    expect(nav.queryByText("#work")).toBeNull();
    expect(nav.getByRole("button", { name: "#kwork-0123 1" })).toBeInTheDocument(); // first ten characters of the sealed name
  });

  it("shows real names outside server view", () => {
    const { nav } = setup();
    expect(nav.queryByText(/0123456789/)).toBeNull();
  });

  it("draws the window's traffic lights as decoration", () => {
    const { container } = render(<TrafficLights />);
    expect(container.firstElementChild).toHaveAttribute("aria-hidden", "true");
    expect(container.querySelectorAll("span")).toHaveLength(3);
  });
});
