/**
 * The in-browser Kanso's state and everything that changes it, as a pure
 * reducer: which notes exist and what they say, which one is open, what the
 * note list shows, how the note is shown, and whether the window shows what
 * the server stores. Nothing here is saved; it lasts as long as the tab.
 */
import { rank } from "./fuzzy";
import { headings, snippet, tagsOf, title, toggleTask } from "./markdown";
import type { Note } from "./notes";

export type Filter =
  | { readonly kind: "all" }
  | { readonly kind: "pinned" }
  | { readonly kind: "tasks" }
  | { readonly kind: "notebook"; readonly name: string }
  | { readonly kind: "tag"; readonly name: string };

export type Mode = "edit" | "preview" | "split";

export interface State {
  readonly notes: readonly Note[];
  readonly selected: string;
  readonly filter: Filter;
  readonly mode: Mode;
  /** Show what the server stores instead of the notes. */
  readonly server: boolean;
  readonly palette: boolean;
  /** The note list's search. */
  readonly query: string;
}

export type Action =
  | { readonly type: "select"; readonly id: string }
  | { readonly type: "edit"; readonly id: string; readonly body: string }
  | { readonly type: "filter"; readonly filter: Filter }
  | { readonly type: "mode"; readonly mode: Mode }
  | { readonly type: "server"; readonly on: boolean }
  | { readonly type: "palette"; readonly open: boolean }
  | { readonly type: "query"; readonly query: string }
  | { readonly type: "toggle-task"; readonly id: string; readonly line: number }
  | { readonly type: "open-wiki"; readonly target: string }
  | { readonly type: "new-note" };

export function initialState(notes: readonly Note[]): State {
  return { notes, selected: notes[0]?.id ?? "", filter: { kind: "all" }, mode: "split", server: false, palette: false, query: "" };
}

function freshId(notes: readonly Note[], base: string): string {
  const stem = base.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "") || "note";
  let id = stem;
  for (let n = 2; notes.some((note) => note.id === id); n++) id = `${stem}-${n}`;
  return id;
}

function update(notes: readonly Note[], id: string, body: (old: string) => string): Note[] {
  return notes.map((note) => (note.id === id ? { ...note, body: body(note.body), edited: 0 } : note));
}

/** The note a [[wiki-link]] points at: the one whose title matches, ignoring case. */
export function findByTitle(notes: readonly Note[], target: string): Note | undefined {
  const wanted = target.trim().toLowerCase();
  return notes.find((note) => title(note.body).toLowerCase() === wanted);
}

export function reducer(state: State, action: Action): State {
  switch (action.type) {
    case "select":
      return state.notes.some((n) => n.id === action.id) ? { ...state, selected: action.id, palette: false } : state;
    case "edit":
      return { ...state, notes: update(state.notes, action.id, () => action.body) };
    case "filter":
      return { ...state, filter: action.filter, query: "" };
    case "mode":
      return { ...state, mode: action.mode };
    case "server":
      return { ...state, server: action.on, palette: false };
    case "palette":
      return { ...state, palette: action.open };
    case "query":
      return { ...state, query: action.query };
    case "toggle-task":
      return { ...state, notes: update(state.notes, action.id, (body) => toggleTask(body, action.line)) };
    case "open-wiki": {
      const found = findByTitle(state.notes, action.target);
      if (found) return { ...state, selected: found.id };
      return create(state, action.target.trim() || "Untitled", false);
    }
    case "new-note":
      return create(state, "Untitled", true);
  }
}

function create(state: State, name: string, blank: boolean): State {
  const current = state.notes.find((n) => n.id === state.selected);
  const notebook = state.filter.kind === "notebook" ? state.filter.name : (current?.notebook ?? "Start here");
  const note: Note = { id: freshId(state.notes, name), notebook, edited: 0, body: blank ? "# " : `# ${name}\n\n` };
  return {
    ...state,
    notes: [note, ...state.notes],
    selected: note.id,
    mode: state.mode === "preview" ? "edit" : state.mode,
    filter: state.filter.kind === "notebook" || state.filter.kind === "all" ? state.filter : { kind: "all" },
    query: "",
  };
}

export function selectedNote(state: State): Note | undefined {
  return state.notes.find((n) => n.id === state.selected);
}

/** The notes the list shows, pinned first, then the most recently edited. */
export function visibleNotes(state: State): Note[] {
  const { filter } = state;
  const shown = state.notes.filter((note) => {
    switch (filter.kind) {
      case "all":
      case "tasks":
        return true;
      case "pinned":
        return note.pinned === true;
      case "notebook":
        return note.notebook === filter.name;
      case "tag":
        return tagsOf(note.body).some((t) => t === filter.name || t.startsWith(`${filter.name}/`));
    }
  });
  const ordered = [...shown].sort((a, b) => Number(b.pinned ?? false) - Number(a.pinned ?? false) || a.edited - b.edited);
  if (state.query.trim() === "") return ordered;
  return rank(state.query, ordered, (note) => `${title(note.body)} ${snippet(note.body)}`);
}

/** Every tag in the library, with how many notes carry it. */
export function allTags(notes: readonly Note[]): { name: string; count: number }[] {
  const counts = new Map<string, number>();
  for (const note of notes) for (const tag of tagsOf(note.body)) counts.set(tag, (counts.get(tag) ?? 0) + 1);
  return [...counts].map(([name, count]) => ({ name, count })).sort((a, b) => a.name.localeCompare(b.name));
}

export interface Destination {
  readonly id: string;
  readonly label: string;
  readonly detail: string;
  /** The heading to scroll to, for a heading inside a note. */
  readonly anchor?: string;
}

/** What quick open offers for a query: notes by title, then headings inside them. */
export function destinations(notes: readonly Note[], query: string): Destination[] {
  const all: Destination[] = [];
  for (const note of notes) {
    const name = title(note.body);
    all.push({ id: note.id, label: name, detail: note.notebook });
    for (const h of headings(note.body).slice(1)) all.push({ id: note.id, label: h.text, detail: name, anchor: h.id });
  }
  const pool = query.trim() === "" ? all.filter((d) => d.anchor === undefined) : all;
  return rank(query, pool, (d) => d.label).slice(0, 8);
}
