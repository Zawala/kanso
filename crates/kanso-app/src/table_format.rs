//! GitHub tables in the source: lining up their pipes, and moving between
//! their cells with Tab, Shift-Tab and Enter.
//!
//! Formatting pads every cell to its column's width as it shows in a
//! monospaced face (CJK characters and most emoji take two columns), keeps
//! each cell's text, honours the alignment row (`:--`, `:-:`, `--:`), gives
//! short rows empty cells, and is idempotent: a formatted table formats to
//! itself. Pure: text in, one edit out. Offsets are UTF-8 byte offsets.

use std::ops::Range;

use unicode_width::UnicodeWidthStr as _;

use crate::markdown_edit::{Edit, line_end, line_start};
use crate::pairing::in_fenced_code;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Align {
    None,
    Left,
    Center,
    Right,
}

/// A table found in the text.
#[derive(Clone, Debug)]
struct Table {
    /// From the start of the header line to the end of the last row, before
    /// its newline.
    range: Range<usize>,
    /// Each line's range in the text, the alignment row second.
    lines: Vec<Range<usize>>,
    /// The header's indent, which every line keeps.
    indent: String,
    /// The cells of the header and of each body row, trimmed.
    rows: Vec<Vec<String>>,
    align: Vec<Align>,
}

/// Where the cursor is in a table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Spot {
    /// The line in the table: 0 the header, 1 the alignment row.
    line: usize,
    col: usize,
    /// Bytes into the cell's trimmed text.
    offset: usize,
}

/// Whether the byte at `ix` is escaped by an odd run of backslashes.
fn escaped(line: &str, ix: usize) -> bool {
    line.as_bytes()[..ix]
        .iter()
        .rev()
        .take_while(|&&b| b == b'\\')
        .count()
        % 2
        == 1
}

/// The pipes in `line` that divide cells: not escaped.
fn pipes(line: &str) -> Vec<usize> {
    line.bytes()
        .enumerate()
        .filter(|&(ix, b)| b == b'|' && !escaped(line, ix))
        .map(|(ix, _)| ix)
        .collect()
}

/// Each cell of a table line, untrimmed, relative to the line; the
/// leading and trailing pipes are optional.
fn cell_ranges(line: &str) -> Vec<Range<usize>> {
    let mut pipes = pipes(line);
    let start = line.len() - line.trim_start().len();
    let end = line.trim_end().len();
    let mut from = start;
    let mut to = end;
    if pipes.first() == Some(&start) {
        from = start + 1;
        pipes.remove(0);
    }
    if end > from && pipes.last() == Some(&(end - 1)) {
        to = end - 1;
        pipes.pop();
    }
    let mut cells = Vec::with_capacity(pipes.len() + 1);
    let mut at = from;
    for pipe in pipes.into_iter().filter(|p| *p >= from && *p < to) {
        cells.push(at..pipe);
        at = pipe + 1;
    }
    cells.push(at..to.max(at));
    cells
}

/// A cell's range without the spaces around its text.
fn trimmed(line: &str, cell: Range<usize>) -> Range<usize> {
    let raw = &line[cell.clone()];
    let start = cell.start + (raw.len() - raw.trim_start().len());
    let end = cell.start + raw.trim_end().len();
    start..end.max(start)
}

fn is_table_line(line: &str) -> bool {
    !pipes(line).is_empty()
}

/// The alignment of each column, if `line` is an alignment row.
fn alignment_row(line: &str) -> Option<Vec<Align>> {
    cell_ranges(line)
        .into_iter()
        .map(|cell| {
            let cell = line[cell].trim();
            let left = cell.starts_with(':');
            let right = cell.len() > 1 && cell.ends_with(':');
            let dashes = cell.trim_start_matches(':').trim_end_matches(':');
            if dashes.is_empty() || !dashes.bytes().all(|b| b == b'-') {
                return None;
            }
            Some(match (left, right) {
                (true, true) => Align::Center,
                (true, false) => Align::Left,
                (false, true) => Align::Right,
                (false, false) => Align::None,
            })
        })
        .collect()
}

