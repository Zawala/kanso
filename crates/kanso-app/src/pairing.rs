//! Auto-pairing as you type: brackets, quotes and Markdown's emphasis and
//! code markers get their closer, typing the closer steps over it,
//! Backspace in an empty pair takes both, and a selection is wrapped
//! rather than replaced. Pure: text and selection in, one edit out.
//!
//! gpui-base's own pairing (`LanguageConfig::auto_closing_pairs`) pairs one
//! typed character at a time and steps over any closer it finds, so typing
//! `**` there gives `**` with nothing closed, and it never wraps a
//! selection. Markdown needs both, and needs to know that `*` at the start
//! of a line is usually a bullet, so Kanso pairs on its own and turns the
//! editor's off.
//!
//! The pairs Kanso put in are remembered, anchored to the start of the text
//! (the opener) and to its end (the closer), so typing between them keeps
//! them; an edit before the opener or after the closer, or the cursor
//! leaving the pair, forgets it. Only a remembered closer is stepped over,
//! and only a remembered empty pair goes with one Backspace.
//!
//! Offsets are UTF-8 byte offsets, as the editor's selection is.

use std::ops::Range;

use crate::markdown_edit::{self, Edit, Format, line_end, line_start};

/// The pairs, by opener. Emphasis and code only pair outside code blocks.
const PAIRS: [(char, &str); 7] = [
    ('(', ")"),
    ('[', "]"),
    ('{', "}"),
    ('"', "\""),
    ('*', "*"),
    ('_', "_"),
    ('`', "`"),
];

/// Characters a new pair may go in front of, besides whitespace and the end
/// of the text: closers and punctuation, so `(|)` stays a pair when another
/// opener is typed inside it.
const PAIR_BEFORE: &str = ")]}>.,;:!?*_`\"'~";

fn closer_for(ch: char) -> Option<&'static str> {
    PAIRS.iter().find(|(open, _)| *open == ch).map(|(_, c)| *c)
}

/// Markdown's own markers, which pair only in prose.
fn is_marker(ch: char) -> bool {
    matches!(ch, '*' | '_' | '`')
}

/// A pair Kanso put in, still standing.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Tracked {
    /// Where the opener starts, from the start of the text.
    open_at: usize,
    open: String,
    /// How far the closer's end is from the end of the text.
    close_from_end: usize,
    /// What is left of the closer, ahead of the cursor.
    close: String,
}

impl Tracked {
    fn open_end(&self) -> usize {
        self.open_at + self.open.len()
    }

    /// Where the closer is in `text`, when the pair still stands there:
    /// the opener and the closer in place, the opener first.
    fn close_range(&self, text: &str) -> Option<Range<usize>> {
        let end = text.len().checked_sub(self.close_from_end)?;
        let start = end.checked_sub(self.close.len())?;
        let open = text.get(self.open_at..self.open_end())?;
        let close = text.get(start..end)?;
        (open == self.open && close == self.close && self.open_end() <= start).then_some(start..end)
    }

    /// Whether `cursor` is inside the pair, between opener and closer.
    fn holds(&self, text: &str, cursor: usize) -> Option<Range<usize>> {
        let close = self.close_range(text)?;
        (self.open_end() <= cursor && cursor <= close.start).then_some(close)
    }
}

/// The pairs Kanso put in that are still open, innermost last.
#[derive(Clone, Debug, Default)]
pub struct Pairs {
    tracked: Vec<Tracked>,
}

/// Whether `at` is inside a fenced code block: an odd number of fence lines
/// (```` ``` ```` or `~~~` after at most three spaces) before its line.
pub fn in_fenced_code(text: &str, at: usize) -> bool {
    let start = line_start(text, at.min(text.len()));
    text[..start]
        .lines()
        .filter(|line| {
            let indent = line.len() - line.trim_start_matches(' ').len();
            let rest = &line[indent..];
            indent <= 3 && (rest.starts_with("```") || rest.starts_with("~~~"))
        })
        .count()
        % 2
        == 1
}

/// A cursor move, with no text changed.
fn move_to(at: usize) -> Edit {
    Edit {
        range: at..at,
        text: String::new(),
        selection: at..at,
    }
}

