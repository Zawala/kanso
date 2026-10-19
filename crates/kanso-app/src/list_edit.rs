//! Tab and Shift-Tab: what they do where the cursor is ([`tab_intent`]),
//! and moving list items in and out a level, their sub-items with them,
//! with numbered lists renumbered. Pure: text in, one edit out. Offsets are
//! UTF-8 byte offsets.

use std::collections::HashSet;
use std::ops::Range;

use crate::markdown_edit::{Edit, ListKind, indent_len, line_end, line_start, list_marker};
use crate::pairing::in_fenced_code;
use crate::table_format::in_table;

/// What Tab does where the cursor is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabIntent {
    /// In a GitHub table: on to the next cell.
    Table,
    /// On a list item: the item goes in a level.
    List,
    /// Anywhere else: the editor's own indent.
    Indent,
}

/// What Tab does with the cursor at `cursor` in `text`.
pub fn tab_intent(text: &str, cursor: usize) -> TabIntent {
    let cursor = floor(text, cursor);
    let start = line_start(text, cursor);
    let line = &text[start..line_end(text, start)];
    let item = list_marker(&line[indent_len(line)..]).is_some();
    let piped = line.contains('|');
    if !item && !piped || in_fenced_code(text, cursor) {
        return TabIntent::Indent;
    }
    if piped && in_table(text, cursor) {
        TabIntent::Table
    } else if item {
        TabIntent::List
    } else {
        TabIntent::Indent
    }
}

fn floor(text: &str, at: usize) -> usize {
    let mut at = at.min(text.len());
    while !text.is_char_boundary(at) {
        at -= 1;
    }
    at
}

/// A list item line: its indent, kind, marker length (with the space after
/// it) and, when numbered, its number and where the digits are.
#[derive(Clone, Copy, Debug)]
struct Item {
    indent: usize,
    kind: ListKind,
    marker: usize,
    number: Option<u64>,
}

fn item(line: &str) -> Option<Item> {
    let indent = indent_len(line);
    let (kind, marker) = list_marker(&line[indent..])?;
    let number = (kind == ListKind::Numbered).then(|| {
        let digits: String = line[indent..]
            .chars()
            .take_while(char::is_ascii_digit)
            .collect();
        digits.parse().unwrap_or(1)
    });
    Some(Item {
        indent,
        kind,
        marker,
        number,
    })
}

/// The lines of the list around `line` (an index into `lines`): items,
/// the indented lines and blank lines between them. Blank lines at its ends
/// aren't part of it.
fn list_block(lines: &[&str], line: usize) -> Range<usize> {
    let part = |l: &str| l.trim().is_empty() || item(l).is_some() || indent_len(l) > 0;
    let mut start = line;
    while start > 0 && part(lines[start - 1]) {
        start -= 1;
    }
    let mut end = line + 1;
    while end < lines.len() && part(lines[end]) {
        end += 1;
    }
    while start < line && lines[start].trim().is_empty() {
        start += 1;
    }
    while end > line + 1 && lines[end - 1].trim().is_empty() {
        end -= 1;
    }
    start..end
}

/// The runs of `lines` that count together: for each item, the first item
/// of its run (the same parent, indent and kind of list).
fn runs(lines: &[String]) -> Vec<Option<usize>> {
    // (indent, numbered, first item of the run)
    let mut stack: Vec<(usize, bool, usize)> = Vec::new();
    lines
        .iter()
        .enumerate()
        .map(|(ix, line)| {
            let it = item(line)?;
            let numbered = it.kind == ListKind::Numbered;
            while stack.last().is_some_and(|top| top.0 > it.indent) {
                stack.pop();
            }
            match stack.last() {
                Some(top) if top.0 == it.indent && top.1 == numbered => Some(top.2),
                _ => {
                    if stack.last().is_some_and(|top| top.0 == it.indent) {
                        stack.pop();
                    }
                    stack.push((it.indent, numbered, ix));
                    Some(ix)
                }
            }
        })
        .collect()
}