/// The table the line at `at` is in: a header row, an alignment row with
/// as many cells, and the rows after them, up to a line without a pipe.
fn table_at(text: &str, at: usize) -> Option<Table> {
    let at = at.min(text.len());
    let here = line_start(text, at);
    if !is_table_line(&text[here..line_end(text, here)]) || in_fenced_code(text, at) {
        return None;
    }
    // The run of lines with pipes around the cursor.
    let mut first = here;
    while first > 0 {
        let prev = line_start(text, first - 1);
        if !is_table_line(&text[prev..first - 1]) {
            break;
        }
        first = prev;
    }
    let mut lines = Vec::new();
    let mut start = first;
    loop {
        let end = line_end(text, start);
        if !is_table_line(&text[start..end]) {
            break;
        }
        lines.push(start..end);
        if end >= text.len() {
            break;
        }
        start = end + 1;
    }
    // The header is the line before the first alignment row with as many
    // cells; earlier lines with pipes aren't part of this table.
    let header = (1..lines.len()).find(|&ix| {
        let head = &text[lines[ix - 1].clone()];
        alignment_row(&text[lines[ix].clone()])
            .is_some_and(|align| align.len() == cell_ranges(head).len())
    })? - 1;
    let lines = lines.split_off(header);
    if lines[0].start > here {
        return None;
    }
    let head = &text[lines[0].clone()];
    let indent = head[..head.len() - head.trim_start_matches(' ').len()].to_owned();
    let mut align = alignment_row(&text[lines[1].clone()])?;
    let mut rows: Vec<Vec<String>> = lines
        .iter()
        .enumerate()
        .filter(|(ix, _)| *ix != 1)
        .map(|(_, range)| {
            let line = &text[range.clone()];
            cell_ranges(line)
                .into_iter()
                .map(|cell| line[trimmed(line, cell)].to_owned())
                .collect()
        })
        .collect();
    let columns = rows
        .iter()
        .map(Vec::len)
        .max()
        .unwrap_or(0)
        .max(align.len());
    for row in &mut rows {
        row.resize(columns, String::new());
    }
    align.resize(columns, Align::None);
    Some(Table {
        range: lines[0].start..lines[lines.len() - 1].end,
        lines,
        indent,
        rows,
        align,
    })
}

impl Table {
    fn columns(&self) -> usize {
        self.align.len()
    }

    /// The rows of the table as lines go: the alignment row is line 1.
    fn row_of_line(line: usize) -> Option<usize> {
        match line {
            0 => Some(0),
            1 => None,
            n => Some(n - 1),
        }
    }

    /// Where `at` is in the table.
    fn spot(&self, text: &str, at: usize) -> Spot {
        let line = self
            .lines
            .iter()
            .position(|range| range.start <= at && at <= range.end)
            .unwrap_or(0);
        let range = self.lines[line].clone();
        let content = &text[range.clone()];
        let rel = at - range.start;
        let cells = cell_ranges(content);
        let col = cells
            .iter()
            .position(|cell| rel <= cell.end)
            .unwrap_or(cells.len() - 1)
            .min(self.columns().saturating_sub(1));
        let inner = cells
            .get(col)
            .map_or(rel..rel, |cell| trimmed(content, cell.clone()));
        let offset = rel.clamp(inner.start, inner.end) - inner.start;
        Spot { line, col, offset }
    }

    /// The widths of the columns as they show, at least three.
    fn widths(&self) -> Vec<usize> {
        (0..self.columns())
            .map(|col| {
                self.rows
                    .iter()
                    .map(|row| row[col].width())
                    .max()
                    .unwrap_or(0)
                    .max(3)
            })
            .collect()
    }