impl Pairs {
    /// Forgets every pair, as when another note opens.
    pub fn clear(&mut self) {
        self.tracked.clear();
    }

    /// Whether no pair is remembered: then only an opener can pair, and
    /// closers, spaces and Backspace type as usual.
    pub fn is_empty(&self) -> bool {
        self.tracked.is_empty()
    }

    /// Whether typing `ch` may pair, step over a closer or end a pair, so
    /// that every other key skips reading the text.
    pub fn may_handle(&self, ch: char) -> bool {
        closer_for(ch).is_some() || (!self.is_empty() && matches!(ch, ')' | ']' | '}' | ' '))
    }

    /// Drops the pairs the text or the cursor has left behind.
    fn prune(&mut self, text: &str, cursor: usize) {
        self.tracked.retain(|p| p.holds(text, cursor).is_some());
    }

    /// The innermost remembered pair around `cursor` with nothing between
    /// opener and closer.
    fn empty_pair_at(&self, text: &str, cursor: usize) -> Option<(usize, Range<usize>)> {
        let ix = self.tracked.len().checked_sub(1)?;
        let pair = &self.tracked[ix];
        let close = pair.holds(text, cursor)?;
        (pair.open_end() == cursor && close.start == cursor).then_some((ix, close))
    }

    /// The edit for typing `ch` with `selection` selected in `text`, when
    /// pairing changes what typing would do; `None` types it as usual.
    pub fn typed(&mut self, text: &str, selection: Range<usize>, ch: char) -> Option<Edit> {
        let selection = clamp(text, selection);
        let at = selection.start;
        self.prune(text, at);
        // Only markers care, and the check reads the text up to the cursor.
        let in_code = is_marker(ch) && in_fenced_code(text, at);
        if !selection.is_empty() {
            self.tracked.clear();
            return wrap(text, selection, ch, in_code);
        }

        // Typed over a closer Kanso put in.
        if let Some(pair) = self.tracked.last()
            && let Some(close) = pair.holds(text, at)
            && close.start == at
            && pair.close.starts_with(ch)
        {
            let empty = pair.open_end() == at;
            let open = pair.open.clone();
            // `*|*` and `*` again: bold, `**|**`.
            if empty && (open == "*" || open == "_") && ch.to_string() == open {
                let ix = self.tracked.len() - 1;
                let pair = &mut self.tracked[ix];
                pair.open = format!("{open}{open}");
                pair.close = format!("{open}{open}");
                return Some(Edit {
                    range: at..at,
                    text: format!("{open}{open}"),
                    selection: at + 1..at + 1,
                });
            }
            let ix = self.tracked.len() - 1;
            let rest = self.tracked[ix].close[ch.len_utf8()..].to_owned();
            if rest.is_empty() {
                self.tracked.pop();
            } else {
                self.tracked[ix].close = rest;
            }
            return Some(move_to(at + ch.len_utf8()));
        }

        // A space in a fresh `*|*` or `_|_`: a bullet or a multiplication,
        // not emphasis. The closer goes.
        if ch == ' '
            && let Some((ix, close)) = self.empty_pair_at(text, at)
            && matches!(self.tracked[ix].open.as_str(), "*" | "_")
        {
            self.tracked.remove(ix);
            return Some(Edit {
                range: at..close.end,
                text: " ".into(),
                selection: at + 1..at + 1,
            });
        }

        // The third backtick at the start of a line: a fenced block.
        if ch == '`' && !in_code {
            let start = line_start(text, at);
            let before = &text[start..at];
            let indent = before.len() - before.trim_start_matches(' ').len();
            let after = &text[at..line_end(text, at)];
            if indent <= 3 && &before[indent..] == "``" && after.trim().is_empty() {
                self.tracked.clear();
                let pad = &before[..indent];
                let block = format!("{pad}```\n{pad}```");
                let cursor = start + indent + 3;
                return Some(Edit {
                    range: start..line_end(text, at),
                    text: block,
                    selection: cursor..cursor,
                });
            }
        }

        let close = closer_for(ch)?;
        if is_marker(ch) && in_code {
            return None;
        }
        let prev = text[..at].chars().next_back();
        let next = text[at..].chars().next();
        if prev == Some('\\') {
            return None;
        }
        let next_ok = next.is_none_or(|n| {
            n.is_whitespace() || (PAIR_BEFORE.contains(n) && !(n == ch && close.starts_with(ch)))
        });
        if !next_ok {
            return None;
        }
        let prev_word = prev.is_some_and(|p| p.is_alphanumeric());
        let symmetric = close.starts_with(ch);
        if symmetric && (prev_word || prev == Some(ch)) {
            return None;
        }
        let open = ch.to_string();
        let after = text.len() - at;
        self.tracked.push(Tracked {
            open_at: at,
            open: open.clone(),
            close_from_end: after,
            close: close.to_owned(),
        });
        let cursor = at + open.len();
        Some(Edit {
            range: at..at,
            text: format!("{open}{close}"),
            selection: cursor..cursor,
        })
    }