/// Numbers the numbered items of the runs in `which` from their first
/// item's number, or from 1 when that first item just moved there.
fn renumber(lines: &mut [String], which: &HashSet<usize>, moved: &HashSet<usize>) {
    let runs = runs(lines);
    let mut next: Vec<(usize, u64)> = Vec::new();
    for ix in 0..lines.len() {
        let Some(run) = runs[ix] else { continue };
        if !which.contains(&run) {
            continue;
        }
        let Some(it) = item(&lines[ix]) else { continue };
        let Some(number) = it.number else { continue };
        let n = match next.iter_mut().find(|(r, _)| *r == run) {
            Some((_, n)) => {
                *n += 1;
                *n
            }
            None => {
                let start = if moved.contains(&ix) { 1 } else { number };
                next.push((run, start));
                start
            }
        };
        let line = &lines[ix];
        let digits = line[it.indent..]
            .bytes()
            .take_while(u8::is_ascii_digit)
            .count();
        lines[ix] = format!("{}{n}{}", &line[..it.indent], &line[it.indent + digits..]);
    }
}

/// Tab (or Shift-Tab, `out`) on list items: each selected item goes in a
/// level (under the item above it) or out a level (to its parent's), with
/// the lines under it; numbered lists the move touches are renumbered.
/// `None` when nothing moves.
pub fn indent_list(text: &str, selection: Range<usize>, out: bool) -> Option<Edit> {
    let selection = floor(text, selection.start)..floor(text, selection.end);
    let selection = selection.start.min(selection.end)..selection.start.max(selection.end);
    // Every line, with where it starts.
    let mut starts = vec![0];
    starts.extend(text.match_indices('\n').map(|(ix, _)| ix + 1));
    let all: Vec<&str> = text.split('\n').collect();
    let line_of = |at: usize| starts.partition_point(|&s| s <= at) - 1;
    let first = line_of(selection.start);
    let last = if selection.end > selection.start && text[..selection.end].ends_with('\n') {
        line_of(selection.end - 1)
    } else {
        line_of(selection.end)
    };
    let block = list_block(&all, first);
    let block = block.start..block.end.max(last + 1).min(all.len());
    let old: Vec<String> = all[block.clone()].iter().map(|s| (*s).to_owned()).collect();
    let mut lines = old.clone();
    let rel = |line: usize| line - block.start;

    // The selected items, each with the lines under it.
    let mut roots: Vec<usize> = Vec::new();
    let mut covered: HashSet<usize> = HashSet::new();
    for line in first..=last {
        let ix = rel(line);
        if covered.contains(&ix) || item(&lines[ix]).is_none() {
            continue;
        }
        roots.push(ix);
        covered.insert(ix);
        covered.extend(subtree(&lines, ix));
    }
    if roots.is_empty() {
        return None;
    }
    let mut deltas = vec![0isize; lines.len()];
    for &root in &roots {
        let it = item(&lines[root]).expect("a root is an item");
        let target = if out {
            match parent(&lines, root) {
                Some(p) => item(&lines[p]).map_or(0, |p| p.indent),
                None => it.indent.saturating_sub(2),
            }
        } else {
            // Under the item above: where its text starts, after "- " or
            // "1. " (a task's box is part of its text).
            let nest = |it: Item| match it.kind {
                ListKind::Numbered => it.marker,
                ListKind::Bulleted | ListKind::Task => 2,
            };
            match sibling_above(&lines, root) {
                Some(s) => {
                    let s = item(&lines[s]).expect("a sibling is an item");
                    s.indent + nest(s)
                }
                None => it.indent + nest(it),
            }
        };
        let delta = target as isize - it.indent as isize;
        for ix in std::iter::once(root).chain(subtree(&lines, root)) {
            deltas[ix] = delta;
        }
    }
    if deltas.iter().all(|d| *d == 0) {
        return None;
    }
    for (ix, delta) in deltas.iter().enumerate() {
        let line = &lines[ix];
        lines[ix] = match delta.cmp(&0) {
            std::cmp::Ordering::Equal => continue,
            std::cmp::Ordering::Greater => format!("{}{line}", " ".repeat(*delta as usize)),
            std::cmp::Ordering::Less => {
                let take = (-delta as usize).min(indent_len(line));
                line[take..].to_owned()
            }
        };
    }
    // Renumber the runs the moved items are in now, and the runs they left.
    let moved: HashSet<usize> = roots.iter().copied().collect();
    let before = runs(&old);
    let after = runs(&lines);
    let left: HashSet<usize> = moved.iter().filter_map(|&m| before[m]).collect();
    let mut which: HashSet<usize> = moved.iter().filter_map(|&m| after[m]).collect();
    for ix in 0..lines.len() {
        if !moved.contains(&ix)
            && before[ix].is_some_and(|run| left.contains(&run))
            && let Some(run) = after[ix]
        {
            which.insert(run);
        }
    }
    renumber(&mut lines, &which, &moved);

    let range = starts[block.start]..starts[block.start] + old.join("\n").len();
    let new_text = lines.join("\n");
    // Each selection end keeps its place in its line's text.
    let map = |at: usize| {
        let ix = rel(line_of(at));
        let new_start: usize = lines[..ix].iter().map(|l| l.len() + 1).sum();
        let col = at - starts[block.start + ix];
        let (old_line, new_line) = (&old[ix], &lines[ix]);
        let shift = new_line.len() as isize - old_line.len() as isize;
        let old_marker_end = item(old_line).map_or(indent_len(old_line), |i| i.indent + i.marker);
        let new_col = if col >= old_marker_end {
            col.saturating_add_signed(shift)
        } else {
            let new_marker_end =
                item(new_line).map_or(indent_len(new_line), |i| i.indent + i.marker);
            (col.saturating_add_signed(deltas[ix])).min(new_marker_end)
        };
        range.start + new_start + new_col.min(new_line.len())
    };
    let selection = map(selection.start)..map(selection.end);
    Some(Edit {
        range,
        text: new_text,
        selection,
    })
}

