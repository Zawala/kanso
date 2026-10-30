// The reveal ranges are carets, one of them.
#![allow(clippy::single_range_in_vec_init)]

use std::ops::Range;

use super::{
    CODE_SIZE, LiveMetrics, LivePalette, LivePlan, UNRESOLVED_TEXT, WidgetKind, WidgetPlan,
    decorations, plan, shown, task_item_start,
};
use gpui_kit::base::input::{Ornament, Replacement};
use gpui_kit::px as raw;
use kanso_engine::live::{LiveBlock, LiveSpan, live_layout};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;

const M: LiveMetrics = LiveMetrics {
    body: 16.,
    rem: 16.,
    zoom: 1.,
    ascent: 14.85,
    descent: 3.77,
};

fn palette() -> LivePalette {
    LivePalette {
        text: gpui_kit::black(),
        heading: None,
        link: gpui_kit::blue(),
        quote: gpui_kit::green(),
        muted: gpui_kit::red(),
        border: gpui_kit::yellow(),
        code_background: gpui_kit::white(),
        code_text: None,
        check: gpui_kit::white(),
        mono: "Mono".into(),
        background: gpui_kit::white(),
        code_radius: raw(4.),
        calendar: "icons/calendar.svg".into(),
    }
}

/// The plan for `text` with the caret at its end (on its last line).
fn plan_at_end(text: &str) -> LivePlan {
    let end = text.len();
    let layout = live_layout(text, &[end..end]);
    plan(
        text,
        &layout,
        &M,
        &palette(),
        &|title| title == "Here",
        &[],
        &[],
    )
}

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-3
}

#[test]
fn headings_take_the_previews_size_and_line_height() {
    let shown = plan_at_end("# Title\n\nText\n").presentation;
    assert_eq!(shown.line_scales, vec![(0, 1.8)]);
    assert_eq!(shown.line_heights.len(), 1);
    assert!(close(shown.line_heights[0].1, 1.25 / 1.65));
    // `# ` is hidden; the heading is set bold.
    assert!(shown.concealed.contains(&(0..2)));
}

#[test]
fn blank_lines_become_the_previews_gaps() {
    let drawn = plan_at_end("# Title\n\nOne\n\nTwo\n");
    let gaps: Vec<(Range<usize>, f32)> = drawn
        .widgets
        .iter()
        .map(|w| match w.kind {
            WidgetKind::Gap(height) => (w.lines.clone(), height),
            _ => panic!("only gaps"),
        })
        .collect();
    // Below an H1 its bottom gap (0.35 of its size); between paragraphs
    // the paragraph gap (0.9 rem).
    assert_eq!(gaps.len(), 2);
    assert_eq!(gaps[0].0, 1..2);
    assert!(close(gaps[0].1, 16. * 1.8 * 0.35));
    assert_eq!(gaps[1].0, 3..4);
    assert!(close(gaps[1].1, 16. * 0.9));
}

#[test]
fn a_heading_after_a_paragraph_keeps_its_section_gap() {
    let drawn = plan_at_end("Text\n\n## Part\n");
    let WidgetKind::Gap(gap) = drawn.widgets[0].kind else {
        panic!("a gap");
    };
    assert!(close(gap, 16. * (4. / 3.) * 1.6));
}

#[test]
fn bullets_hang_in_their_indent_column() {
    let text = "- a\n  - b\n- c\n\nEnd\n";
    let shown = plan_at_end(text).presentation;
    assert_eq!(
        shown.line_insets,
        vec![(0, raw(22.4)), (1, raw(44.8)), (2, raw(22.4))]
    );
    let bullets: Vec<(&str, Range<usize>)> = shown
        .replacements
        .iter()
        .map(|r| (r.text.as_ref(), r.range.clone()))
        .collect();
    assert_eq!(bullets, vec![("•", 0..2), ("◦", 4..8), ("•", 10..12)]);
    let column = shown.replacements[0].column.expect("a column");
    assert!(column.hanging);
    assert!(close(f32::from(column.width), 22.4));
    assert!(close(f32::from(column.gap), 22.4 * 0.3));
    // Items after a list's first breathe a quarter em apart; a nested
    // list's first item sits right under its parent.
    assert_eq!(shown.line_space, vec![(2, raw(4.), raw(0.))]);
}

