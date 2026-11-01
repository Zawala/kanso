//! Live mode: a note's Markdown drawn in place, the way Typora does it.
//! What a mark marks is drawn (bold, italic, struck through, as code, as a
//! link, as a heading, as a list bullet, as a quote, as a rule), and the
//! mark itself is hidden until the caret comes to it:
//!
//! - An inline mark (`**`, `_`, `~~`, backticks, a link's brackets and URL,
//!   a wiki-link's brackets, inline math's `$`s) shows while a caret or a
//!   selection touches its element, from the element's first byte to just
//!   past its last.
//! - A block mark (a heading's `#`s or underline, a quote's `>`, a list
//!   bullet, number or task box, a rule, the indent of a block's lines)
//!   shows while a caret or a selection is on its line.
//! - A complex block (code, a table, display math, HTML, front matter, a
//!   paragraph of images) is drawn by the preview's renderer as a widget
//!   while no caret or selection is on any of its lines, and is its plain
//!   source while one is.
//!
//! [`live_layout`] says which bytes to hide, what to draw for them and which
//! spans to style; the editors draw it. Offsets are UTF-8 bytes into the
//! text; a phone converts them with [`crate::text`].
//!
//! Inline images, inline HTML, autolinks, bare URLs and escapes are left
//! as the highlighted source.

use std::ops::Range;

use markdown::mdast::Node;

/// What Live mode draws for a note.
///
/// An editor hides the bytes of [`LiveLayout::conceal`] and of every
/// marker's range (the two never overlap), draws each marker in its place,
/// and draws each widget in place of its lines.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LiveLayout {
    /// The inline marks and heading marks not drawn: sorted, disjoint,
    /// never empty, on character boundaries and never holding a line break.
    /// Never touching a reveal range, and never on a widget's lines.
    pub conceal: Vec<Range<usize>>,
    /// The spans drawn in a style, sorted by start, revealed or not. They
    /// nest (a strong span in a heading), so they can overlap.
    pub styles: Vec<(Range<usize>, LiveStyle)>,
    /// Each heading's line (0-based) and level, in order, revealed or not.
    /// A Setext heading is at its text's first line.
    pub headings: Vec<(usize, u8)>,
    /// More spans drawn in a style, sorted by start, revealed or not. They
    /// can overlap each other and the styles.
    pub spans: Vec<(Range<usize>, LiveSpan)>,
    /// The links and wiki-links, in order, revealed or not, with where they
    /// go: an editor can tell a wiki-link to a missing note.
    pub links: Vec<LiveLink>,
    /// Every line in exactly one block, in order: the leaf blocks (a list
    /// item's paragraph, a quote's heading) with what they are in, and the
    /// lines between them as [`BlockKind::Blank`].
    pub blocks: Vec<LiveBlock>,
    /// The block marks hidden and drawn as something else (or as nothing):
    /// sorted, disjoint, never empty, on character boundaries, each within
    /// one line. Only on lines no reveal range is on, never on a widget's.
    pub markers: Vec<LiveMarker>,
    /// The complex blocks drawn by the preview's renderer, in order, on
    /// lines of their own no reveal range is on.
    pub widgets: Vec<LiveWidget>,
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

/// More of how a span of a note is drawn in Live mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LiveSpan {
    /// A block quote, all of it, `depth` quotes deep (1 for `>`).
    Quote { depth: usize },
    /// The first block of a done task (`[x]`), which the preview draws
    /// muted.
    TaskDone,
    /// Inline math, without its `$`s.
    InlineMath,
}

/// A link in a note.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveLink {
    /// All of it, marks included.
    pub range: Range<usize>,
    /// The text drawn for it (styled [`LiveStyle::Link`]).
    pub text: Range<usize>,
    pub target: LinkTarget,
}

/// Where a link goes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LinkTarget {
    /// A URL as the link gives it (`[a](url)`, `<url>`, a bare URL).
    Url(String),
    /// A reference link's label (`[a][label]`, `[label]`), its definition
    /// elsewhere in the note.
    Reference(String),
    /// A wiki-link's note, and heading when it has one.
    Wiki {
        title: String,
        heading: Option<String>,
    },
}

