/**
 * The Tasks view: every checkbox with a date, from every note, under
 * overdue, today, coming up and done. Ticking one here ticks it in its note,
 * as in the app.
 */
import { Check } from "lucide-react";
import { tasks, title } from "../../lib/markdown";
import { type Note, dayFrom } from "../../lib/notes";

export interface TaskRow {
  readonly id: string;
  readonly note: string;
  readonly line: number;
  readonly text: string;
  readonly due: string | null;
  readonly checked: boolean;
}

export type Group = "Overdue" | "Today" | "Upcoming" | "No date" | "Done";
export const GROUPS: readonly Group[] = ["Overdue", "Today", "Upcoming", "No date", "Done"];

/** Which heading a task sits under, given today's date as `YYYY-MM-DD`. */
export function groupOf(task: Pick<TaskRow, "due" | "checked">, today: string): Group {
  if (task.checked) return "Done";
  if (task.due === null) return "No date";
  if (task.due < today) return "Overdue";
  if (task.due === today) return "Today";
  return "Upcoming";
}

export function taskRows(notes: readonly Note[]): TaskRow[] {
  return notes.flatMap((note) => tasks(note.body).map((t) => ({ id: note.id, note: title(note.body), ...t })));
}

export interface TasksViewProps {
  notes: readonly Note[];
  today: Date;
  onToggle: (id: string, line: number) => void;
  onOpen: (id: string) => void;
}

function friendly(due: string, today: Date): string {
  if (due === dayFrom(today, 0)) return "Today";
  if (due === dayFrom(today, 1)) return "Tomorrow";
  if (due === dayFrom(today, -1)) return "Yesterday";
  const [y, m, d] = due.split("-").map(Number);
  return new Date(y as number, (m as number) - 1, d).toLocaleDateString("en-GB", { weekday: "short", day: "numeric", month: "short" });
}

export function TasksView({ notes, today, onToggle, onOpen }: TasksViewProps) {
  const now = dayFrom(today, 0);
  const rows = taskRows(notes);
  return (
    <div className="font-[family-name:var(--font-ui)] text-[13px]" data-testid="tasks-view">
      <h2 className="text-[1.35rem] font-bold tracking-tight">Tasks</h2>
      <p className="mt-1 text-fg-3">From every note. Tick one here and it's ticked in the note too.</p>
      {GROUPS.map((group) => {
        const inGroup = rows.filter((r) => groupOf(r, now) === group);
        if (inGroup.length === 0) return null;
        return (
          <section key={group} className="mt-6">
            <h3 className={"mb-1.5 text-[11px] font-semibold tracking-wide uppercase " + (group === "Overdue" ? "text-bad" : "text-fg-3")}>
              {group} <span className="font-normal text-fg-3">{inGroup.length}</span>
            </h3>
            <ul className="grid">
              {inGroup.map((row) => (
                <li key={`${row.id}:${row.line}`} className="flex items-center gap-3 border-b border-line/70 py-2 last:border-0">
                  <button
                    type="button"
                    role="checkbox"
                    aria-checked={row.checked}
                    aria-label={`${row.checked ? "Mark as not done" : "Mark as done"}: ${row.text}`}
                    onClick={() => onToggle(row.id, row.line)}
                    className={
                      "grid size-4 shrink-0 cursor-pointer place-items-center rounded-full border transition-colors " +
                      (row.checked ? "border-accent bg-accent text-btn-fg" : "border-fg-3 hover:border-accent")
                    }
                  >
                    {row.checked && <Check className="size-2.5" strokeWidth={3.5} />}
                  </button>
                  <button type="button" onClick={() => onOpen(row.id)} className="min-w-0 flex-1 cursor-pointer text-left">
                    <span className={"block truncate " + (row.checked ? "text-fg-3 line-through" : "text-fg")}>{row.text}</span>
                    <span className="block truncate text-[11.5px] text-fg-3">{row.note}</span>
                  </button>
                  {row.due && (
                    <span
                      className={
                        "shrink-0 rounded-[4px] px-1.5 py-0.5 text-[11px] " +
                        (group === "Overdue" ? "bg-bad/12 text-bad" : "bg-kw/12 text-kw")
                      }
                    >
                      {friendly(row.due, today)}
                    </span>
                  )}
                </li>
              ))}
            </ul>
          </section>
        );
      })}
    </div>
  );
}
