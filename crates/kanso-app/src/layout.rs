//! What the panes share so the window reads as one surface: pane widths, a
//! top band and a bottom band of the same height in every pane, and the room
//! macOS's traffic lights take in the leftmost pane's top band.
//!
//! The title bar is transparent and the app moves the window itself: a press
//! in a top band that no control claims, then a move, drags the window, and
//! a double-click there does what the system setting says (zoom or
//! minimize). [`BandDrag`] decides when.

use gpui_kit::{Pixels, Point};

/// The band at the top of every pane: the title bar's row.
pub const TOP_BAND_HEIGHT: f32 = 52.;
/// The band at the bottom of every pane: counts, sync and list controls.
pub const BOTTOM_BAND_HEIGHT: f32 = 36.;

pub const SIDEBAR_WIDTH: f32 = 220.;
pub const NOTE_LIST_WIDTH: f32 = 320.;

/// Where macOS puts the close button, from the window's top left. The three
/// buttons are about 14 points across, 20 apart.
pub const TRAFFIC_LIGHTS: (f32, f32) = (16., 18.);
/// The start of the leftmost pane's top band that the traffic lights cover.
pub const TRAFFIC_LIGHT_INSET: f32 = 76.;

/// How far the pointer moves with the button down before a press in a top
/// band becomes a window drag, so a click that wobbles stays a click.
pub const DRAG_THRESHOLD: f32 = 3.;

/// The room at the start of a pane's top band: the traffic lights' in the
/// leftmost pane, and none in full screen, where they hide with the menu bar.
// no-prop-test: a two-way choice; `leading_inset_examples` covers it.
pub fn leading_inset(leftmost: bool, fullscreen: bool) -> f32 {
    if leftmost && !fullscreen {
        TRAFFIC_LIGHT_INSET
    } else {
        0.
    }
}

/// What a pointer event in the window asks of it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BandAction {
    None,
    /// Hand the drag to the system: the window follows the pointer.
    StartMove,
    /// A double-click on the title bar: zoom or minimize, as set in System
    /// Settings.
    TitlebarDoubleClick,
}

/// Turns presses and moves in the top bands into window drags. A press
/// counts when it lands in a band and no control under it claimed it.
#[derive(Clone, Copy, Debug, Default)]
pub struct BandDrag {
    /// Where an unclaimed press in a band went down, until it is released,
    /// moved far enough or handed over.
    pressed_at: Option<Point<Pixels>>,
}

impl BandDrag {
    pub fn is_armed(&self) -> bool {
        self.pressed_at.is_some()
    }

    /// The left button went down at `position`, the `click_count`th click in
    /// a row. `claimed` is whether a control under the pointer took it.
    pub fn down(
        &mut self,
        position: Point<Pixels>,
        click_count: usize,
        claimed: bool,
    ) -> BandAction {
        self.pressed_at = None;
        if claimed || f32::from(position.y) >= TOP_BAND_HEIGHT || f32::from(position.y) < 0. {
            return BandAction::None;
        }
        if click_count == 2 {
            return BandAction::TitlebarDoubleClick;
        }
        if click_count == 1 {
            self.pressed_at = Some(position);
        }
        BandAction::None
    }

    /// The pointer moved to `position`, with the left button `pressed` or
    /// not.
    pub fn moved(&mut self, position: Point<Pixels>, pressed: bool) -> BandAction {
        let Some(from) = self.pressed_at else {
            return BandAction::None;
        };
        if !pressed {
            self.pressed_at = None;
            return BandAction::None;
        }
        let (dx, dy) = (
            f32::from(position.x - from.x),
            f32::from(position.y - from.y),
        );
        if dx.hypot(dy) >= DRAG_THRESHOLD {
            self.pressed_at = None;
            BandAction::StartMove
        } else {
            BandAction::None
        }
    }

