//! A render-only presentation of the code editor's text: ranges laid out at zero
//! width or drawn as other text, lines drawn larger, with space around them, inset
//! or ornamented, and runs of lines drawn as an element of the app's.
//!
//! Nothing here changes the text, the selections, or editing. Every offset keeps
//! addressing the source, so a caret inside a concealed range sits at the point
//! the range was hidden at, and a selection over one still selects its bytes.
use std::fmt;
use std::ops::Range;
use std::rc::Rc;

use gpui::{
    AnyElement, App, Context, EventEmitter, HighlightStyle, Hsla, Pixels, SharedString, Window, px,
};
use ropey::Rope;

use super::{EditorState, InputBaseState, InputModeKind, RopeExt as _};

/// A render-only presentation for an [`EditorState`], set with
/// [`EditorState::set_presentation`].
///
/// Input is checked against the current text when it is set: a concealed range
/// that is empty, out of bounds, off a char boundary, crosses a newline, or
/// overlaps an earlier one is dropped, and so is a scale that is not finite,
/// not above zero, `1.0`, or for a line the text does not have. The other fields are
/// checked the same way (see each), and nothing in them can make it panic.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct EditorPresentation {
    /// UTF-8 byte ranges laid out at zero width and not painted. Each lies
    /// within one line (it may end at the line's end, not past its newline).
    pub concealed: Vec<Range<usize>>,
    /// `(buffer line, scale)`: the line's text is shaped `scale` times larger
    /// (or smaller, below `1.0`), and each of its visual rows is `scale` line
    /// heights tall.
    pub line_scales: Vec<(usize, f32)>,
    /// Source ranges drawn as other text. Each is checked like a concealed
    /// range, must not overlap a concealed range or an earlier replacement, and
    /// its text must be one line.
    pub replacements: Vec<Replacement>,
    /// `(buffer line, above, below)`: empty space added above the line's first
    /// visual row and below its last. Both must be finite and not negative.
    pub line_space: Vec<(usize, Pixels, Pixels)>,
    /// `(buffer line, inset)`: every visual row of the line starts `inset`
    /// further right, and the line wraps that much narrower. Must be finite
    /// and above zero.
    pub line_insets: Vec<(usize, Pixels)>,
    /// `(buffer line, ornament)`: drawn with the line, beneath its text. They
    /// change no layout. A line may have several.
    pub line_ornaments: Vec<(usize, Ornament)>,
    /// Runs of whole lines drawn as an element instead of their text.
    pub block_widgets: Vec<BlockWidget>,
    /// `(buffer line, factor)`: each visual row of the line is `factor` times as
    /// tall as its scale alone makes it (the editor's line height times the
    /// line's scale), its text centred in it as before. Must be finite, above
    /// zero and not `1.0`.
    pub line_heights: Vec<(usize, f32)>,
    /// The height of a row at scale `1.0`, instead of the window's line height
    /// (which is rounded to whole pixels), so rows can follow another
    /// renderer's exactly. Must be finite and above zero.
    pub line_height: Option<Pixels>,
}

/// A source range drawn as `text` instead, see [`EditorPresentation::replacements`].
///
/// The text is shaped at its line's font size (times the line's scale) in the
/// editor's text style refined by `style`, and takes the room of the source.
/// It is atomic for the caret: a caret before it is at its start, after it at
/// its end, and one inside the source is drawn at its start. A click on its
/// left half lands at the start and on its right half at the end, unless it is
/// `clickable`: a click on a clickable replacement leaves the caret where it is
/// and is reported as [`EditorPresentationEvent::ReplacementClicked`].
#[derive(Debug, Clone, PartialEq)]
pub struct Replacement {
    /// UTF-8 byte range of the source, within one line.
    pub range: Range<usize>,
    /// What is drawn in its place.
    pub text: SharedString,
    /// Applied over the editor's text style for `text`.
    pub style: HighlightStyle,
    /// Whether a click on it is reported instead of placing the caret.
    pub clickable: bool,
    /// The font family `text` is shaped in, instead of the editor's.
    pub family: Option<SharedString>,
    /// `text`'s size relative to its line's. Must be finite and above zero;
    /// anything else is taken as `1.0`.
    pub size: f32,
    /// A column `text` is set in, right-aligned, instead of its own width.
    pub column: Option<ReplacementColumn>,
    /// A box drawn behind `text`, with room either side of it.
    pub chip: Option<ReplacementChip>,
    /// An image drawn in place of `text` (which is then not drawn).
    pub image: Option<ReplacementImage>,
}

