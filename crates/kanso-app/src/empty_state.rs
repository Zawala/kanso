//! What the editor pane shows when a notebook has no notes yet, or the
//! library none at all: what is missing, the two ways to fill it, the
//! command palette, and that notes stay plain Markdown. The note list beside
//! it says so quietly.

use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{Icon, StyledExt as _, h_flex, v_flex};
use gpui_kit::*;

use crate::keys::shortcut_chip;
use crate::theme::Colors;
use crate::workspace::{ImportInkdrop, NewNote, OpenCommandPalette};

use gpui_kit::assets::IconName as Lucide;

/// The empty-notebook block. Its buttons and the palette row send the app's
/// own actions from `focus`, which must be inside the workspace so they
/// reach its handlers.
#[derive(IntoElement)]
pub struct EmptyNotebook {
    title: SharedString,
    message: &'static str,
    focus: FocusHandle,
}

impl EmptyNotebook {
    /// The block under `title`, such as "No notes in Research".
    pub fn titled(title: impl Into<SharedString>, focus: FocusHandle) -> Self {
        EmptyNotebook {
            title: title.into(),
            message: "Create your first note or import notes.",
            focus,
        }
    }

    /// The words for a library with no notes at all, as on first run.
    pub fn first_run(self) -> Self {
        EmptyNotebook {
            message: "Create your first note, or bring your notes over from Inkdrop.",
            ..self
        }
    }
}

// no-prop-test: layout and three action dispatches; the snapshots show it
// and the note list's window test sends them.
impl RenderOnce for EmptyNotebook {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = Colors::get(cx);
        let new_focus = self.focus.clone();
        let import_focus = self.focus.clone();
        let palette_focus = self.focus;
        // The shortcut as bound, so the row never shows a stale one.
        let palette_key = shortcut_chip(&OpenCommandPalette, window, cx);
        let divider = || div().w_full().h(px(1.)).bg(colors.divider);
        v_flex()
            .w_full()
            .max_w(px(328.))
            .items_center()
            .child(
                Icon::new(Lucide::FileText)
                    .size(px(36.))
                    .text_color(colors.text_muted),
            )
            .child(
                div()
                    .mt(px(18.))
                    .text_size(px(22.))
                    .line_height(px(28.))
                    .font_semibold()
                    .text_color(colors.text)
                    .text_center()
                    .child(self.title),
            )
            .child(
                div()
                    .mt(px(6.))
                    .text_size(px(14.))
                    .line_height(px(20.))
                    .text_color(colors.text_secondary)
                    .text_center()
                    .child(self.message),
            )
            .child(
                h_flex()
                    .mt(px(22.))
                    .gap_2()
                    .flex_wrap()
                    .justify_center()
                    .child(
                        Button::new("empty-new-note")
                            .primary()
                            .icon(Lucide::SquarePen)
                            .label("New note")
                            .on_click(move |_, window, cx| {
                                new_focus.dispatch_action(&NewNote, window, cx);
                            }),
                    )
                    .child(
                        Button::new("empty-import")
                            .icon(Lucide::Upload)
                            .label("Import notes")
                            .tooltip("Import from Inkdrop")
                            .on_click(move |_, window, cx| {
                                import_focus.dispatch_action(&ImportInkdrop, window, cx);
                            }),
                    ),
            )
            .child(divider().mt(px(26.)))
            .child(
                h_flex()
                    .id("empty-open-palette")
                    .w_full()
                    .h(px(36.))
                    .my(px(2.))
                    .px(px(6.))
                    .gap_2()
                    .rounded(px(6.))
                    .cursor_pointer()
                    .hover(|el| el.bg(colors.row_hover))
                    .text_size(px(13.))
                    .text_color(colors.text_secondary)
                    .on_click(move |_, window, cx| {
                        palette_focus.dispatch_action(&OpenCommandPalette, window, cx);
                    })
                    .child(
                        Icon::new(Lucide::Command)
                            .size(px(14.))
                            .text_color(colors.text_muted),
                    )
                    .child(div().flex_1().child("Open command palette"))
                    .children(palette_key),
            )
            .child(divider())
            .child(
                h_flex()
                    .mt(px(18.))
                    .gap_1p5()
                    .text_size(px(12.))
                    .text_color(colors.text_muted)
                    .child(Icon::new(Lucide::Lock).size(px(12.)))
                    .child("Markdown notes stay portable."),
            )
    }
}
