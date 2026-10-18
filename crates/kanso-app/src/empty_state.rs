//! What a pane shows when a notebook has no notes yet: what is missing, the
//! two ways to fill it, where the commands are, and that notes stay plain
//! Markdown. The note list draws it narrow; the editor can draw it at full
//! size.

use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::kbd::Kbd;
use gpui_kit::component::{Icon, Sizable as _, StyledExt as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::theme::Colors;
use crate::workspace::{ImportInkdrop, NewNote};

use gpui_kit::assets::IconName as Lucide;

/// The empty-notebook block. Its buttons send the app's own actions from
/// `focus`, which must be inside the workspace so they reach its handlers.
#[derive(IntoElement)]
pub struct EmptyNotebook {
    title: SharedString,
    focus: FocusHandle,
    compact: bool,
}

impl EmptyNotebook {
    /// "No notes in Research".
    pub fn new(notebook: &str, focus: FocusHandle) -> Self {
        Self::titled(format!("No notes in {notebook}"), focus)
    }

    /// The same block under another title, such as "No notes yet".
    pub fn titled(title: impl Into<SharedString>, focus: FocusHandle) -> Self {
        EmptyNotebook {
            title: title.into(),
            focus,
            compact: false,
        }
    }

    /// Smaller type and buttons, for a column as narrow as the note list.
    pub fn compact(mut self) -> Self {
        self.compact = true;
        self
    }
}

// no-prop-test: layout and two action dispatches; the snapshots show it.
impl RenderOnce for EmptyNotebook {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = Colors::get(cx);
        let compact = self.compact;
        let new_focus = self.focus.clone();
        let import_focus = self.focus;
        let divider = || div().w_full().h(px(1.)).bg(colors.divider);
        let button_size = |button: Button| if compact { button.small() } else { button };
        v_flex()
            .w_full()
            .max_w(px(if compact { 272. } else { 328. }))
            .items_center()
            .child(
                Icon::new(Lucide::FileText)
                    .size(px(if compact { 28. } else { 36. }))
                    .text_color(colors.text_muted),
            )
            .child(
                div()
                    .mt(px(if compact { 14. } else { 18. }))
                    .text_size(px(if compact { 16. } else { 22. }))
                    .line_height(px(if compact { 22. } else { 28. }))
                    .font_semibold()
                    .text_color(colors.text)
                    .text_center()
                    .child(self.title),
            )
            .child(
                div()
                    .mt(px(6.))
                    .text_size(px(if compact { 12.5 } else { 14. }))
                    .line_height(px(if compact { 18. } else { 20. }))
                    .text_color(colors.text_secondary)
                    .text_center()
                    .child("Create your first note or import notes."),
            )
            .child(
                h_flex()
                    .mt(px(if compact { 18. } else { 22. }))
                    .gap_2()
                    .flex_wrap()
                    .justify_center()
                    .child(button_size(
                        Button::new("empty-new-note")
                            .primary()
                            .icon(Lucide::SquarePen)
                            .label("New note")
                            .on_click(move |_, window, cx| {
                                new_focus.dispatch_action(&NewNote, window, cx);
                            }),
                    ))
                    .child(button_size(
                        Button::new("empty-import")
                            .icon(Lucide::Upload)
                            .label("Import notes")
                            .on_click(move |_, window, cx| {
                                import_focus.dispatch_action(&ImportInkdrop, window, cx);
                            }),
                    )),
            )
            .child(divider().mt(px(if compact { 20. } else { 26. })))
            // A hint, not a button: the palette's own shortcut opens it.
            .child(
                h_flex()
                    .w_full()
                    .h(px(36.))
                    .px_1()
                    .gap_2()
                    .text_size(px(if compact { 12.5 } else { 13. }))
                    .text_color(colors.text_secondary)
                    .child(
                        Icon::new(Lucide::Command)
                            .size(px(14.))
                            .text_color(colors.text_muted),
                    )
                    .child(div().flex_1().child("Open command palette"))
                    .child(Kbd::new(
                        Keystroke::parse("cmd-k").expect("a valid keystroke"),
                    )),
            )
            .child(divider())
            .child(
                h_flex()
                    .mt(px(if compact { 14. } else { 18. }))
                    .gap_1p5()
                    .text_size(px(12.))
                    .text_color(colors.text_muted)
                    .child(Icon::new(Lucide::Lock).size(px(12.)))
                    .child("Markdown notes stay portable."),
            )
            .when(compact, |el| el.px_2())
    }
}
