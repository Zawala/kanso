//! Live mode in the note's editor: the Markdown drawn in place. The engine
//! ([`kanso_engine::live`]) says which marks to hide, which spans to style
//! and which lines are headings; this hands the editor the hidden ranges and
//! the larger heading lines as its presentation, and the styles as text
//! decorations.
//!
//! The layout is worked out again whenever the text changes or the caret
//! moves to other lines, at once, so a hidden range never outlives the text
//! it was hidden in.

use std::ops::Range;

use gpui_kit::base::input::{
    EditorPresentation, RopeExt as _, TextDecoration, TextDecorationCollection,
};
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::input::EditorState;
use gpui_kit::*;
use kanso_engine::live::{LiveLayout, LiveStyle, heading_scale, live_layout};

use crate::zoom::px;

/// The theme's styles Live mode draws its spans in, beyond bold, italic and
/// struck through.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LivePalette {
    /// Inline code's text.
    pub code: HighlightStyle,
    /// A link's or a wiki-link's text.
    pub link: HighlightStyle,
    /// A heading's text.
    pub heading: HighlightStyle,
}

impl LivePalette {
    /// The editor theme's own Markdown styles: its code span (or literal),
    /// link text and title captures.
    pub fn from_theme(cx: &App) -> LivePalette {
        let theme = &cx.theme().highlight_theme;
        let code = theme
            .style("text.code.span")
            .or_else(|| theme.style("text.literal"))
            .unwrap_or_default();
        let link = theme.style("link_text").unwrap_or_default();
        let heading = theme.style("title").unwrap_or_default();
        LivePalette {
            code: HighlightStyle {
                // Only the colour: the code font stays the editor's.
                color: code.color,
                ..Default::default()
            },
            link: HighlightStyle {
                color: link.color,
                ..Default::default()
            },
            heading: HighlightStyle {
                color: heading.color,
                ..Default::default()
            },
        }
    }

    /// How `style` draws its span.
    pub fn style(&self, style: LiveStyle) -> HighlightStyle {
        match style {
            LiveStyle::Strong => HighlightStyle {
                font_weight: Some(FontWeight::BOLD),
                ..Default::default()
            },
            LiveStyle::Emphasis => HighlightStyle {
                font_style: Some(FontStyle::Italic),
                ..Default::default()
            },
            LiveStyle::Strikethrough => HighlightStyle {
                strikethrough: Some(StrikethroughStyle {
                    thickness: px(1.),
                    color: None,
                }),
                ..Default::default()
            },
            LiveStyle::Code => self.code,
            LiveStyle::Link => HighlightStyle {
                underline: Some(UnderlineStyle {
                    thickness: px(1.),
                    color: None,
                    wavy: false,
                }),
                ..self.link
            },
            LiveStyle::Heading(_) => HighlightStyle {
                font_weight: Some(FontWeight::BOLD),
                ..self.heading
            },
        }
    }
}

/// The editor presentation for `layout`: its hidden ranges, and its
/// headings' lines at their scale (body-sized headings aren't scaled).
pub fn presentation(layout: &LiveLayout) -> EditorPresentation {
    EditorPresentation {
        concealed: layout.conceal.clone(),
        line_scales: layout
            .headings
            .iter()
            .map(|&(line, level)| (line, heading_scale(level)))
            .filter(|&(_, scale)| scale > 1.)
            .collect(),
        ..Default::default()
    }
}

/// The text decorations for `layout`'s styles: sorted, disjoint and never
/// empty, each part of the text in the styles of every span over it, an
/// inner (shorter) span's winning where two set the same property.
pub fn decorations(layout: &LiveLayout, palette: &LivePalette) -> Vec<TextDecoration> {
    let spans: Vec<(Range<usize>, HighlightStyle)> = layout
        .styles
        .iter()
        .filter(|(range, _)| range.start < range.end)
        .map(|(range, style)| (range.clone(), palette.style(*style)))
        .collect();
    let mut bounds: Vec<usize> = spans
        .iter()
        .flat_map(|(range, _)| [range.start, range.end])
        .collect();
    bounds.sort_unstable();
    bounds.dedup();
    let mut order: Vec<usize> = (0..spans.len()).collect();
    order.sort_by_key(|&ix| spans[ix].0.start);
    let mut next = 0;
    // The spans over the current part, outermost first.
    let mut open: Vec<usize> = Vec::new();
    let mut out: Vec<TextDecoration> = Vec::new();
    for pair in bounds.windows(2) {
        let (start, end) = (pair[0], pair[1]);
        open.retain(|&ix| spans[ix].0.end > start);
        while next < order.len() && spans[order[next]].0.start <= start {
            open.push(order[next]);
            next += 1;
        }
        if open.is_empty() {
            continue;
        }
        open.sort_by_key(|&ix| std::cmp::Reverse(spans[ix].0.len()));
        let style = open.iter().fold(HighlightStyle::default(), |style, &ix| {
            style.highlight(spans[ix].1)
        });
        match out.last_mut() {
            Some(last) if last.range.end == start && last.style == style => last.range.end = end,
            _ => out.push(TextDecoration::new(start..end, style)),
        }
    }
    out
}