    /// The table formatted, and where each line's cells' text starts in it,
    /// relative to the table: `[line][col]`.
    fn render(&self) -> (String, Vec<Vec<usize>>) {
        let widths = self.widths();
        let mut out = String::new();
        let mut starts = Vec::with_capacity(self.rows.len() + 1);
        let mut line = |cells: Vec<(String, usize)>, out: &mut String| {
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(&self.indent);
            out.push('|');
            let mut row = Vec::with_capacity(cells.len());
            for (cell, left) in cells {
                out.push(' ');
                row.push(out.len() + left);
                out.push_str(&cell);
                out.push_str(" |");
            }
            starts.push(row);
        };
        let pad = |text: &str, width: usize, align: Align| -> (String, usize) {
            let room = width.saturating_sub(text.width());
            let left = match align {
                Align::Right => room,
                Align::Center => room / 2,
                Align::None | Align::Left => 0,
            };
            (
                format!("{}{text}{}", " ".repeat(left), " ".repeat(room - left)),
                left,
            )
        };
        for (ix, row) in self.rows.iter().enumerate() {
            let cells = row
                .iter()
                .zip(&widths)
                .zip(&self.align)
                .map(|((text, width), align)| pad(text, *width, *align))
                .collect();
            line(cells, &mut out);
            if ix == 0 {
                let dashes = widths
                    .iter()
                    .zip(&self.align)
                    .map(|(width, align)| {
                        let cell = match align {
                            Align::None => "-".repeat(*width),
                            Align::Left => format!(":{}", "-".repeat(width - 1)),
                            Align::Right => format!("{}:", "-".repeat(width - 1)),
                            Align::Center => format!(":{}:", "-".repeat(width - 2)),
                        };
                        (cell, 0)
                    })
                    .collect();
                line(dashes, &mut out);
            }
        }
        (out, starts)
    }

    /// Where the text of the cell at `line`, `col` starts in the table as
    /// it is in `text`, relative to the table, and how long it is.
    fn cell_in_place(&self, text: &str, line: usize, col: usize) -> Range<usize> {
        let range = self.lines[line].clone();
        let content = &text[range.clone()];
        let cells = cell_ranges(content);
        let base = range.start - self.range.start;
        match cells.get(col) {
            Some(cell) => {
                let inner = trimmed(content, cell.clone());
                base + inner.start..base + inner.end
            }
            None => base + content.len()..base + content.len(),
        }
    }
}

/// The text of the table at `at`, formatted, with the cursor kept in its
/// cell; `None` outside a table.
pub fn format_table(text: &str, at: usize) -> Option<Edit> {
    let table = table_at(text, at)?;
    let spot = table.spot(text, at);
    let (out, starts) = table.render();
    let row = Table::row_of_line(spot.line);
    let len = row.map_or(0, |row| table.rows[row][spot.col].len());
    let cursor = table.range.start + starts[spot.line][spot.col] + spot.offset.min(len);
    Some(Edit {
        range: table.range,
        text: out,
        selection: cursor..cursor,
    })
}

/// Whether `at` is in a table.
pub fn in_table(text: &str, at: usize) -> bool {
    table_at(text, at).is_some()
}

/// A table's new text, and where each line's cells' text is in it.
struct Laid {
    text: String,
    /// `[line][col]`, relative to the table.
    cells: Vec<Vec<Range<usize>>>,
}

