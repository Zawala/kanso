//! The note history dialog: earlier versions, newest first, with a preview
//! and a way to bring one back.

use gpui_kit::base::text::TextView;
use gpui_kit::component::{ActiveTheme as _, StyledExt as _, WindowExt as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_store::Revision;

use crate::editor::NoteEditor;
use crate::note_list::relative_time;

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

impl Render for HistoryView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.revisions.is_empty() {
            return div()
                .py_8()
                .text_color(cx.theme().muted_foreground)
                .child("No earlier versions yet. Kanso keeps one every few minutes while you edit, and one before every change made elsewhere.")
                .into_any_element();
        }
        let now = kanso_core::now_ms();
        let rows = self.revisions.iter().enumerate().map(|(ix, revision)| {
            let selected = ix == self.selected;
            div()
                .id(("revision", ix))
                .px_2()
                .py_1()
                .rounded_md()
                .cursor_pointer()
                .when(selected, |el| el.bg(cx.theme().list_active))
                .when(!selected, |el| el.hover(|el| el.bg(cx.theme().list_hover)))
                .child(div().text_sm().child(relative_time(revision.saved_at, now)))
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
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
        h_flex()
            .h(px(440.))
            .gap_3()
            .items_start()
            .child(
                div()
                    .id("revisions")
                    .w(px(190.))
                    .h_full()
                    .overflow_y_scroll()
                    .child(v_flex().gap_0p5().children(rows)),
            )
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .border_l_1()
                    .border_color(cx.theme().border)
                    .pl_3()
                    .child(div().font_semibold().pb_2().child(title))
                    .child(
                        div()
                            .id("revision-body")
                            .flex_1()
                            .min_h_0()
                            .overflow_y_scroll()
                            .track_scroll(&self.scroll)
                            .child(TextView::markdown(("revision-text", self.selected), body)),
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
            .title("Note History")
            .width(px(820.))
            .child(view.clone())
            .show_cancel(has_revisions)
            .cancel_text("Close")
            .ok_text(if has_revisions {
                "Restore This Version"
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
