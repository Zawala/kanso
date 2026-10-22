//! Zoom: the whole app larger or smaller, in every window at once, as
//! View › Zoom In, Zoom Out and Actual Size (⌘=, ⌘− and ⌘0) set it.
//!
//! GPUI has no window-wide scale apart from the display's, so zoom works
//! in two places. Kanso's own sizes are written with [`px`], which this
//! module shadows: every file imports it ahead of `gpui_kit::*`, and it
//! multiplies by the zoom in force. The component library's buttons,
//! inputs, menus and dialogs are sized in rems, which follow the theme's
//! font size; [`apply_to_theme`] scales that. A size measured from the
//! window (its width, the pointer, a pane's bounds) is already zoomed:
//! [`unscaled`] turns it back into the units the layout is written in.

use std::cell::Cell;

use gpui_kit::component::Theme;
use gpui_kit::{Pixels, actions};

use crate::workspace::AppState;

actions!(kanso, [ZoomIn, ZoomOut, ActualSize]);

/// The smallest and largest zoom, in percent.
pub const ZOOM_RANGE: (u32, u32) = (80, 150);
/// How far one Zoom In or Zoom Out goes, in percent.
pub const ZOOM_STEP: u32 = 10;
/// Actual size.
pub const DEFAULT_ZOOM: u32 = 100;

/// The component library's text size at actual size, and its monospace
/// one: the theme's own defaults, which Kanso's themes leave alone.
const BASE_FONT_SIZE: f32 = 16.;
const BASE_MONO_FONT_SIZE: f32 = 13.;

thread_local! {
    /// The zoom in force as a factor (1 is actual size). Windows are drawn
    /// on the main thread; each test thread starts at actual size.
    static SCALE: Cell<f32> = const { Cell::new(1.) };
}

/// A zoom as Kanso shows and keeps it: a whole step within
/// [`ZOOM_RANGE`]. A value from an edited settings file is brought to the
/// nearest one.
pub fn normalize(percent: u32) -> u32 {
    let clamped = percent.clamp(ZOOM_RANGE.0, ZOOM_RANGE.1);
    let steps = (clamped - ZOOM_RANGE.0 + ZOOM_STEP / 2) / ZOOM_STEP;
    (ZOOM_RANGE.0 + steps * ZOOM_STEP).min(ZOOM_RANGE.1)
}

/// The zoom `steps` steps from `percent` (negative is smaller), stopping
/// at either end of [`ZOOM_RANGE`].
pub fn stepped(percent: u32, steps: i32) -> u32 {
    let from = i64::from(normalize(percent));
    let to = from + i64::from(steps) * i64::from(ZOOM_STEP);
    let to = to.clamp(i64::from(ZOOM_RANGE.0), i64::from(ZOOM_RANGE.1));
    u32::try_from(to).map_or(DEFAULT_ZOOM, normalize)
}

/// The zoom in force, as a factor.
pub fn scale() -> f32 {
    SCALE.with(Cell::get)
}

/// `value` layout units at the zoom in force. Shadows `gpui_kit::px`;
/// a size that must stay in the window's own pixels (one measured from
/// it, or handed to the system) uses `gpui_kit::px`.
pub fn px(value: f32) -> Pixels {
    gpui_kit::px(value * scale())
}

/// A size measured in the window, in the layout's own units: what [`px`]
/// would have been given to draw it.
pub fn unscaled(value: Pixels) -> f32 {
    f32::from(value) / scale()
}

/// Puts `percent` in force for the next frame, without saving it.
pub fn set(percent: u32, cx: &mut gpui_kit::App) {
    SCALE.with(|scale| scale.set(normalize(percent) as f32 / 100.));
    apply_to_theme(cx);
    cx.refresh_windows();
}

/// Scales the component library's text, and so its rems, to the zoom.
pub fn apply_to_theme(cx: &mut gpui_kit::App) {
    let theme = Theme::global_mut(cx);
    theme.font_size = px(BASE_FONT_SIZE);
    theme.mono_font_size = px(BASE_MONO_FONT_SIZE);
}

/// Zooms `steps` steps from the saved zoom (0 is back to actual size),
/// saves it, and shows every window at it.
pub fn step(steps: i32, cx: &mut gpui_kit::App) {
    let current = cx.global::<AppState>().config.appearance.zoom;
    let next = if steps == 0 {
        DEFAULT_ZOOM
    } else {
        stepped(current, steps)
    };
    if next != current {
        AppState::update_config(cx, |config| config.appearance.zoom = next);
    }
    set(next, cx);
}