    /// The edit for Backspace at `cursor`: both halves of an empty pair
    /// Kanso put in; `None` deletes as usual.
    pub fn backspace(&mut self, text: &str, cursor: usize) -> Option<Edit> {
        let cursor = clamp(text, cursor..cursor).start;
        self.prune(text, cursor);
        let (ix, close) = self.empty_pair_at(text, cursor)?;
        let pair = self.tracked.remove(ix);
        Some(Edit {
            range: pair.open_at..close.end,
            text: String::new(),
            selection: pair.open_at..pair.open_at,
        })
    }

    /// How many pairs are remembered.
    #[cfg(test)]
    fn len(&self) -> usize {
        self.tracked.len()
    }
}

/// Typing an opener over a selection wraps it, keeping what was inside
/// selected, so typing `*` twice makes it bold. Backticks around several
/// lines make a fenced block; other markers leave a selection that spans
/// lines to be typed over.
fn wrap(text: &str, selection: Range<usize>, ch: char, in_code: bool) -> Option<Edit> {
    let close = closer_for(ch)?;
    let selected = &text[selection.clone()];
    let multiline = selected.contains('\n');
    if ch == '`' && multiline && !in_code {
        return Some(markdown_edit::format(Format::Code, text, selection));
    }
    if multiline && is_marker(ch) || ch == '"' && multiline {
        return None;
    }
    if is_marker(ch) && in_code {
        return None;
    }
    let open_len = ch.len_utf8();
    Some(Edit {
        range: selection.clone(),
        text: format!("{ch}{selected}{close}"),
        selection: selection.start + open_len..selection.end + open_len,
    })
}

