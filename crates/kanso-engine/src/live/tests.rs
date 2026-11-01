use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;

use super::*;

/// The text without its concealed bytes and markers' bytes: what is left
/// of the source.
fn shown(text: &str, reveal: &[Range<usize>]) -> String {
    draw(text, reveal, false)
}

/// The text as Live mode draws it: concealed bytes gone, each marker drawn
/// as a symbol (`•`, `◦`, `1.`, `☐`, `☑`, `┃`, `──`) and each widget's
/// lines as `[kind]`.
fn drawn(text: &str, reveal: &[Range<usize>]) -> String {
    draw(text, reveal, true)
}

fn draw(text: &str, reveal: &[Range<usize>], symbols: bool) -> String {
    let layout = live_layout(text, reveal);
    let mut hidden: Vec<(Range<usize>, String)> = layout
        .conceal
        .iter()
        .map(|r| (r.clone(), String::new()))
        .collect();
    for m in &layout.markers {
        let symbol = match &m.kind {
            MarkerKind::Bullet { glyph, .. } => format!("{glyph} "),
            MarkerKind::Number { text, .. } => format!("{text} "),
            MarkerKind::Task { checked: false, .. } => "☐ ".into(),
            MarkerKind::Task { checked: true, .. } => "☑ ".into(),
            MarkerKind::Quote { .. } => "┃".into(),
            MarkerKind::Rule => "──".into(),
            MarkerKind::Hidden { .. } | MarkerKind::Indent | MarkerKind::Underline => String::new(),
        };
        hidden.push((
            m.range.clone(),
            if symbols { symbol } else { String::new() },
        ));
    }
    if symbols {
        let starts: Vec<usize> = std::iter::once(0)
            .chain(text.match_indices('\n').map(|(at, _)| at + 1))
            .collect();
        for w in &layout.widgets {
            let end = starts.get(w.lines.end).map_or(text.len(), |next| next - 1);
            let name = match &w.kind {
                BlockKind::Code { lang: Some(lang) } => format!("[code {lang}]"),
                BlockKind::Code { lang: None } => "[code]".into(),
                kind => format!("[{kind:?}]").to_lowercase(),
            };
            hidden.push((starts[w.lines.start]..end, name));
        }
    }
    hidden.sort_by_key(|(r, _)| r.start);
    let mut out = String::new();
    let mut at = 0;
    for (range, with) in hidden {
        assert!(range.start >= at, "overlap at {range:?}");
        out.push_str(&text[at..range.start]);
        out.push_str(&with);
        at = range.end;
    }
    out.push_str(&text[at..]);
    out
}

/// The text of each span styled `style`.
fn styled(text: &str, style: LiveStyle) -> Vec<&str> {
    live_layout(text, &[])
        .styles
        .into_iter()
        .filter(|(_, s)| *s == style)
        .map(|(r, _)| &text[r])
        .collect()
}

/// The text of each span drawn `span`.
fn spanned(text: &str, span: LiveSpan) -> Vec<&str> {
    live_layout(text, &[])
        .spans
        .into_iter()
        .filter(|(_, s)| *s == span)
        .map(|(r, _)| &text[r])
        .collect()
}

/// Each block's kind and lines.
fn blocks(text: &str) -> Vec<(BlockKind, Range<usize>)> {
    live_layout(text, &[])
        .blocks
        .into_iter()
        .map(|b| (b.kind, b.lines))
        .collect()
}

#[test]
fn atx_headings_hide_their_marks_and_are_styled_and_listed() {
    let text = "# One\n\npara\n### Three ###\n> ## Quoted";
    assert_eq!(shown(text, &[]), "One\n\npara\nThree\nQuoted");
    assert_eq!(styled(text, LiveStyle::Heading(1)), ["One"]);
    assert_eq!(styled(text, LiveStyle::Heading(3)), ["Three"]);
    assert_eq!(live_layout(text, &[]).headings, [(0, 1), (3, 3), (4, 2)]);
}