/// The buffer lines (first to last, inclusive) the caret and the active
/// selection touch: what the editor reveals.
fn revealed_lines(state: &EditorState) -> Range<usize> {
    let text = state.text();
    let selection = state.selected_range();
    let cursor = state.cursor();
    let line = |offset: usize| text.offset_to_point(offset.min(text.len())).row;
    let first = line(selection.start.min(cursor));
    let last = line(selection.end.max(cursor));
    first..last + 1
}

/// The caret's and the active selection's ranges, as the engine reveals
/// them.
fn reveal_ranges(state: &EditorState) -> Vec<Range<usize>> {
    let cursor = state.cursor();
    vec![state.selected_range(), cursor..cursor]
}

/// What a layout was worked out for: the text's version and length, the
/// revealed lines and the theme's styles.
type Shown = (u64, usize, Range<usize>, LivePalette);

/// The editor's Live presentation and the decorations that go with it.
#[derive(Default)]
pub struct LiveMarks {
    collection: Option<TextDecorationCollection>,
    /// What the presentation in the editor was worked out for, and the
    /// presentation the editor kept.
    shown: Option<(Shown, EditorPresentation)>,
}

impl LiveMarks {
    /// Draws `body` in Live mode, working the layout out again if the text
    /// (`version`), the lines the caret is on, or the theme changed since,
    /// or the editor dropped the presentation (a new text).
    pub fn refresh(&mut self, body: &Entity<EditorState>, version: u64, cx: &mut App) {
        let palette = LivePalette::from_theme(cx);
        let state = body.read(cx);
        let key: Shown = (version, state.text().len(), revealed_lines(state), palette);
        if let Some((shown, kept)) = &self.shown
            && *shown == key
            && *kept == state.editor_presentation()
        {
            return;
        }
        let text = state.value();
        let layout = live_layout(&text, &reveal_ranges(state));
        let decorations = decorations(&layout, &palette);
        let kept = body.update(cx, |state, cx| {
            state.set_presentation(Some(presentation(&layout)), cx);
            state.editor_presentation()
        });
        self.shown = Some((key, kept));
        match &self.collection {
            Some(collection) => collection.set(decorations, cx),
            None if decorations.is_empty() => {}
            None => {
                self.collection = Some(body.update(cx, |state, cx| {
                    state.create_decorations_collection(decorations, cx)
                }));
            }
        }
    }

    /// Draws `body` as plain source again.
    pub fn clear(&mut self, body: &Entity<EditorState>, cx: &mut App) {
        if self.shown.take().is_none() {
            return;
        }
        body.update(cx, |state, cx| state.set_presentation(None, cx));
        if let Some(collection) = &self.collection {
            collection.clear(cx);
        }
    }

