/**
 * Tests for CopyCommand with an injected clipboard: the command is shown,
 * success reads "Copied", failure and a missing clipboard read "Select to
 * copy", and the label returns to "Copy" after two seconds (fake timers).
 */
import { act, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { CopyCommand } from "./CopyCommand";

beforeEach(() => vi.useFakeTimers());
afterEach(() => vi.useRealTimers());

const click = async () => {
  await act(async () => {
    fireEvent.click(screen.getByRole("button", { name: "Copy install command" }));
  });
};

describe("CopyCommand", () => {
  it("shows the command and an idle button", () => {
    render(<CopyCommand command="brew install kanso" label="install command" />);
    expect(screen.getByText("brew install kanso")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Copy install command" })).toHaveTextContent("Copy");
  });

  it("copies the command and says Copied, then resets after 2s", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    render(<CopyCommand command="brew install kanso" label="install command" clipboard={{ writeText }} />);
    await click();
    expect(writeText).toHaveBeenCalledWith("brew install kanso");
    expect(screen.getByRole("button")).toHaveTextContent("Copied");
    act(() => void vi.advanceTimersByTime(1999));
    expect(screen.getByRole("button")).toHaveTextContent("Copied");
    act(() => void vi.advanceTimersByTime(1));
    expect(screen.getByRole("button")).toHaveTextContent(/^Copy$/);
  });

  it("says Select to copy when the clipboard rejects, then resets", async () => {
    const writeText = vi.fn().mockRejectedValue(new Error("denied"));
    render(<CopyCommand command="x" label="install command" clipboard={{ writeText }} />);
    await click();
    expect(screen.getByRole("button")).toHaveTextContent("Select to copy");
    act(() => void vi.advanceTimersByTime(2000));
    expect(screen.getByRole("button")).toHaveTextContent(/^Copy$/);
  });

  it("falls back to navigator.clipboard, and fails when there is none", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    vi.stubGlobal("navigator", { clipboard: { writeText } });
    const { unmount } = render(<CopyCommand command="x" label="install command" />);
    await click();
    expect(writeText).toHaveBeenCalledWith("x");
    expect(screen.getByRole("button")).toHaveTextContent("Copied");
    unmount();

    vi.stubGlobal("navigator", {});
    render(<CopyCommand command="x" label="install command" />);
    await click();
    expect(screen.getByRole("button")).toHaveTextContent("Select to copy");
    vi.unstubAllGlobals();
  });

  it("clears its timer on unmount", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    const { unmount } = render(<CopyCommand command="x" label="install command" clipboard={{ writeText }} />);
    await click();
    unmount();
    expect(vi.getTimerCount()).toBe(0);
  });
});
