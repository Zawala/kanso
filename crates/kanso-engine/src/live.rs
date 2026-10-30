//! Live mode: a note's Markdown drawn in place. Off the lines the caret or
//! the selection touch, the marks (`#`, `**`, `_`, `~~`, backticks, a
//! link's brackets and URL, a wiki-link's brackets) are hidden, and what
//! they mark is drawn bold, italic, struck through, as code or as a link;
//! headings are drawn larger on every line.
//!
//! [`live_layout`] says which bytes to hide and which spans to style; the
//! editors draw it. Offsets are UTF-8 bytes into the text; a phone converts
//! them with [`crate::text`].
//!
//! Images, HTML, tables' pipes, code blocks, front matter, math, list
//! bullets, quotes and the like are left as the highlighted source, as are
//! autolinks, bare URLs and escapes.

use std::ops::Range;

use markdown::mdast::Node;

/// What Live mode draws for a note.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LiveLayout {
    /// The bytes not drawn: sorted, disjoint, never empty, on character
    /// boundaries and never holding a line break. Only on lines no caret
    /// or selection touches.
    pub conceal: Vec<Range<usize>>,
    /// The spans drawn in a style, sorted by start. On every line, revealed
    /// ones too. They nest (a strong span in a heading), so they can
    /// overlap.
    pub styles: Vec<(Range<usize>, LiveStyle)>,
    /// Each heading's line (0-based) and level, in order, revealed or not.
    /// A Setext heading is at its text's first line.
    pub headings: Vec<(usize, u8)>,
}

/// How a span of a note is drawn in Live mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LiveStyle {
    Strong,
    Emphasis,
    Strikethrough,
    /// Inline code, without its backticks.
    Code,
    /// A link's or a wiki-link's text.
    Link,
    /// A heading's text, without its `#`s, at this level.
    Heading(u8),
}

/// How much larger a heading of `level` is drawn than body text.
pub fn heading_scale(level: u8) -> f32 {
    match level {
        1 => 1.6,
        2 => 1.4,
        3 => 1.2,
        4 => 1.1,
        _ => 1.0,
    }
}

/// What Live mode draws for `text` with carets and selections at `reveal`
/// (byte ranges; a caret is an empty one). A range reveals every line from
/// the one its start is on to the one its end is on, so a caret at the end
/// of a line reveals it. Pure; one parse of the text.
pub fn live_layout(text: &str, reveal: &[Range<usize>]) -> LiveLayout {
    let lines = Lines::new(text, reveal);
    let parsed = std::panic::catch_unwind(|| markdown::to_mdast(text, &parse_options()));
    let Ok(Ok(root)) = parsed else {
        // A panic or a parse failure draws the note as written.
        return LiveLayout::default();
    };
    let mut walk = Walk {
        text,
        lines: &lines,
        layout: LiveLayout::default(),
        texts: Vec::new(),
        ids: 0,
    };
    walk.node(&root, 0);
    walk.wiki_links();
    let Walk { mut layout, .. } = walk;
    layout.conceal = tidy(text, layout.conceal);
    layout
        .styles
        .retain(|(r, _)| r.start < r.end && r.end <= text.len() && on_boundaries(text, r));
    layout.styles.sort_by_key(|(r, _)| (r.start, r.end));
    layout.headings.sort_unstable();
    layout.headings.dedup_by_key(|(line, _)| *line);
    layout
}

/// The constructs the preview reads: GFM, front matter and math.
fn parse_options() -> markdown::ParseOptions {
    let mut options = markdown::ParseOptions::gfm();
    options.constructs.frontmatter = true;
    options.constructs.math_text = true;
    options.constructs.math_flow = true;
    options
}

/// The text's lines and which of them the reveal ranges touch.
struct Lines<'a> {
    text: &'a str,
    /// Where each line starts.
    starts: Vec<usize>,
    revealed: Vec<bool>,
}

