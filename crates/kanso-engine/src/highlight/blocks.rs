//! Cutting a note into blocks that highlight on their own.
//!
//! A block runs from a line that starts in column 0 with something other
//! than whitespace and follows a blank line, to the next such line. That
//! boundary ends every container Markdown has (a list item needs its
//! indentation, a quote its `>`, and blank lines end both), so the syntax
//! tree of a note never crosses it, with three exceptions that swallow blank
//! lines and unindented text: a fenced code block, an HTML block that ends
//! at a marker (`<script>`, `<!--`, and the like) and front matter. The
//! cutter keeps track of those. When it is unsure, it doesn't cut: a block
//! that is too large costs only cache hits, never a wrong color.
//!
//! Being unsure is the usual case for a fence or an HTML block indented one
//! to three spaces, which is a top-level one or one inside a list item, and
//! for anything indented further, which may be code or a list item's. The
//! cutter follows every reading at once (a set of "worlds") and cuts only
//! when no reading has such a block open at the top level.
//!
//! The grammar (tree-sitter-md) is read as it is, not as CommonMark has it,
//! where they differ: it closes a fence at any line that ends with the
//! fence, at any indentation, and it ends only some HTML blocks on their own
//! line. And a note that opens with a rule is read as having front matter
//! between that rule and a later one by the grammar's injection query, up to
//! the first heading, so it has no cuts before that heading.
//!
//! The scan is one pass over the lines and parses nothing. What it can't
//! see is a syntax error, whose recovery reaches into what follows; the
//! caller finds those (`super::assemble`).

use std::cell::OnceCell;
use std::ops::Range;

/// The most readings followed at once; past it the rest of the note is one
/// block.
const MAX_WORLDS: usize = 64;

/// What ends an HTML block of the kinds that can contain blank lines.
///
/// The names are the grammar's own kinds (tree-sitter-md 0.5.3's scanner,
/// which differs from CommonMark in two places that matter here): there are
/// three tags of the first kind (`script`, `pre`, `style`), which any of
/// their end tags ends, and a `<!X` declaration is ended by a `>` that the
/// grammar finds only in odd places, so it is taken not to end at all.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum End {
    Script,
    Pre,
    Style,
    Comment,
    Instruction,
    Declaration,
    Cdata,
}

impl End {
    /// The text that ends the block, when one line holding it does.
    fn marker(self) -> Option<&'static str> {
        match self {
            End::Script => Some("</script>"),
            End::Pre => Some("</pre>"),
            End::Style => Some("</style>"),
            End::Comment => Some("-->"),
            End::Instruction => Some("?>"),
            End::Declaration => None,
            End::Cdata => Some("]]>"),
        }
    }

    /// Whether `text` holds the marker.
    fn within(self, text: &str) -> bool {
        self.marker().is_some_and(|marker| text.contains(marker))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    /// A fenced code block opened with `len` fence characters `ch`.
    Fence { ch: u8, len: usize },
    /// An HTML block that ends at a line holding the marker.
    Html(End),
    /// An HTML block that ends at a blank line.
    Lines,
}

/// A block that is open: at the top level (`indent` is `None`) or inside a
/// list item, where `indent` is the column the opening line started at (the
/// item's content starts at or before it).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Open {
    kind: Kind,
    indent: Option<usize>,
}

/// One reading of the text so far: `None` for nothing open.
type World = Option<Open>;

/// What a line is made of.
struct Line<'a> {
    blank: bool,
    /// Columns of leading whitespace, tabs counting to the next multiple of 4.
    indent: usize,
    /// The line without its leading whitespace.
    rest: &'a str,
    /// `rest` in ASCII lower case, made when first needed.
    lower: OnceCell<String>,
}