/// An image a [`Replacement`] is drawn as, like a typeset formula: `size`,
/// with its baseline `baseline` from its top on the row's baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct ReplacementImage {
    pub image: std::sync::Arc<gpui::Image>,
    pub size: gpui::Size<Pixels>,
    pub baseline: Pixels,
}

impl Replacement {
    /// `range` drawn as `text`, in the editor's style, not clickable.
    pub fn new(range: Range<usize>, text: impl Into<SharedString>) -> Self {
        Self {
            range,
            text: text.into(),
            style: HighlightStyle::default(),
            clickable: false,
            family: None,
            size: 1.,
            column: None,
            chip: None,
            image: None,
        }
    }

    /// The size its text is shaped at, for a line shaped at `font_size`.
    pub(crate) fn font_size(&self, font_size: Pixels) -> Pixels {
        if self.size.is_finite() && self.size > 0. {
            font_size * self.size
        } else {
            font_size
        }
    }

    /// How it is placed for text `text_width` wide: how wide it is laid out,
    /// where its text starts (from where it is laid out), and how far its
    /// column hangs to the left of where it is laid out.
    pub(crate) fn placement(&self, text_width: Pixels) -> (Pixels, Pixels, Pixels) {
        if let Some(image) = &self.image {
            return (image.size.width, px(0.), px(0.));
        }
        if let Some(chip) = &self.chip {
            let padding = chip.padding.max(px(0.));
            let icon = chip
                .icon
                .as_ref()
                .map_or(px(0.), |icon| icon.size + icon.gap);
            return (text_width + icon + padding * 2., padding + icon, px(0.));
        }
        match self.column {
            Some(column) => {
                let gap = column.gap.max(px(0.));
                let width = column.width.max(text_width + gap);
                if column.hanging {
                    (px(0.), -(gap + text_width), width)
                } else {
                    (width, width - gap - text_width, px(0.))
                }
            }
            None => (text_width, px(0.), px(0.)),
        }
    }
}

/// A box behind a [`Replacement`]'s text, like a code span's or a tag's: the
/// replacement is `padding` wider either side, and the box spans it.
#[derive(Debug, Clone, PartialEq)]
pub struct ReplacementChip {
    pub padding: Pixels,
    pub background: Hsla,
    pub radius: Pixels,
    pub height: ChipHeight,
    /// An icon before the text, in the text's colour.
    pub icon: Option<ChipIcon>,
    /// A dashed line this colour along the bottom of the row, under it.
    pub dashed_bottom: Option<Hsla>,
}

/// An icon in a [`ReplacementChip`]: the SVG asset at `path`, `size` square,
/// centred on the row, `gap` before the text.
#[derive(Debug, Clone, PartialEq)]
pub struct ChipIcon {
    pub path: SharedString,
    pub size: Pixels,
    pub gap: Pixels,
}

/// How tall a [`ReplacementChip`] is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChipHeight {
    /// Around its text's capitals: their height and `size * 0.125 +
    /// descent / 2` either side, as a code span's. Its text keeps the row's
    /// baseline.
    CapHeight,
    /// This tall, centred on the row, as is its text.
    Fixed(Pixels),
}

/// A column a [`Replacement`]'s text is set in, like a list's marker column:
/// at least `width` wide, its text right-aligned `gap` short of its right edge.
/// A wider text widens the column.
///
/// A `hanging` column takes no room: it hangs to the left of where the
/// replacement is (into the line's inset, like an outside list marker), so
/// the text after it starts where the line's wrapped rows do. A click on it
/// is a click on the replacement.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReplacementColumn {
    pub width: Pixels,
    pub gap: Pixels,
    pub hanging: bool,
}

/// Something drawn with a line, see [`EditorPresentation::line_ornaments`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Ornament {
    /// A vertical bar the full height of the line, its space included, `x`
    /// from the left edge of the text area (whatever the line's inset).
    Bar {
        x: Pixels,
        width: Pixels,
        color: Hsla,
    },
    /// A horizontal line across the text area, centred on the line's rows.
    Rule { thickness: Pixels, color: Hsla },
    /// A task's box: a square `size` wide with corners of `radius`, `x` from
    /// the left edge of the text area, centred on the line's first row. Its
    /// border is `border` wide in `border_color`; a `fill` fills it, and a
    /// `check` draws a check mark in it (a stroke `size * 0.1` wide).
    Check {
        x: Pixels,
        size: Pixels,
        radius: Pixels,
        border: Pixels,
        border_color: Hsla,
        fill: Option<Hsla>,
        check: Option<Hsla>,
    },
}

