//! The editor's bottom band: where the cursor is, and how long the note is.
//!
//! It is its own view so the editor doesn't redraw for it. The text editor
//! has no event for a moving cursor and notifies on every caret blink, so
//! the footer watches those notifications and only redraws when the line or
//! column changed. Words and characters are counted after typing pauses.

use std::time::Duration;

use gpui_kit::component::h_flex;
use gpui_kit::component::input::{EditorState, InputEvent, Position};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::layout::BOTTOM_BAND_HEIGHT;
use crate::preview::TEXT_INSET;
use crate::text_stats::{format_count, plural, word_count};
use crate::theme::Colors;

/// How long typing pauses before the counts catch up.
const COUNT_DELAY: Duration = Duration::from_millis(250);

pub struct EditorFooter {
    body: Entity<EditorState>,
    cursor: Position,
    words: usize,
    chars: usize,
    /// Whether there is a note, and a cursor in it to report.
    has_note: bool,
    show_cursor: bool,
    count_task: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
}

impl EditorFooter {
    pub fn new(body: Entity<EditorState>, cx: &mut Context<Self>) -> Self {
        let subscriptions = vec![
            cx.observe(&body, |this, body, cx| {
                let cursor = body.read(cx).cursor_position();
                if cursor != this.cursor {
                    this.cursor = cursor;
                    cx.notify();
                }
            }),
            cx.subscribe(&body, |this, _, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    this.count_soon(cx);
                }
            }),
        ];
        EditorFooter {
            body,
            cursor: Position::default(),
            words: 0,
            chars: 0,
            has_note: false,
            show_cursor: true,
            count_task: None,
            _subscriptions: subscriptions,
        }
    }

    /// A note was opened (or closed): count it now.
    pub fn note_opened(&mut self, has_note: bool, cx: &mut Context<Self>) {
        self.has_note = has_note;
        self.count_task = None;
        self.count(cx);
        cx.notify();
    }

    /// The cursor means nothing in Preview, where there is no source.
    pub fn set_show_cursor(&mut self, show: bool, cx: &mut Context<Self>) {
        if self.show_cursor != show {
            self.show_cursor = show;
            cx.notify();
        }
    }

    fn count_soon(&mut self, cx: &mut Context<Self>) {
        self.count_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(COUNT_DELAY).await;
            this.update(cx, |this, cx| this.count(cx)).ok();
        }));
    }

    fn count(&mut self, cx: &mut Context<Self>) {
        let text = self.body.read(cx).value();
        let counts = (word_count(&text), text.chars().count());
        if counts != (self.words, self.chars) {
            (self.words, self.chars) = counts;
            cx.notify();
        }
    }

    pub fn label(&self) -> String {
        footer_label(
            self.show_cursor.then_some(self.cursor),
            self.words,
            self.chars,
        )
    }
}

/// "Ln 12, Col 4 · 412 words · 2,841 chars", from a 0-based cursor.
// no-prop-test: joins fixed words around `format_count`, whose properties
// cover the numbers; `footer_label_examples` checks the layout.
pub fn footer_label(cursor: Option<Position>, words: usize, chars: usize) -> String {
    let counts = format!(
        "{} · {}",
        plural(words, "word", "words"),
        plural(chars, "char", "chars")
    );
    match cursor {
        Some(cursor) => format!(
            "Ln {}, Col {} · {counts}",
            format_count(cursor.line as usize + 1),
            format_count(cursor.character as usize + 1)
        ),
        None => counts,
    }
}

impl Render for EditorFooter {
    // no-prop-test: layout only; the snapshots show it.
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        h_flex()
            .id("editor-footer")
            .h(px(BOTTOM_BAND_HEIGHT))
            .w_full()
            .flex_none()
            .px(px(TEXT_INSET))
            .gap_4()
            .border_t_1()
            .border_color(colors.divider)
            .bg(colors.editor)
            .text_size(px(12.))
            .text_color(colors.text_muted)
            .when(self.has_note, |el| {
                el.child(div().flex_1().min_w_0().truncate().child(self.label()))
                    .child(div().flex_none().child("Markdown"))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::footer_label;
    use gpui_kit::component::input::Position;

    #[test]
    fn footer_label_examples() {
        assert_eq!(
            footer_label(Some(Position::new(11, 3)), 412, 2841),
            "Ln 12, Col 4 · 412 words · 2,841 chars"
        );
        assert_eq!(footer_label(None, 1, 1), "1 word · 1 char");
    }
}
