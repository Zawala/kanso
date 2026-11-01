//! The clipboard sticky: a sticky (`sticky.rs`) that lists the last texts
//! copied while Kanso ran (`clipboard_history.rs`), newest first; a click
//! copies one again. "Show Clipboard History" in the palette opens it.
//!
//! The rows are masked until hovered, or until the eye in the header shows
//! them all, since a sticky stays on screen, in screenshots and in shared
//! screens too. The header also pauses collecting, and forgets every copy.
//! The sticky opens again where it was left, with its colour; what it
//! showed is gone at the next launch.

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::notification::Notification;
use gpui_kit::component::{ActiveTheme as _, Icon, Selectable as _, Sizable as _, WindowExt as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::clipboard_history::{self, ClipboardHistory, History, first_line, mask};
use crate::clipboard_watch::Support;
use crate::clock::Clock;
use crate::config::{StickyColor, StickyFrame};
use crate::note_window::{Frame, frame_of};
use crate::theme::Colors;
use crate::workspace::AppState;

actions!(kanso, [ShowClipboardHistory, CloseClipboardSticky]);

pub fn bind_keys(cx: &mut App) {
    cx.bind_keys([KeyBinding::new(
        "secondary-w",
        CloseClipboardSticky,
        Some("ClipboardSticky"),
    )]);
    cx.on_action(|_: &ShowClipboardHistory, cx| {
        // Not from inside a window's update: the windows are looked through.
        cx.defer(open);
    });
}

/// The clipboard sticky's window, when open.
pub(crate) fn find(cx: &mut App) -> Option<(AnyWindowHandle, Entity<ClipboardSticky>)> {
    cx.windows().into_iter().find_map(|handle| {
        let view = handle
            .update(cx, |root, _, cx| {
                let root = root.downcast::<gpui_kit::base::Root>().ok()?;
                root.read(cx)
                    .view()
                    .clone()
                    .downcast::<ClipboardSticky>()
                    .ok()
            })
            .ok()
            .flatten()?;
        Some((handle, view))
    })
}

/// Whether the clipboard sticky is open.
pub fn is_open(cx: &mut App) -> bool {
    find(cx).is_some()
}

/// Opens the clipboard sticky where it was left, or brings it forward.
pub fn open(cx: &mut App) {
    if let Some((handle, _)) = find(cx) {
        handle
            .update(cx, |_, window, _| window.activate_window())
            .ok();
        return;
    }
    let saved = cx.global::<AppState>().config.layout.clipboard_sticky;
    let color = saved.map(|f| f.color).unwrap_or(StickyColor::Blue);
    let frame = saved.map(|f| (f.x, f.y, f.width, f.height));
    let opened = crate::sticky::open(frame, cx, move |layer_shell, frame, window, cx| {
        cx.new(|cx| ClipboardSticky::new(color, layer_shell.then_some(frame), window, cx))
    });
    let Some((handle, view)) = opened else {
        return;
    };
    handle
        .update(cx, |_, window, cx| {
            view.update(cx, |view, cx| view.remember_frame(window, cx));
            let view = view.clone();
            window.on_window_should_close(cx, move |_, cx| {
                view.update(cx, |view, cx| view.put_away(cx));
                true
            });
        })
        .ok();
}

/// Opens the clipboard sticky at launch if it was open at the last quit.
pub fn restore(cx: &mut App) {
    let open_at_quit = cx
        .global::<AppState>()
        .config
        .layout
        .clipboard_sticky
        .is_some_and(|f| !f.closed);
    if open_at_quit {
        open(cx);
    }
}

pub struct ClipboardSticky {
    color: StickyColor,
    /// A layer surface's frame, which the compositor keeps.
    fixed_frame: Option<Frame>,
    /// Every row shown in full, not only the one hovered.
    shown: bool,
    hovered: Option<usize>,
    closing: bool,
    focus: FocusHandle,
    _history: Subscription,
}

impl ClipboardSticky {
    fn new(
        color: StickyColor,
        fixed_frame: Option<Frame>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        window.set_window_title("Clipboard");
        let history = cx.global::<ClipboardHistory>().history.clone();
        let observed = cx.observe(&history, |_, _, cx| cx.notify());
        // Kept where it is as it moves.
        cx.observe_window_bounds(window, |this, window, cx| this.remember_frame(window, cx))
            .detach();
        let focus = cx.focus_handle();
        focus.focus(window, cx);
        ClipboardSticky {
            color,
            fixed_frame,
            shown: false,
            hovered: None,
            closing: false,
            focus,
            _history: observed,
        }
    }

    fn history(&self, cx: &App) -> Entity<History> {
        cx.global::<ClipboardHistory>().history.clone()
    }

    /// Keeps where the window is and its colour, to open there again.
    fn remember_frame(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.closing {
            return;
        }
        let (x, y, width, height) = self
            .fixed_frame
            .unwrap_or_else(|| frame_of(window.bounds()));
        let frame = StickyFrame {
            x,
            y,
            width,
            height,
            color: self.color,
            closed: false,
        };
        if cx.global::<AppState>().config.layout.clipboard_sticky == Some(frame) {
            return;
        }
        AppState::update_config_soon(cx, move |config| {
            config.layout.clipboard_sticky = Some(frame);
        });
    }

    /// Not opened at the next launch; where it was is kept.
    fn put_away(&mut self, cx: &mut Context<Self>) {
        self.closing = true;
        AppState::update_config(cx, |config| {
            if let Some(frame) = config.layout.clipboard_sticky.as_mut() {
                frame.closed = true;
            }
        });
    }

    pub(crate) fn copy(&mut self, ix: usize, window: &mut Window, cx: &mut Context<Self>) {
        let text = self
            .history(cx)
            .read(cx)
            .items
            .get(ix)
            .map(|clip| clip.text.clone());
        if let Some(text) = text {
            cx.write_to_clipboard(ClipboardItem::new_string(text));
            window.push_notification(Notification::info("Copied."), cx);
        }
    }

    fn render_header(&self, dark: bool, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let paused = self.history(cx).read(cx).paused;
        let title = if paused {
            "Clipboard · paused"
        } else {
            "Clipboard"
        };
        let button = |id: &'static str, icon: Lucide, tooltip: &'static str| {
            Button::new(id)
                .ghost()
                .xsmall()
                .icon(Icon::new(icon).text_color(colors.text_secondary))
                .tooltip(tooltip)
        };
        let shown = self.shown;
        crate::sticky::header(
            title.into(),
            self.color,
            dark,
            self.fixed_frame.is_none(),
            cx,
        )
        .child(
            button(
                "clipboard-pause",
                Lucide::CirclePause,
                if paused { "Resume" } else { "Pause" },
            )
            .selected(paused)
            .on_click(cx.listener(|this, _, _, cx| {
                this.history(cx).update(cx, |history, cx| {
                    history.paused = !history.paused;
                    cx.notify();
                });
            })),
        )
        .child(
            button(
                "clipboard-show",
                if shown { Lucide::EyeOff } else { Lucide::Eye },
                if shown {
                    "Hide contents"
                } else {
                    "Show contents"
                },
            )
            .on_click(cx.listener(|this, _, _, cx| {
                this.shown = !this.shown;
                cx.notify();
            })),
        )
        .child(
            button("clipboard-clear", Lucide::Trash, "Forget all copies").on_click(cx.listener(
                |this, _, _, cx| {
                    this.history(cx).update(cx, |history, cx| {
                        history.clear();
                        cx.notify();
                    });
                },
            )),
        )
        .child(
            button("clipboard-color", Lucide::Palette, "Change colour").on_click(cx.listener(
                |this, _, window, cx| {
                    this.color = this.color.next();
                    this.remember_frame(window, cx);
                    cx.notify();
                },
            )),
        )
        .child(
            button("clipboard-close", Lucide::Close, "Close").on_click(|_, window, cx| {
                window.dispatch_action(CloseClipboardSticky.boxed_clone(), cx);
            }),
        )
    }

    /// What the sticky says with no copies to show, and the button that
    /// helps.
    fn render_empty(&self, cx: &mut Context<Self>) -> AnyElement {
        let colors = Colors::get(cx);
        let support = cx.global::<ClipboardHistory>().support;
        let (message, action): (&str, Option<AnyElement>) = if !clipboard_history::enabled(cx) {
            (
                "Clipboard history is off. Turned on, it keeps the last 15 texts you copy, in memory only, while Kanso runs. Passwords and text that looks like a key are skipped.",
                Some(
                    Button::new("clipboard-turn-on")
                        .primary()
                        .small()
                        .label("Turn On")
                        .on_click(|_, _, cx| clipboard_history::set_enabled(true, cx))
                        .into_any_element(),
                ),
            )
        } else if support == Support::NeedsXWayland {
            (
                "This desktop lets only the window in front read the clipboard. Turn on Run under XWayland in Settings › General and restart Kanso.",
                Some(
                    Button::new("clipboard-settings")
                        .outline()
                        .small()
                        .label("Open Settings")
                        .on_click(|_, _, cx| {
                            cx.dispatch_action(&crate::settings::OpenGeneralSettings)
                        })
                        .into_any_element(),
                ),
            )
        } else if support == Support::Unsupported {
            ("Clipboard history isn't available on this system.", None)
        } else if self.history(cx).read(cx).paused {
            ("Paused: copies aren't kept until you resume.", None)
        } else {
            (
                "Nothing copied yet. Copy some text and it shows here.",
                None,
            )
        };
        div()
            .flex()
            .flex_col()
            .gap_2()
            .p_3()
            .text_sm()
            .text_color(colors.text_secondary)
            .child(message)
            .children(action)
            .into_any_element()
    }

    fn render_rows(&self, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let colors = Colors::get(cx);
        let mono = cx.theme().mono_font_family.clone();
        let now = Clock::now_ms(cx);
        let history = self.history(cx);
        let items: Vec<_> = history.read(cx).items.iter().cloned().collect();
        items
            .into_iter()
            .enumerate()
            .map(|(ix, clip)| {
                let open = self.shown || self.hovered == Some(ix);
                let line = if open {
                    first_line(&clip.text).to_owned()
                } else {
                    mask(&clip.text)
                };
                let more = clip.text.trim().lines().count().saturating_sub(1);
                div()
                    .id(("clip", ix))
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_3()
                    .py_1()
                    .w_full()
                    .cursor_pointer()
                    .hover(|el| el.bg(colors.hover))
                    .on_hover(cx.listener(move |this, hovered: &bool, _, cx| {
                        if *hovered {
                            this.hovered = Some(ix);
                        } else if this.hovered == Some(ix) {
                            this.hovered = None;
                        }
                        cx.notify();
                    }))
                    .on_click(cx.listener(move |this, _, window, cx| this.copy(ix, window, cx)))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .text_ellipsis()
                            .text_sm()
                            .font_family(mono.clone())
                            .text_color(colors.text)
                            .child(line),
                    )
                    .when(more > 0, |el| {
                        el.child(
                            div()
                                .flex_none()
                                .text_xs()
                                .text_color(colors.text_secondary)
                                .child(format!("+{more}")),
                        )
                    })
                    .child(
                        div()
                            .flex_none()
                            .text_xs()
                            .text_color(colors.text_secondary)
                            .child(ago(now - clip.copied_ms)),
                    )
                    .into_any_element()
            })
            .collect()
    }
}