#[test]
fn setext_headings_hide_their_underline() {
    let text = "Title\n=====\n\nSub\n---\n";
    let layout = live_layout(text, &[]);
    assert_eq!(layout.headings, [(0, 1), (3, 2)]);
    assert!(layout.conceal.is_empty());
    assert_eq!(shown(text, &[]), "Title\n\n\nSub\n\n");
    assert_eq!(styled(text, LiveStyle::Heading(1)), ["Title"]);
    // A caret on the text shows the underline too.
    assert_eq!(shown(text, &[0..0]), "Title\n=====\n\nSub\n\n");
    assert_eq!(
        blocks(text),
        [
            (BlockKind::Heading(1), 0..2),
            (BlockKind::Blank, 2..3),
            (BlockKind::Heading(2), 3..5),
            (BlockKind::Blank, 5..6),
        ]
    );
}

#[test]
fn an_empty_heading_hides_nothing() {
    let layout = live_layout("##\n", &[]);
    assert_eq!(layout.headings, [(0, 2)]);
    assert!(layout.conceal.is_empty());
}

#[test]
fn inline_marks_are_hidden_and_their_text_styled() {
    let text = "a **b** _c_ *d* __e__ ~~f~~ `g` ``h`i``";
    assert_eq!(shown(text, &[]), "a b c d e f g h`i");
    assert_eq!(styled(text, LiveStyle::Strong), ["b", "e"]);
    assert_eq!(styled(text, LiveStyle::Emphasis), ["c", "d"]);
    assert_eq!(styled(text, LiveStyle::Strikethrough), ["f"]);
    assert_eq!(styled(text, LiveStyle::Code), ["g", "h`i"]);
}

#[test]
fn nested_marks_are_all_hidden() {
    let text = "***both*** **a _b_ c**";
    assert_eq!(shown(text, &[]), "both a b c");
    assert_eq!(styled(text, LiveStyle::Emphasis).len(), 2);
}

#[test]
fn inline_marks_show_only_while_a_caret_touches_their_element() {
    let text = "x **bold** and _it_ y";
    // On the line, but outside both.
    assert_eq!(shown(text, &[0..0]), "x bold and it y");
    // At either end of the strong element, and inside it.
    assert_eq!(shown(text, &[2..2]), "x **bold** and it y");
    assert_eq!(shown(text, &[10..10]), "x **bold** and it y");
    assert_eq!(shown(text, &[5..5]), "x **bold** and it y");
    // Just past it.
    assert_eq!(shown(text, &[11..11]), "x bold and it y");
    // A selection over both.
    assert_eq!(shown(text, &[4..16]), text);
    // Backwards too.
    assert_eq!(shown(text, &[16..4]), text);
}

#[test]
fn a_caret_in_an_inner_element_shows_the_outer_marks_too() {
    let text = "**a _b_ c**";
    assert_eq!(shown(text, &[5..5]), text);
    // In the outer one only.
    assert_eq!(shown(text, &[2..2]), "**a b c**");
}

#[test]
fn inline_math_hides_its_dollars() {
    let text = "so $a*b*c$ and $$x$$ here";
    assert_eq!(shown(text, &[]), "so a*b*c and x here");
    assert_eq!(spanned(text, LiveSpan::InlineMath), ["a*b*c", "x"]);
    assert_eq!(shown(text, &[4..4]), "so $a*b*c$ and x here");
}

#[test]
fn links_hide_brackets_and_destination() {
    let text = "see [the **plan**](https://x.y \"Title\") and [ref][r] [c][] [s]\n\n[r]: https://r\n[c]: https://c\n[s]: https://s";
    assert_eq!(
        shown(text, &[]).lines().next(),
        Some("see the plan and ref c s")
    );
    assert_eq!(
        styled(text, LiveStyle::Link),
        ["the **plan**", "ref", "c", "s"]
    );
    let links = live_layout(text, &[]).links;
    assert_eq!(links[0].target, LinkTarget::Url("https://x.y".into()));
    assert_eq!(
        &text[links[0].range.clone()],
        "[the **plan**](https://x.y \"Title\")"
    );
    assert_eq!(links[1].target, LinkTarget::Reference("r".into()));
    // The definitions are blocks of their own.
    assert_eq!(blocks(text)[2], (BlockKind::Definition, 2..3));
}

#[test]
fn autolinks_bare_urls_and_escapes_keep_their_marks() {
    let text = "<https://a.b> https://c.d www.e.f \\*no\\* \\[x](y)";
    assert_eq!(shown(text, &[]), text);
    let links = live_layout(text, &[]).links;
    assert_eq!(links[0].target, LinkTarget::Url("https://a.b".into()));
}