/// A block of a note: lines `lines` (0-based, end exclusive).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveBlock {
    pub lines: Range<usize>,
    /// Its bytes, as the parser spans it (for a blank run, its lines).
    pub source: Range<usize>,
    pub kind: BlockKind,
    /// How many list items it is in (0 outside lists).
    pub list_depth: usize,
    /// How many quotes it is in (0 outside quotes).
    pub quote_depth: usize,
    /// Whether it is the first block of a list item, on the line with the
    /// item's marker.
    pub item_start: bool,
    /// Which of the note's top-level blocks (0-based) it is in; `None` for
    /// lines between them. Blocks of one top-level list or quote share it.
    pub top: Option<usize>,
    /// Whether a reveal range is on any of its lines.
    pub revealed: bool,
}

/// What a block is.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum BlockKind {
    Paragraph,
    Heading(u8),
    /// `---`, `***` or `___`.
    Rule,
    /// Fenced or indented code; a fence's language (its info string's first
    /// word), such as `mermaid`, `excalidraw` or `math`.
    Code {
        lang: Option<String>,
    },
    Table,
    /// Display math: `$$` on lines of their own, or a paragraph of `$$…$$`.
    Math,
    Html,
    FrontMatter,
    /// A paragraph of nothing but images.
    Image,
    /// A link reference definition (`[a]: url`), which the preview omits.
    Definition,
    /// Lines in no other block: blank, or holding only container marks
    /// such as a lone `>`.
    Blank,
}

impl BlockKind {
    /// Whether a block of this kind is drawn as a widget off the caret.
    pub fn is_widget(&self) -> bool {
        matches!(
            self,
            Self::Code { .. }
                | Self::Table
                | Self::Math
                | Self::Html
                | Self::FrontMatter
                | Self::Image
        )
    }
}

/// A block mark hidden and drawn as something else.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveMarker {
    /// The source hidden: `- `, `1. `, `- [ ] `, `> `, an indent, a rule.
    pub range: Range<usize>,
    /// Its line (0-based).
    pub line: usize,
    pub kind: MarkerKind,
}

/// What a hidden block mark is drawn as. `depth` is how many list items
/// (for list marks) or quotes (for quote marks) it is in, the mark's own
/// counted: a top-level bullet is depth 0, a top-level quote's `>` depth 1.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MarkerKind {
    /// A list bullet: `glyph` as the preview draws it at that depth.
    Bullet { depth: usize, glyph: &'static str },
    /// An ordered list's number as the preview draws it (`3.`, `B.`, `c.`).
    Number { depth: usize, text: String },
    /// A task's box (with its bullet or number), checked or not.
    Task { depth: usize, checked: bool },
    /// A list mark the preview draws nothing for: an item right under a
    /// task.
    Hidden { depth: usize },
    /// A quote's `>`, drawn as a bar.
    Quote { depth: usize },
    /// The indent before a block's text, drawn as nothing: the editor
    /// insets the line by its block's depths instead.
    Indent,
    /// A rule's text, drawn as a rule.
    Rule,
    /// A Setext heading's `===` or `---` line, drawn as nothing.
    Underline,
}

/// A complex block drawn by the preview's renderer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveWidget {
    /// Its lines (0-based, end exclusive), which hold nothing else.
    pub lines: Range<usize>,
    /// Its Markdown's bytes.
    pub source: Range<usize>,
    pub kind: BlockKind,
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

/// The preview's bullets, by depth (the last for any deeper).
const BULLETS: [&str; 5] = ["•", "◦", "▪", "‣", "⁃"];

/// An ordered list item's number as the preview draws it: `start + index`
/// at the top level (or when the list starts at 0), else a letter, upper
/// case one level down and lower case below.
fn ordinal_text(start: Option<u32>, index: usize, depth: usize) -> String {
    let ordinal = start
        .unwrap_or(1)
        .saturating_add(u32::try_from(index).unwrap_or(u32::MAX));
    if depth == 0 || start == Some(0) {
        return format!("{ordinal}.");
    }
    let letters = if depth == 1 {
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZ"
    } else {
        b"abcdefghijklmnopqrstuvwxyz"
    };
    let letter = letters[ordinal.saturating_sub(1) as usize % letters.len()];
    format!("{}.", letter as char)
}

