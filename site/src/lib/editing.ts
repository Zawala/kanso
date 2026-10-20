/**
 * What the in-browser editor does with a key before the textarea does: the
 * same small kindnesses as the app's editor. Brackets and quotes pair,
 * typing a closer steps over one that's already there, Backspace removes an
 * empty pair, Return carries a list on (and ends it on an empty item), and
 * Tab indents a list item. Each function takes the text and selection and
 * returns the new ones, or null to let the browser handle the key.
 */

export interface TextState {
  readonly value: string;
  readonly start: number;
  readonly end: number;
}

export const PAIRS: Readonly<Record<string, string>> = {
  "(": ")",
  "[": "]",
  "{": "}",
  '"': '"',
  "`": "`",
};

const CLOSERS = new Set(Object.values(PAIRS));
const WORD = /[\p{L}\p{N}_]/u;
const ITEM = /^(\s*)([-*+]|(\d{1,9})([.)]))(\s+)(\[[ xX]\]\s+)?(.*)$/;

function lineAt(value: string, index: number): { from: number; to: number; text: string } {
  const from = value.lastIndexOf("\n", index - 1) + 1;
  const newline = value.indexOf("\n", index);
  const to = newline === -1 ? value.length : newline;
  return { from, to, text: value.slice(from, to) };
}

function replace(state: TextState, from: number, to: number, text: string, caret = from + text.length): TextState {
  return { value: state.value.slice(0, from) + text + state.value.slice(to), start: caret, end: caret };
}

/** A typed character. */
export function onType(state: TextState, char: string): TextState | null {
  const { value, start, end } = state;
  const close = PAIRS[char];
  if (close !== undefined && start !== end) {
    const inner = value.slice(start, end);
    return { value: value.slice(0, start) + char + inner + close + value.slice(end), start: start + 1, end: end + 1 };
  }
  if (CLOSERS.has(char) && start === end && value[start] === char) {
    return { value, start: start + 1, end: start + 1 };
  }
  if (close === undefined || start !== end) return null;
  const before = value[start - 1];
  const after = value[start];
  // A quote after a letter is an apostrophe or a closing quote; a pair
  // only opens before a space, a closer or the end of the line.
  if ((char === '"' || char === "`") && before !== undefined && WORD.test(before)) return null;
  if (after !== undefined && !/[\s)\]}]/.test(after)) return null;
  return replace(state, start, end, char + close, start + 1);
}

/** Backspace: an empty pair goes in one keystroke. */
export function onBackspace(state: TextState): TextState | null {
  const { value, start, end } = state;
  if (start !== end || start === 0) return null;
  const open = value[start - 1] as string;
  const close = PAIRS[open];
  if (close === undefined || value[start] !== close) return null;
  return replace(state, start - 1, start + 1, "");
}

/** Return: carry the list on, or end it on an empty item. */
export function onEnter(state: TextState): TextState | null {
  const { value, start, end } = state;
  if (start !== end) return null;
  const line = lineAt(value, start);
  const m = ITEM.exec(line.text);
  if (!m) return null;
  const [, indent = "", bullet = "", number, delimiter = "", gap = "", box, body = ""] = m;
  if (body.trim() === "" && start === line.to) {
    return replace(state, line.from, line.to, "");
  }
  const marker = number === undefined ? bullet : `${Number(number) + 1}${delimiter}`;
  const next = `\n${indent}${marker}${gap}${box === undefined ? "" : "[ ] "}`;
  return replace(state, start, end, next);
}

/** Tab or Shift-Tab on a list item: indent or outdent it by one level. */
export function onTab(state: TextState, outdent: boolean): TextState | null {
  const { value, start, end } = state;
  const line = lineAt(value, start);
  if (end > line.to || !ITEM.test(line.text)) return null;
  if (!outdent) {
    return { value: value.slice(0, line.from) + "  " + value.slice(line.from), start: start + 2, end: end + 2 };
  }
  const remove = line.text.startsWith("  ") ? 2 : line.text.startsWith("\t") || line.text.startsWith(" ") ? 1 : 0;
  if (remove === 0) return state;
  const shift = (i: number) => Math.max(line.from, i - remove);
  return { value: value.slice(0, line.from) + value.slice(line.from + remove), start: shift(start), end: shift(end) };
}
