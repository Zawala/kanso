/**
 * The note list, like the app's: a heading with the count and a new-note
 * button, a search field, and a row per note with its title, a line of its
 * text, when it was edited and how many of its tasks are done. In server
 * view the titles and text are shown as the server holds them.
 */
import { Pin, Plus, Search } from "lucide-react";
import { cn } from "../../lib/cn";
import { snippet, tasks, title } from "../../lib/markdown";
import { type Note, editedLabel } from "../../lib/notes";

export interface NoteListProps {
  heading: string;
  notes: readonly Note[];
  selected: string;
  query: string;
  server: boolean;
  sealed: (text: string) => string;
  onSelect: (id: string) => void;
  onQuery: (query: string) => void;
  onNew: () => void;
}

export function NoteList({ heading, notes, selected, query, server, sealed, onSelect, onQuery, onNew }: NoteListProps) {
  return (
    <div className="flex h-full flex-col bg-list font-[family-name:var(--font-ui)] text-[13px]">
      <div className="flex items-center gap-2 px-4 pt-3.5 pb-2">
        <h2 className="min-w-0 flex-1 truncate text-[13px] font-semibold text-fg">
          {heading} <span className="font-normal text-fg-3">{notes.length}</span>
        </h2>
        <button
          type="button"
          onClick={onNew}
          aria-label="New note"
          disabled={server}
          className="grid size-6 cursor-pointer place-items-center rounded-md text-fg-2 transition-colors hover:bg-row-hover disabled:cursor-not-allowed disabled:opacity-40"
        >
          <Plus className="size-4" />
        </button>
      </div>
      <label className="mx-3 mb-2 flex items-center gap-2 rounded-md bg-row px-2.5 py-1.5 text-fg-3">
        <Search className="size-3.5 shrink-0" />
        <input
          value={query}
          onChange={(e) => onQuery(e.target.value)}
          placeholder="Search notes"
          aria-label="Search notes"
          className="min-w-0 flex-1 bg-transparent text-fg outline-none placeholder:text-fg-3"
        />
      </label>
      <ul className="min-h-0 flex-1 overflow-y-auto px-2 pb-2" aria-label="Notes">
        {notes.length === 0 && <li className="px-2 py-6 text-center text-fg-3">No notes here.</li>}
        {notes.map((note) => {
          const done = tasks(note.body);
          const name = title(note.body);
          const text = snippet(note.body);
          return (
            <li key={note.id}>
              <button
                type="button"
                aria-current={note.id === selected ? "true" : undefined}
                onClick={() => onSelect(note.id)}
                className={cn(
                  "grid w-full cursor-pointer gap-0.5 rounded-lg px-2.5 py-2 text-left transition-colors",
                  note.id === selected ? "bg-row" : "hover:bg-row-hover",
                )}
              >
                <span className="flex items-baseline gap-2">
                  {note.pinned && <Pin className="size-3 shrink-0 self-center text-fg-3" />}
                  <span className={cn("min-w-0 flex-1 truncate font-semibold text-fg", server && "font-mono text-[11.5px] font-normal")}>
                    {server ? sealed(name) : name}
                  </span>
                  {done.length > 0 && !server && (
                    <span className="shrink-0 rounded-[4px] bg-fill px-1 text-[10.5px] text-fg-2 tabular-nums">
                      {done.filter((t) => t.checked).length}/{done.length}
                    </span>
                  )}
                  <span className="shrink-0 text-[11px] text-fg-3">{editedLabel(note.edited)}</span>
                </span>
                <span className={cn("line-clamp-2 text-[12px] leading-snug text-fg-2", server && "font-mono text-[11px] break-all")}>
                  {server ? sealed(text) : text}
                </span>
              </button>
            </li>
          );
        })}
      </ul>
    </div>
  );
}