/// What Live mode draws for `text` with carets and selections at `reveal`
/// (byte ranges; a caret is an empty one, and a range can run backwards).
/// A range is on every line from the one its start is on to the one its
/// end is on, so a caret at the end of a line is on it; it touches an
/// element when it meets the element's bytes or either end of them. Pure;
/// one parse of the text.
pub fn live_layout(text: &str, reveal: &[Range<usize>]) -> LiveLayout {
    let lines = Lines::new(text, reveal);
    let Ok(root) = kanso_render::markdown_safe::to_mdast(text, &parse_options()) else {
        // A panic or a parse failure draws the note as written: every line
        // in a block of its own, as the layout promises.
        return LiveLayout {
            blocks: (0..lines.starts.len())
                .map(|line| LiveBlock {
                    lines: line..line + 1,
                    source: lines.starts[line]..lines.content(line).end,
                    kind: BlockKind::Blank,
                    list_depth: 0,
                    quote_depth: 0,
                    item_start: false,
                    top: None,
                    revealed: lines.revealed[line],
                })
                .collect(),
            ..LiveLayout::default()
        };
    };
    let count = lines.starts.len();
    let mut walk = Walk {
        text,
        claimed: lines.starts.clone(),
        lines: &lines,
        layout: LiveLayout::default(),
        texts: Vec::new(),
        ids: 0,
        leaves: Vec::new(),
        item_start: false,
        widget_lines: vec![false; count],
        line_quote: vec![0; count],
        line_list: vec![0; count],
        line_top: vec![None; count],
    };
    let ctx = Ctx {
        parent: 0,
        list_depth: 0,
        quote_depth: 0,
        top: None,
        todo: false,
        block: true,
    };
    walk.ids += 1;
    for (ix, child) in root.children().into_iter().flatten().enumerate() {
        if let Some(span) = walk.span(child) {
            let (first, last) = walk.lines_of(&span);
            walk.line_top[first..=last].fill(Some(ix));
        }
        walk.node(
            child,
            Ctx {
                parent: 1,
                top: Some(ix),
                ..ctx
            },
        );
    }
    walk.wiki_links();
    walk.finish()
}

/// The constructs the preview reads: GFM, front matter and math.
fn parse_options() -> markdown::ParseOptions {
    let mut options = markdown::ParseOptions::gfm();
    options.constructs.frontmatter = true;
    options.constructs.math_text = true;
    options.constructs.math_flow = true;
    options
}

/// The text's lines, which of them the reveal ranges are on, and the
/// ranges.
struct Lines<'a> {
    text: &'a str,
    /// Where each line starts.
    starts: Vec<usize>,
    revealed: Vec<bool>,
    /// The reveal ranges, forwards and inside the text.
    reveal: Vec<(usize, usize)>,
}

