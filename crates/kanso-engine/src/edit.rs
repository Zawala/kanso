//! Edits to the Markdown source without a window: what the formatting
//! toolbar does ([`markdown`]), auto-pairing as you type ([`pairing`]), Tab
//! and Enter in lists ([`list`]) and tables ([`table`]). Each is pure: the
//! note's text and the selection in, at most one [`Edit`] out, which the
//! editor applies as one change, so one undo step reverts it.
//!
//! Offsets are UTF-8 byte offsets, as the desktop editor's selection is. An
//! editor that counts in UTF-16 units converts with [`crate::text`].
//!
//! [`tab`] and [`enter`] are the rules the desktop editor follows for those
//! keys, in one place; [`enter_continuing_lists`] adds continuing a list
//! item, which the desktop doesn't do (yet).

use std::ops::Range;

pub mod list;
pub mod markdown;
pub mod pairing;
pub mod table;

/// The selected bytes of a text, `start..end` with an empty range for a
/// caret.
pub type Selection = Range<usize>;

/// One replacement: `range` of the old text becomes `text`, and then
/// `selection` (in the new text) is selected.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Edit {
    pub range: Range<usize>,
    pub text: String,
    pub selection: Selection,
}

impl Edit {
    /// The text after the edit.
    pub fn apply(&self, text: &str) -> String {
        let mut out = String::with_capacity(text.len() + self.text.len());
        out.push_str(&text[..self.range.start]);
        out.push_str(&self.text);
        out.push_str(&text[self.range.end..]);
        out
    }
}

/// Tab (or Shift-Tab, with `out`): the next (previous) table cell in a
/// table, a list item in (out) a level with what hangs under it, and
/// otherwise `None`, which leaves Tab to the editor's own indent.
/// `format_tables` lines the table up as the cursor leaves it.
pub fn tab(text: &str, selection: Selection, out: bool, format_tables: bool) -> Option<Edit> {
    match list::tab_intent(text, selection.start) {
        list::TabIntent::Table => table::next_cell(text, selection.start, out, format_tables),
        list::TabIntent::List => list::indent_list(text, selection, out),
        list::TabIntent::Indent => None,
    }
}

/// Enter at the end of a table row adds a row; in an empty last row it
/// leaves the table. `None` for anything else, and for a selection, which
/// Enter replaces.
pub fn enter(text: &str, selection: Selection, format_tables: bool) -> Option<Edit> {
    if !selection.is_empty() {
        return None;
    }
    table::enter_in_table(text, selection.start, format_tables)
}