/// Builds the element of a [`BlockWidget`].
pub type BlockWidgetRenderer = Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>;

/// Whole buffer lines drawn as an element, see
/// [`EditorPresentation::block_widgets`].
///
/// The lines' text is not drawn: the first line becomes one row as tall as the
/// element, and the others are hidden the way folded lines are, so vertical
/// moves step over them in one row. The element is laid out at the width of
/// the text area; its measured height replaces `height` from the next frame
/// on, remembered by `key` until a widget with that key is no longer set.
///
/// Mouse events inside the element reach it. A press it leaves alone does not
/// place the caret, and a click it leaves alone is reported as
/// [`EditorPresentationEvent::BlockClicked`]; an element that handles a click
/// itself should stop its propagation. Selections and edits address the
/// source as usual: an edit touching the lines drops the widget.
///
/// `lines` must be in the text and must not overlap an earlier widget, and
/// `height` must be finite and above zero.
#[derive(Clone)]
pub struct BlockWidget {
    /// The buffer lines, `start..end`.
    pub lines: Range<usize>,
    /// The height to lay out before the element is measured.
    pub height: Pixels,
    /// Identifies the element's content, so its measured height is kept.
    pub key: u64,
    /// Builds the element, each frame it is visible.
    pub render: BlockWidgetRenderer,
}

impl fmt::Debug for BlockWidget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BlockWidget")
            .field("lines", &self.lines)
            .field("height", &self.height)
            .field("key", &self.key)
            .finish_non_exhaustive()
    }
}

/// Widgets are equal when their lines, height and key are: the key stands for
/// what the renderer draws.
impl PartialEq for BlockWidget {
    fn eq(&self, other: &Self) -> bool {
        self.lines == other.lines && self.height == other.height && self.key == other.key
    }
}

/// What a click on a presentation reports, emitted by the editor's state.
#[derive(Debug, Clone, PartialEq)]
pub enum EditorPresentationEvent {
    /// A [`Replacement`] with `clickable` set was clicked; the caret did not move.
    ReplacementClicked { range: Range<usize> },
    /// A [`BlockWidget`] was clicked where its element did not handle it.
    BlockClicked { lines: Range<usize> },
}

impl<M: InputModeKind> EventEmitter<EditorPresentationEvent> for InputBaseState<M> {}

/// A presentation checked against the text: every list sorted (by start or by
/// line) and free of the items validation drops.
#[derive(Clone, Default)]
pub(crate) struct ValidPresentation {
    pub(crate) concealed: Rc<[Range<usize>]>,
    pub(crate) line_scales: Rc<[(usize, f32)]>,
    pub(crate) replacements: Rc<[Replacement]>,
    pub(crate) line_space: Rc<[(usize, Pixels, Pixels)]>,
    pub(crate) line_insets: Rc<[(usize, Pixels)]>,
    pub(crate) line_ornaments: Rc<[(usize, Ornament)]>,
    pub(crate) blocks: Rc<[BlockWidget]>,
    pub(crate) line_heights: Rc<[(usize, f32)]>,
    pub(crate) line_height: Option<Pixels>,
}

impl PartialEq for ValidPresentation {
    fn eq(&self, other: &Self) -> bool {
        self.concealed == other.concealed
            && self.line_scales == other.line_scales
            && self.replacements == other.replacements
            && self.line_space == other.line_space
            && self.line_insets == other.line_insets
            && self.line_ornaments == other.line_ornaments
            && self.blocks == other.blocks
            && self.line_heights == other.line_heights
            && self.line_height == other.line_height
    }
}

impl ValidPresentation {
    pub(crate) fn is_empty(&self) -> bool {
        self.concealed.is_empty()
            && self.line_scales.is_empty()
            && self.replacements.is_empty()
            && self.line_space.is_empty()
            && self.line_insets.is_empty()
            && self.line_ornaments.is_empty()
            && self.blocks.is_empty()
            && self.line_heights.is_empty()
            && self.line_height.is_none()
    }