impl<'a> Line<'a> {
    fn new(text: &'a str) -> Line<'a> {
        let text = text.strip_suffix('\r').unwrap_or(text);
        let mut indent = 0;
        for (at, c) in text.char_indices() {
            match c {
                ' ' => indent += 1,
                '\t' => indent += 4 - indent % 4,
                _ => {
                    return Line {
                        blank: false,
                        indent,
                        rest: &text[at..],
                        lower: OnceCell::new(),
                    };
                }
            }
        }
        Line {
            blank: true,
            indent,
            rest: "",
            lower: OnceCell::new(),
        }
    }

    fn lower(&self) -> &str {
        self.lower.get_or_init(|| self.rest.to_ascii_lowercase())
    }

    /// A fence's character and length, when the line starts one.
    fn fence(&self) -> Option<(u8, usize)> {
        let ch = *self.rest.as_bytes().first()?;
        if ch != b'`' && ch != b'~' {
            return None;
        }
        let len = self.rest.bytes().take_while(|b| *b == ch).count();
        // A backtick fence's info string has no backtick: that is a code span.
        let code_span = ch == b'`' && self.rest[len..].contains('`');
        (len >= 3 && !code_span).then_some((ch, len))
    }

    /// Whether the line closes a fence of `len` `ch`s. Markdown wants the
    /// line to be only that; the grammar closes at any line that ends with
    /// that many (`code ```` does), which is what counts here.
    fn closes(&self, ch: u8, len: usize) -> bool {
        let text = self.rest.trim_end_matches([' ', '\t']);
        text.bytes().rev().take_while(|b| *b == ch).count() >= len
    }

    /// The HTML block this line opens that ends at a marker, and whether
    /// the marker already ends the line.
    fn html(&self) -> Option<(End, bool)> {
        let lower = self.lower();
        let tail = lower.strip_prefix('<')?;
        for (name, end) in [
            ("script", End::Script),
            ("pre", End::Pre),
            ("style", End::Style),
        ] {
            if let Some(after) = tail.strip_prefix(name) {
                return Some((end, end.within(after)));
            }
        }
        let after = |prefix: &str| tail.strip_prefix(prefix);
        if let Some(after) = after("!--") {
            Some((End::Comment, End::Comment.within(after)))
        } else if let Some(after) = after("?") {
            Some((End::Instruction, End::Instruction.within(after)))
        } else if let Some(after) = after("![cdata[") {
            Some((End::Cdata, End::Cdata.within(after)))
        } else if self.rest[1..]
            .strip_prefix('!')
            .is_some_and(|after| after.starts_with(|c: char| c.is_ascii_uppercase()))
        {
            Some((End::Declaration, false))
        } else {
            None
        }
    }
}

/// The worlds after `line`, given the worlds before it.
fn step(worlds: &[World], line: &Line) -> Vec<World> {
    let mut next: Vec<World> = Vec::with_capacity(worlds.len() + 2);
    let mut push = |world: World| {
        if !next.contains(&world) {
            next.push(world);
        }
    };
    for world in worlds {
        match world {
            None => plain(line, &mut push),
            Some(open) => open_world(*open, line, &mut push),
        }
    }
    next
}

/// A line read with nothing open.
fn plain(line: &Line, push: &mut impl FnMut(World)) {
    if line.blank {
        push(None);
        return;
    }
    let (i, rest) = (line.indent, line.rest);
    let mut bare = true;
    if let Some((ch, len)) = line.fence() {
        let kind = Kind::Fence { ch, len };
        // Up to three columns in, it opens a fence, at the top level or in a
        // list item; further in, it may be code.
        if i <= 3 {
            push(Some(Open { kind, indent: None }));
            bare = false;
        }
        if i >= 1 {
            push(Some(Open {
                kind,
                indent: Some(i),
            }));
        }
    } else if let Some((end, closed)) = line.html()
        // One that opens and ends on a line is nothing to follow.
        && !closed
    {
        // It may not be one (the grammar knows fewer than CommonMark does),
        // so the line stays plain text too.
        let kind = Kind::Html(end);
        if i <= 3 {
            push(Some(Open { kind, indent: None }));
        }
        if i >= 1 {
            push(Some(Open {
                kind,
                indent: Some(i),
            }));
        }
    }
    if rest.starts_with('<') {
        // An HTML block of the kinds that end at a blank line, or text.
        if i <= 3 {
            push(Some(Open {
                kind: Kind::Lines,
                indent: None,
            }));
        }
        if i >= 1 {
            push(Some(Open {
                kind: Kind::Lines,
                indent: Some(i),
            }));
        }
    }
    if bare {
        push(None);
    }
}

/// A line read inside an open block.
fn open_world(open: Open, line: &Line, push: &mut impl FnMut(World)) {
    let stay = Some(open);
    // Blank lines end the kinds that end at one, and nothing else.
    if line.blank {
        push(if open.kind == Kind::Lines { None } else { stay });
        return;
    }
    let ends = || match open.kind {
        Kind::Fence { ch, len } => line.closes(ch, len),
        Kind::Html(end) => end.within(line.lower()),
        Kind::Lines => false,
    };
    match open.indent {
        None => {
            // The grammar closes a fence whatever the indentation of the line.
            push(if ends() { None } else { stay });
        }
        Some(k) => {
            // Unindented text ends a list item, and with it what is in it.
            if line.indent == 0 {
                plain(line, push);
                return;
            }
            // The item's content starts at or before column `k`; a line
            // indented less may be outside it.
            if line.indent < k {
                plain(line, push);
            }
            if ends() {
                push(None);
                if line.indent > 4 {
                    push(stay);
                }
            } else {
                push(stay);
            }
        }
    }
}

/// Whether some reading has a block open at the top level that swallows
/// blank lines and unindented text.
fn swallowing(worlds: &[World]) -> bool {
    worlds.iter().any(|world| {
        matches!(
            world,
            Some(Open {
                kind: Kind::Fence { .. } | Kind::Html(_),
                indent: None,
            })
        )
    })
}

/// Whether the line is a thematic break, or looks like one: only `-`, `*`
/// and `_`, and spaces, three or more of them.
fn is_rule(line: &str) -> bool {
    let line = line.trim_matches([' ', '\t', '\r']);
    line.chars()
        .all(|c| matches!(c, '-' | '*' | '_' | ' ' | '\t'))
        && line.chars().filter(|c| *c != ' ' && *c != '\t').count() >= 3
}

/// Whether the line is an ATX heading: one to six `#`, then a space or the
/// end of the line, up to three columns in.
fn is_heading(line: &Line) -> bool {
    let hashes = line.rest.bytes().take_while(|b| *b == b'#').count();
    line.indent <= 3 && (1..=6).contains(&hashes) && line.rest[hashes..].starts_with([' ', '\t'])
        || line.indent <= 3 && (1..=6).contains(&hashes) && line.rest.len() == hashes
}

/// Cuts `text` into consecutive ranges that cover it, each a block. The
/// range of a block holds the blank lines after it.
pub(super) fn cut(text: &str) -> Vec<Range<usize>> {
    if text.is_empty() {
        return Vec::new();
    }
    // A lone carriage return is a line ending too; not worth reading.
    let bytes = text.as_bytes();
    if bytes
        .iter()
        .enumerate()
        .any(|(i, b)| *b == b'\r' && bytes.get(i + 1) != Some(&b'\n'))
    {
        return std::iter::once(0..text.len()).collect();
    }

    let lines: Vec<(usize, &str)> = {
        let mut start = 0;
        text.split_inclusive('\n')
            .map(|raw| {
                let at = start;
                start += raw.len();
                (at, raw.strip_suffix('\n').unwrap_or(raw))
            })
            .collect()
    };

    // Front matter: a first line of `---` or `+++` and a later one the same.
    let mut skip_to = 0;
    if let Some((_, first)) = lines.first() {
        let fence = first.trim_end_matches([' ', '\t', '\r']);
        if (fence == "---" || fence == "+++")
            && let Some(closing) = lines[1..]
                .iter()
                .position(|(_, l)| l.trim_end_matches([' ', '\t', '\r']) == fence)
        {
            skip_to = closing + 2;
        }
    }

    // A note that opens with a rule has its first section, up to the first
    // heading, read as front matter by the grammar's injection query:
    // whatever sits between that rule and a later one is colored as YAML. So
    // that reading doesn't depend on where the cuts fall, there are none
    // until that heading.
    let mut holding = skip_to == 0
        && lines
            .iter()
            .find(|(_, l)| !Line::new(l).blank)
            .is_some_and(|(_, l)| is_rule(l));

    let mut cuts = vec![0];
    let mut worlds: Vec<World> = vec![None];
    let mut previous_blank = false;
    let mut stopped = false;
    for (index, (start, raw)) in lines.iter().enumerate() {
        if index < skip_to {
            continue;
        }
        let line = Line::new(raw);
        let candidate = previous_blank
            && !raw.is_empty()
            && raw.chars().next().is_some_and(|c| !c.is_whitespace());
        if holding && candidate && worlds.iter().all(Option::is_none) && is_heading(&line) {
            holding = false;
        }
        if candidate && !holding && !stopped && !swallowing(&worlds) {
            cuts.push(*start);
        }
        worlds = step(&worlds, &line);
        if worlds.len() > MAX_WORLDS {
            stopped = true;
            worlds.truncate(MAX_WORLDS);
        }
        previous_blank = line.blank;
    }
    cuts.push(text.len());
    cuts.windows(2).map(|w| w[0]..w[1]).collect()
}

#[cfg(test)]
mod tests {
    use super::cut;

