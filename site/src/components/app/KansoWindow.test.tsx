/**
 * Integration tests for KansoWindow, the in-browser app: it renders the
 * tour's welcome note, opens notes from the list, [[links]] and quick open,
 * switches Edit/Preview/Split, shows real ciphertext in Server view once
 * sealing resolves, gathers and ticks tasks, folds to one pane when narrow
 * (through a fake ResizeObserver) with a notes drawer, and opaqueId.
 * jsdom lacks Element.scrollTo, scrollIntoView, CSS.escape and a usable
 * Web Crypto, so they are stubbed (Node's webcrypto for the latter).
 */
import { webcrypto } from "node:crypto";
import { act, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import fc from "fast-check";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { COMPACT_WIDTH, KansoWindow, opaqueId } from "./KansoWindow";

const TODAY = new Date(2026, 9, 20);
const scrollIntoView = vi.fn();

beforeEach(() => {
  vi.stubGlobal("crypto", webcrypto);
  vi.stubGlobal("CSS", { escape: (s: string) => s });
  Element.prototype.scrollTo = vi.fn() as unknown as typeof Element.prototype.scrollTo;
  Element.prototype.scrollIntoView = scrollIntoView;
  scrollIntoView.mockClear();
});
afterEach(() => vi.unstubAllGlobals());

const mount = (props: { today?: Date; className?: string } = { today: TODAY }) => {
  const user = userEvent.setup();
  const { unmount } = render(<KansoWindow {...props} />);
  return {
    user,
    unmount,
    editor: () => screen.getByRole("textbox", { name: /, Markdown$/ }) as HTMLTextAreaElement,
    library: () => within(screen.getByRole("navigation", { name: "Library" })),
    list: () => within(screen.getAllByRole("list", { name: "Notes" })[0] as HTMLElement),
    mode: (name: string) => screen.getByRole("button", { name }),
    serverToggle: () => screen.getByRole("button", { name: "Server view" }),
  };
};

/** A ResizeObserver stand-in that the test can drive. */
function fakeResizeObserver() {
  const state: { callback: ResizeObserverCallback | null; disconnected: boolean } = { callback: null, disconnected: false };
  vi.stubGlobal(
    "ResizeObserver",
    class {
      constructor(cb: ResizeObserverCallback) {
        state.callback = cb;
      }
      observe() {}
      unobserve() {}
      disconnect() {
        state.disconnected = true;
      }
    },
  );
  const resize = (width: number | null) =>
    act(() => state.callback?.(width === null ? [] : ([{ contentRect: { width } }] as unknown as ResizeObserverEntry[]), {} as ResizeObserver));
  return { state, resize };
}

describe("opaqueId", () => {
  it.each([
    ["", "811c9dc5"],
    ["a", "e40c292c"],
    ["foobar", "bf9cf968"],
  ])("is FNV-1a of %j: %s", (input, expected) => {
    expect(opaqueId(input)).toBe(expected);
  });

  it("is always eight lower case hex digits, and stable", () => {
    fc.assert(
      fc.property(fc.oneof(fc.string(), fc.string({ unit: "binary" })), (id) => /^[0-9a-f]{8}$/.test(opaqueId(id)) && opaqueId(id) === opaqueId(id)),
    );
  });

  it("tells different note ids of the tour apart", () => {
    const ids = ["welcome", "writing", "links", "tasks", "capture", "diagrams", "search", "history", "sync", "publishing", "agents", "source", "sunday", "garden"];
    expect(new Set(ids.map(opaqueId)).size).toBe(ids.length);
  });
});

describe("KansoWindow", () => {
  it("opens on the welcome note, in split mode, with the editor and the preview", () => {
    const { editor, library } = mount();
    expect(screen.getByRole("region", { name: "Kanso, running in your browser" })).toBeInTheDocument();
    expect(editor().value).toContain("# Welcome to Kanso");
    expect(within(screen.getByTestId("preview")).getByRole("heading", { level: 1, name: "Welcome to Kanso" })).toBeInTheDocument();
    expect(library().getByRole("button", { name: /^All Notes/ })).toHaveAttribute("aria-current", "true");
    expect(screen.getByRole("button", { name: "Split" })).toHaveAttribute("aria-pressed", "true");
  });

  it("works without a today prop, and takes a className", () => {
    mount({ className: "mine" });
    expect(screen.getByRole("region")).toHaveClass("mine");
  });

  it("opens a note when it is clicked in the list", async () => {
    const { user, editor, list } = mount();
    await user.click(list().getByRole("button", { name: /^Writing/ }));
    expect(editor().value).toContain("# Writing");
    expect(list().getByRole("button", { name: /^Writing/ })).toHaveAttribute("aria-current", "true");
    expect(Element.prototype.scrollTo).toHaveBeenCalled();
  });

  it("shows the note's place and edit time, and word and character counts that follow typing", async () => {
    const { user, editor } = mount();
    expect(screen.getByText("Welcome to Kanso", { selector: "p span.text-fg" })).toBeInTheDocument();
    expect(screen.getByText("Edited 2 min ago")).toBeInTheDocument();
    const words = () => Number((screen.getByText(/ words$/, { selector: "footer span" }).textContent ?? "").replace(/[^\d]/g, ""));
    const before = words();
    await user.click(editor());
    await user.keyboard("{Control>}{End}{/Control}");
    fireEvent.change(editor(), { target: { value: `${editor().value} brand new words` } });
    expect(words()).toBe(before + 3);
    expect(screen.getByText(/Ln \d+, Col \d+/)).toBeInTheDocument();
  });

  describe("modes", () => {
    it("switches between Edit, Preview and Split", async () => {
      const { user, mode } = mount();
      await user.click(mode("Edit"));
      expect(screen.queryByTestId("preview")).toBeNull();
      expect(screen.getByRole("textbox", { name: /Markdown/ })).toBeInTheDocument();
      expect(mode("Edit")).toHaveAttribute("aria-pressed", "true");
      await user.click(mode("Preview"));
      expect(screen.getByTestId("preview")).toBeInTheDocument();
      expect(screen.queryByRole("textbox", { name: /Markdown/ })).toBeNull();
      await user.click(mode("Split"));
      expect(screen.getByTestId("preview")).toBeInTheDocument();
      expect(screen.getByRole("textbox", { name: /Markdown/ })).toBeInTheDocument();
    });

    it("edits the note and shows the change in the preview", async () => {
      const { editor } = mount();
      fireEvent.change(editor(), { target: { value: "# Changed title\n\ntext" } });
      expect(within(screen.getByTestId("preview")).getByRole("heading", { name: "Changed title" })).toBeInTheDocument();
    });
  });

  describe("links and tasks in the preview", () => {
    it("ticks a task in the preview, changing the note's text", async () => {
      const { user, editor } = mount();
      const box = within(screen.getByTestId("preview")).getAllByRole("checkbox", { name: "Mark as done" })[0] as HTMLElement;
      await user.click(box);
      expect(editor().value).toContain("- [x] Write something in [[Writing]]");
    });

    it("opens the note a [[wiki-link]] names", async () => {
      const { user, editor } = mount();
      await user.click(within(screen.getByTestId("preview")).getByRole("button", { name: "Writing" }));
      expect(editor().value).toContain("# Writing");
    });

    it("makes a note for a [[wiki-link]] that names none", async () => {
      const { user, editor, list } = mount();
      fireEvent.change(editor(), { target: { value: "# Hub\n\nSee [[Brand New Idea]]" } });
      await user.click(within(screen.getByTestId("preview")).getByRole("button", { name: "Brand New Idea" }));
      expect(editor()).toHaveValue("# Brand New Idea\n\n");
      expect(list().getByRole("button", { name: /^Brand New Idea/ })).toHaveAttribute("aria-current", "true");
    });
  });

  describe("the note list and the sidebar", () => {
    it("starts a new note", async () => {
      const { user, editor } = mount();
      await user.click(screen.getByRole("button", { name: "New note" }));
      expect(editor()).toHaveValue("# ");
      expect(editor()).toHaveAccessibleName("Untitled, Markdown");
    });

    it("searches the list", async () => {
      const { user, list } = mount();
      await user.type(screen.getAllByRole("textbox", { name: "Search notes" })[0] as HTMLElement, "garden");
      const buttons = list().getAllByRole("button");
      expect(buttons.length).toBeLessThan(14);
      expect(list().getByRole("button", { name: /^Garden plan/ })).toBeInTheDocument();
    });

    it("filters by notebook, by pinned and by tag, and names the list for the filter", async () => {
      const { user, list, library } = mount();
      await user.click(library().getByRole("button", { name: /^Privacy/ }));
      expect(screen.getByRole("heading", { level: 2, name: /^Privacy/ })).toBeInTheDocument();
      expect(list().getAllByRole("button")).toHaveLength(2);
      await user.click(library().getByRole("button", { name: /^Pinned/ }));
      expect(screen.getByRole("heading", { level: 2, name: /^Pinned/ })).toBeInTheDocument();
      await user.click(library().getByRole("button", { name: /^#writing/ }));
      expect(screen.getByRole("heading", { level: 2, name: /^#writing/ })).toBeInTheDocument();
      expect(list().getByRole("button", { name: /^Links/ })).toBeInTheDocument();
      await user.click(library().getByRole("button", { name: /^All Notes/ }));
      expect(screen.getByRole("heading", { level: 2, name: /^All Notes/ })).toBeInTheDocument();
    });
  });

  describe("tasks", () => {
    it("shows every dated task, and ticking one ticks it in its note", async () => {
      const { user, editor, library } = mount();
      await user.click(library().getByRole("button", { name: /^Tasks/ }));
      const view = within(screen.getByTestId("tasks-view"));
      expect(view.getByRole("heading", { level: 3, name: /^Overdue/ })).toBeInTheDocument();
      expect(view.getByRole("heading", { level: 3, name: /^Today/ })).toBeInTheDocument();
      expect(view.getByRole("heading", { level: 3, name: /^Upcoming/ })).toBeInTheDocument();
      expect(screen.getByText("Tasks", { selector: "p span.text-fg" })).toBeInTheDocument();
      for (const name of ["Edit", "Preview", "Split"]) expect(screen.getByRole("button", { name })).toBeDisabled();

      await user.click(view.getByRole("checkbox", { name: "Mark as done: Book the plumber" }));
      expect(view.getByRole("checkbox", { name: "Mark as not done: Book the plumber" })).toBeInTheDocument();

      await user.click(view.getByText("Book the plumber"));
      expect(screen.queryByTestId("tasks-view")).toBeNull();
      expect(editor().value).toContain("- [x] Book the plumber @2026-10-20");
      expect(library().getByRole("button", { name: /^All Notes/ })).toHaveAttribute("aria-current", "true");
    });

    it("counts the open tasks in the sidebar", async () => {
      const { library } = mount();
      expect(library().getByRole("button", { name: /^Tasks/ })).toHaveTextContent(/Tasks\s*\d+/);
    });
  });

  describe("server view", () => {
    it("shows the opaque ids, the size and the real ciphertext once sealing is done", async () => {
      const { user, serverToggle } = mount();
      await user.click(serverToggle());
      const pane = within(screen.getByTestId("server-pane"));
      expect(pane.getByText(`n_${opaqueId("welcome")}`)).toBeInTheDocument();
      expect(pane.getByText(`b_${opaqueId("Start here")}`)).toBeInTheDocument();
      const ciphertext = () => screen.getByTestId("ciphertext").textContent as string;
      await waitFor(() => expect(ciphertext()).toMatch(/^[A-Za-z0-9+/]{40,}=*$/));
      expect(ciphertext()).not.toContain("Welcome");
      await waitFor(() => expect(pane.getByText(/^[\d,]+ bytes$/)).toBeInTheDocument());
      expect(pane.getByText(/^[0-9a-f]{8}$/)).toBeInTheDocument();
      expect(screen.getByText("Sealed in this tab with AES-256-GCM")).toBeInTheDocument();
      for (const name of ["Edit", "Preview", "Split"]) expect(screen.getByRole("button", { name })).toBeDisabled();
      expect(serverToggle()).toHaveAttribute("aria-pressed", "true");
    });

    it("shows placeholders before sealing finishes, and no real text anywhere in the list", async () => {
      const { serverToggle, list } = mount();
      fireEvent.click(serverToggle());
      expect(screen.getByTestId("ciphertext")).toHaveTextContent(/^·+$/);
      await waitFor(() => expect(screen.getByTestId("ciphertext").textContent).not.toMatch(/·/));
      expect(list().queryByText("Welcome to Kanso")).toBeNull();
      expect(screen.getByRole("button", { name: "New note" })).toBeDisabled();
    });

    it("seals the sidebar's notebook names too", async () => {
      const { user, serverToggle, library } = mount();
      await user.click(serverToggle());
      await waitFor(() => expect(library().queryByText("Privacy")).toBeNull());
      expect(library().queryByText("Inside Kanso")).toBeNull();
    });

    it("makes a new key on request, changing the ciphertext and fingerprint", async () => {
      const { user, serverToggle } = mount();
      await user.click(serverToggle());
      const pane = within(screen.getByTestId("server-pane"));
      await waitFor(() => expect(pane.getByText(/^[0-9a-f]{8}$/)).toBeInTheDocument());
      const [fingerprint, ciphertext] = [pane.getByText(/^[0-9a-f]{8}$/).textContent, screen.getByTestId("ciphertext").textContent];
      await user.click(pane.getByRole("button", { name: "New key" }));
      await waitFor(() => expect(pane.getByText(/^[0-9a-f]{8}$/).textContent).not.toBe(fingerprint));
      expect(screen.getByTestId("ciphertext").textContent).not.toBe(ciphertext);
    });

    it("goes back to the note when switched off, with the text intact", async () => {
      const { user, serverToggle, editor } = mount();
      await user.click(serverToggle());
      await waitFor(() => expect(screen.getByTestId("ciphertext").textContent).not.toMatch(/·/));
      await user.click(serverToggle());
      expect(screen.queryByTestId("server-pane")).toBeNull();
      expect(editor().value).toContain("# Welcome to Kanso");
    });

    it("can be turned on from the sidebar's sync status", async () => {
      const { user, library } = mount();
      await user.click(library().getByRole("button", { name: /This browser/ }));
      expect(screen.getByTestId("server-pane")).toBeInTheDocument();
    });

    it("shows the tasks view, not the server pane, when Tasks is chosen in server view", async () => {
      const { user, serverToggle, library } = mount();
      await user.click(serverToggle());
      await user.click(library().getByRole("button", { name: /^Tasks/ }));
      expect(screen.getByTestId("tasks-view")).toBeInTheDocument();
    });
  });

  describe("quick open", () => {
    it("opens with Cmd-K, filters, and Enter opens the chosen note", async () => {
      const { user, editor } = mount();
      await user.keyboard("{Meta>}k{/Meta}");
      const dialog = screen.getByRole("dialog", { name: "Quick open" });
      expect(dialog).toBeInTheDocument();
      await user.keyboard("encrypted");
      await user.keyboard("{Enter}");
      expect(screen.queryByRole("dialog")).toBeNull();
      expect(editor().value).toContain("# Encrypted sync");
    });

    it("opens with Ctrl-K, and with Cmd-P only while focus is inside the window", async () => {
      const { user, editor } = mount();
      await user.keyboard("{Control>}k{/Control}");
      expect(screen.getByRole("dialog")).toBeInTheDocument();
      await user.keyboard("{Escape}");
      expect(screen.queryByRole("dialog")).toBeNull();

      await user.keyboard("{Meta>}p{/Meta}");
      expect(screen.queryByRole("dialog")).toBeNull();
      editor().focus();
      await user.keyboard("{Meta>}p{/Meta}");
      expect(screen.getByRole("dialog")).toBeInTheDocument();
    });

    it("ignores K without a modifier, and with Shift or Alt", async () => {
      const { user } = mount();
      await user.keyboard("k");
      await user.keyboard("{Meta>}{Shift>}k{/Shift}{/Meta}");
      await user.keyboard("{Meta>}{Alt>}k{/Alt}{/Meta}");
      expect(screen.queryByRole("dialog")).toBeNull();
    });

    it("opens from the footer button, and closes on a press outside", async () => {
      const { user } = mount();
      await user.click(screen.getByRole("button", { name: "⌘P Quick open" }));
      expect(screen.getByRole("dialog")).toBeInTheDocument();
      fireEvent.mouseDown(screen.getByTestId("palette"));
      expect(screen.queryByRole("dialog")).toBeNull();
    });

    it("jumps to a heading inside a note, switching Edit to Split and scrolling to it", async () => {
      const { user, mode, editor } = mount();
      fireEvent.change(editor(), { target: { value: "# Hub\n\n## Deep Heading\n\ntext" } });
      await user.click(mode("Edit"));
      await user.keyboard("{Meta>}k{/Meta}Deep");
      await user.keyboard("{Enter}");
      expect(mode("Split")).toHaveAttribute("aria-pressed", "true");
      await waitFor(() => expect(scrollIntoView).toHaveBeenCalledWith({ block: "start" }));
      expect(scrollIntoView.mock.contexts[0]).toHaveAttribute("id", "note-deep-heading");
    });

    it("does not change the mode for a heading when the preview is already showing", async () => {
      const { user, mode, editor } = mount();
      fireEvent.change(editor(), { target: { value: "# Hub\n\n## Deep Heading" } });
      await user.click(mode("Preview"));
      await user.keyboard("{Meta>}k{/Meta}Deep{Enter}");
      expect(mode("Preview")).toHaveAttribute("aria-pressed", "true");
      await waitFor(() => expect(scrollIntoView).toHaveBeenCalled());
    });

    it("leaves the Tasks view when a note is chosen from quick open", async () => {
      const { user, library, editor } = mount();
      await user.click(library().getByRole("button", { name: /^Tasks/ }));
      await user.keyboard("{Meta>}k{/Meta}history{Enter}");
      expect(screen.queryByTestId("tasks-view")).toBeNull();
      expect(editor().value).toContain("# History");
    });
  });

  describe("on a narrow screen", () => {
    it("shows one pane at a time: split becomes preview, with no Split button", async () => {
      const { resize } = fakeResizeObserver();
      const { user, mode } = mount();
      expect(mode("Split")).toBeInTheDocument();
      resize(COMPACT_WIDTH - 1);
      await waitFor(() => expect(screen.queryByRole("button", { name: "Split" })).toBeNull());
      expect(mode("Preview")).toHaveAttribute("aria-pressed", "true");
      expect(screen.queryByRole("textbox", { name: /Markdown/ })).toBeNull();
      await user.click(mode("Edit"));
      expect(screen.getByRole("textbox", { name: /Markdown/ })).toBeInTheDocument();
    });

    it("goes back to split when it widens again, treats an empty report as wide, and stops observing on unmount", async () => {
      const { resize, state } = fakeResizeObserver();
      const { mode, unmount } = mount();
      resize(300);
      await waitFor(() => expect(screen.queryByRole("button", { name: "Split" })).toBeNull());
      resize(COMPACT_WIDTH);
      await waitFor(() => expect(mode("Split")).toHaveAttribute("aria-pressed", "true"));
      resize(300);
      resize(null);
      await waitFor(() => expect(mode("Split")).toBeInTheDocument());
      expect(state.disconnected).toBe(false);
      unmount();
      expect(state.disconnected).toBe(true);
    });

    it("jumping to a heading from Edit goes to Preview rather than Split", async () => {
      const { resize } = fakeResizeObserver();
      const { user, mode, editor } = mount();
      resize(400);
      await user.click(mode("Edit"));
      fireEvent.change(editor(), { target: { value: "# Hub\n\n## Deep Heading" } });
      await user.keyboard("{Meta>}k{/Meta}Deep{Enter}");
      await waitFor(() => expect(mode("Preview")).toHaveAttribute("aria-pressed", "true"));
    });

    it("opens the notes drawer from Show notes and a note chosen in it closes the drawer", async () => {
      const { user, editor } = mount();
      expect(screen.queryByTestId("drawer")).toBeNull();
      await user.click(screen.getByRole("button", { name: "Show notes" }));
      const drawer = within(screen.getByTestId("drawer"));
      await user.click(drawer.getByRole("button", { name: /^Sunday/ }));
      expect(screen.queryByTestId("drawer")).toBeNull();
      expect(editor().value).toContain("# Sunday");
    });

    it("filters from the drawer, and Tasks closes it onto the tasks view", async () => {
      const { user } = mount();
      await user.click(screen.getByRole("button", { name: "Show notes" }));
      const drawer = within(screen.getByTestId("drawer"));
      await user.click(drawer.getByRole("button", { name: "Pinned" }));
      expect(drawer.getByRole("button", { name: "Pinned" })).toHaveAttribute("aria-pressed", "true");
      expect(drawer.getAllByRole("list", { name: "Notes" })[0]?.querySelectorAll("li")).toHaveLength(2);
      await user.click(drawer.getByRole("button", { name: "All" }));
      expect(drawer.getByRole("button", { name: "All" })).toHaveAttribute("aria-pressed", "true");
      await user.click(drawer.getByRole("button", { name: "Tasks" }));
      expect(screen.queryByTestId("drawer")).toBeNull();
      expect(screen.getByTestId("tasks-view")).toBeInTheDocument();
    });

    it("closes with either Hide notes control, and a new note made in it closes it too", async () => {
      const { user, editor } = mount();
      await user.click(screen.getByRole("button", { name: "Show notes" }));
      const [close, backdrop] = screen.getAllByRole("button", { name: "Hide notes" });
      expect(backdrop).toBeDefined();
      await user.click(close as HTMLElement);
      expect(screen.queryByTestId("drawer")).toBeNull();
      await user.click(screen.getByRole("button", { name: "Show notes" }));
      await user.click(screen.getAllByRole("button", { name: "Hide notes" })[1] as HTMLElement);
      expect(screen.queryByTestId("drawer")).toBeNull();
      await user.click(screen.getByRole("button", { name: "Show notes" }));
      await user.click(within(screen.getByTestId("drawer")).getByRole("button", { name: "New note" }));
      expect(screen.queryByTestId("drawer")).toBeNull();
      expect(editor()).toHaveValue("# ");
    });
  });
});
