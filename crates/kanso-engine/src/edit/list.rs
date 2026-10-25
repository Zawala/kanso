//! Tab and Shift-Tab: what they do where the cursor is ([`tab_intent`]),
//! and moving list items in and out a level, their sub-items with them,
//! with numbered lists renumbered. Pure: text in, one edit out. Offsets are
//! UTF-8 byte offsets.

use std::collections::HashSet;
use std::ops::Range;

use super::Edit;
use super::markdown::{ListKind, indent_len, line_end, line_start, list_marker};
use super::pairing::in_fenced_code;
use super::table::in_table;

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
        // A selection that ends at the start of the line after the block
        // (a drag down whole lines) keeps that place, past the change.
        if line_of(at) >= block.end {
            return at + new_text.len() - (range.end - range.start);
        }
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

/// The largest number a numbered item can have and still be an item
/// ([`list_marker`] reads at most nine digits).
const MAX_NUMBER: u64 = 999_999_999;

/// Enter on a list item: the text after the caret starts the next item,
/// with the same kind of marker (a task's box unchecked, a number one more
/// with the numbers after it counting on), at the same indent. Enter on an
/// empty item ends the list instead: its marker goes, and the line is left
/// empty.
///
/// `None` when the selection isn't a caret, the caret is before the
/// marker, on no item, or in a fenced block, or the number can't grow:
/// Enter then types its newline as usual.
pub fn enter_in_list(text: &str, selection: Range<usize>) -> Option<Edit> {
    let cursor = floor(text, selection.start);
    if cursor != floor(text, selection.end) || in_fenced_code(text, cursor) {
        return None;
    }
    let start = line_start(text, cursor);
    let end = line_end(text, cursor);
    let line = &text[start..end];
    let it = item(line)?;
    if cursor < start + it.indent + it.marker {
        return None;
    }
    if line[it.indent + it.marker..].trim().is_empty() {
        return Some(Edit {
            range: start..end,
            text: String::new(),
            selection: start..start,
        });
    }
    let rest = &line[it.indent..];
    let next_marker = match it.number {
        Some(number) => {
            let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
            if number >= MAX_NUMBER {
                return None;
            }
            format!("{}{} ", number + 1, &rest[digits..=digits])
        }
        None if it.kind == ListKind::Task => format!("{} [ ] ", &rest[..1]),
        None => format!("{} ", &rest[..1]),
    };
    let prefix = format!("\n{}{next_marker}", &line[..it.indent]);
    let selection = cursor + prefix.len();

    // The numbers after the new item count on, as far as its run goes.
    let mut renumbered: Vec<(Range<usize>, String)> = Vec::new();
    if let Some(number) = it.number {
        let mut next = number + 2;
        let mut at = end;
        while at < text.len() {
            let from = at + 1;
            let to = line_end(text, from);
            at = to;
            let other = &text[from..to];
            if other.trim().is_empty() || in_fenced_code(text, from) {
                continue;
            }
            match item(other) {
                Some(o) if o.indent == it.indent && o.kind == ListKind::Numbered => {
                    if next > MAX_NUMBER {
                        return None;
                    }
                    let digits = other[o.indent..]
                        .bytes()
                        .take_while(u8::is_ascii_digit)
                        .count();
                    let digits = from + o.indent..from + o.indent + digits;
                    renumbered.push((digits, next.to_string()));
                    next += 1;
                }
                Some(o) if o.indent > it.indent => {}
                None if indent_len(other) > it.indent => {}
                _ => break,
            }
        }
    }
    let Some(last) = renumbered
        .last()
        .map(|(digits, _)| line_end(text, digits.start))
    else {
        return Some(Edit {
            range: cursor..cursor,
            text: prefix,
            selection: selection..selection,
        });
    };
    // The rest of the caret's line, then the lines down to the last number
    // that changed, each with its new number.
    let mut new = prefix;
    new.push_str(&text[cursor..end]);
    let mut copied = end;
    for (digits, number) in &renumbered {
        new.push_str(&text[copied..digits.start]);
        new.push_str(number);
        copied = digits.end;
    }
    new.push_str(&text[copied..last]);
    Some(Edit {
        range: cursor..last,
        text: new,
        selection: selection..selection,
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

    /// The text after Enter with the cursor at `|` in `text`, with the
    /// cursor where the edit leaves it; unchanged when Enter does nothing.
    fn enter(text: &str) -> String {
        let at = text.find('|').unwrap();
        let text = text.replace('|', "");
        match enter_in_list(&text, at..at) {
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
    fn enter_continues_the_item() {
        assert_eq!(enter("- a|"), "- a\n- |");
        assert_eq!(enter("* a|"), "* a\n* |");
        assert_eq!(enter("+ a|\n+ b"), "+ a\n+ |\n+ b");
        // Nested items stay at their level.
        assert_eq!(enter("- a\n  - b|"), "- a\n  - b\n  - |");
        // The text after the caret moves to the new item.
        assert_eq!(enter("- ab|cd"), "- ab\n- |cd");
        assert_eq!(enter("- a|\n\nafter"), "- a\n- |\n\nafter");
        // Multibyte text.
        assert_eq!(enter("- 日本|語"), "- 日本\n- |語");
    }

    #[test]
    fn enter_on_a_task_starts_an_unchecked_one() {
        assert_eq!(enter("- [ ] a|"), "- [ ] a\n- [ ] |");
        assert_eq!(enter("- [x] done|"), "- [x] done\n- [ ] |");
        assert_eq!(enter("  * [X] done|"), "  * [X] done\n  * [ ] |");
    }

    #[test]
    fn enter_on_a_numbered_item_counts_on() {
        assert_eq!(enter("1. a|"), "1. a\n2. |");
        assert_eq!(enter("1. a|\n2. b\n3. c"), "1. a\n2. |\n3. b\n4. c");
        assert_eq!(enter("1. a\n2. b|\n3. c"), "1. a\n2. b\n3. |\n4. c");
        assert_eq!(enter("9) a|\n10) b"), "9) a\n10) |\n11) b");
        // Only the run the item is in is renumbered.
        assert_eq!(
            enter("1. a|\n   1. x\n2. b\n\n- z\n3. q"),
            "1. a\n2. |\n   1. x\n3. b\n\n- z\n3. q"
        );
        // A list that starts at 5 keeps its start.
        assert_eq!(enter("5. a|\n6. b"), "5. a\n6. |\n7. b");
        assert_eq!(
            enter("1. a|\n```\n2. in code\n```"),
            "1. a\n2. |\n```\n2. in code\n```"
        );
    }

    #[test]
    fn enter_on_an_empty_item_ends_the_list() {
        assert_eq!(enter("- a\n- |"), "- a\n|");
        assert_eq!(enter("- a\n  - |"), "- a\n|");
        assert_eq!(enter("1. a\n2. |"), "1. a\n|");
        assert_eq!(enter("- [ ] a\n- [ ] |"), "- [ ] a\n|");
        assert_eq!(enter("- [ ]|"), "|");
        assert_eq!(enter("- |\ntext"), "|\ntext");
        // Spaces only.
        assert_eq!(enter("- a\n-   |"), "- a\n|");
    }

    #[test]
    fn enter_elsewhere_is_enter() {
        // Not a list, a quote, a table row, or code.
        assert_eq!(enter("plain|"), "plain|");
        assert_eq!(enter("> - a|"), "> - a|");
        assert!(enter_in_list("| a | b |\n|---|---|\n| c | d |", 31..31).is_none());
        assert_eq!(enter("```\n- a|\n```"), "```\n- a|\n```");
        // Before the marker, or inside it.
        assert_eq!(enter("|- a"), "|- a");
        assert_eq!(enter("-| a"), "-| a");
        assert_eq!(enter("- [|x] a"), "- [|x] a");
        // A selection is replaced by Enter, not continued.
        assert!(enter_in_list("- abc", 3..5).is_none());
        // A number that can't grow.
        assert_eq!(enter("999999999. a|"), "999999999. a|");
    }

    /// Dragging down whole list lines ends the selection at the start of
    /// the line after them, which isn't part of the list.
    #[test]
    fn a_selection_ending_below_the_list_moves_its_items() {
        let text = "- a\n- b\nafter";
        let edit = indent_list(text, 0..8, false).unwrap();
        let new = edit.apply(text);
        assert_eq!(new, "  - a\n  - b\nafter");
        // The selection still ends at the start of "after".
        assert_eq!(edit.selection, 2..new.len() - "after".len());
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

        /// Enter on an item with text adds exactly one line: an item of the
        /// same kind and indent, with a task's box unchecked, holding what
        /// was after the caret, with the caret at its start; every other
        /// line is kept.
        #[test]
        fn prop_enter_adds_one_item_of_the_same_kind(
            text in list(),
            line in 0usize..8,
            cut in 0usize..6,
        ) {
            let lines: Vec<&str> = text.split('\n').collect();
            let line = line % lines.len();
            let it = item(lines[line]).unwrap();
            let start: usize = lines[..line].iter().map(|l| l.len() + 1).sum();
            let at = start + it.indent + it.marker + cut.min(lines[line].len() - it.indent - it.marker);
            let at = floor(&text, at);
            let edit = enter_in_list(&text, at..at).unwrap();
            let new = edit.apply(&text);
            let new_lines: Vec<&str> = new.split('\n').collect();
            prop_assert_eq!(new_lines.len(), lines.len() + 1);
            // The caret's line is cut at the caret, the next line is the new item.
            prop_assert_eq!(new_lines[line], &text[start..at]);
            let made = item(new_lines[line + 1]).unwrap();
            prop_assert_eq!(made.indent, it.indent);
            prop_assert_eq!(made.kind, it.kind);
            if it.kind == ListKind::Task {
                let bullet = &lines[line][it.indent..=it.indent];
                let box_ = format!("{bullet} [ ] ");
                prop_assert!(new_lines[line + 1][made.indent..].starts_with(&box_));
            }
            if let Some(number) = it.number {
                prop_assert_eq!(made.number, Some(number + 1));
            }
            let rest = &text[at..start + lines[line].len()];
            prop_assert_eq!(&new_lines[line + 1][made.indent + made.marker..], rest);
            // The caret is at the start of that text.
            let made_at: usize = new_lines[..=line].iter().map(|l| l.len() + 1).sum();
            let caret = made_at + made.indent + made.marker;
            prop_assert_eq!(edit.selection.clone(), caret..caret);
            // Lines below keep their words, kinds and indents; only numbers move.
            for (old, new) in lines[line + 1..].iter().zip(&new_lines[line + 2..]) {
                let (o, n) = (item(old).unwrap(), item(new).unwrap());
                prop_assert_eq!((o.indent, o.kind), (n.indent, n.kind));
                prop_assert_eq!(&old[o.indent + o.marker..], &new[n.indent + n.marker..]);
                if o.number.is_none() {
                    prop_assert_eq!(old, new);
                }
            }
        }

        /// Enter on the item it just made ends the list: that line is left
        /// empty and nothing else changes.
        #[test]
        fn prop_enter_twice_ends_the_list(text in list(), line in 0usize..8) {
            let lines: Vec<&str> = text.split('\n').collect();
            let line = line % lines.len();
            let end: usize = lines[..=line].iter().map(|l| l.len() + 1).sum::<usize>() - 1;
            let first = enter_in_list(&text, end..end).unwrap();
            let once = first.apply(&text);
            let caret = first.selection.start;
            let second = enter_in_list(&once, caret..caret).unwrap();
            let mut expected: Vec<&str> = once.split('\n').collect();
            expected[line + 1] = "";
            prop_assert_eq!(second.apply(&once), expected.join("\n"));
            prop_assert_eq!(second.selection.clone(), second.range.start..second.range.start);
        }

        /// In a numbered list that counts 1, 2, 3, …, Enter on any item
        /// leaves it counting by one.
        #[test]
        fn prop_enter_keeps_a_numbered_list_counting(n in 1usize..9, pick in 0usize..9, sub in any::<bool>()) {
            let pick = pick % n;
            let mut lines: Vec<String> = Vec::new();
            for i in 1..=n {
                lines.push(format!("{i}. w{i}"));
                if sub && i % 2 == 0 {
                    lines.push("   1. inner".to_owned());
                }
            }
            let text = lines.join("\n");
            let at = text.find(&format!("w{}", pick + 1)).unwrap() + 2;
            let edit = enter_in_list(&text, at..at).unwrap();
            let new = edit.apply(&text);
            let top: Vec<u64> = new.lines().filter_map(item).filter(|i| i.indent == 0).filter_map(|i| i.number).collect();
            prop_assert_eq!(top, (1..=(n as u64 + 1)).collect::<Vec<_>>());
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
