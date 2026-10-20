/**
 * The window's sidebar, laid out like the app's: the library's views, the
 * notebooks with their counts, the tags, and the sync status at the foot.
 * In server view the names are shown as the server holds them: sealed.
 */
import { CheckSquare, FileText, Folder, Lock, Pin } from "lucide-react";
import type { ReactNode } from "react";
import { cn } from "../../lib/cn";
import { type Filter, allTags } from "../../lib/library";
import { NOTEBOOKS, type Note } from "../../lib/notes";
import { Logo } from "../Logo";
import { taskRows } from "./TasksView";

export interface SidebarProps {
  notes: readonly Note[];
  filter: Filter;
  server: boolean;
  /** How the server holds a name, in server view. */
  sealed: (text: string) => string;
  onFilter: (filter: Filter) => void;
  onServer: (on: boolean) => void;
}

function same(a: Filter, b: Filter): boolean {
  if (a.kind !== b.kind) return false;
  return !("name" in a) || a.name === (b as typeof a).name;
}

export function TrafficLights() {
  return (
    <div aria-hidden="true" className="flex gap-2">
      <span className="size-3 rounded-full bg-[#ff5f57] ring-1 ring-black/10 ring-inset" />
      <span className="size-3 rounded-full bg-[#febc2e] ring-1 ring-black/10 ring-inset" />
      <span className="size-3 rounded-full bg-[#28c840] ring-1 ring-black/10 ring-inset" />
    </div>
  );
}

export function Sidebar({ notes, filter, server, sealed, onFilter, onServer }: SidebarProps) {
  const open = taskRows(notes).filter((t) => !t.checked).length;
  const row = (target: Filter, icon: ReactNode, label: string, count?: number) => {
    const active = same(filter, target);
    return (
      <button
        key={target.kind + ("name" in target ? `:${target.name}` : "")}
        type="button"
        aria-current={active ? "true" : undefined}
        onClick={() => onFilter(target)}
        className={cn(
          "flex w-full cursor-pointer items-center gap-2 rounded-md px-2 py-[5px] text-left text-fg-2 transition-colors",
          active ? "bg-side-active text-fg" : "hover:bg-side-hover",
        )}
      >
        <span className="text-fg-3 [&_svg]:size-[15px]">{icon}</span>
        <span className="min-w-0 flex-1 truncate">{label}</span>
        {count !== undefined && <span className="text-[11.5px] text-fg-3 tabular-nums">{count}</span>}
      </button>
    );
  };

  return (
    <nav aria-label="Library" className="flex h-full flex-col bg-sidebar font-[family-name:var(--font-ui)] text-[13px]">
      <div className="px-4 pt-4 pb-3">
        <TrafficLights />
      </div>
      <div className="min-h-0 flex-1 overflow-y-auto px-2 pb-3">
        <div className="grid gap-px">
          {row({ kind: "all" }, <FileText />, "All Notes", notes.length)}
          {row({ kind: "pinned" }, <Pin />, "Pinned", notes.filter((n) => n.pinned).length)}
          {row({ kind: "tasks" }, <CheckSquare />, "Tasks", open)}
        </div>
        <p className="mt-5 mb-1 px-2 text-[11px] font-semibold text-fg-3">Notebooks</p>
        <div className="grid gap-px">
          {NOTEBOOKS.map((name) =>
            row({ kind: "notebook", name }, <Folder />, server ? sealed(name) : name, notes.filter((n) => n.notebook === name).length),
          )}
        </div>
        <p className="mt-5 mb-1.5 px-2 text-[11px] font-semibold text-fg-3">Tags</p>
        <div className="flex flex-wrap gap-1 px-1.5">
          {allTags(notes).map((tag) => (
            <button
              key={tag.name}
              type="button"
              aria-pressed={filter.kind === "tag" && filter.name === tag.name}
              onClick={() => onFilter({ kind: "tag", name: tag.name })}
              className={cn(
                "max-w-full cursor-pointer truncate rounded-[5px] border border-line px-1.5 py-0.5 text-[11.5px] text-fg-2 transition-colors hover:bg-side-hover",
                filter.kind === "tag" && filter.name === tag.name && "border-transparent bg-side-active text-fg",
              )}
            >
              #{server ? sealed(tag.name).slice(0, 10) : tag.name} <span className="text-fg-3">{tag.count}</span>
            </button>
          ))}
        </div>
      </div>
      <button
        type="button"
        onClick={() => onServer(!server)}
        className="flex cursor-pointer items-center gap-2 border-t border-line px-4 py-3 text-left text-[12px] text-fg-2 transition-colors hover:bg-side-hover"
      >
        <span className="grid size-7 shrink-0 place-items-center rounded-full bg-fill text-fg">
          <Logo className="size-4" />
        </span>
        <span className="min-w-0 flex-1 leading-tight">
          <span className="block truncate text-fg">This browser</span>
          <span className="flex items-center gap-1 text-fg-3">
            <span className="size-1.5 rounded-full bg-ok" /> Synced, end-to-end
          </span>
        </span>
        <Lock className={cn("size-3.5", server ? "text-accent" : "text-fg-3")} />
      </button>
    </nav>
  );
}