    fn blocks(text: &str) -> Vec<&str> {
        cut(text).into_iter().map(|r| &text[r]).collect()
    }

    #[test]
    fn paragraphs_are_cut_after_blank_lines() {
        assert_eq!(blocks("a\n\nb\n\n\nc"), ["a\n\n", "b\n\n\n", "c"]);
        assert_eq!(blocks("a\nb\n\nc\n"), ["a\nb\n\n", "c\n"]);
        assert_eq!(blocks(""), Vec::<&str>::new());
        assert_eq!(blocks("one"), ["one"]);
    }

    #[test]
    fn indented_lines_continue_a_block() {
        assert_eq!(blocks("- a\n\n  b\n\n- c\n"), ["- a\n\n  b\n\n", "- c\n"]);
        assert_eq!(blocks("a\n\n    code\n\nb"), ["a\n\n    code\n\n", "b"]);
        assert_eq!(blocks("a\n\n\tcode\n\nb"), ["a\n\n\tcode\n\n", "b"]);
    }

    #[test]
    fn fences_swallow_blank_lines_and_unindented_text() {
        assert_eq!(
            blocks("```\na\n\nb\n```\n\nc"),
            ["```\na\n\nb\n```\n\n", "c"]
        );
        assert_eq!(blocks("~~~\na\n\nb\n```\n\nc"), ["~~~\na\n\nb\n```\n\nc"]);
        // Unclosed: the rest of the note.
        assert_eq!(blocks("x\n\n```\na\n\nb"), ["x\n\n", "```\na\n\nb"]);
        // A longer fence needs a longer closer.
        assert_eq!(
            blocks("````\n```\n\nb\n````\n\nc"),
            ["````\n```\n\nb\n````\n\n", "c"]
        );
    }

