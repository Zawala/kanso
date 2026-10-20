/**
 * Tests for Palette (quick open): it takes focus, filters notes and
 * headings as you type, moves with the arrow keys (clamped), opens with
 * Enter or a click, and closes on Escape or a press outside the dialog.
 */
import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { Palette } from "./Palette";
import type { Note } from "../../lib/notes";

const notes: Note[] = [
  { id: "alpha", notebook: "Work", edited: 1, body: "# Alpha\n\n## Plan the week\n\n## Review" },
  { id: "beta", notebook: "Home", edited: 2, body: "# Beta" },
  { id: "gamma", notebook: "Home", edited: 3, body: "# Gamma" },
];

const setup = () => {
  const onOpen = vi.fn();
  const onClose = vi.fn();
  const user = userEvent.setup();
  render(<Palette notes={notes} onOpen={onOpen} onClose={onClose} />);
  return { onOpen, onClose, user, input: screen.getByRole("combobox", { name: "Search notes and headings" }) };
};
const selected = () => screen.getAllByRole("option").map((o) => o.getAttribute("aria-selected"));

describe("Palette", () => {
  it("takes focus and lists the notes, with their notebooks, selecting the first", () => {
    const { input } = setup();
    expect(input).toHaveFocus();
    expect(screen.getByRole("dialog", { name: "Quick open" })).toBeInTheDocument();
    expect(screen.getAllByRole("option").map((o) => o.textContent)).toEqual(["AlphaWork", "BetaHome", "GammaHome"]);
    expect(selected()).toEqual(["true", "false", "false"]);
  });

  it("filters as you type, and offers headings inside notes", async () => {
    const { user } = setup();
    await user.keyboard("plan");
    const options = screen.getAllByRole("option");
    expect(options).toHaveLength(1);
    expect(options[0]).toHaveTextContent("Plan the weekAlpha");
  });

  it("says when nothing matches, and Enter then does nothing", async () => {
    const { user, onOpen } = setup();
    await user.keyboard("zzzq{Enter}");
    expect(screen.getByText(/No notes match/)).toHaveTextContent("zzzq");
    expect(onOpen).not.toHaveBeenCalled();
  });

  it("moves with the arrow keys and stops at both ends", async () => {
    const { user } = setup();
    await user.keyboard("{ArrowUp}");
    expect(selected()).toEqual(["true", "false", "false"]);
    await user.keyboard("{ArrowDown}");
    expect(selected()).toEqual(["false", "true", "false"]);
    await user.keyboard("{ArrowDown}{ArrowDown}{ArrowDown}");
    expect(selected()).toEqual(["false", "false", "true"]);
    await user.keyboard("{ArrowUp}");
    expect(selected()).toEqual(["false", "true", "false"]);
  });

  it("opens the selected destination on Enter", async () => {
    const { user, onOpen } = setup();
    await user.keyboard("{ArrowDown}{Enter}");
    expect(onOpen).toHaveBeenCalledWith({ id: "beta", label: "Beta", detail: "Home" });
  });

  it("opens a heading with its anchor", async () => {
    const { user, onOpen } = setup();
    await user.keyboard("review{Enter}");
    expect(onOpen).toHaveBeenCalledWith({ id: "alpha", label: "Review", detail: "Alpha", anchor: "review" });
  });

  it("resets the selection to the top when the query changes", async () => {
    const { user } = setup();
    await user.keyboard("{ArrowDown}{ArrowDown}a");
    expect(selected()[0]).toBe("true");
  });

  it("opens an option on click and follows the mouse with the selection", async () => {
    const { user, onOpen } = setup();
    const gamma = screen.getByRole("option", { name: /Gamma/ });
    await user.hover(gamma);
    expect(selected()).toEqual(["false", "false", "true"]);
    await user.click(gamma);
    expect(onOpen).toHaveBeenCalledWith(expect.objectContaining({ id: "gamma" }));
  });

  it("closes on Escape", async () => {
    const { user, onClose } = setup();
    await user.keyboard("{Escape}");
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("closes on a press outside the dialog, but not inside it", () => {
    const { onClose, input } = setup();
    fireEvent.mouseDown(input);
    fireEvent.mouseDown(screen.getByRole("dialog"));
    expect(onClose).not.toHaveBeenCalled();
    fireEvent.mouseDown(screen.getByTestId("palette"));
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("ignores other keys", async () => {
    const { user, onOpen, onClose } = setup();
    await user.keyboard("{Shift}{Tab}");
    expect(onOpen).not.toHaveBeenCalled();
    expect(onClose).not.toHaveBeenCalled();
  });
});
