/**
 * Kanso, sketched in the browser: the app's window with its sidebar, note
 * list and editor, working on the tour's library. It edits, previews and
 * splits like the app, opens notes from [[links]] and ⌘P, gathers tasks,
 * and in server view shows the whole library the way the sync server holds
 * it. On a narrow screen the list folds away behind a button.
 */
import { Columns2, Eye, List, Lock, PenLine, RefreshCw, X } from "lucide-react";
import { type ReactNode, useEffect, useMemo, useReducer, useRef, useState } from "react";
import { cn } from "../../lib/cn";
import { type Destination, type Filter, type Mode, initialState, reducer, selectedNote, visibleNotes } from "../../lib/library";
import { snippet, tagsOf, title, wordCount } from "../../lib/markdown";
import { type Note, editedLabel, library } from "../../lib/notes";
import { type Caret, Editor } from "./Editor";
import { NoteList } from "./NoteList";
import { Palette } from "./Palette";
import { Preview } from "./Preview";
import { Sidebar, TrafficLights } from "./Sidebar";
import { TasksView } from "./TasksView";
import { useSealed } from "./useSealed";

/** Below this width (in px) the window shows one pane at a time. */
export const COMPACT_WIDTH = 720;

const MODES: readonly { mode: Mode; label: string; icon: ReactNode; wide?: boolean }[] = [
  { mode: "edit", label: "Edit", icon: <PenLine /> },
  { mode: "preview", label: "Preview", icon: <Eye /> },
  { mode: "split", label: "Split", icon: <Columns2 />, wide: true },
];

/** A stable, meaningless id for a note, as the server would list it. */
export function opaqueId(id: string): string {
  let h = 0x811c9dc5;
  for (const char of id) h = Math.imul(h ^ (char.codePointAt(0) as number), 0x01000193) >>> 0;
  return h.toString(16).padStart(8, "0");
}

function headingFor(filter: Filter): string {
  switch (filter.kind) {
    case "all":
    case "tasks":
      return "All Notes";
    case "pinned":
      return "Pinned";
    case "notebook":
      return filter.name;
    case "tag":
      return `#${filter.name}`;
  }
}

export interface KansoWindowProps {
  /** Today, for due dates; injectable for tests. */
  today?: Date;
  className?: string;
}