/// The lines under the item at `ix`: those after it indented more deeply,
/// with blank lines between them.
fn subtree(lines: &[String], ix: usize) -> Vec<usize> {
    let indent = indent_len(&lines[ix]);
    let mut out = Vec::new();
    let mut pending = Vec::new();
    for (j, line) in lines.iter().enumerate().skip(ix + 1) {
        if line.trim().is_empty() {
            pending.push(j);
            continue;
        }
        if indent_len(line) <= indent {
            break;
        }
        out.append(&mut pending);
        out.push(j);
    }
    out
}

/// The item above `ix` at its level, before any item less indented.
fn sibling_above(lines: &[String], ix: usize) -> Option<usize> {
    let indent = indent_len(&lines[ix]);
    for j in (0..ix).rev() {
        let Some(it) = item(&lines[j]) else { continue };
        if it.indent == indent {
            return Some(j);
        }
        if it.indent < indent {
            return None;
        }
    }
    None
}

/// The item `ix` is under: the nearest one above it less indented.
fn parent(lines: &[String], ix: usize) -> Option<usize> {
    let indent = indent_len(&lines[ix]);
    (0..ix)
        .rev()
        .find(|&j| item(&lines[j]).is_some_and(|it| it.indent < indent))
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

    /// The text after Tab (or Shift-Tab) with the cursor at `|` in `text`.
    fn tab(text: &str, out: bool) -> String {
        let at = text.find('|').unwrap();
        let text = text.replace('|', "");
        match indent_list(&text, at..at, out) {
            Some(edit) => {
                let new = edit.apply(&text);
                format!(
                    "{}|{}",
                    &new[..edit.selection.start],
                    &new[edit.selection.start..]
                )
            }
            None => format!("{}|{}", &text[..at], &text[at..]),
        }
    }

    #[test]
    fn intents() {
        assert_eq!(tab_intent("- a", 3), TabIntent::List);
        assert_eq!(tab_intent("1. a", 0), TabIntent::List);
        assert_eq!(tab_intent("- [ ] a", 5), TabIntent::List);
        assert_eq!(tab_intent("plain", 2), TabIntent::Indent);
        assert_eq!(tab_intent("| a | b |\n|---|---|", 3), TabIntent::Table);
        assert_eq!(tab_intent("a | b", 3), TabIntent::Indent);
        assert_eq!(tab_intent("```\n- a\n```", 6), TabIntent::Indent);
    }

    #[test]
    fn bullets_go_in_and_out() {
        assert_eq!(tab("- a\n- b|", false), "- a\n  - b|");
        assert_eq!(tab("- a\n  - b|", true), "- a\n- b|");
        // The first item has nothing to go under, but goes in anyway.
        assert_eq!(tab("- |a", false), "  - |a");
        assert_eq!(tab("- |a", true), "- |a");
        // Sub-items come along.
        assert_eq!(
            tab("- a\n- |b\n  - c\n- d", false),
            "- a\n  - |b\n    - c\n- d"
        );
        assert_eq!(tab("- [ ] a\n- [x] |b", false), "- [ ] a\n  - [x] |b");
    }

    #[test]
    fn numbered_lists_renumber() {
        assert_eq!(tab("1. a\n2. |b\n3. c", false), "1. a\n   1. |b\n2. c");
        assert_eq!(tab("1. a\n   1. |b\n2. c", true), "1. a\n2. |b\n3. c");
        // A list that starts at 5 keeps its start.
        assert_eq!(tab("5. a\n6. b\n7. |c", false), "5. a\n6. b\n   1. |c");
        // Lazy numbering elsewhere is left alone.
        assert_eq!(
            tab("1. a\n1. b\n\n- x\n- |y", false),
            "1. a\n1. b\n\n- x\n  - |y"
        );
    }

    #[test]
    fn selections_move_every_item() {
        let text = "- a\n- b\n- c";
        let edit = indent_list(text, 4..text.len(), false).unwrap();
        assert_eq!(edit.apply(text), "- a\n  - b\n  - c");
    }

    /// Lists of bullets, tasks and numbers, nested a level or two.
    fn list() -> impl Strategy<Value = String> {
        prop::collection::vec(
            (
                prop::sample::select(vec!["", "  ", "    ", "   "]),
                prop::sample::select(vec!["- ", "* ", "- [ ] ", "1. ", "2. ", "10. "]),
                "[a-z]{1,5}",
            ),
            1..8,
        )
        .prop_map(|lines| {
            lines
                .into_iter()
                .map(|(indent, marker, text)| format!("{indent}{marker}{text}"))
                .collect::<Vec<_>>()
                .join("\n")
        })
    }

    /// The words of each line, without markers or indent.
    fn words(text: &str) -> Vec<String> {
        text.lines()
            .map(|line| {
                let it = item(line).unwrap();
                line[it.indent + it.marker..].to_owned()
            })
            .collect()
    }

    proptest! {
        #![proptest_config(config())]

        /// Tab and Shift-Tab change only indents and numbers: every line
        /// keeps its words, its kind of marker and its place.
        #[test]
        fn prop_only_indents_and_numbers_change(text in list(), at in 0usize..80, out: bool) {
            let at = floor(&text, at);
            if let Some(edit) = indent_list(&text, at..at, out) {
                let new = edit.apply(&text);
                prop_assert_eq!(words(&new), words(&text));
                let kinds = |t: &str| t.lines().map(|l| item(l).unwrap().kind).collect::<Vec<_>>();
                prop_assert_eq!(kinds(&new), kinds(&text));
                prop_assert!(edit.selection.end <= new.len());
            }
        }

        /// Tab moves the cursor's item in, Shift-Tab out; never the other
        /// way.
        #[test]
        fn prop_tab_goes_in_shift_tab_out(text in list(), at in 0usize..80) {
            let at = floor(&text, at);
            let line = text[..at].matches('\n').count();
            let indent = |t: &str| indent_len(t.lines().nth(line).unwrap());
            if let Some(edit) = indent_list(&text, at..at, false) {
                prop_assert!(indent(&edit.apply(&text)) > indent(&text));
            }
            if let Some(edit) = indent_list(&text, at..at, true) {
                prop_assert!(indent(&edit.apply(&text)) < indent(&text));
            }
        }

        /// Round trip: in a tidy list (one level of bullets), Tab then
        /// Shift-Tab gives the text back.
        #[test]
        fn prop_tab_then_shift_tab_round_trips(items in prop::collection::vec("[a-z]{1,5}", 2..6), pick in 1usize..6) {
            let text = items.iter().map(|w| format!("- {w}")).collect::<Vec<_>>().join("\n");
            let pick = pick.min(items.len() - 1);
            let at = text.lines().take(pick).map(|l| l.len() + 1).sum::<usize>() + 2;
            let tabbed = indent_list(&text, at..at, false).unwrap();
            let once = tabbed.apply(&text);
            let back = indent_list(&once, tabbed.selection.clone(), true).unwrap();
            prop_assert_eq!(back.apply(&once), text);
            prop_assert_eq!(back.selection, at..at);
        }

        /// After Tab or Shift-Tab, a numbered run the move touched counts
        /// up by one.
        #[test]
        fn prop_numbered_runs_count_up(n in 2usize..7, pick in 1usize..7, out: bool) {
            let pick = pick.min(n - 1);
            let text = (1..=n).map(|i| format!("{i}. w{i}")).collect::<Vec<_>>().join("\n");
            let at = text.lines().take(pick).map(|l| l.len() + 1).sum::<usize>();
            if let Some(edit) = indent_list(&text, at..at, out) {
                let new = edit.apply(&text);
                let top: Vec<u64> = new.lines().filter_map(item).filter(|i| i.indent == 0).filter_map(|i| i.number).collect();
                prop_assert_eq!(top, (1..=(n as u64 - 1)).collect::<Vec<_>>());
            }
        }
    }
}