impl<'a> Lines<'a> {
    fn new(text: &'a str, reveal: &[Range<usize>]) -> Self {
        let starts: Vec<usize> = std::iter::once(0)
            .chain(text.match_indices('\n').map(|(at, _)| at + 1))
            .collect();
        let reveal: Vec<(usize, usize)> = reveal
            .iter()
            .map(|r| {
                let (a, b) = (r.start.min(r.end), r.start.max(r.end));
                (a.min(text.len()), b.min(text.len()))
            })
            .collect();
        let mut lines = Self {
            text,
            revealed: vec![false; starts.len()],
            starts,
            reveal,
        };
        for &(a, b) in &lines.reveal {
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
        let end = if self.text.as_bytes()[start..end].ends_with(b"\r") {
            end - 1
        } else {
            end
        };
        start..end
    }

    /// Whether a reveal range touches `span`: meets it or either end.
    fn touches(&self, span: &Range<usize>) -> bool {
        self.reveal
            .iter()
            .any(|&(a, b)| a <= span.end && b >= span.start)
    }

    /// Whether a reveal range is on any of lines `first..=last`.
    fn any_revealed(&self, first: usize, last: usize) -> bool {
        self.revealed[first..=last].iter().any(|&r| r)
    }
}

/// Where the walk is.
#[derive(Clone, Copy)]
struct Ctx {
    /// The parent node's id.
    parent: usize,
    list_depth: usize,
    quote_depth: usize,
    top: Option<usize>,
    /// Whether the nearest list item around is a task: the preview draws
    /// no bullet for the items of a list right in a task.
    todo: bool,
    /// Whether the node is a block (its parent holds blocks).
    block: bool,
}

/// A leaf block found by the walk.
struct Leaf {
    source: Range<usize>,
    lines: Range<usize>,
    kind: BlockKind,
    list_depth: usize,
    quote_depth: usize,
    item_start: bool,
    top: Option<usize>,
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
    leaves: Vec<Leaf>,
    /// Whether the next leaf is a list item's first.
    item_start: bool,
    /// Per line, how far into it the block marks found so far reach.
    claimed: Vec<usize>,
    widget_lines: Vec<bool>,
    /// Per line, the quotes, list items and top-level block it is in.
    line_quote: Vec<usize>,
    line_list: Vec<usize>,
    line_top: Vec<Option<usize>>,
}

impl Walk<'_> {
    fn span(&self, node: &Node) -> Option<Range<usize>> {
        range(node).filter(|r| r.end <= self.text.len() && on_boundaries(self.text, r))
    }

    /// The first and last line of `span`.
    fn lines_of(&self, span: &Range<usize>) -> (usize, usize) {
        let first = self.lines.line_of(span.start);
        let last = self
            .lines
            .line_of(span.end.saturating_sub(1).max(span.start));
        (first, last)
    }