/// An ordered selection inside the text, on character boundaries.
fn clamp(text: &str, selection: Range<usize>) -> Range<usize> {
    let floor = |offset: usize| {
        let mut offset = offset.min(text.len());
        while !text.is_char_boundary(offset) {
            offset -= 1;
        }
        offset
    };
    let (a, b) = (floor(selection.start), floor(selection.end));
    a.min(b)..a.max(b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 512,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    /// The text and cursor after typing `keys` into `text` at `cursor`, with
    /// pairing doing what it does and every other key typed as usual.
    fn type_keys(text: &str, cursor: usize, keys: &str) -> (String, usize, Pairs) {
        let mut pairs = Pairs::default();
        let mut text = text.to_owned();
        let mut sel = cursor..cursor;
        for key in keys.chars() {
            if key == '\u{8}' {
                match pairs.backspace(&text, sel.start) {
                    Some(edit) => {
                        text = edit.apply(&text);
                        sel = edit.selection;
                    }
                    None if sel.start > 0 => {
                        let prev = text[..sel.start].chars().next_back().unwrap();
                        let at = sel.start - prev.len_utf8();
                        text.replace_range(at..sel.start, "");
                        sel = at..at;
                    }
                    None => {}
                }
                continue;
            }
            match pairs.typed(&text, sel.clone(), key) {
                Some(edit) => {
                    text = edit.apply(&text);
                    sel = edit.selection;
                }
                None => {
                    text.replace_range(sel.clone(), &key.to_string());
                    let at = sel.start + key.len_utf8();
                    sel = at..at;
                }
            }
        }
        (text, sel.start, pairs)
    }

    fn typed(text: &str, keys: &str) -> String {
        let (text, cursor, _) = type_keys(text, text.len(), keys);
        format!("{}|{}", &text[..cursor], &text[cursor..])
    }

    #[test]
    fn brackets_and_quotes_pair() {
        assert_eq!(typed("", "("), "(|)");
        assert_eq!(typed("", "(x)"), "(x)|");
        assert_eq!(typed("", "[x](y)"), "[x](y)|");
        assert_eq!(typed("", "- [ ] "), "- [ ] |");
        assert_eq!(typed("", "\"hi\""), "\"hi\"|");
        assert_eq!(typed("", "{[(\"\""), "{[(\"\"|)]}");
        // Before a word, an opener stays alone.
        assert_eq!(typed_at("word", 0, "("), "(|word");
        // After a word, quotes don't pair: inches, a closing quote.
        assert_eq!(typed("5", "\""), "5\"|");
    }

    fn typed_at(text: &str, cursor: usize, keys: &str) -> String {
        let (text, cursor, _) = type_keys(text, cursor, keys);
        format!("{}|{}", &text[..cursor], &text[cursor..])
    }

    #[test]
    fn emphasis_pairs_and_turns_bold() {
        assert_eq!(typed("a ", "*"), "a *|*");
        assert_eq!(typed("a ", "**"), "a **|**");
        assert_eq!(typed("a ", "**b**"), "a **b**|");
        assert_eq!(typed("a ", "*b*"), "a *b*|");
        assert_eq!(typed("a ", "_b_ "), "a _b_ |");
        // A tilde isn't paired: ~/Documents.
        assert_eq!(typed("a ", "~/"), "a ~/|");
        // A bullet, or 2 * 3: the space takes the closer away.
        assert_eq!(typed("", "* item"), "* item|");
        assert_eq!(typed("2 ", "* 3"), "2 * 3|");
        // Inside a word: snake_case, a*b.
        assert_eq!(typed("snake", "_case"), "snake_case|");
        assert_eq!(typed("a", "*b"), "a*b|");
        // Escaped.
        assert_eq!(typed("\\", "*"), "\\*|");
    }

    #[test]
    fn backticks_pair_and_fence() {
        assert_eq!(typed("", "`"), "`|`");
        assert_eq!(typed("", "`code`"), "`code`|");
        assert_eq!(typed("", "``"), "``|");
        assert_eq!(typed("", "```"), "```|\n```");
        assert_eq!(typed("", "```rust"), "```rust|\n```");
        assert_eq!(typed("  ", "```"), "  ```|\n  ```");
        // Closing a block that is open: just backticks.
        assert_eq!(typed("```\ncode\n", "```"), "```\ncode\n```|");
        // No emphasis pairs inside code, brackets do.
        assert_eq!(typed("```\n", "*("), "```\n*(|)");
    }

    #[test]
    fn backspace_takes_an_empty_pair() {
        assert_eq!(typed("", "(\u{8}"), "|");
        assert_eq!(typed("", "**\u{8}"), "|");
        // Emptied again, it is still a pair.
        assert_eq!(typed("", "(x\u{8}\u{8}"), "|");
        // A pair typed by hand is left alone.
        assert_eq!(typed_at("()", 1, "\u{8}"), "|)");
    }

    #[test]
    fn selections_wrap() {
        let mut pairs = Pairs::default();
        let edit = pairs.typed("a word here", 2..6, '*').unwrap();
        assert_eq!(edit.apply("a word here"), "a *word* here");
        assert_eq!(edit.selection, 3..7);
        let text = edit.apply("a word here");
        let edit = pairs.typed(&text, 3..7, '*').unwrap();
        assert_eq!(edit.apply(&text), "a **word** here");
        let edit = pairs.typed("see this", 4..8, '[').unwrap();
        assert_eq!(edit.apply("see this"), "see [this]");
        let edit = pairs.typed("a\nb", 0..3, '`').unwrap();
        assert_eq!(edit.apply("a\nb"), "```\na\nb\n```");
        assert!(pairs.typed("a\nb", 0..3, '*').is_none());
        assert!(pairs.typed("ab", 0..2, 'x').is_none());
    }

    #[test]
    fn typing_outside_forgets_the_pair() {
        let (text, _, mut pairs) = type_keys("x", 1, "(");
        assert_eq!(text, "x()");
        assert_eq!(pairs.len(), 1);
        // The cursor went back before the opener: the closer is plain text.
        assert!(pairs.typed(&text, 0..0, ')').is_none());
        assert_eq!(pairs.len(), 0);
    }

    /// Plain text and Markdown, with the characters that pair.
    fn text() -> impl Strategy<Value = String> {
        "[a-z é\n*_`()\\[\\]\"~]{0,24}"
    }

    proptest! {
        #![proptest_config(config())]

        /// Invariant: an edit is inside the text, on character boundaries,
        /// with its selection inside the new text.
        #[test]
        fn prop_edits_are_valid(text in text(), a in 0usize..30, b in 0usize..30, ch in prop::sample::select(vec!['(', '[', '{', '"', '*', '_', '`', '~', ')', ' ', 'x'])) {
            let mut pairs = Pairs::default();
            let sel = clamp(&text, a..b);
            if let Some(edit) = pairs.typed(&text, sel, ch) {
                prop_assert!(edit.range.end <= text.len());
                prop_assert!(text.is_char_boundary(edit.range.start) && text.is_char_boundary(edit.range.end));
                let new = edit.apply(&text);
                prop_assert!(edit.selection.end <= new.len());
                prop_assert!(new.is_char_boundary(edit.selection.start) && new.is_char_boundary(edit.selection.end));
            }
        }

        /// Wrapping keeps the selected text, between an opener and its
        /// closer, and keeps it selected.
        #[test]
        fn prop_wrapping_keeps_the_selection(text in "[a-z ]{1,20}", a in 0usize..20, b in 0usize..20, ch in prop::sample::select(vec!['(', '[', '{', '"', '*', '_', '`'])) {
            let sel = clamp(&text, a..b);
            prop_assume!(!sel.is_empty());
            let mut pairs = Pairs::default();
            let edit = pairs.typed(&text, sel.clone(), ch).unwrap();
            let new = edit.apply(&text);
            prop_assert_eq!(&new[edit.selection.clone()], &text[sel.clone()]);
            let close = closer_for(ch).unwrap();
            prop_assert_eq!(&new[edit.selection.end..edit.selection.end + close.len()], close);
            prop_assert_eq!(new.chars().nth(text[..sel.start].chars().count()), Some(ch));
        }

        /// Metamorphic: typing an opener, some words, and the closer gives
        /// the same text as typing them with pairing off; the cursor ends
        /// after the closer.
        #[test]
        fn prop_typing_the_closer_steps_over_it(before in "[a-z]{0,6}( |\n)", inner in "[a-z]{0,6}", ch in prop::sample::select(vec!['(', '[', '{', '"'])) {
            let close = closer_for(ch).unwrap();
            let keys = format!("{ch}{inner}{close}");
            let (text, cursor, _) = type_keys(&before, before.len(), &keys);
            prop_assert_eq!(&text, &format!("{before}{keys}"));
            prop_assert_eq!(cursor, text.len());
        }

        /// Round trip: an opener then Backspace leaves the text as it was.
        #[test]
        fn prop_backspace_undoes_a_fresh_pair(before in "[a-z]{0,6}( |\n)?", after in "( [a-z]{0,6})?", ch in prop::sample::select(vec!['(', '[', '{', '"', '*', '_', '`'])) {
            let text = format!("{before}{after}");
            let (typed, cursor, _) = type_keys(&text, before.len(), &format!("{ch}\u{8}"));
            prop_assert_eq!(typed, text);
            prop_assert_eq!(cursor, before.len());
        }

        /// Typing between a pair keeps it: its closer is still stepped over.
        #[test]
        fn prop_pairs_survive_typing_inside(inner in "[a-z ]{0,8}", tail in "( [a-z]{0,6})?") {
            let text = format!("x {tail}");
            let (typed, cursor, _) = type_keys(&text, 2, &format!("({inner})"));
            prop_assert_eq!(&typed, &format!("x ({inner}){tail}"));
            prop_assert_eq!(cursor, 2 + inner.len() + 2);
        }
    }
}
