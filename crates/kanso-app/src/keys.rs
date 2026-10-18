//! Keyboard shortcuts on screen: one bordered chip for a shortcut wherever
//! one shows outside a row or a tooltip (the search box, the empty states,
//! the palette's detail pane, the "esc" hints), so they all read alike and
//! none goes stale when a binding changes.

use gpui_kit::*;

use crate::palette::shortcut;
use crate::theme::Colors;

/// A key or a whole shortcut ("⌘K", "⇧⌘F", "esc") in a bordered chip.
// no-prop-test: drawing; the snapshots show it.
pub(crate) fn keycap(text: impl Into<SharedString>, cx: &App) -> Div {
    let colors = Colors::get(cx);
    div()
        .flex_none()
        .min_w(px(20.))
        .h(px(20.))
        .px(px(5.))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(4.))
        .border_1()
        .border_color(colors.divider)
        .bg(colors.editor)
        .text_size(px(11.5))
        .text_color(colors.text_secondary)
        .child(text.into())
}

/// The chip for the keys that run `action` now, or nothing when it has no
/// binding.
// no-prop-test: reads the keymap into `keycap`; `shortcut` has the tests.
pub(crate) fn shortcut_chip(action: &dyn Action, window: &Window, cx: &App) -> Option<Div> {
    shortcut(action, window).map(|keys| keycap(keys.concat(), cx))
}