/// Answers the zoom actions in every window, whatever has focus (the graph
/// takes the same keys for its own zoom while it has focus).
pub fn register(cx: &mut gpui_kit::App) {
    cx.on_action(|_: &ZoomIn, cx| step(1, cx));
    cx.on_action(|_: &ZoomOut, cx| step(-1, cx));
    cx.on_action(|_: &ActualSize, cx| step(0, cx));
}

pub fn bind_keys(cx: &mut gpui_kit::App) {
    use gpui_kit::KeyBinding;
    // `secondary` is ⌘ on macOS and Ctrl elsewhere. ⌘= is ⌘+ without
    // Shift; both zoom in.
    cx.bind_keys([
        KeyBinding::new("secondary-=", ZoomIn, None),
        KeyBinding::new("secondary-+", ZoomIn, None),
        KeyBinding::new("secondary--", ZoomOut, None),
        KeyBinding::new("secondary-0", ActualSize, None),
    ]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    #[test]
    fn steps_through_the_range() {
        assert_eq!(stepped(100, 1), 110);
        assert_eq!(stepped(100, -1), 90);
        assert_eq!(stepped(150, 1), 150);
        assert_eq!(stepped(80, -1), 80);
        assert_eq!(stepped(80, 7), 150);
        assert_eq!(stepped(150, -7), 80);
        assert_eq!(normalize(0), 80);
        assert_eq!(normalize(104), 100);
        assert_eq!(normalize(105), 110);
        assert_eq!(normalize(u32::MAX), 150);
    }

    #[gpui_kit::test]
    fn px_follows_the_zoom(cx: &mut gpui_kit::TestAppContext) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            set(120, cx);
            assert_eq!(px(10.), gpui_kit::px(12.));
            assert_eq!(unscaled(gpui_kit::px(12.)), 10.);
            assert_eq!(Theme::global(cx).font_size, gpui_kit::px(16. * 1.2));
            set(DEFAULT_ZOOM, cx);
            assert_eq!(px(10.), gpui_kit::px(10.));
        });
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        })]

        /// A kept zoom is a whole step within the range, the nearest one,
        /// and keeping it again changes nothing.
        #[test]
        fn prop_normalize_is_the_nearest_step(percent in any::<u32>()) {
            let kept = normalize(percent);
            prop_assert!((ZOOM_RANGE.0..=ZOOM_RANGE.1).contains(&kept));
            prop_assert_eq!((kept - ZOOM_RANGE.0) % ZOOM_STEP, 0);
            prop_assert_eq!(normalize(kept), kept);
            let wanted = percent.clamp(ZOOM_RANGE.0, ZOOM_RANGE.1);
            prop_assert!(kept.abs_diff(wanted) <= ZOOM_STEP / 2);
        }

        /// A step moves one step unless it is at that end already; going
        /// back undoes it away from the ends, and steps add up.
        #[test]
        fn prop_steps_stay_in_range_and_add_up(
            percent in any::<u32>(),
            a in -20i32..20,
            b in -20i32..20,
        ) {
            let from = normalize(percent);
            let one = stepped(from, a);
            prop_assert!((ZOOM_RANGE.0..=ZOOM_RANGE.1).contains(&one));
            prop_assert_eq!(normalize(one), one);
            let free = i64::from(from) + i64::from(a) * i64::from(ZOOM_STEP);
            if (i64::from(ZOOM_RANGE.0)..=i64::from(ZOOM_RANGE.1)).contains(&free) {
                prop_assert_eq!(i64::from(one), free);
                prop_assert_eq!(stepped(one, -a), from);
            }
            if a.signum() == b.signum() || a == 0 || b == 0 {
                prop_assert_eq!(stepped(stepped(from, a), b), stepped(from, a + b));
            }
        }

        /// `unscaled` undoes `px` at every zoom.
        #[test]
        fn prop_unscaled_undoes_px(steps in -3i32..6, value in 0f32..4000.) {
            SCALE.with(|scale| scale.set(stepped(DEFAULT_ZOOM, steps) as f32 / 100.));
            let back = unscaled(px(value));
            prop_assert!((back - value).abs() <= value.max(1.) * 1e-5);
            SCALE.with(|scale| scale.set(1.));
        }
    }
}