    fn node(&mut self, node: &Node, ctx: Ctx) {
        self.ids += 1;
        let id = self.ids;
        let Some(span) = self.span(node) else {
            return self.children(node, Ctx { parent: id, ..ctx });
        };
        let inline = Ctx {
            parent: id,
            block: false,
            ..ctx
        };
        match node {
            Node::Code(code) => {
                let lang = code
                    .lang
                    .as_deref()
                    .and_then(|l| l.split_whitespace().next())
                    .map(str::to_owned);
                self.leaf(span, BlockKind::Code { lang }, ctx);
            }
            Node::Math(_) => self.leaf(span, BlockKind::Math, ctx),
            Node::Yaml(_) | Node::Toml(_) => self.leaf(span, BlockKind::FrontMatter, ctx),
            Node::Table(_) => self.leaf(span, BlockKind::Table, ctx),
            Node::Definition(_) => self.leaf(span, BlockKind::Definition, ctx),
            Node::Html(_) if ctx.block => self.leaf(span, BlockKind::Html, ctx),
            Node::ThematicBreak(_) => {
                let line = self.lines.line_of(span.start);
                let from = self.claimed[line].max(span.start.min(self.lines.content(line).end));
                self.marker(from..self.lines.content(line).end, MarkerKind::Rule);
                self.leaf(span, BlockKind::Rule, ctx);
            }
            Node::Paragraph(_) => {
                if kanso_render::html::display_math_paragraph(node, &self.text[span.clone()])
                    .is_some()
                {
                    self.leaf(span, BlockKind::Math, ctx);
                } else if images_only(node) {
                    self.leaf(span, BlockKind::Image, ctx);
                } else {
                    self.leaf(span.clone(), BlockKind::Paragraph, ctx);
                    self.indents(&span);
                    self.children(node, inline);
                }
            }
            Node::Heading(heading) => {
                let level = heading.depth.clamp(1, 6);
                let (first, last) = self.lines_of(&span);
                self.layout.headings.push((first, level));
                self.leaf(span.clone(), BlockKind::Heading(level), ctx);
                self.indents(&span);
                let atx = self.text[span.clone()].starts_with('#');
                if let Some(inner) = self.inner(node, &span) {
                    if atx {
                        self.conceal_block(span.start..inner.start);
                        self.conceal_block(inner.end..span.end);
                    }
                    self.layout.styles.push((inner, LiveStyle::Heading(level)));
                }
                if !atx && last > first && !self.lines.any_revealed(first, last) {
                    let content = self.lines.content(last);
                    self.marker(self.claimed[last]..content.end, MarkerKind::Underline);
                }
                self.children(node, inline);
            }
            Node::Blockquote(_) => {
                let depth = ctx.quote_depth + 1;
                let (first, last) = self.lines_of(&span);
                for line in first..=last {
                    self.line_quote[line] = self.line_quote[line].max(depth);
                    self.quote_mark(line, if line == first { span.start } else { 0 }, depth);
                }
                self.layout.spans.push((span, LiveSpan::Quote { depth }));
                self.children(
                    node,
                    Ctx {
                        parent: id,
                        quote_depth: depth,
                        block: true,
                        ..ctx
                    },
                );
            }
            Node::List(list) => {
                let mut index = 0;
                for child in &list.children {
                    if let Node::ListItem(item) = child {
                        self.ids += 1;
                        let item_id = self.ids;
                        self.list_item(child, item.checked, list.start, list.ordered, index, ctx);
                        self.children(
                            child,
                            Ctx {
                                parent: item_id,
                                list_depth: ctx.list_depth + 1,
                                todo: item.checked.is_some(),
                                block: true,
                                ..ctx
                            },
                        );
                        index += 1;
                    } else {
                        self.node(child, ctx);
                    }
                }
            }
            Node::Root(_) | Node::FootnoteDefinition(_) | Node::ListItem(_) => self.children(
                node,
                Ctx {
                    parent: id,
                    block: true,
                    ..ctx
                },
            ),
            // Left as the source: nothing in them is hidden.
            Node::Html(_)
            | Node::Image(_)
            | Node::ImageReference(_)
            | Node::FootnoteReference(_) => {}
            Node::Text(_) => self.texts.push((span, ctx.parent)),
            Node::Strong(_) | Node::Emphasis(_) | Node::Delete(_) => {
                let style = match node {
                    Node::Strong(_) => LiveStyle::Strong,
                    Node::Emphasis(_) => LiveStyle::Emphasis,
                    _ => LiveStyle::Strikethrough,
                };
                if let Some(inner) = self.inner(node, &span) {
                    self.conceal_inline(&span, &inner);
                    self.layout.styles.push((inner, style));
                }
                self.children(node, inline);
            }
            Node::InlineCode(_) | Node::InlineMath(_) => {
                let mark = if matches!(node, Node::InlineCode(_)) {
                    b'`'
                } else {
                    b'$'
                };
                let source = &self.text.as_bytes()[span.clone()];
                let open = source.iter().take_while(|&&b| b == mark).count();
                let close = source.iter().rev().take_while(|&&b| b == mark).count();
                if open > 0 && open == close && 2 * open < source.len() {
                    let inner = span.start + open..span.end - close;
                    self.conceal_inline(&span, &inner);
                    if mark == b'`' {
                        self.layout.styles.push((inner, LiveStyle::Code));
                    } else {
                        self.layout.spans.push((inner, LiveSpan::InlineMath));
                    }
                }
            }
            Node::Link(_) | Node::LinkReference(_) => {
                if let Some(inner) = self.inner(node, &span) {
                    // An autolink or a bare URL keeps its marks.
                    if self.text[span.clone()].starts_with('[') {
                        self.conceal_inline(&span, &inner);
                    }
                    let target = match node {
                        Node::Link(link) => LinkTarget::Url(link.url.clone()),
                        Node::LinkReference(link) => LinkTarget::Reference(
                            link.label
                                .clone()
                                .unwrap_or_else(|| link.identifier.clone()),
                        ),
                        _ => unreachable!(),
                    };
                    self.layout.links.push(LiveLink {
                        range: span.clone(),
                        text: inner.clone(),
                        target,
                    });
                    self.layout.styles.push((inner, LiveStyle::Link));
                }
                self.children(node, inline);
            }
            _ => self.children(node, inline),
        }
    }

    fn children(&mut self, node: &Node, ctx: Ctx) {
        for child in node.children().into_iter().flatten() {
            self.node(child, ctx);
        }
    }

