/**
 * The Markdown editor: a textarea laid exactly over a coloured copy of its
 * text, so it types like a textarea and reads like the app's editor. The
 * keys the app treats kindly (pairs, Return in lists, Tab, Backspace) go
 * through lib/editing first. It reports the caret's line and column.
 */
import { type KeyboardEvent, useLayoutEffect, useRef } from "react";
import { type TextState, onBackspace, onEnter, onTab, onType } from "../../lib/editing";
import { highlight } from "../../lib/markdown";

export interface Caret {
  readonly line: number;
  readonly column: number;
}

export interface EditorProps {
  value: string;
  onChange: (value: string) => void;
  onCaret?: (caret: Caret) => void;
  label: string;
}

export function caretAt(value: string, index: number): Caret {
  const before = value.slice(0, index);
  const line = before.split("\n").length;
  return { line, column: index - before.lastIndexOf("\n") };
}

export function Editor({ value, onChange, onCaret, label }: EditorProps) {
  const area = useRef<HTMLTextAreaElement>(null);
  const pending = useRef<{ start: number; end: number } | null>(null);

  useLayoutEffect(() => {
    const el = area.current;
    if (el && pending.current) {
      el.setSelectionRange(pending.current.start, pending.current.end);
      pending.current = null;
    }
  });

  const report = () => {
    const el = area.current;
    if (el && onCaret) onCaret(caretAt(el.value, el.selectionStart));
  };

  const apply = (next: TextState | null, event: KeyboardEvent): void => {
    if (next === null) return;
    event.preventDefault();
    if (next.value !== value) {
      // The caret is restored once React has rewritten the textarea's text.
      pending.current = { start: next.start, end: next.end };
      onChange(next.value);
    } else {
      // Nothing re-renders, so move the caret now; leaving it pending would
      // pull it back on the next, unrelated keystroke.
      area.current?.setSelectionRange(next.start, next.end);
    }
    if (onCaret) onCaret(caretAt(next.value, next.start));
  };

  const onKeyDown = (event: KeyboardEvent<HTMLTextAreaElement>) => {
    if (event.nativeEvent.isComposing || event.metaKey || event.ctrlKey || event.altKey) return;
    const el = event.currentTarget;
    const state = { value: el.value, start: el.selectionStart, end: el.selectionEnd };
    if (event.key === "Enter" && !event.shiftKey) apply(onEnter(state), event);
    else if (event.key === "Tab") apply(onTab(state, event.shiftKey), event);
    else if (event.key === "Backspace") apply(onBackspace(state), event);
    else if (event.key.length === 1) apply(onType(state, event.key), event);
  };

  const lines = highlight(value);

  return (
    <div className="grid min-h-full">
      <pre aria-hidden="true" className="source pointer-events-none m-0 [grid-area:1/1] text-fg">
        {lines.map((spans, i) => (
          <span key={i}>
            {spans.map((span, k) => (
              <span key={k} className={span.tone === "plain" ? undefined : `tone-${span.tone}`}>
                {span.text}
              </span>
            ))}
            {i < lines.length - 1 ? "\n" : "\n "}
          </span>
        ))}
      </pre>
      <textarea
        ref={area}
        aria-label={label}
        value={value}
        spellCheck={false}
        autoCapitalize="sentences"
        onChange={(e) => {
          onChange(e.target.value);
          if (onCaret) onCaret(caretAt(e.target.value, e.target.selectionStart));
        }}
        onKeyDown={onKeyDown}
        onSelect={report}
        onClick={report}
        className="source m-0 resize-none overflow-hidden border-0 bg-transparent p-0 text-transparent caret-accent outline-none [grid-area:1/1] selection:bg-accent/25 selection:text-transparent focus-visible:outline-none"
      />
    </div>
  );
}