    pub fn up(&mut self) {
        self.pressed_at = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::px;

    fn point_at(x: f32, y: f32) -> Point<Pixels> {
        gpui_kit::point(px(x), px(y))
    }
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 512,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    #[derive(Clone, Debug)]
    enum Event {
        Down {
            x: f32,
            y: f32,
            clicks: usize,
            claimed: bool,
        },
        Move {
            x: f32,
            y: f32,
            pressed: bool,
        },
        Up,
    }

    fn event() -> impl Strategy<Value = Event> {
        prop_oneof![
            (0f32..400., -10f32..120., 1usize..4, any::<bool>()).prop_map(
                |(x, y, clicks, claimed)| Event::Down {
                    x,
                    y,
                    clicks,
                    claimed
                }
            ),
            (0f32..400., -10f32..120., any::<bool>()).prop_map(|(x, y, pressed)| Event::Move {
                x,
                y,
                pressed
            }),
            Just(Event::Up),
        ]
    }

    proptest! {
        #![proptest_config(config())]

        /// Model-based: a window move starts only after a single, unclaimed
        /// press inside a top band, while the button stays down with no
        /// release or other press in between, once the pointer is at least
        /// the threshold away from the press; and at most once per press.
        /// A double-click acts only inside a band and when unclaimed.
        #[test]
        fn prop_moves_start_only_from_unclaimed_band_presses(events in prop::collection::vec(event(), 0..40)) {
            let mut drag = BandDrag::default();
            // The model: the live press, and whether it has moved the window.
            let mut press: Option<(f32, f32)> = None;
            for event in events {
                match event {
                    Event::Down { x, y, clicks, claimed } => {
                        let action = drag.down(point_at(x, y), clicks, claimed);
                        let in_band = (0. ..TOP_BAND_HEIGHT).contains(&y);
                        let double = in_band && !claimed && clicks == 2;
                        prop_assert_eq!(action == BandAction::TitlebarDoubleClick, double);
                        prop_assert!(action != BandAction::StartMove);
                        press = (in_band && !claimed && clicks == 1).then_some((x, y));
                    }
                    Event::Move { x, y, pressed } => {
                        let action = drag.moved(point_at(x, y), pressed);
                        let far = press.is_some_and(|(px, py)| (x - px).hypot(y - py) >= DRAG_THRESHOLD);
                        let expected = pressed && far;
                        prop_assert_eq!(action == BandAction::StartMove, expected, "{:?}", press);
                        if !pressed || expected {
                            press = None;
                        }
                    }
                    Event::Up => {
                        drag.up();
                        press = None;
                    }
                }
                prop_assert_eq!(drag.is_armed(), press.is_some());
            }
        }
    }

    proptest! {
        #![proptest_config(config())]

        /// The threshold decides: after an unclaimed press in a band, a move
        /// with the button down starts a window move exactly when it went at
        /// least the threshold away, in any direction.
        #[test]
        fn prop_threshold_decides_a_drag(
            x in 0f32..400.,
            y in 0f32..TOP_BAND_HEIGHT,
            angle in 0f32..std::f32::consts::TAU,
            distance in 0f32..2. * DRAG_THRESHOLD,
        ) {
            // Rounding makes a move right at the threshold go either way.
            prop_assume!((distance - DRAG_THRESHOLD).abs() > 1e-3);
            let mut drag = BandDrag::default();
            prop_assert_eq!(drag.down(point_at(x, y), 1, false), BandAction::None);
            let to = point_at(x + distance * angle.cos(), y + distance * angle.sin());
            let expected = if distance >= DRAG_THRESHOLD {
                BandAction::StartMove
            } else {
                BandAction::None
            };
            prop_assert_eq!(drag.moved(to, true), expected);
            prop_assert_eq!(drag.is_armed(), expected == BandAction::None);
        }
    }

    #[test]
    fn band_drag_examples() {
        let mut drag = BandDrag::default();
        // A click on a control in the band never drags.
        assert_eq!(drag.down(point_at(100., 20.), 1, true), BandAction::None);
        assert_eq!(drag.moved(point_at(140., 20.), true), BandAction::None);
        // A press on empty band, a wobble, then a real move.
        assert_eq!(drag.down(point_at(100., 20.), 1, false), BandAction::None);
        assert_eq!(drag.moved(point_at(101., 21.), true), BandAction::None);
        assert_eq!(drag.moved(point_at(102.9, 20.), true), BandAction::None);
        assert_eq!(drag.moved(point_at(103., 20.), true), BandAction::StartMove);
        assert_eq!(drag.moved(point_at(110., 20.), true), BandAction::None);
        // A press that is released doesn't drag on the next move.
        assert_eq!(drag.down(point_at(100., 20.), 1, false), BandAction::None);
        drag.up();
        assert_eq!(drag.moved(point_at(110., 20.), true), BandAction::None);
        assert_eq!(drag.down(point_at(100., 20.), 1, false), BandAction::None);
        assert_eq!(drag.moved(point_at(110., 20.), true), BandAction::StartMove);
        assert_eq!(drag.moved(point_at(130., 20.), true), BandAction::None);
        // Below the band is the panes' content.
        assert_eq!(drag.down(point_at(100., 60.), 2, false), BandAction::None);
        assert_eq!(
            drag.down(point_at(100., 51.), 2, false),
            BandAction::TitlebarDoubleClick
        );
    }

    #[test]
    fn leading_inset_examples() {
        assert_eq!(leading_inset(true, false), TRAFFIC_LIGHT_INSET);
        assert_eq!(leading_inset(true, true), 0.);
        assert_eq!(leading_inset(false, false), 0.);
        // The inset clears the third light with a margin.
        assert!(TRAFFIC_LIGHTS.0 + 2. * 20. + 14. < TRAFFIC_LIGHT_INSET);
        assert!(TRAFFIC_LIGHTS.1 + 14. < TOP_BAND_HEIGHT);
    }
}