/// How long ago a copy was made, shortly: "now", "5m", "2h". Pure.
pub fn ago(ms: i64) -> String {
    let minutes = ms.max(0) / 60_000;
    match minutes {
        0 => "now".to_owned(),
        1..=59 => format!("{minutes}m"),
        _ => format!("{}h", minutes / 60),
    }
}

impl Render for ClipboardSticky {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let dark = cx.theme().is_dark();
        let rows = self.render_rows(cx);
        let body = if rows.is_empty() {
            self.render_empty(cx)
        } else {
            div()
                .id("clipboard-rows")
                .size_full()
                .py_1()
                .overflow_y_scroll()
                .children(rows)
                .into_any_element()
        };
        div()
            .size_full()
            .relative()
            .flex()
            .flex_col()
            .key_context("ClipboardSticky")
            .track_focus(&self.focus)
            .bg(colors.editor)
            .text_color(cx.theme().foreground)
            .on_action(cx.listener(|this, _: &CloseClipboardSticky, window, cx| {
                this.put_away(cx);
                window.defer(cx, |window, _| window.remove_window());
            }))
            .child(self.render_header(dark, cx))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .bg(crate::sticky::wash(self.color, dark))
                    .child(body),
            )
            .when(self.fixed_frame.is_none(), |el| {
                el.children(crate::sticky::resize_handles())
            })
    }
}

#[cfg(test)]
mod tests {
    use super::ago;

    #[test]
    fn says_how_long_ago_shortly() {
        assert_eq!(ago(-5), "now");
        assert_eq!(ago(59_000), "now");
        assert_eq!(ago(5 * 60_000), "5m");
        assert_eq!(ago(2 * 3_600_000 + 1), "2h");
    }
}