    fn to_presentation(&self) -> EditorPresentation {
        EditorPresentation {
            concealed: self.concealed.to_vec(),
            line_scales: self.line_scales.to_vec(),
            replacements: self.replacements.to_vec(),
            line_space: self.line_space.to_vec(),
            line_insets: self.line_insets.to_vec(),
            line_ornaments: self.line_ornaments.to_vec(),
            block_widgets: self.blocks.to_vec(),
            line_heights: self.line_heights.to_vec(),
            line_height: self.line_height,
        }
    }
}

/// Look up the value of `line` in a list sorted by line with one entry per line.
pub(crate) fn line_value<T: Copy>(items: &[(usize, T)], line: usize) -> Option<T> {
    items
        .binary_search_by_key(&line, |(l, _)| *l)
        .ok()
        .map(|ix| items[ix].1)
}

/// Whether `range` lies within one line of `text`, on char boundaries, and is not empty.
fn is_line_range(text: &Rope, range: &Range<usize>) -> bool {
    range.start < range.end
        && range.end <= text.len()
        && text.is_char_boundary(range.start)
        && text.is_char_boundary(range.end)
        && text.offset_to_point(range.start).row == text.offset_to_point(range.end).row
}

/// Sort `items` by line and keep the first of each line.
fn first_per_line<T>(mut items: Vec<(usize, T)>) -> Vec<(usize, T)> {
    items.sort_by_key(|(line, _)| *line);
    items.dedup_by_key(|(line, _)| *line);
    items
}

fn finite_px(value: Pixels) -> bool {
    f32::from(value).is_finite()
}

impl EditorPresentation {
    /// Whether the presentation changes nothing.
    pub fn is_empty(&self) -> bool {
        self.concealed.is_empty()
            && self.line_scales.is_empty()
            && self.replacements.is_empty()
            && self.line_space.is_empty()
            && self.line_insets.is_empty()
            && self.line_ornaments.is_empty()
            && self.block_widgets.is_empty()
            && self.line_heights.is_empty()
            && self.line_height.is_none()
    }

    /// The valid part of this presentation for `text`: sorted, disjoint ranges,
    /// per-line items sorted by line (the first of duplicates wins, except
    /// ornaments, which keep every one in order), and disjoint widgets.
    pub(crate) fn validate(&self, text: &Rope) -> ValidPresentation {
        let mut concealed: Vec<Range<usize>> = self
            .concealed
            .iter()
            .filter(|range| is_line_range(text, range))
            .cloned()
            .collect();
        concealed.sort_by_key(|range| range.start);
        let mut end = 0;
        concealed.retain(|range| {
            let keep = range.start >= end;
            if keep {
                end = range.end;
            }
            keep
        });

        let mut replacements: Vec<Replacement> = self
            .replacements
            .iter()
            .filter(|r| is_line_range(text, &r.range) && !r.text.contains(['\n', '\r']))
            .cloned()
            .collect();
        replacements.sort_by_key(|r| r.range.start);
        let mut end = 0;
        replacements.retain(|r| {
            let overlaps_concealed = {
                let ix = concealed.partition_point(|c| c.end <= r.range.start);
                concealed.get(ix).is_some_and(|c| c.start < r.range.end)
            };
            let keep = r.range.start >= end && !overlaps_concealed;
            if keep {
                end = r.range.end;
            }
            keep
        });

        let lines = text.lines_len();
        let line_scales = first_per_line(
            self.line_scales
                .iter()
                .copied()
                .filter(|&(line, scale)| {
                    line < lines && scale.is_finite() && scale > 0. && scale != 1.
                })
                .collect(),
        );
        let line_space = {
            let mut items: Vec<(usize, Pixels, Pixels)> = self
                .line_space
                .iter()
                .copied()
                .filter(|&(line, above, below)| {
                    line < lines
                        && finite_px(above)
                        && finite_px(below)
                        && above >= px(0.)
                        && below >= px(0.)
                        && (above > px(0.) || below > px(0.))
                })
                .collect();
            items.sort_by_key(|(line, _, _)| *line);
            items.dedup_by_key(|(line, _, _)| *line);
            items
        };
        let line_insets = first_per_line(
            self.line_insets
                .iter()
                .copied()
                .filter(|&(line, inset)| line < lines && finite_px(inset) && inset > px(0.))
                .collect(),
        );
        let mut line_ornaments: Vec<(usize, Ornament)> = self
            .line_ornaments
            .iter()
            .copied()
            .filter(|(line, ornament)| {
                *line < lines
                    && match *ornament {
                        Ornament::Bar { x, width, .. } => {
                            finite_px(x) && finite_px(width) && width > px(0.)
                        }
                        Ornament::Rule { thickness, .. } => {
                            finite_px(thickness) && thickness > px(0.)
                        }
                        Ornament::Check {
                            x,
                            size,
                            radius,
                            border,
                            ..
                        } => {
                            finite_px(x)
                                && finite_px(size)
                                && size > px(0.)
                                && finite_px(radius)
                                && finite_px(border)
                        }
                    }
            })
            .collect();
        line_ornaments.sort_by_key(|(line, _)| *line);

        let mut blocks: Vec<BlockWidget> = self
            .block_widgets
            .iter()
            .filter(|b| {
                b.lines.start < b.lines.end
                    && b.lines.end <= lines
                    && finite_px(b.height)
                    && b.height > px(0.)
            })
            .cloned()
            .collect();
        blocks.sort_by_key(|b| b.lines.start);
        let mut end = 0;
        blocks.retain(|b| {
            let keep = b.lines.start >= end;
            if keep {
                end = b.lines.end;
            }
            keep
        });

        let line_heights = first_per_line(
            self.line_heights
                .iter()
                .copied()
                .filter(|&(line, factor)| {
                    line < lines && factor.is_finite() && factor > 0. && factor != 1.
                })
                .collect(),
        );

        ValidPresentation {
            line_height: self
                .line_height
                .filter(|&height| finite_px(height) && height > px(0.)),
            line_heights: line_heights.into(),
            concealed: concealed.into(),
            line_scales: line_scales.into(),
            replacements: replacements.into(),
            line_space: line_space.into(),
            line_insets: line_insets.into(),
            line_ornaments: line_ornaments.into(),
            blocks: blocks.into(),
        }
    }
}