#[test]
fn wiki_links_hide_brackets_and_an_alias_hides_the_target() {
    let text = "see [[Note]] and [[Other#H|alias]] and `[[code]]` ![[embed]]";
    assert_eq!(
        shown(text, &[]),
        "see Note and alias and [[code]] ![[embed]]"
    );
    assert_eq!(styled(text, LiveStyle::Link), ["Note", "alias"]);
    let links = live_layout(text, &[]).links;
    assert_eq!(
        links[1].target,
        LinkTarget::Wiki {
            title: "Other".into(),
            heading: Some("H".into())
        }
    );
    // A caret at a wiki-link's end shows it whole.
    assert_eq!(
        shown(text, &[12..12]),
        "see [[Note]] and alias and [[code]] ![[embed]]"
    );
}

#[test]
fn complex_blocks_are_widgets_off_the_caret() {
    for (text, kind) in [
        (
            "```rust\n# not **bold**\n```",
            BlockKind::Code {
                lang: Some("rust".into()),
            },
        ),
        ("    **indented code**", BlockKind::Code { lang: None }),
        (
            "```mermaid\ngraph TD\n```",
            BlockKind::Code {
                lang: Some("mermaid".into()),
            },
        ),
        ("---\ntitle: **x**\n---", BlockKind::FrontMatter),
        ("<div>**x**</div>", BlockKind::Html),
        ("![**alt**](img.png)", BlockKind::Image),
        ("![a](a.png) ![b](b.png)", BlockKind::Image),
        ("$$\n**x**\n$$", BlockKind::Math),
        ("$$x = 1$$", BlockKind::Math),
        (
            "| **a** | `b` |\n|---|---|\n| [c](d) | e |",
            BlockKind::Table,
        ),
    ] {
        let layout = live_layout(text, &[]);
        let lines = text.lines().count();
        assert_eq!(
            layout.widgets,
            [LiveWidget {
                lines: 0..lines,
                source: 0..text.len(),
                kind: kind.clone(),
            }],
            "{text}"
        );
        assert!(
            layout.conceal.is_empty() && layout.markers.is_empty(),
            "{text}"
        );
        assert!(layout.headings.is_empty(), "{text}");
        assert_eq!(blocks(text), [(kind, 0..lines)], "{text}");
        // Entered, they are their plain source.
        let inside = live_layout(text, &[text.len()..text.len()]);
        assert!(inside.widgets.is_empty(), "{text}");
        assert_eq!(shown(text, &[text.len()..text.len()]), text, "{text}");
    }
}

#[test]
fn a_widget_shows_its_source_while_any_of_its_lines_is_revealed() {
    let text = "para\n\n```\na\nb\n```\n\nafter";
    assert_eq!(drawn(text, &[]), "para\n\n[code]\n\nafter");
    assert_eq!(drawn(text, &[11..11]), text);
    // A caret on the next line leaves it a widget.
    assert_eq!(drawn(text, &[20..20]), "para\n\n[code]\n\nafter");
}

#[test]
fn a_block_sharing_its_line_with_a_list_mark_is_no_widget() {
    let text = "- ![a](b.png)\n- ```\n  x\n  ```";
    assert!(live_layout(text, &[]).widgets.is_empty());
    let text = "> ```\n> x\n> ```";
    assert_eq!(drawn(text, &[]), "[code]");
}

#[test]
fn inline_images_and_html_are_left_alone() {
    let text = "a ![**alt**](img.png) <b>x</b>";
    let layout = live_layout(text, &[]);
    assert!(layout.widgets.is_empty());
    assert_eq!(shown(text, &[]), text);
}

#[test]
fn bullets_are_drawn_as_the_preview_draws_them_at_each_depth() {
    let text = "- a\n  * b\n    + c\n      - d\n        - e\n          - f";
    assert_eq!(drawn(text, &[]), "• a\n◦ b\n▪ c\n‣ d\n⁃ e\n⁃ f");
    let depths: Vec<usize> = live_layout(text, &[])
        .blocks
        .iter()
        .map(|b| b.list_depth)
        .collect();
    assert_eq!(depths, [1, 2, 3, 4, 5, 6]);
}