    /// Whether a Live presentation is drawn.
    #[cfg(test)]
    pub fn is_shown(&self) -> bool {
        self.shown.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::{LivePalette, decorations, presentation};
    use gpui_kit::{FontWeight, HighlightStyle};
    use kanso_engine::live::{LiveLayout, LiveStyle, heading_scale, live_layout};
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    fn palette() -> LivePalette {
        LivePalette {
            code: HighlightStyle::color(gpui_kit::red()),
            link: HighlightStyle::color(gpui_kit::blue()),
            heading: HighlightStyle::color(gpui_kit::green()),
        }
    }

    #[test]
    fn headings_scale_and_marks_hide() {
        let text = "# Title\n\nSome **bold** and `code`.\n##### Small\n";
        let layout = live_layout(text, &[]);
        let shown = presentation(&layout);
        assert_eq!(shown.concealed, layout.conceal);
        assert!(shown.concealed.contains(&(0..2)));
        // H1 is larger; H5 is body-sized and not scaled.
        assert_eq!(shown.line_scales, vec![(0, heading_scale(1))]);
    }

    #[test]
    fn styles_become_disjoint_decorations() {
        let text = "## A [link](u) and **b**\n";
        let layout = live_layout(text, &[]);
        let marks = decorations(&layout, &palette());
        let link = text.find("link").unwrap();
        let at = |offset: usize| {
            marks
                .iter()
                .find(|mark| mark.range.contains(&offset))
                .map(|mark| mark.style)
        };
        // The link wins the colour inside the heading, and keeps it bold.
        let style = at(link).expect("the link is styled");
        assert_eq!(style.color, Some(gpui_kit::blue()));
        assert_eq!(style.font_weight, Some(FontWeight::BOLD));
        assert!(style.underline.is_some());
        let heading = at(3).expect("the heading is styled");
        assert_eq!(heading.color, Some(gpui_kit::green()));
        for pair in marks.windows(2) {
            assert!(pair[0].range.end <= pair[1].range.start);
        }
    }

    #[test]
    fn plain_text_has_no_presentation() {
        let layout = live_layout("Just words.\n", &[]);
        assert!(presentation(&layout).is_empty());
        assert!(decorations(&layout, &palette()).is_empty());
    }

    fn arb_layout() -> impl Strategy<Value = LiveLayout> {
        let style = prop_oneof![
            Just(LiveStyle::Strong),
            Just(LiveStyle::Emphasis),
            Just(LiveStyle::Strikethrough),
            Just(LiveStyle::Code),
            Just(LiveStyle::Link),
            (1u8..=6).prop_map(LiveStyle::Heading),
        ];
        (
            prop::collection::vec((0usize..200, 0usize..200), 0..12),
            prop::collection::vec(((0usize..200, 0usize..60), style), 0..16),
            prop::collection::vec((0usize..40, 1u8..=6), 0..8),
        )
            .prop_map(|(conceal, styles, headings)| LiveLayout {
                conceal: conceal
                    .into_iter()
                    .map(|(a, b)| a.min(b)..a.max(b))
                    .collect(),
                styles: styles
                    .into_iter()
                    .map(|((start, len), style)| (start..start + len, style))
                    .collect(),
                headings,
                ..LiveLayout::default()
            })
    }

    /// Invariant: the hidden ranges pass through as they are, and every
    /// scaled line is a heading's, drawn larger than body text.
    fn presentation_keeps_the_layout(layout: &LiveLayout) -> Result<(), TestCaseError> {
        let shown = presentation(layout);
        prop_assert_eq!(&shown.concealed, &layout.conceal);
        for &(line, scale) in &shown.line_scales {
            prop_assert!(scale > 1.);
            let heading = layout
                .headings
                .iter()
                .any(|&(l, level)| l == line && heading_scale(level) == scale);
            prop_assert!(heading);
        }
        let scaled = layout
            .headings
            .iter()
            .filter(|&&(_, level)| heading_scale(level) > 1.)
            .count();
        prop_assert_eq!(shown.line_scales.len(), scaled);
        Ok(())
    }

    /// Invariant: the decorations are sorted, disjoint and non-empty, and
    /// cover exactly the bytes some style covers.
    fn decorations_cover_the_styles(layout: &LiveLayout) -> Result<(), TestCaseError> {
        let marks = decorations(layout, &palette());
        for mark in &marks {
            prop_assert!(mark.range.start < mark.range.end);
        }
        for pair in marks.windows(2) {
            prop_assert!(pair[0].range.end <= pair[1].range.start);
        }
        for offset in 0..260 {
            let styled = layout
                .styles
                .iter()
                .any(|(range, _)| range.contains(&offset));
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
        fn prop_presentation_keeps_the_layout(layout in arb_layout()) {
            presentation_keeps_the_layout(&layout)?;
        }

        #[test]
        fn prop_decorations_cover_the_styles(layout in arb_layout()) {
            decorations_cover_the_styles(&layout)?;
        }
    }
}