impl EditorState {
    /// Set, or with `None` clear, the render-only presentation of the text.
    ///
    /// Off by default; without one the editor lays out exactly as before. The
    /// presentation is validated against the current text (see
    /// [`EditorPresentation`]). Edits carry it along: a concealed range or
    /// replacement an edit touches is dropped, and later ones shift; the
    /// per-line items of the lines an edit spans are dropped (an edit within
    /// one line keeps that line's), and later lines' shift with the line count;
    /// a block widget whose lines an edit touches is dropped. Replacing the
    /// whole text clears it. It is meant to be set again after every change, so
    /// this only keeps a stale presentation safe until then.
    ///
    /// Concealing and replacing are ignored while inline tokens are shown.
    pub fn set_presentation(
        &mut self,
        presentation: Option<EditorPresentation>,
        cx: &mut Context<Self>,
    ) {
        let presentation = presentation
            .map(|presentation| presentation.validate(&self.text))
            .unwrap_or_default();
        // A widget's renderer may change under the same key, so a frame is due
        // whenever widgets are set.
        let has_blocks = !presentation.blocks.is_empty();
        if self.display_map.set_presentation(presentation, cx) || has_blocks {
            cx.notify();
        }
    }

    /// The presentation in effect: what [`Self::set_presentation`] kept,
    /// carried through any edits since.
    pub fn editor_presentation(&self) -> EditorPresentation {
        self.display_map.presentation().to_presentation()
    }
}

impl<M: InputModeKind> InputBaseState<M> {
    /// The height of a row at scale `1.0`: the presentation's, or the window's line
    /// height.
    pub(crate) fn presented_line_height(&self, window: &Window) -> Pixels {
        self.display_map
            .presentation()
            .line_height
            .unwrap_or_else(|| window.line_height())
    }