#[test]
fn ordered_items_are_numbered_as_the_preview_numbers_them() {
    let text = "3. a\n4. b\n9. c\n   1. d\n   1. e\n      1) f\n      1) g";
    assert_eq!(drawn(text, &[]), "3. a\n4. b\n5. c\nA. d\nB. e\na. f\nb. g");
    // A list starting at zero stays numeric at any depth.
    assert_eq!(drawn("- x\n\n  0. y\n  0. z", &[]), "• x\n\n0. y\n1. z");
}

#[test]
fn tasks_are_drawn_as_boxes_and_done_ones_are_muted() {
    let text = "- [ ] open\n- [x] done\n- [X] also\n1. [ ] numbered";
    assert_eq!(drawn(text, &[]), "☐ open\n☑ done\n☑ also\n☐ numbered");
    assert_eq!(spanned(text, LiveSpan::TaskDone), ["done", "also"]);
    let checked: Vec<bool> = live_layout(text, &[])
        .markers
        .iter()
        .map(|m| matches!(m.kind, MarkerKind::Task { checked: true, .. }))
        .collect();
    assert_eq!(checked, [false, true, true, false]);
}

#[test]
fn items_right_under_a_task_have_no_bullet() {
    let text = "- [ ] task\n  - sub\n    - deeper";
    assert_eq!(drawn(text, &[]), "☐ task\nsub\n▪ deeper");
}

#[test]
fn a_list_mark_shows_while_a_caret_is_on_its_line() {
    let text = "- one\n- two\n  more";
    assert_eq!(drawn(text, &[]), "• one\n• two\nmore");
    assert_eq!(drawn(text, &[6..6]), "• one\n- two\nmore");
    // The continuation line's indent is its own.
    assert_eq!(drawn(text, &[14..14]), "• one\n• two\n  more");
}

#[test]
fn empty_items_are_blocks_with_their_mark() {
    let text = "- \n-";
    assert_eq!(drawn(text, &[]), "• \n• ");
    assert!(live_layout(text, &[]).blocks.iter().all(|b| b.item_start));
}

#[test]
fn quotes_hide_their_marks_and_are_spanned() {
    let text = "> a\n>\n> > b\n> - c\nlazy";
    assert_eq!(drawn(text, &[]), "┃a\n┃\n┃┃b\n┃• c\nlazy");
    assert_eq!(spanned(text, LiveSpan::Quote { depth: 1 }), [text]);
    assert_eq!(spanned(text, LiveSpan::Quote { depth: 2 }), ["> b"]);
    // Revealed per line.
    assert_eq!(drawn(text, &[4..4]), "┃a\n>\n┃┃b\n┃• c\nlazy");
    let layout = live_layout(text, &[]);
    let quotes: Vec<usize> = layout.blocks.iter().map(|b| b.quote_depth).collect();
    // The lazy line is the item's paragraph's.
    assert_eq!(quotes, [1, 1, 2, 1]);
    assert_eq!(layout.blocks[1].kind, BlockKind::Blank);
}

#[test]
fn a_quote_in_a_list_item_and_a_list_in_a_quote() {
    assert_eq!(drawn("- > q\n  > r", &[]), "• ┃q\n┃r");
    assert_eq!(drawn("> - a\n>   b", &[]), "┃• a\n┃b");
}

#[test]
fn rules_are_drawn_off_the_caret() {
    let text = "a\n\n---\n\n***\n> ___";
    assert_eq!(drawn(text, &[]), "a\n\n──\n\n──\n┃──");
    assert_eq!(drawn(text, &[4..4]), "a\n\n---\n\n──\n┃──");
    assert_eq!(
        blocks(text),
        [
            (BlockKind::Paragraph, 0..1),
            (BlockKind::Blank, 1..2),
            (BlockKind::Rule, 2..3),
            (BlockKind::Blank, 3..4),
            (BlockKind::Rule, 4..5),
            (BlockKind::Rule, 5..6),
        ]
    );
}