/// The table (formatted when `format`), with an empty row after the line
/// `new_row_after` when there is one.
fn lay_out(mut table: Table, text: &str, format: bool, new_row_after: Option<usize>) -> Laid {
    if format {
        if let Some(line) = new_row_after {
            let at = Table::row_of_line(line).unwrap_or(0) + 1;
            table.rows.insert(at, vec![String::new(); table.columns()]);
        }
        let (out, starts) = table.render();
        let cells = starts
            .iter()
            .enumerate()
            .map(|(line, row)| {
                let lens: Vec<usize> = match Table::row_of_line(line) {
                    Some(r) => table.rows[r].iter().map(String::len).collect(),
                    None => vec![0; row.len()],
                };
                row.iter().zip(lens).map(|(s, l)| *s..*s + l).collect()
            })
            .collect();
        return Laid { text: out, cells };
    }
    let mut out = text[table.range.clone()].to_owned();
    let mut cells: Vec<Vec<Range<usize>>> = (0..table.lines.len())
        .map(|line| {
            (0..table.columns())
                .map(|col| table.cell_in_place(text, line, col))
                .collect()
        })
        .collect();
    if let Some(line) = new_row_after {
        let line = line.max(1);
        let at = table.lines[line].end - table.range.start;
        let columns = table.columns();
        // Two spaces a cell, so what is typed in one has a space each side.
        let row = format!("\n{}|{}", table.indent, "  |".repeat(columns));
        out.insert_str(at, &row);
        let shift = row.len();
        for later in cells.iter_mut().skip(line + 1) {
            for cell in later.iter_mut() {
                *cell = cell.start + shift..cell.end + shift;
            }
        }
        let first = at + 1 + table.indent.len() + 2;
        let new: Vec<Range<usize>> = (0..columns)
            .map(|col| first + 3 * col..first + 3 * col)
            .collect();
        cells.insert(line + 1, new);
    }
    Laid { text: out, cells }
}

/// Tab (or Shift-Tab, `back`) in a table: on to the next cell, or back to
/// the one before, selecting its text, past the alignment row. Tab in the
/// last cell adds a row. With `format`, the table is formatted on the way.
pub fn next_cell(text: &str, at: usize, back: bool, format: bool) -> Option<Edit> {
    let table = table_at(text, at)?;
    let spot = table.spot(text, at);
    let columns = table.columns();
    let lines = table.lines.len();
    // Cells in reading order, as (line, col), without the alignment row.
    let order: Vec<(usize, usize)> = (0..lines)
        .filter(|line| *line != 1)
        .flat_map(|line| (0..columns).map(move |col| (line, col)))
        .collect();
    let here = if spot.line == 1 {
        // On the alignment row: as if at the end of the header.
        order.iter().position(|&(l, c)| l == 0 && c == columns - 1)
    } else {
        order
            .iter()
            .position(|&(l, c)| l == spot.line && c == spot.col)
    }
    .unwrap_or(0);
    let range = table.range.clone();
    let (target, add_row) = if back {
        (order[here.saturating_sub(1)], None)
    } else if here + 1 < order.len() {
        (order[here + 1], None)
    } else {
        ((lines, 0), Some(lines - 1))
    };
    let laid = lay_out(table, text, format, add_row);
    let cell = laid.cells[target.0][target.1].clone();
    Some(Edit {
        range: range.clone(),
        text: laid.text,
        selection: range.start + cell.start..range.start + cell.end,
    })
}

