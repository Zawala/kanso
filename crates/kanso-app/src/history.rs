//! The note history dialog: earlier versions, newest first, with a preview
//! and a way to bring one back.

use gpui_kit::base::text::TextView;
use gpui_kit::component::{ActiveTheme as _, StyledExt as _, WindowExt as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_store::Revision;

use crate::clock::Clock;
use crate::editor::NoteEditor;
use crate::grouping::as_title;
use crate::theme::Colors;

/// Why a version was kept, from the origin of the write that replaced it.
pub fn origin_label(origin: &str) -> &'static str {
    match origin {
        "ui" => "Edited in Kanso",
        "api" => "Replaced through the API",
        "sync" => "Replaced by sync",
        "import" => "Replaced by an import",
        _ => "Saved version",
    }
}

/// The dialog's width and its body's height, and the width of its list of
/// versions.
pub const DIALOG_WIDTH: f32 = 820.;
const BODY_HEIGHT: f32 = 440.;
const LIST_WIDTH: f32 = 200.;

pub struct HistoryView {
    revisions: Vec<Revision>,
    selected: usize,
    scroll: ScrollHandle,
}

impl HistoryView {
    fn selected(&self) -> Option<&Revision> {
        self.revisions.get(self.selected)
    }
}

fn field(revision: &Revision, key: &str) -> String {
    revision
        .fields
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_owned()
}

// no-prop-test: drawing; the history snapshots show it in both themes.
impl Render for HistoryView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        if self.revisions.is_empty() {
            return div()
                .py_8()
                .text_size(px(13.5))
                .text_color(colors.text_secondary)
                .child("No earlier versions yet. Kanso keeps one every few minutes while you edit, and one before every change made elsewhere.")
                .into_any_element();
        }
        let rows = self.revisions.iter().enumerate().map(|(ix, revision)| {
            let selected = ix == self.selected;
            v_flex()
                .id(("revision", ix))
                .px(px(10.))
                .py(px(7.))
                .gap(px(1.))
                .rounded(px(6.))
                .cursor_pointer()
                // The selected version as the selected note in the list.
                .when(selected, |el| el.bg(colors.row_active))
                .when(!selected, |el| el.hover(|el| el.bg(colors.row_hover)))
                .child(
                    div()
                        .text_size(px(13.))
                        .line_height(px(18.))
                        .font_medium()
                        .text_color(colors.text)
                        // As the editor's header says when the note was
                        // edited, and the list names the day.
                        .child(as_title(&Clock::edited_label(cx, revision.saved_at))),
                )
                .child(
                    div()
                        .text_size(px(12.))
                        .line_height(px(16.))
                        .text_color(colors.quiet(selected))
                        .child(origin_label(&revision.origin)),
                )
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.selected = ix;
                    cx.notify();
                }))
        });
        let (title, body) = self
            .selected()
            .map(|r| (field(r, "title"), field(r, "body")))
            .unwrap_or_default();
        let dark = cx.theme().is_dark();
        h_flex()
            .h(px(BODY_HEIGHT))
            .gap(px(14.))
            .items_start()
            .child(
                div()
                    .id("revisions")
                    .w(px(LIST_WIDTH))
                    .flex_none()
                    .h_full()
                    .overflow_y_scroll()
                    .child(v_flex().gap(px(2.)).children(rows)),
            )
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .border_l_1()
                    .border_color(colors.divider)
                    .pl(px(18.))
                    .child(
                        div()
                            .pb(px(10.))
                            .text_size(px(18.))
                            .line_height(px(24.))
                            .font_semibold()
                            .text_color(colors.text)
                            .child(title),
                    )
                    .child(
                        div()
                            .id("revision-body")
                            .flex_1()
                            .min_h_0()
                            .overflow_y_scroll()
                            .track_scroll(&self.scroll)
                            // The version as the preview shows a note, a size
                            // smaller to suit the dialog.
                            .child(
                                TextView::markdown(("revision-text", self.selected), body)
                                    .style(crate::preview::text_style(colors, dark))
                                    .text_size(px(14.))
                                    .line_height(relative(1.6))
                                    .text_color(colors.text),
                            ),
                    ),
            )
            .into_any_element()
    }
}

/// Opens the history of the note in `editor`.
pub fn open_history(editor: Entity<NoteEditor>, window: &mut Window, cx: &mut App) {
    let Some(id) = editor.read(cx).note_id().map(str::to_owned) else {
        return;
    };
    let store = cx.global::<crate::workspace::AppState>().store.clone();
    let revisions = store.revisions(&id).unwrap_or_default();
    let has_revisions = !revisions.is_empty();
    let view = cx.new(|_| HistoryView {
        revisions,
        selected: 0,
        scroll: ScrollHandle::new(),
    });
    window.open_alert_dialog(cx, move |alert, _, _| {
        let view_for_ok = view.clone();
        let editor = editor.clone();
        alert
            .title("Note history")
            .width(px(DIALOG_WIDTH))
            .child(view.clone())
            .show_cancel(has_revisions)
            .cancel_text("Close")
            .ok_text(if has_revisions {
                "Restore this version"
            } else {
                "Close"
            })
            .on_ok(move |_, window, cx| {
                let chosen = view_for_ok
                    .read(cx)
                    .selected()
                    .map(|r| (field(r, "title"), field(r, "body")));
                if let Some((title, body)) = chosen {
                    editor.update(cx, |editor, cx| editor.restore(title, body, window, cx));
                }
                true
            })
    });
}

#[cfg(test)]
mod tests {
    use super::origin_label;

    #[test]
    fn every_origin_has_a_label() {
        for origin in [
            kanso_store::Origin::Ui,
            kanso_store::Origin::Api,
            kanso_store::Origin::Sync,
            kanso_store::Origin::Import,
        ] {
            assert_ne!(origin_label(origin.as_str()), "Saved version");
        }
    }
}