#[test]
fn blocks_say_what_they_are_in() {
    let text = "# T\n\npara\nmore\n\n- a\n\n  b\n- c\n\n> q";
    let layout = live_layout(text, &[7..7]);
    let summary: Vec<_> = layout
        .blocks
        .iter()
        .map(|b| {
            (
                b.kind.clone(),
                b.lines.clone(),
                b.list_depth,
                b.item_start,
                b.top,
                b.revealed,
            )
        })
        .collect();
    use BlockKind::*;
    assert_eq!(
        summary,
        [
            (Heading(1), 0..1, 0, false, Some(0), false),
            (Blank, 1..2, 0, false, None, false),
            (Paragraph, 2..4, 0, false, Some(1), true),
            (Blank, 4..5, 0, false, None, false),
            (Paragraph, 5..6, 1, true, Some(2), false),
            (Blank, 6..7, 1, false, Some(2), false),
            (Paragraph, 7..8, 1, false, Some(2), false),
            (Paragraph, 8..9, 1, true, Some(2), false),
            (Blank, 9..10, 0, false, None, false),
            (Paragraph, 10..11, 0, false, Some(3), false),
        ]
    );
    assert_eq!(&text[layout.blocks[2].source.clone()], "para\nmore");
}

#[test]
fn marks_on_different_lines_show_together() {
    let text = "*one\ntwo*";
    assert_eq!(shown(text, &[]), "one\ntwo");
    assert_eq!(shown(text, &[0..0]), text);
    assert_eq!(shown(text, &[9..9]), text);
    let text = "[a](\nhttps://x)";
    assert_eq!(shown(text, &[]), "a\n");
    assert_eq!(shown(text, &[0..0]), text);
}

#[test]
fn crlf_line_breaks_are_never_hidden() {
    let text = "# A\r\n**b**\r\n- c\r\n";
    assert_eq!(drawn(text, &[]), "A\r\nb\r\n• c\r\n");
    assert_eq!(live_layout(text, &[]).headings, [(0, 1)]);
}

#[test]
fn offsets_are_bytes_past_wide_characters() {
    let text = "é日🙂 **ü** [[Ñ|ö]]\n- 🙂";
    assert_eq!(drawn(text, &[]), "é日🙂 ü ö\n• 🙂");
    assert_eq!(styled(text, LiveStyle::Strong), ["ü"]);
}

#[test]
fn reveal_ranges_past_the_end_are_the_last_line() {
    let text = "a\n**b**";
    assert_eq!(shown(text, &[99..120]), text);
    assert_eq!(shown("", &[5..5]), "");
    assert_eq!(
        live_layout("", &[]).blocks,
        [LiveBlock {
            lines: 0..1,
            source: 0..0,
            kind: BlockKind::Blank,
            list_depth: 0,
            quote_depth: 0,
            item_start: false,
            top: None,
            revealed: false,
        }]
    );
}

#[test]
fn revealing_everything_draws_the_source() {
    let text = "# One\n- [x] **two**\n> three\n---\n```\nc\n```";
    let layout = live_layout(text, &[0..text.len()]);
    assert!(layout.conceal.is_empty() && layout.markers.is_empty());
    assert!(layout.widgets.is_empty());
    assert_eq!(layout.headings, [(0, 1)]);
    assert!(layout.blocks.iter().all(|b| b.revealed));
}

// no-prop-test: a fixed table of levels.
#[test]
fn heading_scales_shrink_with_the_level() {
    let scales: Vec<f32> = (0..=7).map(heading_scale).collect();
    assert_eq!(scales, [1.0, 1.6, 1.4, 1.2, 1.1, 1.0, 1.0, 1.0]);
}

fn config() -> ProptestConfig {
    ProptestConfig {
        cases: 512,
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
        ..ProptestConfig::default()
    }
}

/// Text made mostly of Markdown's marks, line breaks and a few wide
/// characters, so constructs and their edges come up often.
fn markdown_text() -> impl Strategy<Value = String> {
    prop::collection::vec(
        prop_oneof![
            4 => prop::sample::select(vec![
                "#", "## ", "**", "*", "_", "__", "~~", "`", "``", "[", "]", "(", ")",
                "[[", "]]", "|", "\\", "<", ">", "> ", "!", "\n", "\r\n", "\n\n", " ", "---",
                "```", "$", "$$", "https://a.b", "\"t\"", "- ", "1. ", "    ", "=", "  ",
                "- [ ] ", "- [x] ", "* ", "3) ", "![a](b)", "|---|", "<div>", "***",
            ])
            .prop_map(str::to_owned),
            2 => "[a-z]{1,4}",
            1 => "[é日🙂]{1,2}",
            1 => "\\PC{0,4}",
        ],
        0..40,
    )
    .prop_map(|pieces| pieces.concat())
}