export function KansoWindow({ today: given, className }: KansoWindowProps) {
  const [today] = useState(() => given ?? new Date());
  const [state, dispatch] = useReducer(reducer, today, (t: Date) => initialState(library(t)));
  const [caret, setCaret] = useState<Caret>({ line: 1, column: 1 });
  const [compact, setCompact] = useState(false);
  const [drawer, setDrawer] = useState(false);
  const root = useRef<HTMLElement>(null);
  const scroller = useRef<HTMLDivElement>(null);
  const note = selectedNote(state);
  const shown = visibleNotes(state);

  useEffect(() => {
    const el = root.current;
    if (!el || typeof ResizeObserver === "undefined") return;
    const observer = new ResizeObserver(([entry]) => setCompact((entry?.contentRect.width ?? Infinity) < COMPACT_WIDTH));
    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (!(event.metaKey || event.ctrlKey) || event.altKey || event.shiftKey) return;
      const key = event.key.toLowerCase();
      const inside = root.current?.contains(document.activeElement) ?? false;
      if (key === "k" || (key === "p" && inside)) {
        event.preventDefault();
        dispatch({ type: "palette", open: true });
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  // Server view seals every name and note the window shows.
  const texts = useMemo(() => {
    if (!state.server) return [];
    const out = new Set<string>(["Start here", "Inside Kanso", "Privacy", "For developers", "Personal"]);
    for (const n of state.notes) {
      out.add(title(n.body));
      out.add(snippet(n.body));
      out.add(n.body);
      for (const tag of tagsOf(n.body)) out.add(tag);
    }
    return [...out];
  }, [state.server, state.notes]);
  const sealing = useSealed(state.server, texts);
  const sealed = (text: string) => sealing.sealed.get(text) ?? "·".repeat(Math.min(24, Math.max(8, text.length)));

  const mode: Mode = compact && state.mode === "split" ? "preview" : state.mode;
  const open = (id: string) => {
    dispatch({ type: "select", id });
    if (state.filter.kind === "tasks") dispatch({ type: "filter", filter: { kind: "all" } });
    setDrawer(false);
    scroller.current?.scrollTo({ top: 0 });
  };
  const goTo = (d: Destination) => {
    open(d.id);
    if (d.anchor) {
      const anchor = d.anchor;
      if (state.mode === "edit") dispatch({ type: "mode", mode: compact ? "preview" : "split" });
      requestAnimationFrame(() => root.current?.querySelector(`#note-${CSS.escape(anchor)}`)?.scrollIntoView({ block: "start" }));
    }
  };
  const onWiki = (target: string) => {
    dispatch({ type: "open-wiki", target });
    scroller.current?.scrollTo({ top: 0 });
  };
  const toggleTask = (line: number) => note && dispatch({ type: "toggle-task", id: note.id, line });

  const list = (
    <NoteList
      heading={headingFor(state.filter)}
      notes={shown}
      selected={state.selected}
      query={state.query}
      server={state.server}
      sealed={sealed}
      onSelect={open}
      onQuery={(query) => dispatch({ type: "query", query })}
      onNew={() => {
        dispatch({ type: "new-note" });
        setDrawer(false);
      }}
    />
  );

  return (
    <section
      ref={root}
      aria-label="Kanso, running in your browser"
      className={cn(
        "@container relative isolate h-[clamp(34rem,80vh,48rem)] overflow-hidden rounded-[12px] bg-editor shadow-window",
        className,
      )}
    >
      <div className="grid h-full grid-cols-[minmax(0,1fr)] @min-[45rem]:grid-cols-[16.5rem_minmax(0,1fr)] @min-[64rem]:grid-cols-[12.5rem_17rem_minmax(0,1fr)]">
        <div className="hidden min-h-0 border-r border-line @min-[64rem]:block">
          <Sidebar
            notes={state.notes}
            filter={state.filter}
            server={state.server}
            sealed={sealed}
            onFilter={(filter) => dispatch({ type: "filter", filter })}
            onServer={(on) => dispatch({ type: "server", on })}
          />
        </div>
        <div className="hidden min-h-0 flex-col border-r border-line @min-[45rem]:flex">
          <div className="px-4 pt-4 @min-[64rem]:hidden">
            <TrafficLights />
          </div>
          <div className="min-h-0 flex-1">{list}</div>
        </div>

        <div className="flex min-h-0 min-w-0 flex-col bg-editor font-[family-name:var(--font-ui)]">
          <header className="flex h-12 shrink-0 items-center gap-2 border-b border-line px-3 text-[13px]">
            <button
              type="button"
              onClick={() => setDrawer(true)}
              aria-label="Show notes"
              className="grid size-7 cursor-pointer place-items-center rounded-md text-fg-2 hover:bg-row-hover @min-[45rem]:hidden"
            >
              <List className="size-4" />
            </button>
            <p className="min-w-0 flex-1 truncate text-fg-3">
              {state.filter.kind === "tasks" ? (
                <span className="text-fg">Tasks</span>
              ) : (
                note && (
                  <>
                    {state.server ? sealed(note.notebook).slice(0, 12) : note.notebook} <span className="px-0.5">/</span>{" "}
                    <span className="text-fg">{state.server ? sealed(title(note.body)).slice(0, 16) : title(note.body)}</span>
                  </>
                )
              )}
            </p>
            {note && state.filter.kind !== "tasks" && (
              <span className="hidden text-[11.5px] text-fg-3 @min-[52rem]:inline">Edited {editedLabel(note.edited).toLowerCase()}</span>
            )}
            <div role="group" aria-label="Show the note as" className="flex rounded-md bg-row p-0.5">
              {MODES.filter((m) => !(compact && m.wide)).map((m) => (
                <button
                  key={m.mode}
                  type="button"
                  aria-pressed={mode === m.mode}
                  disabled={state.server || state.filter.kind === "tasks"}
                  onClick={() => dispatch({ type: "mode", mode: m.mode })}
                  className={cn(
                    "flex cursor-pointer items-center gap-1.5 rounded-[5px] px-2 py-1 text-[12px] text-fg-2 transition-colors disabled:cursor-not-allowed disabled:opacity-50 [&_svg]:size-3.5",
                    mode === m.mode && "bg-elevated text-fg shadow-[0_1px_2px_rgb(0_0_0/0.08)]",
                  )}
                >
                  {m.icon}
                  <span className="hidden @min-[36rem]:inline">{m.label}</span>
                </button>
              ))}
            </div>
            <button
              type="button"
              aria-pressed={state.server}
              onClick={() => dispatch({ type: "server", on: !state.server })}
              className={cn(
                "flex cursor-pointer items-center gap-1.5 rounded-md px-2.5 py-1.5 text-[12px] font-medium transition-colors",
                state.server ? "bg-btn text-btn-fg" : "text-fg-2 ring-1 ring-line hover:bg-row-hover",
              )}
            >
              <Lock className="size-3.5" />
              <span>Server view</span>
            </button>
          </header>

          <div ref={scroller} className="relative min-h-0 flex-1 overflow-y-auto">
            {state.filter.kind === "tasks" ? (
              <div className="mx-auto max-w-[40rem] px-6 py-7">
                <TasksView
                  notes={state.notes}
                  today={today}
                  onToggle={(id, line) => dispatch({ type: "toggle-task", id, line })}
                  onOpen={open}
                />
              </div>
            ) : state.server && note ? (
              <ServerPane note={note} sealed={sealed} size={sealing.sizes.get(note.body)} fingerprint={sealing.fingerprint} onRekey={sealing.rekey} />
            ) : note ? (
              <Panes
                note={note}
                mode={mode}
                onEdit={(body) => dispatch({ type: "edit", id: note.id, body })}
                onCaret={setCaret}
                onToggleTask={toggleTask}
                onOpenWiki={onWiki}
              />
            ) : null}
          </div>

          <footer className="flex h-7 shrink-0 items-center gap-3 border-t border-line px-3 text-[11px] text-fg-3 tabular-nums">
            {note && state.filter.kind !== "tasks" && !state.server ? (
              <>
                <span>
                  Ln {caret.line}, Col {caret.column}
                </span>
                <span>{wordCount(note.body).toLocaleString("en")} words</span>
                <span className="hidden @min-[30rem]:inline">{note.body.length.toLocaleString("en")} chars</span>
              </>
            ) : (
              <span>{state.server ? "Sealed in this tab with AES-256-GCM" : `${state.notes.length} notes`}</span>
            )}
            <button
              type="button"
              onClick={() => dispatch({ type: "palette", open: true })}
              className="ml-auto cursor-pointer rounded px-1 hover:text-fg-2"
            >
              ⌘P Quick open
            </button>
          </footer>
        </div>
      </div>

      {drawer && (
        <div className="absolute inset-0 z-20 flex" data-testid="drawer">
          <div className="pop flex w-[min(20rem,85%)] flex-col border-r border-line bg-list shadow-xl">
            <div className="flex items-center gap-1 px-2 pt-2">
              {(
                [
                  [{ kind: "all" }, "All"],
                  [{ kind: "pinned" }, "Pinned"],
                  [{ kind: "tasks" }, "Tasks"],
                ] as const
              ).map(([filter, label]) => (
                <button
                  key={label}
                  type="button"
                  aria-pressed={state.filter.kind === filter.kind}
                  onClick={() => {
                    dispatch({ type: "filter", filter });
                    if (filter.kind === "tasks") setDrawer(false);
                  }}
                  className={cn(
                    "cursor-pointer rounded-md px-2.5 py-1 text-[12px] text-fg-2",
                    state.filter.kind === filter.kind && "bg-row text-fg",
                  )}
                >
                  {label}
                </button>
              ))}
              <button
                type="button"
                onClick={() => setDrawer(false)}
                aria-label="Hide notes"
                className="ml-auto grid size-7 cursor-pointer place-items-center rounded-md text-fg-2 hover:bg-row-hover"
              >
                <X className="size-4" />
              </button>
            </div>
            <div className="min-h-0 flex-1">{list}</div>
          </div>
          <button type="button" aria-label="Hide notes" className="flex-1 cursor-default bg-fg/10" onClick={() => setDrawer(false)} />
        </div>
      )}

      {state.palette && (
        <Palette
          notes={state.notes}
          onOpen={(d) => {
            dispatch({ type: "palette", open: false });
            goTo(d);
          }}
          onClose={() => dispatch({ type: "palette", open: false })}
        />
      )}
    </section>
  );
}

function Panes({
  note,
  mode,
  onEdit,
  onCaret,
  onToggleTask,
  onOpenWiki,
}: {
  note: Note;
  mode: Mode;
  onEdit: (body: string) => void;
  onCaret: (caret: Caret) => void;
  onToggleTask: (line: number) => void;
  onOpenWiki: (target: string) => void;
}) {
  const editor = <Editor key={note.id} value={note.body} onChange={onEdit} onCaret={onCaret} label={`${title(note.body)}, Markdown`} />;
  const preview = <Preview source={note.body} onToggleTask={onToggleTask} onOpenWiki={onOpenWiki} />;
  if (mode === "split") {
    return (
      <div className="grid min-h-full grid-cols-2">
        <div className="border-r border-line px-6 py-6">{editor}</div>
        <div className="px-7 py-6">{preview}</div>
      </div>
    );
  }
  return <div className="mx-auto max-w-[44rem] px-6 py-7 @min-[45rem]:px-10">{mode === "edit" ? editor : preview}</div>;
}

function ServerPane({
  note,
  sealed,
  size,
  fingerprint,
  onRekey,
}: {
  note: Note;
  sealed: (text: string) => string;
  size: number | undefined;
  fingerprint: string | null;
  onRekey: () => void;
}) {
  return (
    <div className="mx-auto max-w-[44rem] px-6 py-7 text-[13px] @min-[45rem]:px-10" data-testid="server-pane">
      <p className="flex items-center gap-2 text-[11px] font-semibold tracking-wide text-accent uppercase">
        <Lock className="size-3.5" /> What the server stores for this note
      </p>
      <dl className="mt-4 grid grid-cols-[auto_1fr] gap-x-6 gap-y-1.5 text-fg-2">
        <dt className="text-fg-3">Note</dt>
        <dd className="font-mono text-[12px]">n_{opaqueId(note.id)}</dd>
        <dt className="text-fg-3">Notebook</dt>
        <dd className="font-mono text-[12px]">b_{opaqueId(note.notebook)}</dd>
        <dt className="text-fg-3">Edited</dt>
        <dd>{editedLabel(note.edited)}</dd>
        <dt className="text-fg-3">Size</dt>
        <dd>{size === undefined ? "…" : `${size.toLocaleString("en")} bytes`}</dd>
      </dl>
      <p className="mt-5 font-mono text-[12px] leading-relaxed break-all text-fg" data-testid="ciphertext">
        {sealed(note.body)}
      </p>
      <div className="mt-6 flex flex-wrap items-center gap-x-4 gap-y-2 border-t border-line pt-4 text-[12px] text-fg-3">
        <span>
          Key <span className="font-mono text-fg-2">{fingerprint ?? "…"}</span>, made in this tab and never sent
        </span>
        <button
          type="button"
          onClick={onRekey}
          className="ml-auto flex cursor-pointer items-center gap-1.5 rounded-md px-2 py-1 text-fg-2 ring-1 ring-line hover:bg-row-hover"
        >
          <RefreshCw className="size-3.5" /> New key
        </button>
      </div>
      <p className="mt-6 text-fg-3">Turn off Server view to read and edit the note.</p>
    </div>
  );
}
