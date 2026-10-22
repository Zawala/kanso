//! The one time Kanso brings up sync to someone who hasn't set it up: a
//! short note at the foot of the note list, once the library shows they
//! use Kanso for real. "Not now" or "Set up sync" puts it away for good;
//! the sidebar's "Set up sync" line stays for whenever they want it.

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{Icon, Sizable as _, StyledExt as _, h_flex, v_flex};
use gpui_kit::*;

use crate::settings::OpenAccountSettings;
use crate::theme::Colors;
use crate::workspace::AppState;
use crate::zoom::px;

/// Notes that show the library is in real use, whatever its age.
pub const MANY_NOTES: u32 = 25;
/// Notes that, a week after Kanso was first opened, show the same.
pub const SOME_NOTES: u32 = 5;
pub const WEEK_MS: i64 = 7 * 24 * 60 * 60 * 1000;

/// Whether to offer sync now: never while signed in or once put away, and
/// otherwise when there are many notes, or some notes a week after Kanso
/// was first opened (`first_seen_ms`).
pub fn offer_sync(
    signed_out: bool,
    dismissed: bool,
    notes: u32,
    first_seen_ms: Option<i64>,
    now_ms: i64,
) -> bool {
    let a_week_in = first_seen_ms.is_some_and(|first| now_ms.saturating_sub(first) >= WEEK_MS);
    signed_out && !dismissed && (notes >= MANY_NOTES || (notes >= SOME_NOTES && a_week_in))
}

/// Puts the offer away for good.
fn dismiss(cx: &mut App) {
    AppState::update_config(cx, |config| config.nudges.sync_offer_dismissed = true);
}

/// The offer, for the foot of the note list.
// no-prop-test: layout and two clicks; `offer_sync` decides when it shows.
pub fn render(colors: &Colors) -> impl IntoElement {
    v_flex()
        .id("sync-offer")
        .debug_selector(|| "sync-offer".into())
        .flex_none()
        .mx(px(10.))
        .mb(px(10.))
        .p(px(12.))
        .gap(px(8.))
        .rounded(px(8.))
        .border_1()
        .border_color(colors.divider)
        .bg(colors.elevated)
        .child(
            h_flex()
                .gap(px(8.))
                .items_start()
                .child(
                    Icon::new(Lucide::Cloud)
                        .mt(px(1.))
                        .size(px(15.))
                        .text_color(colors.text_secondary),
                )
                .child(
                    v_flex()
                        .flex_1()
                        .min_w_0()
                        .gap(px(2.))
                        .child(
                            div()
                                .text_size(px(13.))
                                .line_height(px(18.))
                                .font_medium()
                                .text_color(colors.text)
                                .child("Use Kanso on another device?"),
                        )
                        .child(
                            div()
                                .text_size(px(12.5))
                                .line_height(px(17.))
                                .text_color(colors.text_secondary)
                                .child("Sync keeps your notes on all of them and backs them up, locked so only you can read them. It's free."),
                        ),
                ),
        )
        .child(
            h_flex()
                .gap(px(6.))
                .justify_end()
                .child(
                    Button::new("sync-offer-later")
                        .ghost()
                        .small()
                        .label("Not now")
                        .on_click(|_, _, cx| dismiss(cx)),
                )
                .child(
                    Button::new("sync-offer-set-up")
                        .primary()
                        .small()
                        .label("Set up sync")
                        .on_click(|_, window, cx| {
                            dismiss(cx);
                            window.dispatch_action(Box::new(OpenAccountSettings), cx);
                        }),
                ),
        )
}

#[cfg(test)]
mod tests {
    use super::{MANY_NOTES, SOME_NOTES, WEEK_MS, offer_sync};
    use proptest::prelude::*;

    #[test]
    fn offer_examples() {
        let now = 100 * WEEK_MS;
        // A fresh library: not yet.
        assert!(!offer_sync(true, false, 3, Some(now), now));
        // Many notes on day one.
        assert!(offer_sync(true, false, MANY_NOTES, Some(now), now));
        // Some notes, a week in.
        assert!(offer_sync(
            true,
            false,
            SOME_NOTES,
            Some(now - WEEK_MS),
            now
        ));
        assert!(!offer_sync(
            true,
            false,
            SOME_NOTES,
            Some(now - WEEK_MS + 1),
            now
        ));
        // Unknown start: only many notes count.
        assert!(!offer_sync(true, false, SOME_NOTES, None, now));
        // Signed in, or put away: never.
        assert!(!offer_sync(false, false, 500, Some(0), now));
        assert!(!offer_sync(true, true, 500, Some(0), now));
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: 256, ..ProptestConfig::default() })]

        /// Never while signed in or put away; more notes or more time never
        /// takes an offer back.
        #[test]
        fn prop_offer_is_monotonic(
            (signed_out, dismissed) in (any::<bool>(), any::<bool>()),
            (notes, more) in (0u32..60, 0u32..60),
            first in proptest::option::of(0i64..(20 * WEEK_MS)),
            (now, later) in ((20 * WEEK_MS)..(40 * WEEK_MS), 0i64..(4 * WEEK_MS)),
        ) {
            let offered = offer_sync(signed_out, dismissed, notes, first, now);
            if !signed_out || dismissed {
                prop_assert!(!offered);
            }
            if offered {
                prop_assert!(offer_sync(signed_out, dismissed, notes + more, first, now));
                prop_assert!(offer_sync(signed_out, dismissed, notes, first, now + later));
            }
        }
    }
}
