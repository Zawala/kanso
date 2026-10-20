//! What the editor pane shows when a notebook has no notes yet, or the
//! library none at all: what is missing, the two ways to fill it, the
//! command palette, and that notes stay plain Markdown. On a first run while
//! signed out, a row offers to sign in to sync instead. The note list beside
//! it says so quietly.

use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{Icon, StyledExt as _, h_flex, v_flex};
use gpui_kit::*;

use gpui_kit::prelude::FluentBuilder as _;

use crate::keys::shortcut_chip;
use crate::services::Services;
use crate::theme::Colors;
use crate::workspace::{ImportLibrary, NewNote, OpenCommandPalette, ShowSignIn};

use gpui_kit::assets::IconName as Lucide;

/// The empty-notebook block. Its buttons and the palette row send the app's
/// own actions from `focus`, which must be inside the workspace so they
/// reach its handlers.
#[derive(IntoElement)]
pub struct EmptyNotebook {
    title: SharedString,
    message: &'static str,
    focus: FocusHandle,
    /// The library has no notes at all.
    first_run: bool,
}

impl EmptyNotebook {
    /// The block under `title`, such as "No notes in Research".
    pub fn titled(title: impl Into<SharedString>, focus: FocusHandle) -> Self {
        EmptyNotebook {
            title: title.into(),
            message: "Create your first note or import notes.",
            focus,
            first_run: false,
        }
    }

    /// The words for a library with no notes at all, as on first run.
    pub fn first_run(self) -> Self {
        EmptyNotebook {
            message: "Create your first note, or import your notes.",
            first_run: true,
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
        let sign_in_focus = self.focus.clone();
        let palette_focus = self.focus;
        let offer_sign_in = self.first_run
            && !cx
                .try_global::<Services>()
                .is_some_and(|services| services.signed_in());
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
                            .tooltip("From Markdown, Obsidian, Notion, Evernote and more")
                            .on_click(move |_, window, cx| {
                                import_focus.dispatch_action(&ImportLibrary, window, cx);
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
            .when(offer_sign_in, |el| {
                el.child(divider()).child(
                    h_flex()
                        .id("empty-sign-in")
                        .debug_selector(|| "empty-sign-in".into())
                        .w_full()
                        .my(px(2.))
                        .px(px(6.))
                        .py(px(7.))
                        .gap_2()
                        .items_start()
                        .rounded(px(6.))
                        .cursor_pointer()
                        .hover(|el| el.bg(colors.row_hover))
                        .text_size(px(13.))
                        .text_color(colors.text_secondary)
                        .on_click(move |_, window, cx| {
                            sign_in_focus.dispatch_action(&ShowSignIn, window, cx);
                        })
                        .child(
                            Icon::new(Lucide::LogIn)
                                .mt(px(2.))
                                .size(px(14.))
                                .text_color(colors.text_muted),
                        )
                        // Too long for one line of the column, so the
                        // question and the action each get one.
                        .child(
                            v_flex()
                                .flex_1()
                                .min_w_0()
                                .line_height(px(18.))
                                .child("Already use Kanso on another device?")
                                .child(div().text_color(colors.text).child("Sign in to sync")),
                        ),
                )
            })
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
