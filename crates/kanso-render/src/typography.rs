//! The numbers the preview and an exported page both set text by.
//!
//! They live here so the two can't drift: the preview reads them when it
//! lays a note out, and [`crate::html`] writes them into a page's
//! stylesheet. Sizes are ems of the note's own text size, or points where
//! noted.

/// The rendered note's line height, a multiple of its text size.
pub const LINE_HEIGHT: f32 = 1.65;

/// The widest prose runs, in ems of the note's size: about 75 characters a
/// line, however wide the window. SF Pro averages about 0.44 em a
/// character in prose.
pub const READABLE_EMS: f32 = 35.;
/// The same measure for notes set in a monospaced face. SF Mono averages
/// about 0.62 em a character.
pub const MONO_READABLE_EMS: f32 = 47.;

/// The gap below ordinary blocks, in ems of the note's size.
pub const PARAGRAPH_GAP: f32 = 0.9;
/// The gap below a heading, relative to the heading's own size.
pub const HEADING_BOTTOM_GAP: f32 = 0.35;

/// A code block's corner radius, in points.
pub const CODE_RADIUS: f32 = 8.;
/// The room above a code block's first line, clear of its language label.
pub const CODE_TOP: f32 = 34.;
/// A code block's horizontal padding, in points.
pub const CODE_PADDING_X: f32 = 16.;
/// The room below a code block's last line, in points.
pub const CODE_PADDING_BOTTOM: f32 = 14.;
/// A code block's line height, a multiple of its own size.
pub const CODE_LINE_HEIGHT: f32 = 1.6;
/// A code block's text size, in points, as the theme sets it.
pub const CODE_SIZE: f32 = 13.;

/// A heading's size and the room above it, relative to the body text (the
/// size) and to the heading itself (the gap).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Heading {
    pub scale: f32,
    pub gap: f32,
}

/// A heading's metrics: the size scale and the gap above it, as the preview
/// sets them. Levels outside 1 to 6 take the sixth's.
pub fn heading(level: u8) -> Heading {
    let (scale, gap) = match level {
        1 => (1.8, 1.1),
        2 => (4. / 3., 1.6),
        3 => (17. / 15., 1.4),
        4 => (1., 1.4),
        5 => (14. / 15., 1.4),
        _ => (13. / 15., 1.4),
    };
    Heading { scale, gap }
}

/// The readable measure for text set at `size`, in the monospaced face or
/// the proportional one.
pub fn readable_width(size: f32, mono: bool) -> f32 {
    size * if mono {
        MONO_READABLE_EMS
    } else {
        READABLE_EMS
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(128))]

        /// Invariant: the measure grows with the text size, and the
        /// monospaced measure is always the wider one.
        #[test]
        fn prop_measure_grows_with_size(a in 6f32..=72., b in 6f32..=72.) {
            let (small, large) = if a <= b { (a, b) } else { (b, a) };
            prop_assert!(readable_width(small, false) <= readable_width(large, false));
            prop_assert!(readable_width(small, true) <= readable_width(large, true));
            prop_assert!(readable_width(small, true) >= readable_width(small, false));
        }

        /// Invariant: every heading level scales down from the one before it,
        /// never grows, and stays within 1 em of the body.
        #[test]
        fn prop_headings_step_down(level in 1u8..=6) {
            let h = heading(level);
            prop_assert!(h.scale > 0. && h.scale <= 1.8);
            prop_assert!(h.gap >= 1.0);
            if level > 1 {
                prop_assert!(heading(level - 1).scale >= h.scale);
            }
        }
    }

    #[test]
    fn the_default_measure_is_the_previews() {
        assert_eq!(readable_width(16., false), 560.);
        assert!(readable_width(16., true) > readable_width(16., false));
        assert_eq!(
            heading(1),
            Heading {
                scale: 1.8,
                gap: 1.1
            }
        );
        assert_eq!(heading(6).scale, 13. / 15.);
    }
}