    #[test]
    fn fences_in_list_items_do_not_confuse_the_cutter() {
        // Indented fence in a list item, closed: cuts resume.
        assert_eq!(
            blocks("- a\n\n  ```\n  x\n\n  y\n  ```\n\nb\n\nc"),
            ["- a\n\n  ```\n  x\n\n  y\n  ```\n\n", "b\n\n", "c"]
        );
        // The item ends before its fence closes, and an indented fence
        // opens at the top level: not cut inside it.
        let text = "- a\n\n  ```\n  code\n\nx\n\n  ```\n  more\n\ny";
        assert!(blocks(text).last().unwrap().contains("more"));
        assert!(blocks(text).iter().all(|b| !b.starts_with("y")));
    }

    #[test]
    fn html_blocks_that_hold_blank_lines_are_kept_whole() {
        assert_eq!(
            blocks("<!--\na\n\nb\n-->\n\nc"),
            ["<!--\na\n\nb\n-->\n\n", "c"]
        );
        assert_eq!(blocks("<!-- a -->\n\nc"), ["<!-- a -->\n\n", "c"]);
        assert_eq!(
            blocks("<script>\na\n\nb\n</script>\n\nc"),
            ["<script>\na\n\nb\n</script>\n\n", "c"]
        );
        assert_eq!(blocks("<pre>\na\n\nb"), ["<pre>\na\n\nb"]);
        assert_eq!(blocks("<div>\na\n\nb"), ["<div>\na\n\n", "b"]);
    }

    #[test]
    fn front_matter_is_one_block_with_what_follows_it() {
        assert_eq!(
            blocks("---\na: 1\n\nb: 2\n---\n\nc"),
            ["---\na: 1\n\nb: 2\n---\n\n", "c"]
        );
        // Not closed: a rule.
        assert_eq!(blocks("---\na\n\nb"), ["---\na\n\nb"]);
        // Only at the very start.
        assert_eq!(
            blocks("x\n\n---\na\n\nb\n---"),
            ["x\n\n", "---\na\n\n", "b\n---"]
        );
    }

    #[test]
    fn a_lone_carriage_return_keeps_the_note_whole() {
        assert_eq!(blocks("a\r\rb"), ["a\r\rb"]);
        assert_eq!(blocks("a\r\n\r\nb"), ["a\r\n\r\n", "b"]);
    }

    #[test]
    fn unusual_whitespace_is_not_a_blank_line_or_a_start() {
        assert_eq!(blocks("a\n\u{a0}\nb"), ["a\n\u{a0}\nb"]);
        assert_eq!(blocks("a\n\n\u{a0}b"), ["a\n\n\u{a0}b"]);
    }
}
