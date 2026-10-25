//! Drawing the lines of a changed section (`history_diff.rs`): the note
//! history and the review queue show a diff the same way.

use gpui_kit::component::{h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::history_diff::{Hunk, LineKind};
use crate::theme::Colors;
use crate::zoom::px;

// no-prop-test: drawing; the history snapshots show it in both themes.
/// The lines of `hunk` with their numbers, signs and changed words marked.
/// `mono` is the monospaced font, when the system has one.
pub fn hunk_lines(hunk: &Hunk, colors: &Colors, mono: Option<&'static str>) -> impl IntoElement {
    let lines = hunk.lines.iter().map(|line| {
        let (bg, mark, sign, sign_color) = match line.kind {
            LineKind::Same => (None, None, " ", colors.text_muted),
            LineKind::Added => (
                Some(Hsla::from(colors.success).opacity(0.13)),
                Some(Hsla::from(colors.success).opacity(0.34)),
                "+",
                colors.success,
            ),
            LineKind::Removed => (
                Some(Hsla::from(colors.danger).opacity(0.13)),
                Some(Hsla::from(colors.danger).opacity(0.34)),
                "−",
                colors.danger,
            ),
        };
        let number = line
            .old_line
            .or(line.new_line)
            .map(|n| n.to_string())
            .unwrap_or_default();
        let highlights: Vec<(std::ops::Range<usize>, HighlightStyle)> = mark
            .map(|mark| {
                line.changed
                    .iter()
                    .map(|r| {
                        (
                            r.clone(),
                            HighlightStyle {
                                background_color: Some(mark),
                                ..Default::default()
                            },
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        let words = if line.text.is_empty() {
            " ".to_owned()
        } else {
            line.text.clone()
        };
        h_flex()
            .items_start()
            .min_h(px(20.))
            .when_some(bg, |el, bg| el.bg(bg))
            .child(
                div()
                    .w(px(34.))
                    .flex_none()
                    .pr(px(6.))
                    .text_right()
                    .text_color(colors.text_muted)
                    .child(number),
            )
            .child(
                div()
                    .w(px(14.))
                    .flex_none()
                    .text_color(sign_color)
                    .child(sign),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .pr(px(8.))
                    .text_color(if line.kind == LineKind::Same {
                        colors.text_secondary
                    } else {
                        colors.text
                    })
                    .child(StyledText::new(words).with_highlights(highlights)),
            )
    });
    v_flex()
        .py(px(4.))
        .text_size(px(12.5))
        .line_height(px(20.))
        .when_some(mono, |el, font| el.font_family(font))
        .children(lines)
}