/// Enter at the end of a table row: a new row below it, the cursor in its
/// first cell. Enter in an empty last row leaves the table, with a blank
/// line after it. `None` anywhere else: Enter as usual.
pub fn enter_in_table(text: &str, at: usize, format: bool) -> Option<Edit> {
    let table = table_at(text, at)?;
    let spot = table.spot(text, at);
    let line = table.lines[spot.line].clone();
    let rest = text[at.min(line.end)..line.end].trim();
    if !(rest.is_empty() || rest == "|") {
        return None;
    }
    let last = table.lines.len() - 1;
    if spot.line == last && spot.line > 1 && table.rows[last - 1].iter().all(String::is_empty) {
        // Leave the table: the empty row goes, a blank line follows it.
        let start = table.lines[last - 1].end;
        return Some(Edit {
            range: start..line.end,
            text: "\n\n".into(),
            selection: start + 2..start + 2,
        });
    }
    let range = table.range.clone();
    let after = spot.line.max(1);
    let laid = lay_out(table, text, format, Some(after));
    let cell = laid.cells[after + 1][0].clone();
    Some(Edit {
        range: range.clone(),
        text: laid.text,
        selection: range.start + cell.start..range.start + cell.start,
    })
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

    fn formatted(text: &str) -> String {
        format_table(text, 0).unwrap().apply(text)
    }

    #[test]
    fn formats_a_table() {
        let text = "|a|b|\n|-|:-:|\n|long cell|x|";
        assert_eq!(
            formatted(text),
            "| a         |  b  |\n| --------- | :-: |\n| long cell |  x  |"
        );
        // Without the outer pipes, short rows and right alignment.
        let text = "Name | Qty\n:-- | --:\nApples | 3\nPears |";
        assert_eq!(
            formatted(text),
            "| Name   | Qty |\n| :----- | --: |\n| Apples |   3 |\n| Pears  |     |"
        );
    }

    #[test]
    fn wide_characters_take_two_columns() {
        let text = "| 名前 | x |\n|---|---|\n| 日本語テキスト | y |";
        assert_eq!(
            formatted(text),
            "| 名前           | x   |\n| -------------- | --- |\n| 日本語テキスト | y   |"
        );
    }

    #[test]
    fn escaped_pipes_stay_in_their_cell() {
        let text = "| a \\| b | c |\n|--|--|\n| 1 | 2 |";
        assert_eq!(
            formatted(text),
            "| a \\| b | c   |\n| ------ | --- |\n| 1      | 2   |"
        );
    }

    #[test]
    fn extra_cells_are_kept() {
        let text = "| a | b |\n|---|---|\n| 1 | 2 | 3 |";
        assert_eq!(
            formatted(text),
            "| a   | b   |     |\n| --- | --- | --- |\n| 1   | 2   | 3   |"
        );
    }

    #[test]
    fn only_tables_format() {
        assert!(format_table("a | b\nnot a table", 0).is_none());
        assert!(format_table("```\n| a | b |\n|---|---|\n```", 5).is_none());
        assert!(format_table("plain", 0).is_none());
        // Text above the header is left alone.
        let text = "x | y\n| a | b |\n|---|---|";
        let edit = format_table(text, text.len()).unwrap();
        assert_eq!(edit.range.start, 6);
    }

    #[test]
    fn the_cursor_stays_in_its_cell() {
        let text = "|a|bc|\n|-|-|\n|x|y|";
        // After the "b".
        let edit = format_table(text, 4).unwrap();
        let out = edit.apply(text);
        assert_eq!(&out[..edit.selection.start], "| a   | b");
    }

    #[test]
    fn tab_moves_between_cells_and_adds_a_row() {
        let text = "| a | b |\n|---|---|\n| c | d |";
        let edit = next_cell(text, 2, false, true).unwrap();
        let out = edit.apply(text);
        assert_eq!(&out[edit.selection.clone()], "b");
        // From the header's last cell, past the alignment row.
        let edit = next_cell(&out, edit.selection.start, false, true).unwrap();
        assert_eq!(&out[edit.selection.clone()], "c");
        let edit = next_cell(text, 26, true, false).unwrap();
        assert_eq!(&text[edit.selection.clone()], "c");
        // The last cell: a new row, the cursor in its first cell.
        let edit = next_cell(text, text.len() - 2, false, true).unwrap();
        let out = edit.apply(text);
        assert_eq!(
            out,
            "| a   | b   |\n| --- | --- |\n| c   | d   |\n|     |     |"
        );
        assert_eq!(edit.selection, 44..44);
        // Unformatted.
        let edit = next_cell(text, text.len() - 2, false, false).unwrap();
        let out = edit.apply(text);
        assert_eq!(out, "| a | b |\n|---|---|\n| c | d |\n|  |  |");
        assert_eq!(edit.selection, 32..32);
    }

    #[test]
    fn enter_adds_a_row_and_leaves_an_empty_one() {
        let text = "| a | b |\n|---|---|\n| c | d |";
        let edit = enter_in_table(text, text.len(), true).unwrap();
        let out = edit.apply(text);
        assert_eq!(
            out,
            "| a   | b   |\n| --- | --- |\n| c   | d   |\n|     |     |"
        );
        let edit = enter_in_table(&out, out.len(), true).unwrap();
        let left = edit.apply(&out);
        assert_eq!(left, "| a   | b   |\n| --- | --- |\n| c   | d   |\n\n");
        assert_eq!(edit.selection.start, left.len());
        // In the middle of a row, Enter is Enter.
        assert!(enter_in_table(text, 22, true).is_none());
    }

    /// A table with cells of words, wide characters and escaped pipes, rows
    /// of uneven length, any alignment and optional outer pipes.
    fn table() -> impl Strategy<Value = String> {
        let cell = "[a-z 名é]{0,6}|\\\\\\|";
        let row = prop::collection::vec(cell, 1..4);
        (
            row.clone(),
            prop::collection::vec(
                prop::sample::select(vec!["---", ":--", "--:", ":-:", "-"]),
                1..4,
            ),
            prop::collection::vec(row, 0..4),
            any::<bool>(),
        )
            .prop_map(|(head, align, body, outer)| {
                let align: Vec<&str> = align.iter().cycle().take(head.len()).copied().collect();
                let line = |cells: &[String]| {
                    // Without outer pipes an empty first or last cell would
                    // read as one.
                    let cells: Vec<String> = cells
                        .iter()
                        .map(|c| {
                            if !outer && c.trim().is_empty() {
                                "x".into()
                            } else {
                                c.clone()
                            }
                        })
                        .collect();
                    let inner = cells.join("|");
                    if outer {
                        format!("|{inner}|")
                    } else {
                        format!("{inner} |")
                    }
                };
                let align: Vec<String> = align.iter().map(|s| s.to_string()).collect();
                let mut lines = vec![line(&head), line(&align)];
                lines.extend(body.iter().map(|r| line(r)));
                lines.join("\n")
            })
    }

    fn cells(text: &str) -> Vec<Vec<String>> {
        let t = table_at(text, 0).unwrap();
        t.rows
            .iter()
            .map(|row| {
                let mut row = row.clone();
                while row.last().is_some_and(String::is_empty) {
                    row.pop();
                }
                row
            })
            .collect()
    }

    proptest! {
        #![proptest_config(config())]

        /// Idempotent: a formatted table formats to itself.
        #[test]
        fn prop_formatting_is_idempotent(text in table()) {
            let once = formatted(&text);
            prop_assert_eq!(formatted(&once), once);
        }

        /// Every cell keeps its text, in its row and column.
        #[test]
        fn prop_formatting_keeps_every_cell(text in table()) {
            prop_assert_eq!(cells(&formatted(&text)), cells(&text));
        }

        /// The pipes line up: every line is as wide as the others.
        #[test]
        fn prop_pipes_line_up(text in table()) {
            let out = formatted(&text);
            let widths: Vec<usize> = out.lines().map(|l| l.width()).collect();
            prop_assert!(widths.windows(2).all(|w| w[0] == w[1]), "{out}");
        }

        /// The alignment row keeps each column's alignment.
        #[test]
        fn prop_alignment_is_kept(text in table()) {
            let before = table_at(&text, 0).unwrap().align;
            let after = table_at(&formatted(&text), 0).unwrap().align;
            prop_assert_eq!(before, after);
        }

        /// Tab through every cell and back: Shift-Tab undoes Tab's move.
        #[test]
        fn prop_tab_then_shift_tab_returns(text in table(), at in any::<prop::sample::Index>()) {
            let text = formatted(&text);
            let at = at.index(text.len() + 1);
            prop_assume!(text.is_char_boundary(at));
            let spot = table_at(&text, at).unwrap().spot(&text, at);
            prop_assume!(spot.line != 1);
            let there = next_cell(&text, at, false, true).unwrap();
            // Tab from the last cell adds a row; that is another property.
            prop_assume!(there.apply(&text) == text);
            let back = next_cell(&text, there.selection.start, true, true).unwrap();
            let start = table_at(&text, at).unwrap().spot(&text, back.selection.start);
            prop_assert_eq!((start.line, start.col), (spot.line, spot.col));
        }
    }
}