impl<'a> Lines<'a> {
    fn new(text: &'a str, reveal: &[Range<usize>]) -> Self {
        let starts: Vec<usize> = std::iter::once(0)
            .chain(text.match_indices('\n').map(|(at, _)| at + 1))
            .collect();
        let mut lines = Self {
            text,
            revealed: vec![false; starts.len()],
            starts,
        };
        for range in reveal {
            let (a, b) = (range.start.min(range.end), range.start.max(range.end));
            let (first, last) = (lines.line_of(a), lines.line_of(b));
            lines.revealed[first..=last].fill(true);
        }
        lines
    }

    /// The line byte `at` is on; past the end, the last.
    fn line_of(&self, at: usize) -> usize {
        self.starts.partition_point(|&start| start <= at) - 1
    }

    /// Line `line` without its line break (`\n` or `\r\n`).
    fn content(&self, line: usize) -> Range<usize> {
        let start = self.starts[line];
        let end = self
            .starts
            .get(line + 1)
            .map_or(self.text.len(), |next| next - 1);
        let end = if self.text[start..end].ends_with('\r') {
            end - 1
        } else {
            end
        };
        start..end
    }
}

struct Walk<'a> {
    text: &'a str,
    lines: &'a Lines<'a>,
    layout: LiveLayout,
    /// Text nodes, in order, with an id of their parent: where wiki-links
    /// can be.
    texts: Vec<(Range<usize>, usize)>,
    /// The nodes walked so far, which numbers them.
    ids: usize,
}

