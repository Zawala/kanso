use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;

use super::*;

/// The text as Live mode draws it off the revealed lines: `text` without
/// its concealed bytes.
fn shown(text: &str, reveal: &[Range<usize>]) -> String {
    let layout = live_layout(text, reveal);
    let mut out = String::new();
    let mut at = 0;
    for range in &layout.conceal {
        out.push_str(&text[at..range.start]);
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

#[test]
fn atx_headings_hide_their_marks_and_are_styled_and_listed() {
    let text = "# One\n\npara\n### Three ###\n> ## Quoted";
    assert_eq!(shown(text, &[]), "One\n\npara\nThree\n> Quoted");
    assert_eq!(styled(text, LiveStyle::Heading(1)), ["One"]);
    assert_eq!(styled(text, LiveStyle::Heading(3)), ["Three"]);
    assert_eq!(live_layout(text, &[]).headings, [(0, 1), (3, 3), (4, 2)]);
}

#[test]
fn setext_headings_are_listed_at_their_first_line_and_hide_nothing() {
    let text = "Title\n=====\n\nSub\n---\n";
    let layout = live_layout(text, &[]);
    assert_eq!(layout.headings, [(0, 1), (3, 2)]);
    assert!(layout.conceal.is_empty());
    assert_eq!(styled(text, LiveStyle::Heading(1)), ["Title"]);
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
}

#[test]
fn autolinks_bare_urls_and_escapes_keep_their_marks() {
    let text = "<https://a.b> https://c.d www.e.f \\*no\\* \\[x](y)";
    assert_eq!(shown(text, &[]), text);
}

#[test]
fn wiki_links_hide_brackets_and_an_alias_hides_the_target() {
    let text = "see [[Note]] and [[Other#H|alias]] and `[[code]]` ![[embed]]";
    assert_eq!(
        shown(text, &[]),
        "see Note and alias and [[code]] ![[embed]]"
    );
    assert_eq!(styled(text, LiveStyle::Link), ["Note", "alias"]);
}

#[test]
fn wiki_links_are_not_found_across_table_cells() {
    let text = "| [[a | b]] |\n|---|---|\n| x | y |";
    assert!(styled(text, LiveStyle::Link).is_empty());
}

#[test]
fn code_blocks_front_matter_html_images_and_math_are_left_alone() {
    for text in [
        "```\n# not **bold**\n```",
        "    **indented code**",
        "---\ntitle: **x**\n---\nbody",
        "<div>**x**</div>",
        "![**alt**](img.png)",
        "$$\n**x**\n$$",
        "$a*b*c$",
    ] {
        let layout = live_layout(text, &[]);
        assert_eq!(shown(text, &[]), text, "{text}");
        assert!(layout.headings.is_empty(), "{text}");
    }
}

#[test]
fn table_cells_hide_inline_marks() {
    let text = "| **a** | `b` |\n|---|---|\n| [c](d) | e |";
    assert_eq!(shown(text, &[]), "| a | b |\n|---|---|\n| c | e |");
}

#[test]
fn a_revealed_line_shows_its_marks_and_keeps_its_styles() {
    let text = "# One\n**two**\n_three_";
    // A caret at the end of the first line.
    assert_eq!(shown(text, &[5..5]), "# One\ntwo\nthree");
    // At the start of the second.
    assert_eq!(shown(text, &[6..6]), "One\n**two**\nthree");
    // A selection over the last two.
    assert_eq!(shown(text, &[8..15]), "One\n**two**\n_three_");
    // Backwards ranges count too.
    assert_eq!(shown(text, &[15..8]), "One\n**two**\n_three_");
    let layout = live_layout(text, &[0..text.len()]);
    assert!(layout.conceal.is_empty());
    assert_eq!(layout.headings, [(0, 1)]);
    assert_eq!(layout.styles.len(), 3);
}

#[test]
fn marks_on_different_lines_hide_with_their_own_line() {
    let text = "*one\ntwo*";
    assert_eq!(shown(text, &[]), "one\ntwo");
    assert_eq!(shown(text, &[0..0]), "*one\ntwo");
    assert_eq!(shown(text, &[9..9]), "one\ntwo*");
    let text = "[a](\nhttps://x)";
    assert_eq!(shown(text, &[]), "a\n");
    assert_eq!(shown(text, &[0..0]), "[a](\n");
}

#[test]
fn crlf_line_breaks_are_never_hidden() {
    let text = "# A\r\n**b**\r\n";
    assert_eq!(shown(text, &[]), "A\r\nb\r\n");
    assert_eq!(live_layout(text, &[]).headings, [(0, 1)]);
}

#[test]
fn offsets_are_bytes_past_wide_characters() {
    let text = "é日🙂 **ü** [[Ñ|ö]]";
    assert_eq!(shown(text, &[]), "é日🙂 ü ö");
    assert_eq!(styled(text, LiveStyle::Strong), ["ü"]);
}

#[test]
fn reveal_ranges_past_the_end_are_the_last_line() {
    let text = "a\n**b**";
    assert_eq!(shown(text, &[99..120]), text);
    assert_eq!(shown("", &[5..5]), "");
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
                "[[", "]]", "|", "\\", "<", ">", "!", "\n", "\r\n", "\n\n", " ", "---",
                "```", "$", "https://a.b", "\"t\"", "- ", "1. ", "    ", "=",
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

/// The lines (0-based) a reveal range touches, as the module documents.
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

proptest! {
    #![proptest_config(config())]

    /// Whatever the text and the carets, the conceal ranges are sorted,
    /// disjoint, not empty, on character boundaries, hold no line break
    /// and are on no revealed line; the styles are inside the text on
    /// character boundaries; headings are in order, one per line.
    #[test]
    fn prop_the_layout_keeps_its_promises(text in markdown_text(), reveal in reveals()) {
        let reveal = to_ranges(&text, &reveal);
        let layout = live_layout(&text, &reveal);
        let revealed = revealed_lines(&text, &reveal);
        for pair in layout.conceal.windows(2) {
            prop_assert!(pair[0].end < pair[1].start, "{:?}", layout.conceal);
        }
        for r in &layout.conceal {
            prop_assert!(r.start < r.end && r.end <= text.len());
            prop_assert!(text.is_char_boundary(r.start) && text.is_char_boundary(r.end));
            prop_assert!(!text[r.clone()].contains(['\n', '\r']));
            let line = text[..r.start].matches('\n').count();
            prop_assert!(!revealed[line], "{:?} on revealed line {}", r, line);
        }
        for (r, _) in &layout.styles {
            prop_assert!(r.start < r.end && r.end <= text.len());
            prop_assert!(text.is_char_boundary(r.start) && text.is_char_boundary(r.end));
        }
        for pair in layout.headings.windows(2) {
            prop_assert!(pair[0].0 < pair[1].0);
        }
        for &(line, level) in &layout.headings {
            prop_assert!(line < revealed.len() && (1..=6).contains(&level));
        }
    }

    /// With the whole text selected nothing is hidden, and the styles and
    /// headings are those of a note with no caret at all.
    #[test]
    fn prop_revealing_everything_hides_nothing(text in markdown_text()) {
        let all = live_layout(&text, &[0..text.len()]);
        let none = live_layout(&text, &[]);
        prop_assert!(all.conceal.is_empty());
        prop_assert_eq!(all.styles, none.styles);
        prop_assert_eq!(all.headings, none.headings);
    }

    /// Any text at all, with any ranges, never panics.
    #[test]
    fn prop_any_text_lays_out(text in "\\PC{0,80}", a in 0usize..200, b in 0usize..200) {
        let layout = live_layout(&text, &[a..b, b..a]);
        for r in &layout.conceal {
            prop_assert!(r.end <= text.len());
        }
    }
}