    /// The source range of the clickable replacement drawn under window `position`, if any.
    pub(crate) fn clickable_replacement_at(
        &self,
        position: gpui::Point<Pixels>,
    ) -> Option<Range<usize>> {
        let replacements = &self.display_map.presentation().replacements;
        if self.masked || !replacements.iter().any(|r| r.clickable) {
            return None;
        }
        let (bounds, last_layout) = (self.last_bounds.as_ref()?, self.last_layout.as_ref()?);
        let inner = position - bounds.origin - gpui::point(last_layout.line_number_width, px(0.));
        let mut y = last_layout.visible_top;
        for (vi, line) in last_layout.lines.iter().enumerate() {
            let height = line.size(last_layout.line_height).height;
            if inner.y >= y && inner.y < y + height {
                let local =
                    line.replacement_at_position(inner - gpui::point(px(0.), y), last_layout)?;
                let start = last_layout.visible_line_byte_offsets.get(vi)?;
                let range = start + local.start..start + local.end;
                return replacements
                    .iter()
                    .find(|r| r.clickable && r.range == range)
                    .map(|r| r.range.clone());
            }
            y += height;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{IntoElement as _, hsla};

    fn valid(presentation: EditorPresentation, text: &str) -> EditorPresentation {
        presentation.validate(&Rope::from(text)).to_presentation()
    }

    #[test]
    fn invalid_concealed_ranges_are_dropped() {
        let text = "# héllo\n**b**\n";
        let presentation = EditorPresentation {
            concealed: vec![
                9..11,
                0..2,
                1..2, // overlaps 0..2
                3..4, // ends inside `é`
                5..5, // empty
                6..9, // crosses the newline
                12..14,
                40..42, // out of bounds
                Range { start: 4, end: 3 },
            ],
            ..Default::default()
        };
        assert_eq!(
            valid(presentation, text).concealed,
            vec![0..2, 9..11, 12..14]
        );
    }

    #[test]
    fn invalid_scales_are_dropped() {
        let presentation = EditorPresentation {
            line_scales: vec![
                (2, 1.2),
                (0, 1.6),
                (0, 1.4),
                (1, 0.5),
                (1, 1.),
                (1, f32::NAN),
                (1, f32::INFINITY),
                (9, 2.),
            ],
            ..Default::default()
        };
        assert_eq!(
            valid(presentation, "a\nb\nc").line_scales,
            vec![(0, 1.6), (1, 0.5), (2, 1.2)]
        );
    }

    fn replacement(range: Range<usize>, text: &str) -> Replacement {
        Replacement::new(range, SharedString::from(text.to_string()))
    }

    #[test]
    fn invalid_replacements_are_dropped() {
        let text = "- [ ] é task\n- b";
        let presentation = EditorPresentation {
            concealed: vec![6..8],
            replacements: vec![
                replacement(14..16, "•"),
                replacement(0..2, "•"),
                replacement(1..3, "x"), // overlaps 0..2
                replacement(2..5, "☐"),
                replacement(6..7, "x"),     // inside a concealed range
                replacement(7..8, "x"),     // ends inside `é` and is concealed
                replacement(9..10, "a\nb"), // two lines
                replacement(11..14, "x"),   // crosses the newline
                replacement(30..31, "x"),   // out of bounds
                replacement(10..10, "x"),   // empty
            ],
            ..Default::default()
        };
        let valid = valid(presentation, text);
        assert_eq!(
            valid
                .replacements
                .iter()
                .map(|r| r.range.clone())
                .collect::<Vec<_>>(),
            vec![0..2, 2..5, 14..16]
        );
    }

    #[test]
    fn replacements_are_placed_in_columns_chips_and_images() {
        let mut r = Replacement::new(0..2, "1.");
        assert_eq!(r.placement(px(10.)), (px(10.), px(0.), px(0.)));
        r.column = Some(ReplacementColumn {
            width: px(22.),
            gap: px(6.),
            hanging: false,
        });
        // Right-aligned in the column, `gap` short of its edge.
        assert_eq!(r.placement(px(10.)), (px(22.), px(6.), px(0.)));
        // A wider text widens the column.
        assert_eq!(r.placement(px(20.)), (px(26.), px(0.), px(0.)));
        r.column = Some(ReplacementColumn {
            width: px(22.),
            gap: px(6.),
            hanging: true,
        });
        // Hanging: no room, the text left of where it is, the column to hit.
        assert_eq!(r.placement(px(10.)), (px(0.), px(-16.), px(22.)));
        let chip = Replacement {
            chip: Some(ReplacementChip {
                padding: px(4.),
                background: hsla(0., 0., 0., 1.),
                radius: px(2.),
                height: ChipHeight::CapHeight,
                icon: Some(ChipIcon {
                    path: "icons/calendar.svg".into(),
                    size: px(12.),
                    gap: px(4.),
                }),
                dashed_bottom: None,
            }),
            ..Replacement::new(0..2, "x")
        };
        assert_eq!(
            chip.placement(px(10.)),
            (px(10. + 16. + 8.), px(20.), px(0.))
        );
        let image = Replacement {
            image: Some(ReplacementImage {
                image: std::sync::Arc::new(gpui::Image::from_bytes(gpui::ImageFormat::Png, vec![])),
                size: gpui::size(px(30.), px(14.)),
                baseline: px(11.),
            }),
            ..Replacement::new(0..2, "")
        };
        assert_eq!(image.placement(px(0.)), (px(30.), px(0.), px(0.)));
        // A size that is not one is taken as the line's.
        let sized = Replacement {
            size: f32::NAN,
            ..Replacement::new(0..1, "x")
        };
        assert_eq!(sized.font_size(px(16.)), px(16.));
    }

    #[test]
    fn invalid_line_heights_and_line_height_are_dropped() {
        let presentation = EditorPresentation {
            line_heights: vec![(1, 0.5), (0, 1.), (0, 0.), (2, f32::NAN), (9, 2.)],
            line_height: Some(px(-1.)),
            ..Default::default()
        };
        let valid = valid(presentation, "a\nb\nc");
        assert_eq!(valid.line_heights, vec![(1, 0.5)]);
        assert_eq!(valid.line_height, None);
        let kept = EditorPresentation {
            line_height: Some(px(26.4)),
            ..Default::default()
        };
        assert_eq!(valid_height(kept), Some(px(26.4)));
    }

    fn valid_height(presentation: EditorPresentation) -> Option<Pixels> {
        valid(presentation, "a").line_height
    }

    #[test]
    fn invalid_line_items_and_widgets_are_dropped() {
        let color = hsla(0., 0., 0., 1.);
        let render: BlockWidgetRenderer = Rc::new(|_, _| gpui::Empty.into_any_element());
        let block = |lines: Range<usize>, height: f32| BlockWidget {
            lines,
            height: px(height),
            key: 1,
            render: render.clone(),
        };
        let presentation = EditorPresentation {
            line_space: vec![
                (2, px(4.), px(0.)),
                (0, px(1.), px(2.)),
                (0, px(9.), px(9.)),
                (1, px(0.), px(0.)),
                (1, px(-1.), px(3.)),
                (1, px(f32::NAN), px(3.)),
                (7, px(1.), px(1.)),
            ],
            line_insets: vec![
                (1, px(12.)),
                (0, px(0.)),
                (2, px(f32::INFINITY)),
                (5, px(3.)),
            ],
            line_ornaments: vec![
                (
                    1,
                    Ornament::Rule {
                        thickness: px(1.),
                        color,
                    },
                ),
                (
                    0,
                    Ornament::Bar {
                        x: px(0.),
                        width: px(3.),
                        color,
                    },
                ),
                (
                    0,
                    Ornament::Bar {
                        x: px(8.),
                        width: px(3.),
                        color,
                    },
                ),
                (
                    0,
                    Ornament::Bar {
                        x: px(8.),
                        width: px(0.),
                        color,
                    },
                ),
                (
                    2,
                    Ornament::Rule {
                        thickness: px(f32::NAN),
                        color,
                    },
                ),
                (
                    4,
                    Ornament::Rule {
                        thickness: px(1.),
                        color,
                    },
                ),
            ],
            block_widgets: vec![
                block(1..3, 40.),
                block(0..2, 40.), // sorts first, so 1..3 overlaps it
                block(2..2, 40.),
                block(2..4, 40.),
                block(0..1, f32::NAN),
            ],
            ..Default::default()
        };
        let valid = valid(presentation, "a\nb\nc");
        assert_eq!(
            valid.line_space,
            vec![(0, px(1.), px(2.)), (2, px(4.), px(0.))]
        );
        assert_eq!(valid.line_insets, vec![(1, px(12.))]);
        assert_eq!(
            valid
                .line_ornaments
                .iter()
                .map(|(line, _)| *line)
                .collect::<Vec<_>>(),
            vec![0, 0, 1]
        );
        assert_eq!(
            valid
                .block_widgets
                .iter()
                .map(|b| b.lines.clone())
                .collect::<Vec<_>>(),
            vec![0..2]
        );
    }
}