    /// A list item's mark: from the indent before its bullet or number to
    /// its text, its task box included.
    fn list_item(
        &mut self,
        item: &Node,
        checked: Option<bool>,
        start: Option<u32>,
        ordered: bool,
        index: usize,
        ctx: Ctx,
    ) {
        let Some(span) = self.span(item) else {
            return;
        };
        let (first, last) = self.lines_of(&span);
        for line in first..=last {
            self.line_list[line] = self.line_list[line].max(ctx.list_depth + 1);
        }
        self.item_start = true;
        if item.children().is_none_or(|c| c.is_empty()) {
            // An empty item still has a line, with its mark.
            self.leaf(span.clone(), BlockKind::Paragraph, ctx);
        }
        let bytes = self.text.as_bytes();
        let content = self.lines.content(first);
        let from = self.claimed[first].max(content.start);
        let mut at = skip_space(bytes, span.start.max(from).min(content.end), content.end);
        let bullet = at;
        if at < content.end && matches!(bytes[at], b'-' | b'*' | b'+') {
            at += 1;
        } else {
            let digits = bytes[at..content.end]
                .iter()
                .take_while(|b| b.is_ascii_digit())
                .count();
            if digits == 0
                || at + digits >= content.end
                || !matches!(bytes[at + digits], b'.' | b')')
            {
                return;
            }
            at += digits + 1;
        }
        let spaces = skip_space(bytes, at, content.end);
        at = if spaces - at > 4 { at + 1 } else { spaces };
        let mut task = None;
        if let Some(done) = checked {
            let boxed = bytes
                .get(at..at + 3)
                .filter(|b| b[0] == b'[' && b[2] == b']' && matches!(b[1], b' ' | b'x' | b'X'));
            if boxed.is_some() && at + 3 <= content.end {
                at = skip_space(bytes, at + 3, content.end);
                task = Some(done);
            }
        }
        // The indent before the mark goes with it, back to the marks
        // before it on the line.
        let mut begin = bullet;
        while begin > from && matches!(bytes[begin - 1], b' ' | b'\t') {
            begin -= 1;
        }
        let depth = ctx.list_depth;
        let kind = match (task.or(checked), ordered) {
            (Some(checked), _) => MarkerKind::Task { depth, checked },
            (None, _) if ctx.todo => MarkerKind::Hidden { depth },
            (None, true) => MarkerKind::Number {
                depth,
                text: ordinal_text(start, index, depth),
            },
            (None, false) => MarkerKind::Bullet {
                depth,
                glyph: BULLETS[depth.min(BULLETS.len() - 1)],
            },
        };
        let first_child = item
            .children()
            .and_then(|c| c.first())
            .and_then(|c| self.span(c));
        if let (Some(true), Some(first_child)) = (checked, first_child) {
            let from = first_child.start.max(at.min(first_child.end));
            if from < first_child.end {
                self.layout
                    .spans
                    .push((from..first_child.end, LiveSpan::TaskDone));
            }
        }
        self.marker(begin..at, kind);
    }

    /// A quote's `>` on `line`, at or after `after`, past the marks before
    /// it on the line.
    fn quote_mark(&mut self, line: usize, after: usize, depth: usize) {
        let bytes = self.text.as_bytes();
        let content = self.lines.content(line);
        let from = self.claimed[line].max(content.start);
        let at = skip_space(bytes, from, content.end);
        if at < content.end && bytes[at] == b'>' && at >= after.min(content.end) {
            let mut end = at + 1;
            if end < content.end && matches!(bytes[end], b' ' | b'\t') {
                end += 1;
            }
            self.marker(from..end, MarkerKind::Quote { depth });
        }
    }

    /// The indent left on each line of a block's `span`, past the marks
    /// before it.
    fn indents(&mut self, span: &Range<usize>) {
        let (first, last) = self.lines_of(span);
        for line in first..=last {
            let content = self.lines.content(line);
            let from = self.claimed[line].max(content.start);
            let to = skip_space(self.text.as_bytes(), from, content.end);
            self.marker(from..to, MarkerKind::Indent);
        }
    }