#[test]
fn a_task_box_is_clickable_and_drawn_like_the_previews() {
    let text = "- [x] Done\n- [ ] Open\n\nEnd\n";
    let shown = plan_at_end(text).presentation;
    let boxes: Vec<&Replacement> = shown.replacements.iter().filter(|r| r.clickable).collect();
    assert_eq!(boxes.len(), 2);
    assert_eq!(boxes[0].range, 0..6);
    // The text starts past the box and its gap, the box hangs before it.
    let column = 16. * (0.9375 + 0.5);
    assert_eq!(shown.line_insets[0], (0, raw(column)));
    let checks: Vec<&Ornament> = shown.line_ornaments.iter().map(|(_, o)| o).collect();
    match checks[0] {
        Ornament::Check {
            x,
            size,
            fill,
            check,
            ..
        } => {
            assert!(close(f32::from(*x), 0.));
            assert!(close(f32::from(*size), 15.));
            assert_eq!(*fill, Some(gpui_kit::blue()));
            assert!(check.is_some());
        }
        other => panic!("a box, not {other:?}"),
    }
    assert!(matches!(checks[1], Ornament::Check { fill: None, .. }));
    assert_eq!(task_item_start(text, &boxes[1].range), Some(11));
}

#[test]
fn quotes_get_their_bar_and_inset() {
    let text = "> One\n>\n> Two\n\nEnd\n";
    let drawn = plan_at_end(text);
    let shown = drawn.presentation;
    assert_eq!(shown.line_insets, vec![(0, raw(17.)), (2, raw(17.))]);
    assert!(
        shown
            .line_ornaments
            .iter()
            .all(|(_, o)| matches!(o, Ornament::Bar { .. }))
    );
    assert_eq!(shown.line_ornaments.len(), 2);
    // The `>` line between the paragraphs is the paragraph gap, beside
    // the bar.
    let gap = &drawn.widgets[0];
    assert_eq!(gap.lines, 1..2);
    assert_eq!(gap.quotes, 1);
}

#[test]
fn a_rule_is_a_hairline_row() {
    let shown = plan_at_end("Text\n\n---\n\nMore\n").presentation;
    assert_eq!(shown.line_heights.len(), 1);
    assert!(close(shown.line_heights[0].1 * M.line(), 1.));
    assert!(
        shown
            .line_ornaments
            .iter()
            .any(|(line, o)| *line == 2 && matches!(o, Ornament::Rule { .. }))
    );
}

#[test]
fn blocks_off_the_caret_are_widgets_and_on_it_their_source() {
    let text = "Text\n\n```rust\nfn a() {}\n```\n\nEnd\n";
    let drawn = plan_at_end(text);
    let blocks: Vec<&WidgetPlan> = drawn
        .widgets
        .iter()
        .filter(|w| matches!(w.kind, WidgetKind::Markdown { .. }))
        .collect();
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].lines, 2..5);
    let WidgetKind::Markdown { source, .. } = &blocks[0].kind else {
        unreachable!()
    };
    assert_eq!(source, "```rust\nfn a() {}\n```");
    // With the caret in it, it is its source.
    let layout = live_layout(text, &[14..14]);
    let revealed = plan(text, &layout, &M, &palette(), &|_| true, &[], &[]);
    assert!(
        revealed
            .widgets
            .iter()
            .all(|w| matches!(w.kind, WidgetKind::Gap(_)))
    );
}

#[test]
fn quoted_and_indented_blocks_lose_their_marks() {
    let text = "> ```\n> a\n> ```\n\nEnd\n";
    let drawn = plan_at_end(text);
    let block = drawn
        .widgets
        .iter()
        .find(|w| matches!(w.kind, WidgetKind::Markdown { .. }))
        .expect("a block");
    assert_eq!(block.quotes, 1);
    let WidgetKind::Markdown { source, .. } = &block.kind else {
        unreachable!()
    };
    assert_eq!(source, "```\na\n```");
}

#[test]
fn links_are_coloured_and_unresolved_wiki_links_quieter() {
    let text = "[a](u) [[Here]] [[Gone]]\n\nEnd\n";
    let drawn = plan_at_end(text);
    let style_at = |needle: &str| {
        let at = text.find(needle).unwrap();
        drawn
            .decorations
            .iter()
            .find(|d| d.range.contains(&at))
            .map(|d| d.style)
            .expect("styled")
    };
    assert_eq!(style_at("a]").color, Some(gpui_kit::blue()));
    assert_eq!(style_at("Here").color, Some(gpui_kit::blue()));
    assert_eq!(
        style_at("Gone").color,
        Some(gpui_kit::white().blend(gpui_kit::blue().opacity(UNRESOLVED_TEXT)))
    );
}

#[test]
fn inline_code_off_the_caret_is_set_in_the_code_face() {
    let text = "Some `code` here\n\nEnd\n";
    let shown = plan_at_end(text).presentation;
    let code = shown
        .replacements
        .iter()
        .find(|r| r.family.is_some())
        .expect("code drawn in its face");
    assert_eq!(code.range, 6..10);
    assert_eq!(code.text.as_ref(), "code");
    assert!(close(code.size, CODE_SIZE));
}