/// Reveal ranges as fractions of the text's length, so they land anywhere
/// (inside characters and past the end too).
fn reveals() -> impl Strategy<Value = Vec<(f64, f64)>> {
    prop::collection::vec((0f64..=1.1, 0f64..=1.1), 0..4)
}

fn to_ranges(text: &str, reveal: &[(f64, f64)]) -> Vec<Range<usize>> {
    let len = text.len() as f64;
    reveal
        .iter()
        .map(|&(a, b)| (a * len) as usize..(b * len) as usize)
        .collect()
}

/// The lines (0-based) a reveal range is on, as the module documents.
fn revealed_lines(text: &str, reveal: &[Range<usize>]) -> Vec<bool> {
    let line_of = |at: usize| {
        text.as_bytes()[..at.min(text.len())]
            .iter()
            .filter(|&&b| b == b'\n')
            .count()
    };
    let mut lines = vec![false; text.split('\n').count()];
    for r in reveal {
        let (a, b) = (r.start.min(r.end), r.start.max(r.end));
        for line in &mut lines[line_of(a)..=line_of(b)] {
            *line = true;
        }
    }
    lines
}

/// Checks hidden ranges: sorted, disjoint, not empty, on character
/// boundaries, within one line, touching no reveal range.
fn check_hidden(
    text: &str,
    ranges: &[Range<usize>],
    reveal: &[Range<usize>],
) -> Result<(), TestCaseError> {
    for pair in ranges.windows(2) {
        prop_assert!(pair[0].end <= pair[1].start, "{:?}", ranges);
    }
    for r in ranges {
        prop_assert!(r.start < r.end && r.end <= text.len());
        prop_assert!(text.is_char_boundary(r.start) && text.is_char_boundary(r.end));
        prop_assert!(!text[r.clone()].contains(['\n', '\r']));
        for q in reveal {
            let (a, b) = (q.start.min(q.end), q.start.max(q.end));
            let (a, b) = (a.min(text.len()), b.min(text.len()));
            prop_assert!(
                !(a <= r.end && b >= r.start),
                "{:?} touches reveal {:?}",
                r,
                q
            );
        }
    }
    Ok(())
}