/// [`enter`], and where that does nothing, Enter on a list item: the next
/// item starts (see [`list::enter_in_list`]).
pub fn enter_continuing_lists(
    text: &str,
    selection: Selection,
    format_tables: bool,
) -> Option<Edit> {
    enter(text, selection.clone(), format_tables).or_else(|| list::enter_in_list(text, selection))
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    use super::markdown::{Format, format as format_with};
    use super::pairing::Pairs;
    use super::*;

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 512,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    // MARK: Generators

    /// Lines with markers of every kind, blank lines and multibyte text.
    fn markdown() -> impl Strategy<Value = String> {
        prop::collection::vec(
            "( {0,2})(#{0,3} |- |1\\. |- \\[[ x]\\] |> |```|~~~|)[a-zé日🙂*_`~\\[\\]() ]{0,10}",
            1..6,
        )
        .prop_map(|lines| lines.join("\n"))
    }

    /// Bullets, tasks and numbers, nested a level or two.
    fn list() -> impl Strategy<Value = String> {
        prop::collection::vec(
            (
                prop::sample::select(vec!["", "  ", "    ", "   "]),
                prop::sample::select(vec!["- ", "* ", "- [ ] ", "- [x] ", "1. ", "2. ", "10. "]),
                "[a-zé日]{1,5}",
            ),
            1..6,
        )
        .prop_map(|lines| {
            lines
                .into_iter()
                .map(|(indent, marker, text)| format!("{indent}{marker}{text}"))
                .collect::<Vec<_>>()
                .join("\n")
        })
    }

    /// A table of words and wide characters, with any alignment and
    /// optional outer pipes.
    fn table() -> impl Strategy<Value = String> {
        let cell = "[a-zé日 ]{1,5}";
        let row = move |n: usize| prop::collection::vec(cell, n);
        (1usize..4, any::<bool>()).prop_flat_map(move |(columns, outer)| {
            (
                row(columns),
                prop::collection::vec(row(columns), 0..3),
                prop::collection::vec(
                    prop::sample::select(vec!["---", ":--", "--:", ":-:"]),
                    columns,
                ),
            )
                .prop_map(move |(head, body, align)| {
                    let line = |cells: &[String]| {
                        let inner = cells.join("|");
                        if outer { format!("|{inner}|") } else { inner }
                    };
                    let mut lines = vec![
                        line(&head),
                        line(&align.iter().map(|a| (*a).to_owned()).collect::<Vec<_>>()),
                    ];
                    lines.extend(body.iter().map(|row| line(row)));
                    lines.join("\n")
                })
        })
    }

    /// A note: blocks of every kind, one after another.
    fn note() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop_oneof![markdown(), list(), table(), "[a-zé日🙂 ]{0,12}"],
            1..4,
        )
        .prop_map(|blocks| blocks.join("\n"))
    }

    /// A note and a selection in it: a caret or a range, both on character
    /// boundaries.
    fn with_selection() -> impl Strategy<Value = (String, Selection)> {
        (note(), 0f64..=1., 0f64..=1., any::<bool>()).prop_map(|(text, a, b, caret)| {
            let at = |f: f64| {
                let mut at = (text.len() as f64 * f) as usize;
                while !text.is_char_boundary(at) {
                    at -= 1;
                }
                at
            };
            let (a, b) = (at(a), at(b));
            let selection = if caret { a..a } else { a.min(b)..a.max(b) };
            (text, selection)
        })
    }

    // MARK: Checks

    /// The letters of `text` in order. A task's box (`[x]`) is a marker, not
    /// text, so its `x` isn't one.
    fn letters(text: &str) -> Vec<char> {
        text.chars()
            .filter(|c| c.is_alphabetic() && !matches!(c, 'x' | 'X'))
            .collect()
    }

    fn is_subsequence(small: &[char], big: &[char]) -> bool {
        let mut rest = big.iter();
        small.iter().all(|c| rest.any(|b| b == c))
    }

    /// E1 and E3 for any edit: its range lies in the old text, ordered, on
    /// character boundaries; its selection lies in the new text the same
    /// way; and what is outside the range is what the new text starts and
    /// ends with.
    fn check_valid(old: &str, edit: &Edit) -> Result<String, TestCaseError> {
        let (start, end) = (edit.range.start, edit.range.end);
        prop_assert!(
            start <= end && end <= old.len(),
            "{:?} in {old:?}",
            edit.range
        );
        prop_assert!(old.is_char_boundary(start) && old.is_char_boundary(end));
        let new = edit.apply(old);
        let (a, b) = (edit.selection.start, edit.selection.end);
        prop_assert!(a <= b && b <= new.len(), "{:?} in {new:?}", edit.selection);
        prop_assert!(new.is_char_boundary(a) && new.is_char_boundary(b));
        prop_assert_eq!(&new[..start], &old[..start]);
        prop_assert_eq!(&new[new.len() - (old.len() - end)..], &old[end..]);
        prop_assert_eq!(new.len(), old.len() - (end - start) + edit.text.len());
        Ok(new)
    }

    const FORMATS: [Format; 18] = [
        Format::Heading(1),
        Format::Heading(2),
        Format::Heading(3),
        Format::Heading(4),
        Format::Heading(5),
        Format::Heading(6),
        Format::Bulleted,
        Format::Numbered,
        Format::Task,
        Format::Quote,
        Format::Code,
        Format::Bold,
        Format::Italic,
        Format::Strikethrough,
        Format::Link,
        Format::Image,
        Format::Table,
        Format::Rule,
    ];

    /// How many letters a template adds: a link's `https`, a table's
    /// column names.
    fn template_letters(format: Format) -> &'static [usize] {
        match format {
            Format::Link | Format::Image => &[0, 5],
            Format::Table => &[12],
            Format::Rule => &[0],
            _ => &[0],
        }
    }

    proptest! {
        #![proptest_config(config())]

        /// E1, E2 and E3 for every format: a valid edit that never loses
        /// text. Wraps and line formats keep the letters exactly; links,
        /// images, tables and rules add only their template's.
        #[test]
        fn prop_formats_are_valid_and_lose_no_text(
            (text, selection) in with_selection(),
            which in 0..FORMATS.len(),
        ) {
            let format = FORMATS[which];
            let edit = format_with(format, &text, selection);
            let new = check_valid(&text, &edit)?;
            let (old, now) = (letters(&text), letters(&new));
            match format {
                Format::Link | Format::Image | Format::Table | Format::Rule => {
                    prop_assert!(is_subsequence(&old, &now));
                    prop_assert!(
                        template_letters(format).contains(&(now.len() - old.len())),
                        "{format:?}: {} → {}", old.len(), now.len()
                    );
                }
                _ => prop_assert_eq!(now, old),
            }
        }

        /// E1, E2 and E3 for Tab and Shift-Tab on list items, with the
        /// words and markers kept (only indents and numbers move).
        #[test]
        fn prop_list_indents_are_valid_and_lose_no_text(
            (text, selection) in with_selection(),
            out: bool,
        ) {
            if let Some(edit) = list::indent_list(&text, selection, out) {
                let new = check_valid(&text, &edit)?;
                prop_assert_eq!(letters(&new), letters(&text));
            }
        }

        /// E1, E2 and E3 for moving through a table's cells, forward and
        /// back, formatting or not; and for Enter in a table, and for
        /// formatting it.
        #[test]
        fn prop_table_edits_are_valid_and_lose_no_text(
            (text, selection) in with_selection(),
            back: bool,
            format: bool,
        ) {
            let at = selection.start;
            for edit in [
                table::next_cell(&text, at, back, format),
                table::enter_in_table(&text, at, format),
                table::format_table(&text, at),
            ].into_iter().flatten() {
                let new = check_valid(&text, &edit)?;
                prop_assert_eq!(letters(&new), letters(&text));
            }
        }

        /// E1, E2 and E3 for Enter on a list item: no text lost, and the
        /// new line holds only a marker.
        #[test]
        fn prop_enter_in_a_list_is_valid_and_loses_no_text((text, selection) in with_selection()) {
            if let Some(edit) = list::enter_in_list(&text, selection) {
                let new = check_valid(&text, &edit)?;
                prop_assert_eq!(letters(&new), letters(&text));
            }
        }

        /// E1, E2 and E3 for typing: any of the characters that pair, one
        /// after another, each applied as its edit says. Typing adds only
        /// the characters typed and their closers, so no letter is lost;
        /// Backspace after it takes only markers away.
        #[test]
        fn prop_typing_and_backspace_are_valid_and_lose_no_text(
            (text, selection) in with_selection(),
            typed in prop::collection::vec(
                prop::sample::select(vec!['(', '[', '{', '"', '*', '_', '`', '~', ')', ']', '}', ' ']),
                1..5,
            ),
            backspaces in 0usize..3,
        ) {
            let mut pairs = Pairs::default();
            let mut text = text;
            let mut selection = selection;
            // The letters the text should have: typing as usual replaces
            // the selection, and its letters with it.
            let mut want = letters(&text);
            for ch in typed {
                if let Some(edit) = pairs.typed(&text, selection.clone(), ch) {
                    text = check_valid(&text, &edit)?;
                    selection = edit.selection;
                } else {
                    text.replace_range(selection.clone(), &ch.to_string());
                    let caret = selection.start + ch.len_utf8();
                    selection = caret..caret;
                    want = letters(&text);
                }
                prop_assert_eq!(letters(&text), want.clone());
            }
            for _ in 0..backspaces {
                if selection.is_empty()
                    && let Some(edit) = pairs.backspace(&text, selection.start)
                {
                    text = check_valid(&text, &edit)?;
                    selection = edit.selection;
                    prop_assert_eq!(letters(&text), want.clone());
                }
            }
        }

        /// E4: Tab does the table's cell move on a table row, the list's
        /// indent on a list item and nothing anywhere else, as the editor
        /// did before the rule moved here.
        #[test]
        fn prop_tab_follows_the_rule((text, selection) in with_selection(), out: bool, format: bool) {
            let got = tab(&text, selection.clone(), out, format);
            let at = selection.start;
            let line = &text[markdown::line_start(&text, at)..markdown::line_end(&text, at)];
            let item = markdown::list_marker(&line[markdown::indent_len(line)..]).is_some();
            if pairing::in_fenced_code(&text, at) || (!item && !line.contains('|')) {
                prop_assert_eq!(got, None);
            } else if let Some(edit) = got {
                let cell = table::next_cell(&text, at, out, format);
                let indent = list::indent_list(&text, selection, out);
                prop_assert!(Some(&edit) == cell.as_ref() || Some(&edit) == indent.as_ref());
            }
        }

        /// E4: Enter is the table's Enter for a caret and nothing for a
        /// selection; with lists continuing, a list's Enter where the table
        /// has none.
        #[test]
        fn prop_enter_follows_the_rule((text, selection) in with_selection(), format: bool) {
            let got = enter(&text, selection.clone(), format);
            if selection.is_empty() {
                prop_assert_eq!(got.clone(), table::enter_in_table(&text, selection.start, format));
            } else {
                prop_assert_eq!(got.clone(), None);
            }
            let continuing = enter_continuing_lists(&text, selection.clone(), format);
            if got.is_some() {
                prop_assert_eq!(continuing, got);
            } else {
                prop_assert_eq!(continuing, list::enter_in_list(&text, selection));
            }
        }

        /// E5: a formatted table formats to itself, and the table a move
        /// between cells leaves (formatting on) is formatted already.
        #[test]
        fn prop_formatting_is_idempotent((text, selection) in with_selection(), back: bool) {
            let at = selection.start;
            if let Some(edit) = table::format_table(&text, at) {
                let once = edit.apply(&text);
                let again = table::format_table(&once, edit.selection.start);
                prop_assert!(again.is_none_or(|e| e.apply(&once) == once));
            }
            if let Some(edit) = table::next_cell(&text, at, back, true) {
                let moved = edit.apply(&text);
                let again = table::format_table(&moved, edit.selection.start);
                prop_assert!(again.is_none_or(|e| e.apply(&moved) == moved));
            }
        }
    }
}
