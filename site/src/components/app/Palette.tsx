/**
 * Quick open, as ⌘P opens it in the app: type a few letters of a note's
 * title or of a heading inside it, choose with the arrows, Return to open.
 * Escape or a click outside closes it.
 */
import { FileText, Hash } from "lucide-react";
import { type KeyboardEvent, useEffect, useRef, useState } from "react";
import { type Destination, destinations } from "../../lib/library";
import type { Note } from "../../lib/notes";

export interface PaletteProps {
  notes: readonly Note[];
  onOpen: (destination: Destination) => void;
  onClose: () => void;
}

export function Palette({ notes, onOpen, onClose }: PaletteProps) {
  const [query, setQuery] = useState("");
  const [active, setActive] = useState(0);
  const input = useRef<HTMLInputElement>(null);
  const results = destinations(notes, query);

  useEffect(() => input.current?.focus(), []);

  const onKeyDown = (event: KeyboardEvent) => {
    if (event.key === "Escape") {
      event.preventDefault();
      onClose();
    } else if (event.key === "ArrowDown") {
      event.preventDefault();
      setActive((a) => Math.min(a + 1, results.length - 1));
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      setActive((a) => Math.max(a - 1, 0));
    } else if (event.key === "Enter") {
      event.preventDefault();
      const chosen = results[active];
      if (chosen) onOpen(chosen);
    }
  };

  return (
    <div className="absolute inset-0 z-30 flex justify-center bg-fg/10 pt-[12%]" onMouseDown={onClose} data-testid="palette">
      <div
        role="dialog"
        aria-label="Quick open"
        onMouseDown={(e) => e.stopPropagation()}
        className="pop h-fit w-[min(34rem,calc(100%-2rem))] overflow-hidden rounded-xl border border-line bg-elevated font-[family-name:var(--font-ui)] text-[13px] shadow-[0_24px_60px_-12px_rgb(0_0_0/0.35)]"
      >
        <input
          ref={input}
          value={query}
          onChange={(e) => {
            setQuery(e.target.value);
            setActive(0);
          }}
          onKeyDown={onKeyDown}
          placeholder="Open a note or heading…"
          aria-label="Search notes and headings"
          role="combobox"
          aria-expanded="true"
          aria-controls="palette-results"
          className="w-full border-b border-line bg-transparent px-4 py-3 text-[15px] text-fg outline-none placeholder:text-fg-3"
        />
        <ul id="palette-results" role="listbox" className="max-h-80 overflow-y-auto p-1.5">
          {results.length === 0 && <li className="px-3 py-2.5 text-fg-3">No notes match “{query}”.</li>}
          {results.map((d, i) => (
            <li
              key={`${d.id}:${d.anchor ?? ""}`}
              role="option"
              aria-selected={i === active}
              onMouseEnter={() => setActive(i)}
              onClick={() => onOpen(d)}
              className={"flex cursor-pointer items-center gap-2.5 rounded-lg px-3 py-2 " + (i === active ? "bg-row" : "")}
            >
              {d.anchor ? <Hash className="size-3.5 shrink-0 text-fg-3" /> : <FileText className="size-3.5 shrink-0 text-fg-3" />}
              <span className="truncate text-fg">{d.label}</span>
              <span className="ml-auto shrink-0 text-[11.5px] text-fg-3">{d.detail}</span>
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}