fn line_kind() -> impl Strategy<Value = &'static str> {
    prop::sample::select(vec![
        "# Head",
        "## Sub",
        "###### Six",
        "Title",
        "===",
        "text **b** `c` [l](u) [[W]] $x$ ~~s~~",
        "",
        "- item",
        "  - nested",
        "1. one",
        "- [ ] task",
        "- [x] done",
        "  continued",
        "> quote",
        ">",
        "> - quoted item",
        "---",
        "```",
        "code",
        "| a | b |",
        "|---|---|",
        "$$",
        "[d]: http://x",
        "![p](x.png)",
    ])
}

fn arb_note() -> impl Strategy<Value = (String, usize)> {
    (
        prop::collection::vec(line_kind(), 0..24),
        any::<prop::sample::Index>(),
    )
        .prop_map(|(lines, at)| {
            let text = lines.join("\n");
            let at = at.index(text.len() + 1);
            let at = (0..=at)
                .rev()
                .find(|&i| text.is_char_boundary(i))
                .unwrap_or(0);
            (text, at)
        })
}

/// Invariant: every block mark is hidden or drawn as something else, the
/// hidden and replaced ranges are sorted and never overlap, every inset,
/// space, height and widget is sound, and the widgets are exactly the
/// layout's off the caret plus gaps over lines nothing is drawn on.
fn plan_is_sound(text: &str, at: usize) -> Result<(), TestCaseError> {
    let layout = live_layout(text, &[at..at]);
    let drawn = plan(text, &layout, &M, &palette(), &|_| true, &[], &[]);
    let shown = &drawn.presentation;
    for marker in &layout.markers {
        let hidden = shown.concealed.contains(&marker.range)
            || shown.replacements.iter().any(|r| r.range == marker.range);
        prop_assert!(hidden, "{:?}", marker);
    }
    for pair in shown.concealed.windows(2) {
        prop_assert!(pair[0].end <= pair[1].start);
    }
    for pair in shown.replacements.windows(2) {
        prop_assert!(pair[0].range.end <= pair[1].range.start);
    }
    for r in &shown.replacements {
        prop_assert!(
            shown
                .concealed
                .iter()
                .all(|c| c.end <= r.range.start || c.start >= r.range.end)
        );
    }
    for &(_, above, below) in &shown.line_space {
        prop_assert!(f32::from(above) >= 0. && f32::from(below) >= 0.);
        prop_assert!(f32::from(above).is_finite() && f32::from(below).is_finite());
    }
    for &(_, inset) in &shown.line_insets {
        prop_assert!(f32::from(inset) > 0. && f32::from(inset).is_finite());
    }
    for &(_, factor) in shown.line_heights.iter().chain(&shown.line_scales) {
        prop_assert!(factor > 0. && factor.is_finite());
    }
    let mut end = 0;
    for widget in &drawn.widgets {
        prop_assert!(widget.lines.start >= end && widget.lines.start < widget.lines.end);
        end = widget.lines.end;
        prop_assert!(widget.height() > 0. && widget.inset >= 0.);
        match widget.kind {
            WidgetKind::Markdown { .. } => {
                prop_assert!(layout.widgets.iter().any(|w| w.lines == widget.lines));
            }
            WidgetKind::Gap(_) => {
                for line in widget.lines.clone() {
                    let block = layout.blocks.iter().find(|b| b.lines.contains(&line));
                    prop_assert!(block.is_some_and(|b| !shown_block(b) && !b.revealed));
                }
            }
        }
    }
    for w in &layout.widgets {
        let found = drawn
            .widgets
            .iter()
            .any(|p| p.lines == w.lines && matches!(p.kind, WidgetKind::Markdown { .. }));
        prop_assert!(found, "{:?}", w.lines);
    }
    Ok(())
}

fn shown_block(block: &LiveBlock) -> bool {
    shown(block)
}

/// Invariant: the decorations are sorted, disjoint and non-empty, and
/// cover exactly the bytes some style or styled span covers.
fn decorations_cover_the_styles(text: &str, at: usize) -> Result<(), TestCaseError> {
    let layout = live_layout(text, &[at..at]);
    let marks = decorations(&layout, &palette(), &|_| true);
    for mark in &marks {
        prop_assert!(mark.range.start < mark.range.end);
    }
    for pair in marks.windows(2) {
        prop_assert!(pair[0].range.end <= pair[1].range.start);
    }
    for offset in 0..text.len() {
        let styled = layout.styles.iter().any(|(r, _)| r.contains(&offset))
            || layout
                .spans
                .iter()
                .any(|(r, s)| r.contains(&offset) && *s != LiveSpan::InlineMath);
        let marked = marks.iter().any(|mark| mark.range.contains(&offset));
        prop_assert_eq!(styled, marked, "offset {}", offset);
    }
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 256,
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
        ..ProptestConfig::default()
    })]

    #[test]
    fn prop_plan_is_sound((text, at) in arb_note()) {
        plan_is_sound(&text, at)?;
    }

    #[test]
    fn prop_decorations_cover_the_styles((text, at) in arb_note()) {
        decorations_cover_the_styles(&text, at)?;
    }
}