proptest! {
    #![proptest_config(config())]

    /// Whatever the text and the carets, the layout keeps every promise
    /// its fields make.
    #[test]
    fn prop_the_layout_keeps_its_promises(text in markdown_text(), reveal in reveals()) {
        let reveal = to_ranges(&text, &reveal);
        let layout = live_layout(&text, &reveal);
        let revealed = revealed_lines(&text, &reveal);
        let lines = revealed.len();
        let line_of = |at: usize| text[..at].matches('\n').count();

        // Conceal and markers: each well formed, and together disjoint.
        check_hidden(&text, &layout.conceal, &reveal)?;
        let markers: Vec<Range<usize>> = layout.markers.iter().map(|m| m.range.clone()).collect();
        check_hidden(&text, &markers, &reveal)?;
        let mut all: Vec<Range<usize>> = layout.conceal.iter().cloned().chain(markers).collect();
        all.sort_by_key(|r| r.start);
        for pair in all.windows(2) {
            prop_assert!(pair[0].end <= pair[1].start, "{:?}", all);
        }
        for m in &layout.markers {
            prop_assert_eq!(m.line, line_of(m.range.start));
            prop_assert!(!revealed[m.line]);
        }

        // Widgets: whole lines in order, none revealed, nothing hidden on
        // them.
        let mut widget_line = vec![false; lines];
        let mut end = 0;
        for w in &layout.widgets {
            prop_assert!(w.lines.start >= end && w.lines.start < w.lines.end && w.lines.end <= lines);
            end = w.lines.end;
            prop_assert!(w.kind.is_widget());
            prop_assert!(w.source.start <= w.source.end && w.source.end <= text.len());
            prop_assert_eq!(line_of(w.source.start), w.lines.start);
            for line in w.lines.clone() {
                prop_assert!(!revealed[line], "widget {:?} on revealed line {}", w, line);
                widget_line[line] = true;
            }
        }
        for r in &all {
            prop_assert!(!widget_line[line_of(r.start)], "{:?} on a widget", r);
        }

        // Blocks: every line in exactly one, in order.
        let mut next = 0;
        for b in &layout.blocks {
            prop_assert_eq!(b.lines.start, next, "{:?}", layout.blocks);
            prop_assert!(b.lines.start < b.lines.end);
            next = b.lines.end;
            prop_assert_eq!(b.revealed, revealed[b.lines.clone()].iter().any(|&r| r));
            prop_assert!(b.source.start <= b.source.end && b.source.end <= text.len());
        }
        prop_assert_eq!(next, lines);

        // Styles and spans inside the text; headings in order.
        for (r, _) in &layout.styles {
            prop_assert!(r.start < r.end && r.end <= text.len());
            prop_assert!(text.is_char_boundary(r.start) && text.is_char_boundary(r.end));
        }
        for (r, _) in &layout.spans {
            prop_assert!(r.start < r.end && r.end <= text.len());
            prop_assert!(text.is_char_boundary(r.start) && text.is_char_boundary(r.end));
        }
        for link in &layout.links {
            prop_assert!(link.range.start <= link.text.start && link.text.end <= link.range.end);
            prop_assert!(link.range.end <= text.len());
        }
        for pair in layout.headings.windows(2) {
            prop_assert!(pair[0].0 < pair[1].0);
        }
        for &(line, level) in &layout.headings {
            prop_assert!(line < lines && (1..=6).contains(&level));
        }
    }

    /// With the whole text selected nothing is hidden and nothing is a
    /// widget, and what is styled is what a note with no caret styles.
    #[test]
    fn prop_revealing_everything_hides_nothing(text in markdown_text()) {
        let all = live_layout(&text, &[0..text.len()]);
        let none = live_layout(&text, &[]);
        prop_assert!(all.conceal.is_empty());
        prop_assert!(all.markers.is_empty());
        prop_assert!(all.widgets.is_empty());
        prop_assert_eq!(&all.styles, &none.styles);
        prop_assert_eq!(&all.spans, &none.spans);
        prop_assert_eq!(&all.links, &none.links);
        prop_assert_eq!(&all.headings, &none.headings);
        let shape = |l: &LiveLayout| -> Vec<_> {
            l.blocks.iter().map(|b| (b.lines.clone(), b.kind.clone(), b.list_depth, b.quote_depth)).collect()
        };
        prop_assert_eq!(shape(&all), shape(&none));
    }

    /// Any text at all, with any ranges, never panics.
    #[test]
    fn prop_any_text_lays_out(text in "\\PC{0,80}", a in 0usize..200, b in 0usize..200) {
        let layout = live_layout(&text, &[a..b, b..a]);
        for r in &layout.conceal {
            prop_assert!(r.end <= text.len());
        }
        for m in &layout.markers {
            prop_assert!(m.range.end <= text.len());
        }
    }
}

/// The note from a bug report: markdown-rs 1.0.0 panics on a setext heading
/// whose underline is followed by a `---` line and a later setext heading.
/// Live mode draws it.
#[test]
fn a_setext_heading_then_a_rule_is_drawn() {
    let text = "Setext\n===\n---\npara **b** _i_ ~~s~~ `c` \u{1F642}\n---\n";
    let layout = live_layout(text, &[]);
    assert!(!layout.conceal.is_empty());
    assert_eq!(
        layout.blocks.iter().map(|b| b.lines.len()).sum::<usize>(),
        6
    );
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 64,
        failure_persistence: None,
        rng_seed: proptest::test_runner::RngSeed::Fixed(0x006b_616e_736f),
        ..ProptestConfig::default()
    })]

    /// Random mixes of setext headings, thematic breaks, `---` lines, front
    /// matter and inline marks never panic the layout, and every line is in
    /// exactly one block, in order.
    #[test]
    fn prop_structural_mixes_never_break_the_layout(
        pieces in prop::collection::vec(
            prop::sample::select(vec![
                "Setext\n===\n",
                "Title\n---\n",
                "---\n",
                "***\n",
                "===\n",
                "\n",
                "---\ntitle: x\n---\n",
                "para **b** _i_ ~~s~~ `c` \u{1F642}\n",
                "# h\n",
            ]),
            0..10,
        ),
    ) {
        let text: String = pieces.concat();
        let layout = live_layout(&text, &[]);
        let mut next = 0;
        for block in &layout.blocks {
            prop_assert_eq!(block.lines.start, next);
            next = block.lines.end;
        }
        prop_assert_eq!(next, text.matches('\n').count() + 1);
    }
}