impl Walk<'_> {
    /// Walks `node`, whose parent has id `parent`.
    fn node(&mut self, node: &Node, parent: usize) {
        self.ids += 1;
        let id = self.ids;
        let Some(span) = range(node).filter(|r| r.end <= self.text.len()) else {
            return self.children(node, id);
        };
        match node {
            // Left as the source: nothing in them is hidden.
            Node::Code(_)
            | Node::Html(_)
            | Node::Image(_)
            | Node::ImageReference(_)
            | Node::Yaml(_)
            | Node::Toml(_)
            | Node::Math(_)
            | Node::InlineMath(_)
            | Node::FootnoteReference(_)
            | Node::Definition(_) => {}
            Node::Text(_) => self.texts.push((span, parent)),
            Node::Heading(heading) => {
                let level = heading.depth.clamp(1, 6);
                let line = self.lines.line_of(span.start);
                self.layout.headings.push((line, level));
                let atx = self.text[span.clone()].starts_with('#');
                if let Some(inner) = self.inner(node, &span) {
                    if atx {
                        self.conceal(span.start..inner.start);
                        self.conceal(inner.end..span.end);
                    }
                    self.layout.styles.push((inner, LiveStyle::Heading(level)));
                }
                self.children(node, id);
            }
            Node::Strong(_) | Node::Emphasis(_) | Node::Delete(_) => {
                let style = match node {
                    Node::Strong(_) => LiveStyle::Strong,
                    Node::Emphasis(_) => LiveStyle::Emphasis,
                    _ => LiveStyle::Strikethrough,
                };
                if let Some(inner) = self.inner(node, &span) {
                    self.conceal(span.start..inner.start);
                    self.conceal(inner.end..span.end);
                    self.layout.styles.push((inner, style));
                }
                self.children(node, id);
            }
            Node::InlineCode(_) => {
                let source = &self.text[span.clone()];
                let open = source.bytes().take_while(|&b| b == b'`').count();
                let close = source.bytes().rev().take_while(|&b| b == b'`').count();
                if open > 0 && open == close && 2 * open < source.len() {
                    let inner = span.start + open..span.end - close;
                    self.conceal(span.start..inner.start);
                    self.conceal(inner.end..span.end);
                    self.layout.styles.push((inner, LiveStyle::Code));
                }
            }
            Node::Link(_) | Node::LinkReference(_) => {
                if let Some(inner) = self.inner(node, &span) {
                    // An autolink or a bare URL keeps its marks.
                    if self.text[span.clone()].starts_with('[') {
                        self.conceal(span.start..inner.start);
                        self.conceal(inner.end..span.end);
                    }
                    self.layout.styles.push((inner, LiveStyle::Link));
                }
                self.children(node, id);
            }
            _ => self.children(node, id),
        }
    }

    fn children(&mut self, node: &Node, id: usize) {
        for child in node.children().into_iter().flatten() {
            self.node(child, id);
        }
    }

    /// The span of `node`'s children, when they sit inside `span` in order;
    /// the marks are what is around it.
    fn inner(&self, node: &Node, span: &Range<usize>) -> Option<Range<usize>> {
        let children = node.children()?;
        let first = range(children.first()?)?;
        let last = range(children.last()?)?;
        (span.start <= first.start && first.start < last.end && last.end <= span.end)
            .then_some(first.start..last.end)
    }

    /// Hides `range`, a line at a time, on the lines no reveal touches.
    fn conceal(&mut self, range: Range<usize>) {
        if range.start >= range.end {
            return;
        }
        let (first, last) = (
            self.lines.line_of(range.start),
            self.lines.line_of(range.end - 1),
        );
        for line in first..=last {
            if self.lines.revealed[line] {
                continue;
            }
            let content = self.lines.content(line);
            let piece = range.start.max(content.start)..range.end.min(content.end);
            if piece.start < piece.end {
                self.layout.conceal.push(piece);
            }
        }
    }

    /// The wiki-links in text nodes: `[[Note]]` hides its brackets, and
    /// `[[Note|Alias]]` its target too. Both brackets must be in text of
    /// one parent, so a link isn't found across table cells or in code.
    fn wiki_links(&mut self) {
        if self.texts.is_empty() || !self.text.contains("[[") {
            return;
        }
        for link in kanso_core::links::wiki_links(self.text) {
            let Range { start, end } = link.range;
            if start > 0 && self.text.as_bytes()[start - 1] == b'\\' {
                continue;
            }
            let (Some(open), Some(close)) = (self.text_at(start), self.text_at(end - 2)) else {
                continue;
            };
            let (open, close) = (&self.texts[open], &self.texts[close]);
            if open.1 != close.1 || open.0.end < start + 2 || close.0.end < end {
                continue;
            }
            let inner = &self.text[start + 2..end - 2];
            let shown = match (link.alias.is_some(), inner.find('|')) {
                (true, Some(bar)) => start + 2 + bar + 1,
                _ => start + 2,
            };
            self.conceal(start..shown);
            self.conceal(end - 2..end);
            if shown < end - 2 {
                self.layout.styles.push((shown..end - 2, LiveStyle::Link));
            }
        }
    }

    /// The index of the text node `at` is in.
    fn text_at(&self, at: usize) -> Option<usize> {
        let ix = self
            .texts
            .partition_point(|(r, _)| r.start <= at)
            .checked_sub(1)?;
        (at < self.texts[ix].0.end).then_some(ix)
    }
}

/// A node's byte span.
fn range(node: &Node) -> Option<Range<usize>> {
    let position = node.position()?;
    (position.start.offset <= position.end.offset)
        .then_some(position.start.offset..position.end.offset)
}

fn on_boundaries(text: &str, range: &Range<usize>) -> bool {
    text.is_char_boundary(range.start) && text.is_char_boundary(range.end)
}

/// The conceal ranges sorted and merged where they overlap, with any that
/// would break the promises of [`LiveLayout::conceal`] left out.
fn tidy(text: &str, mut ranges: Vec<Range<usize>>) -> Vec<Range<usize>> {
    ranges.retain(|r| {
        r.start < r.end
            && r.end <= text.len()
            && on_boundaries(text, r)
            && !text[r.clone()].contains(['\n', '\r'])
    });
    ranges.sort_by_key(|r| r.start);
    let mut merged: Vec<Range<usize>> = Vec::with_capacity(ranges.len());
    for range in ranges {
        match merged.last_mut() {
            Some(last) if range.start <= last.end => last.end = last.end.max(range.end),
            _ => merged.push(range),
        }
    }
    merged
}

// The reveal ranges are carets and selections, often one of them.
#[cfg(test)]
#[allow(clippy::single_range_in_vec_init, clippy::reversed_empty_ranges)]
mod tests;
