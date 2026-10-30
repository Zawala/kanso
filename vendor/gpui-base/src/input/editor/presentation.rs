//! A render-only presentation of the code editor's text: ranges laid out at zero
//! width, and lines drawn larger.
//!
//! Nothing here changes the text, the selections, or editing. Every offset keeps
//! addressing the source, so a caret inside a concealed range sits at the point
//! the range was hidden at, and a selection over one still selects its bytes.
use std::ops::Range;
use std::rc::Rc;

use gpui::Context;
use ropey::Rope;

use super::{EditorState, RopeExt as _};

/// A render-only presentation for an [`EditorState`], set with
/// [`EditorState::set_presentation`].
///
/// Input is checked against the current text when it is set: a concealed range
/// that is empty, out of bounds, off a char boundary, crosses a newline, or
/// overlaps an earlier one is dropped, and so is a scale that is not finite,
/// not above `1.0`, or for a line the text does not have.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct EditorPresentation {
    /// UTF-8 byte ranges laid out at zero width and not painted. Each lies
    /// within one line (it may end at the line's end, not past its newline).
    pub concealed: Vec<Range<usize>>,
    /// `(buffer line, scale)`: the line's text is shaped `scale` times larger,
    /// and each of its visual rows is `scale` line heights tall.
    pub line_scales: Vec<(usize, f32)>,
}

impl EditorPresentation {
    /// Whether the presentation conceals or scales nothing.
    pub fn is_empty(&self) -> bool {
        self.concealed.is_empty() && self.line_scales.is_empty()
    }

    /// The valid part of this presentation for `text`: sorted, disjoint
    /// concealed ranges, and scales sorted by line (the first of duplicates
    /// wins).
    pub(crate) fn validate(&self, text: &Rope) -> (Rc<[Range<usize>]>, Rc<[(usize, f32)]>) {
        let mut concealed: Vec<Range<usize>> = self
            .concealed
            .iter()
            .filter(|range| {
                range.start < range.end
                    && range.end <= text.len()
                    && text.is_char_boundary(range.start)
                    && text.is_char_boundary(range.end)
                    && text.offset_to_point(range.start).row == text.offset_to_point(range.end).row
            })
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

        let lines = text.lines_len();
        let mut line_scales: Vec<(usize, f32)> = self
            .line_scales
            .iter()
            .copied()
            .filter(|&(line, scale)| line < lines && scale.is_finite() && scale > 1.)
            .collect();
        line_scales.sort_by_key(|&(line, _)| line);
        line_scales.dedup_by_key(|&mut (line, _)| line);

        (concealed.into(), line_scales.into())
    }
}

impl EditorState {
    /// Set, or with `None` clear, the render-only presentation of the text.
    ///
    /// Off by default; without one the editor lays out exactly as before. The
    /// presentation is validated against the current text (see
    /// [`EditorPresentation`]). Edits carry it along: a concealed range an edit
    /// touches is dropped, and later ones shift; the scales of the lines an edit
    /// spans are dropped (an edit within one line keeps that line's), and later
    /// lines' shift with the line count. Replacing the whole text clears it.
    /// It is meant to be set again after every change, so this only keeps a
    /// stale presentation safe until then.
    ///
    /// Concealing is ignored while inline tokens are shown.
    pub fn set_presentation(
        &mut self,
        presentation: Option<EditorPresentation>,
        cx: &mut Context<Self>,
    ) {
        let (concealed, line_scales) = presentation
            .map(|presentation| presentation.validate(&self.text))
            .unwrap_or_else(|| (Rc::from([]), Rc::from([])));
        if self
            .display_map
            .set_presentation(concealed, line_scales, cx)
        {
            cx.notify();
        }
    }

    /// The presentation in effect: what [`Self::set_presentation`] kept,
    /// carried through any edits since.
    pub fn editor_presentation(&self) -> EditorPresentation {
        EditorPresentation {
            concealed: self.display_map.concealed().to_vec(),
            line_scales: self.display_map.line_scales().to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid(presentation: EditorPresentation, text: &str) -> EditorPresentation {
        let (concealed, line_scales) = presentation.validate(&Rope::from(text));
        EditorPresentation {
            concealed: concealed.to_vec(),
            line_scales: line_scales.to_vec(),
        }
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
            line_scales: vec![],
        };
        assert_eq!(
            valid(presentation, text).concealed,
            vec![0..2, 9..11, 12..14]
        );
    }

    #[test]
    fn invalid_scales_are_dropped() {
        let presentation = EditorPresentation {
            concealed: vec![],
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
        };
        assert_eq!(
            valid(presentation, "a\nb\nc").line_scales,
            vec![(0, 1.6), (2, 1.2)]
        );
    }
}