    /// Notes a block mark, claiming its line up to its end.
    fn marker(&mut self, range: Range<usize>, kind: MarkerKind) {
        if range.start >= range.end {
            return;
        }
        let line = self.lines.line_of(range.start);
        self.claimed[line] = self.claimed[line].max(range.end);
        self.layout.markers.push(LiveMarker { range, line, kind });
    }

    /// Notes a leaf block, and a widget when it is one off the reveal.
    fn leaf(&mut self, source: Range<usize>, kind: BlockKind, ctx: Ctx) {
        let (first, last) = self.lines_of(&source);
        let item_start = std::mem::take(&mut self.item_start);
        if kind.is_widget() && !self.lines.any_revealed(first, last) && self.alone(&source) {
            self.widget_lines[first..=last].fill(true);
            self.layout.widgets.push(LiveWidget {
                lines: first..last + 1,
                source: source.clone(),
                kind: kind.clone(),
            });
        }
        self.leaves.push(Leaf {
            source,
            lines: first..last + 1,
            kind,
            list_depth: ctx.list_depth,
            quote_depth: ctx.quote_depth,
            item_start,
            top: ctx.top,
        });
    }

    /// Whether `span` has its lines to itself: nothing but quote marks and
    /// spaces before it on its first, nothing but spaces after it on its
    /// last.
    fn alone(&self, span: &Range<usize>) -> bool {
        let (first, last) = self.lines_of(span);
        let bytes = self.text.as_bytes();
        let before = &bytes[self.lines.starts[first]..span.start];
        let after =
            &bytes[span.end.min(self.lines.content(last).end)..self.lines.content(last).end];
        before.iter().all(|b| matches!(b, b' ' | b'\t' | b'>'))
            && after.iter().all(|b| matches!(b, b' ' | b'\t'))
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

    /// Hides an inline element's marks (what of `span` is outside
    /// `inner`) unless a reveal touches the element.
    fn conceal_inline(&mut self, span: &Range<usize>, inner: &Range<usize>) {
        if self.lines.touches(span) {
            return;
        }
        self.conceal(span.start..inner.start, false);
        self.conceal(inner.end..span.end, false);
    }

    /// Hides a heading's marks on the lines no reveal is on.
    fn conceal_block(&mut self, range: Range<usize>) {
        self.conceal(range, true);
    }

    /// Hides `range`, a line at a time (on unrevealed lines only, if
    /// `by_line`).
    fn conceal(&mut self, range: Range<usize>, by_line: bool) {
        if range.start >= range.end {
            return;
        }
        let (first, last) = (
            self.lines.line_of(range.start),
            self.lines.line_of(range.end - 1),
        );
        for line in first..=last {
            if by_line && self.lines.revealed[line] {
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
            if shown < end - 2 {
                self.conceal_inline(&(start..end), &(shown..end - 2));
                self.layout.styles.push((shown..end - 2, LiveStyle::Link));
                self.layout.links.push(LiveLink {
                    range: start..end,
                    text: shown..end - 2,
                    target: LinkTarget::Wiki {
                        title: link.title,
                        heading: link.heading,
                    },
                });
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

    /// The layout, tidied to keep its promises.
    fn finish(self) -> LiveLayout {
        let Walk {
            text,
            lines,
            mut layout,
            leaves,
            widget_lines,
            line_quote,
            line_list,
            line_top,
            ..
        } = self;
        let off_widgets = |r: &Range<usize>| !widget_lines[lines.line_of(r.start)];

        let mut markers = std::mem::take(&mut layout.markers);
        markers.retain(|m| {
            let r = &m.range;
            r.start < r.end
                && r.end <= text.len()
                && on_boundaries(text, r)
                && !text[r.clone()].contains(['\n', '\r'])
                && !lines.revealed[m.line]
                && off_widgets(r)
        });
        markers.sort_by_key(|m| m.range.start);
        let mut kept: Vec<LiveMarker> = Vec::with_capacity(markers.len());
        for marker in markers {
            if kept
                .last()
                .is_none_or(|last| last.range.end <= marker.range.start)
            {
                kept.push(marker);
            }
        }
        layout.markers = kept;

        let mut conceal = tidy(text, std::mem::take(&mut layout.conceal));
        conceal.retain(off_widgets);
        layout.conceal = subtract(conceal, &layout.markers);

        layout
            .styles
            .retain(|(r, _)| r.start < r.end && r.end <= text.len() && on_boundaries(text, r));
        layout.styles.sort_by_key(|(r, _)| (r.start, r.end));
        layout
            .spans
            .retain(|(r, _)| r.start < r.end && r.end <= text.len() && on_boundaries(text, r));
        layout.spans.sort_by_key(|(r, _)| (r.start, r.end));
        layout.links.sort_by_key(|l| (l.range.start, l.range.end));
        layout.headings.sort_unstable();
        layout.headings.dedup_by_key(|(line, _)| *line);
        layout.widgets.sort_by_key(|w| w.lines.start);
        let mut end = 0;
        layout.widgets.retain(|w| {
            let keep = w.lines.start >= end;
            if keep {
                end = w.lines.end;
            }
            keep
        });

        // The leaves in order, one per line at most, and the lines between
        // them as blank runs.
        let mut leaves = leaves;
        leaves.sort_by_key(|l| (l.lines.start, l.source.start));
        let count = lines.starts.len();
        let mut blocks: Vec<LiveBlock> = Vec::with_capacity(leaves.len() * 2 + 1);
        let mut next = 0;
        let blank = |blocks: &mut Vec<LiveBlock>, from: usize, to: usize| {
            let mut at = from;
            while at < to {
                let top = line_top[at];
                let mut end = at + 1;
                while end < to && line_top[end] == top {
                    end += 1;
                }
                blocks.push(LiveBlock {
                    lines: at..end,
                    source: lines.starts[at]..lines.content(end - 1).end,
                    kind: BlockKind::Blank,
                    list_depth: line_list[at],
                    quote_depth: line_quote[at],
                    item_start: false,
                    top,
                    revealed: lines.any_revealed(at, end - 1),
                });
                at = end;
            }
        };
        for leaf in leaves {
            if leaf.lines.start < next {
                continue;
            }
            blank(&mut blocks, next, leaf.lines.start);
            next = leaf.lines.end;
            blocks.push(LiveBlock {
                revealed: lines.any_revealed(leaf.lines.start, leaf.lines.end - 1),
                lines: leaf.lines,
                source: leaf.source,
                kind: leaf.kind,
                list_depth: leaf.list_depth,
                quote_depth: leaf.quote_depth,
                item_start: leaf.item_start,
                top: leaf.top,
            });
        }
        blank(&mut blocks, next, count);
        layout.blocks = blocks;
        layout
    }
}

/// Whether a paragraph holds images and nothing else but white space.
fn images_only(node: &Node) -> bool {
    let Some(children) = node.children() else {
        return false;
    };
    let mut any = false;
    for child in children {
        match child {
            Node::Image(_) | Node::ImageReference(_) => any = true,
            Node::Break(_) => {}
            Node::Text(text) if text.value.trim().is_empty() => {}
            _ => return false,
        }
    }
    any
}

/// Past the spaces and tabs from `at`, up to `end`.
fn skip_space(bytes: &[u8], mut at: usize, end: usize) -> usize {
    while at < end && matches!(bytes[at], b' ' | b'\t') {
        at += 1;
    }
    at
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

/// `ranges` (sorted, disjoint) without the markers' bytes (sorted,
/// disjoint). Marks start and end on ASCII bytes, so the pieces stay on
/// character boundaries.
fn subtract(ranges: Vec<Range<usize>>, markers: &[LiveMarker]) -> Vec<Range<usize>> {
    let mut out = Vec::with_capacity(ranges.len());
    for range in ranges {
        let mut at = range.start;
        let first = markers.partition_point(|m| m.range.end <= range.start);
        for marker in &markers[first..] {
            if marker.range.start >= range.end {
                break;
            }
            if marker.range.start > at {
                out.push(at..marker.range.start);
            }
            at = at.max(marker.range.end);
        }
        if at < range.end {
            out.push(at..range.end);
        }
    }
    out
}

// The reveal ranges are carets and selections, often one of them.
#[cfg(test)]
#[allow(clippy::single_range_in_vec_init, clippy::reversed_empty_ranges)]
mod tests;
