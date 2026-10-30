/// DisplayMap: Public facade for Editor/Input display mapping.
///
/// This combines WrapMap and FoldMap to provide a unified API:
/// - BufferPoint ↔ DisplayPoint conversion
/// - Fold management (candidates, toggle, query)
/// - Automatic projection updates on text/layout changes
use std::collections::HashMap;
use std::ops::Range;

use gpui::{App, Font, Pixels, px};
use ropey::Rope;

use super::fold_map::FoldMap;
use super::folding::FoldRange;
pub use super::text_wrapper::WrappingIndent;
use super::text_wrapper::{LineFrame, LineItem, WrapDisplayPoint};
use super::wrap_map::WrapMap;
use super::{BufferPoint, DisplayPoint};
use crate::input::Point as TreeSitterPoint;
use crate::input::display_map::WrapPoint;
use crate::input::presentation::{BlockWidget, ValidPresentation, line_value};
use crate::input::rope_ext::RopeExt as _;

/// DisplayMap is the main interface for Editor/Input coordinate mapping.
///
/// It manages the two-layer projection:
/// 1. Buffer → Wrap (soft-wrapping)
/// 2. Wrap → Display (folding)
///
/// Editor/Input only needs to work with BufferPoint and DisplayPoint.
pub struct DisplayMap {
    wrap_map: WrapMap,
    fold_map: FoldMap,
    /// Visible display rows of the presentation's lines that are not one line height a row
    /// (scaled lines, lines with space, block widgets), sorted, rebuilt with the fold
    /// projection. Empty without such a presentation, which keeps rows uniformly tall.
    scaled_rows: Vec<ScaledRows>,
    /// Sum over `scaled_rows` of `(scale - 1) * rows`, in line heights.
    scaled_extra: f32,
    /// Sum over `scaled_rows` of their extra height in pixels: their space, and how much
    /// taller than a line height a widget's row is.
    scaled_extra_px: Pixels,
    /// Wrap rows hidden by block widgets, which the scroll height leaves out.
    block_hidden_rows: usize,
    /// Measured heights of block widgets by key, which replace the heights they were set with.
    block_heights: HashMap<u64, Pixels>,
    /// The window's device pixels per logical pixel, which leaves round to; `0.` until known.
    device_scale: f32,
    /// The room below the last line of each of the presentation's leaves that rounds the
    /// leaf up to whole device pixels, by that line.
    leaf_pads: HashMap<usize, Pixels>,
}

/// The display rows of one buffer line that are not one line height each.
struct ScaledRows {
    rows: Range<usize>,
    /// Each row is `scale` line heights plus `fixed` pixels tall.
    scale: f32,
    fixed: Pixels,
    /// Space above the first row and below the last.
    above: Pixels,
    below: Pixels,
    /// Extra height, in line heights, of the scaled rows above these.
    extra_before: f32,
    /// Extra height, in pixels, of the rows above these.
    extra_before_px: Pixels,
}

impl ScaledRows {
    /// The height of each row, without the space.
    fn row_height(&self, line_height: Pixels) -> Pixels {
        line_height * self.scale + self.fixed
    }
}

impl DisplayMap {
    pub fn new(font: Font, font_size: Pixels, wrap_width: Option<Pixels>) -> Self {
        Self {
            wrap_map: WrapMap::new(font, font_size, wrap_width),
            fold_map: FoldMap::new(),
            scaled_rows: Vec::new(),
            scaled_extra: 0.,
            scaled_extra_px: px(0.),
            block_hidden_rows: 0,
            block_heights: HashMap::new(),
            device_scale: 0.,
            leaf_pads: HashMap::new(),
        }
    }

    // ==================== Core Coordinate Mapping ====================

    /// Convert buffer position to display position
    pub fn buffer_pos_to_display_pos(&self, pos: BufferPoint) -> DisplayPoint {
        // Buffer → Wrap
        let wrap_pos = self.wrap_map.buffer_pos_to_wrap_pos(pos);

        // Wrap → Display
        if let Some(display_row) = self.fold_map.wrap_row_to_display_row(wrap_pos.row) {
            DisplayPoint::new(display_row, wrap_pos.col)
        } else {
            // Cursor is in a folded region, find nearest visible row
            let display_row = self.fold_map.nearest_visible_display_row(wrap_pos.row);
            DisplayPoint::new(display_row, 0) // Column 0 at fold boundary
        }
    }

    /// Convert display position to buffer position
    pub fn display_pos_to_buffer_pos(&self, pos: DisplayPoint) -> BufferPoint {
        // Display → Wrap
        let wrap_row = self.fold_map.display_row_to_wrap_row(pos.row).unwrap_or(0);

        // Wrap → Buffer
        let wrap_pos = WrapPoint::new(wrap_row, pos.col);
        self.wrap_map.wrap_pos_to_buffer_pos(wrap_pos)
    }

    /// Get total number of visible display rows
    #[inline]
    pub fn display_row_count(&self) -> usize {
        self.fold_map.display_row_count()
    }

    /// Get the buffer line for a given display row
    pub fn display_row_to_buffer_line(&self, display_row: usize) -> usize {
        // Display → Wrap
        let wrap_row = self
            .fold_map
            .display_row_to_wrap_row(display_row)
            .unwrap_or(0);

        // Wrap → Buffer line
        self.wrap_map.wrap_row_to_buffer_line(wrap_row)
    }

    /// Get the display row range for a buffer line: [start, end)
    /// Returns None if the buffer line is completely hidden
    pub fn buffer_line_to_display_row_range(&self, line: usize) -> Option<Range<usize>> {
        // Buffer line → Wrap row range
        let wrap_row_range = self.wrap_map.buffer_line_to_wrap_row_range(line);

        // Find first and last visible display rows in this range
        let mut first_display_row = None;
        let mut last_display_row = None;

        for wrap_row in wrap_row_range {
            if let Some(display_row) = self.fold_map.wrap_row_to_display_row(wrap_row) {
                if first_display_row.is_none() {
                    first_display_row = Some(display_row);
                }
                last_display_row = Some(display_row);
            }
        }

        if let (Some(start), Some(end)) = (first_display_row, last_display_row) {
            Some(start..end + 1)
        } else {
            None // Completely folded
        }
    }

    /// Check if a buffer line is completely hidden
    #[inline]
    pub fn is_buffer_line_hidden(&self, line: usize) -> bool {
        self.buffer_line_to_display_row_range(line).is_none()
    }

    /// First display row of a buffer line. If the line is fully folded, returns the
    /// nearest visible display row.
    pub fn buffer_line_to_display_row(&self, line: usize) -> usize {
        match self.buffer_line_to_display_row_range(line) {
            Some(range) => range.start,
            None => {
                let wrap_row = self.wrap_map.buffer_line_to_first_wrap_row(line);
                self.fold_map.nearest_visible_display_row(wrap_row)
            }
        }
    }

    /// Set fold candidates (from tree-sitter/LSP)
    pub fn set_fold_candidates(&mut self, candidates: Vec<FoldRange>) {
        self.fold_map.set_candidates(candidates);
        self.rebuild_fold_projection();
    }

    /// Set a fold at the given start_line (must be in candidates)
    pub fn set_folded(&mut self, start_line: usize, folded: bool) {
        self.fold_map.set_folded(start_line, folded);
        self.rebuild_fold_projection();
    }

    /// Toggle fold at the given start_line
    pub fn toggle_fold(&mut self, start_line: usize) {
        self.fold_map.toggle_fold(start_line);
        self.rebuild_fold_projection();
    }

    /// Check if a line is currently folded
    #[inline]
    pub fn is_folded_at(&self, start_line: usize) -> bool {
        self.fold_map.is_folded_at(start_line)
    }

    /// Check if a line is a fold candidate
    #[inline]
    pub fn is_fold_candidate(&self, start_line: usize) -> bool {
        self.fold_map.is_fold_candidate(start_line)
    }

    /// Get all currently folded ranges
    #[inline]
    pub fn folded_ranges(&self) -> &[FoldRange] {
        self.fold_map.folded_ranges()
    }

    /// Clear all folds
    pub fn clear_folds(&mut self) {
        self.fold_map.clear_folds();
        self.rebuild_fold_projection();
    }

    // ==================== Text and Layout Updates ====================

    /// Adjust folds and candidates for a text edit before updating the wrap map.
    ///
    /// Must be called with the OLD text (before replacement) and the edit range/new_text
    /// so we can compute which old lines were affected.
    pub fn adjust_folds_for_edit(&mut self, old_text: &Rope, range: &Range<usize>, new_text: &str) {
        if self.fold_map.folded_ranges().is_empty() && self.fold_map.fold_candidates().is_empty() {
            return;
        }

        let edit_start_line = old_text.offset_to_point(range.start).row;
        let edit_end_line = old_text.offset_to_point(range.end.min(old_text.len())).row;

        let old_lines_in_range = edit_end_line.saturating_sub(edit_start_line);
        let new_lines_in_range = new_text.chars().filter(|c| *c == '\n').count();
        let line_delta = new_lines_in_range as isize - old_lines_in_range as isize;

        self.fold_map
            .adjust_folds_for_edit(edit_start_line, edit_end_line, line_delta);
    }

    /// Incrementally update fold candidates after a text edit.
    ///
    /// Extracts new fold candidates only within the edited byte range
    /// and merges them with existing (already adjusted) candidates.
    pub fn update_fold_candidates_for_edit(
        &mut self,
        extract_fold_ranges: impl FnOnce(Range<usize>, &Rope) -> Vec<FoldRange>,
        edit_byte_range: Range<usize>,
        new_text: &Rope,
    ) {
        let new_start_line = new_text.offset_to_point(edit_byte_range.start).row;
        let new_end_line = new_text
            .offset_to_point(edit_byte_range.end.min(new_text.len()))
            .row;

        let new_candidates = extract_fold_ranges(edit_byte_range, new_text);
        self.fold_map
            .merge_candidates_for_edit(new_start_line, new_end_line, new_candidates);
    }

    /// Update text (incremental or full)
    pub fn on_text_changed(
        &mut self,
        changed_text: &Rope,
        range: &Range<usize>,
        new_text: &Rope,
        cx: &mut App,
    ) {
        self.wrap_map
            .on_text_changed(changed_text, range, new_text, cx);
        self.rebuild_fold_projection();
    }

    /// Update layout parameters (wrap width or font)
    pub(crate) fn set_inline_metrics(
        &mut self,
        metrics: std::rc::Rc<[(std::ops::Range<usize>, Pixels)]>,
        cx: &mut App,
    ) {
        self.wrap_map.set_inline_metrics(metrics, cx);
        self.rebuild_fold_projection();
    }

    pub fn on_layout_changed(&mut self, wrap_width: Option<Pixels>, cx: &mut App) {
        self.wrap_map.on_layout_changed(wrap_width, cx);
        self.rebuild_fold_projection();
    }

    /// Set the wrapping indent for continuation lines.
    pub fn set_wrapping_indent(&mut self, wrapping_indent: WrappingIndent, cx: &mut App) {
        self.wrap_map.set_wrapping_indent(wrapping_indent, cx);
        self.rebuild_fold_projection();
    }

    /// Set font parameters
    pub fn set_font(&mut self, font: Font, font_size: Pixels, cx: &mut App) {
        self.wrap_map.set_font(font, font_size, cx);
        self.rebuild_fold_projection();
    }

    /// Ensure text is prepared (initializes wrapper if needed)
    pub fn ensure_text_prepared(&mut self, text: &Rope, cx: &mut App) {
        let did_initialize = self.wrap_map.ensure_text_prepared(text, cx);
        if did_initialize {
            self.rebuild_fold_projection();
        }
    }

    /// Initialize with text
    pub fn set_text(&mut self, text: &Rope, cx: &mut App) {
        self.wrap_map.set_text(text, cx);
        self.rebuild_fold_projection();
    }

    // ==================== Presentation ====================

    /// Replace the render-only presentation, already validated against the text (see
    /// `EditorPresentation`). Returns whether anything changed.
    pub(crate) fn set_presentation(
        &mut self,
        presentation: ValidPresentation,
        cx: &mut App,
    ) -> bool {
        // Measured heights are kept only for the widgets still set.
        if !self.block_heights.is_empty() {
            self.block_heights
                .retain(|key, _| presentation.blocks.iter().any(|b| b.key == *key));
        }
        let changed = self.wrap_map.set_presentation(presentation, cx);
        if changed {
            self.rebuild_fold_projection();
        }
        changed
    }

    /// The validated presentation, carried through any edits since it was set.
    #[inline]
    pub(crate) fn presentation(&self) -> &ValidPresentation {
        self.wrap_map.wrapper().presentation()
    }

    /// The concealed byte ranges, sorted and disjoint.
    #[inline]
    pub(crate) fn concealed(&self) -> &[Range<usize>] {
        self.wrap_map.wrapper().concealed()
    }

    /// Whether a presentation changes anything.
    #[inline]
    pub(crate) fn has_presentation(&self) -> bool {
        !self.presentation().is_empty()
    }

    /// Whether some display rows are not one line height tall.
    #[inline]
    pub(crate) fn has_uneven_rows(&self) -> bool {
        !self.scaled_rows.is_empty()
    }

    /// The scale of buffer line `line`: `1.0` unless the presentation scales it.
    #[inline]
    pub(crate) fn line_scale(&self, line: usize) -> f32 {
        self.wrap_map.wrapper().line_scale(line)
    }

    /// How many line heights tall each row of buffer line `line` is: its scale times its
    /// line height factor.
    #[inline]
    pub(crate) fn row_scale(&self, line: usize) -> f32 {
        self.wrap_map.wrapper().row_scale(line)
    }

    /// The block widget whose first line is `line`.
    pub(crate) fn block_at(&self, line: usize) -> Option<&BlockWidget> {
        let blocks = &self.presentation().blocks;
        blocks
            .binary_search_by_key(&line, |b| b.lines.start)
            .ok()
            .map(|ix| &blocks[ix])
    }

    /// The height a block widget is laid out at: its measured one once it has been drawn.
    pub(crate) fn block_height(&self, block: &BlockWidget) -> Pixels {
        self.block_heights
            .get(&block.key)
            .copied()
            .unwrap_or(block.height)
    }

    /// Record the measured height of the widget with `key`. Returns whether the layout
    /// changed.
    pub(crate) fn set_block_height(&mut self, key: u64, height: Pixels) -> bool {
        if !f32::from(height).is_finite() || height <= px(0.) {
            return false;
        }
        let Some(block) = self.presentation().blocks.iter().find(|b| b.key == key) else {
            return false;
        };
        if (self.block_height(block) - height).abs() < px(0.5) {
            return false;
        }
        self.block_heights.insert(key, height);
        self.rebuild_scaled_rows();
        true
    }

    /// Set the window's device pixels per logical pixel, which the presentation's leaves
    /// round to. Returns whether the layout changed.
    pub(crate) fn set_device_scale(&mut self, scale: f32) -> bool {
        if self.device_scale == scale || !scale.is_finite() || scale <= 0. {
            return false;
        }
        self.device_scale = scale;
        if self.presentation().leaves.is_empty() {
            return false;
        }
        self.rebuild_scaled_rows();
        true
    }

    /// The room the presentation gives buffer line `line` around and before its rows.
    pub(crate) fn line_frame(&self, line: usize) -> LineFrame {
        let presentation = self.presentation();
        if presentation.line_space.is_empty()
            && presentation.line_insets.is_empty()
            && presentation.blocks.is_empty()
            && self.leaf_pads.is_empty()
        {
            return LineFrame::default();
        }
        if let Some(block) = self.block_at(line) {
            return LineFrame {
                block: Some(self.block_height(block)),
                ..LineFrame::default()
            };
        }
        let (above, below) = self.line_space(line);
        LineFrame {
            above,
            below,
            inset: line_value(&presentation.line_insets, line).unwrap_or(px(0.)),
            block: None,
        }
    }

    /// The space above and below buffer line `line`.
    fn line_space(&self, line: usize) -> (Pixels, Pixels) {
        let space = &self.presentation().line_space;
        let (above, below) = space
            .binary_search_by_key(&line, |(l, _, _)| *l)
            .map_or((px(0.), px(0.)), |ix| (space[ix].1, space[ix].2));
        let pad = self.leaf_pads.get(&line).copied().unwrap_or(px(0.));
        (above, below + pad)
    }

    /// Work out the room that rounds each leaf up to whole device pixels: its rows (as
    /// many as it wraps to now), the space between them and its extra.
    fn rebuild_leaf_pads(&mut self) {
        self.leaf_pads.clear();
        let presentation = self.presentation().clone();
        let (Some(line_height), scale) = (presentation.line_height, self.device_scale) else {
            return;
        };
        if scale <= 0. {
            return;
        }
        for (lines, extra) in presentation.leaves.iter() {
            let mut height = *extra;
            for line in lines.clone() {
                let Some(rows) = self.buffer_line_to_display_row_range(line) else {
                    continue;
                };
                height += line_height * self.row_scale(line) * rows.len() as f32;
                let space = &presentation.line_space;
                if let Ok(ix) = space.binary_search_by_key(&line, |(l, _, _)| *l) {
                    if line != lines.start {
                        height += space[ix].1;
                    }
                    if line + 1 != lines.end {
                        height += space[ix].2;
                    }
                }
            }
            let device = f32::from(height) * scale;
            // Within a rounding error of a whole device pixel is on it: a scaled row's
            // height is a product of floats.
            let pad = px(((device - 1e-3).ceil() - device).max(0.) / scale);
            if pad > px(0.) {
                self.leaf_pads.insert(lines.end - 1, pad);
            }
        }
    }

    /// The y of the top of display row `row`, in content space.
    ///
    /// Rows are `line_height` tall except those of scaled lines, so without a presentation
    /// this is `line_height * row`. The space above a line belongs to its first row, and the
    /// space below to its last.
    pub(crate) fn display_row_top(&self, row: usize, line_height: Pixels) -> Pixels {
        if self.scaled_rows.is_empty() {
            return line_height * row;
        }
        let ix = self.scaled_rows.partition_point(|s| s.rows.start < row);
        let Some(scaled) = ix.checked_sub(1).map(|ix| &self.scaled_rows[ix]) else {
            return line_height * row;
        };
        let inside = row.min(scaled.rows.end) - scaled.rows.start;
        let space = if row >= scaled.rows.end {
            scaled.above + scaled.below
        } else {
            scaled.above
        };
        line_height * (row as f32 + scaled.extra_before + (scaled.scale - 1.) * inside as f32)
            + scaled.extra_before_px
            + scaled.fixed * inside as f32
            + space
    }

    /// The height of display row `row`, its share of its line's space included.
    pub(crate) fn display_row_height(&self, row: usize, line_height: Pixels) -> Pixels {
        let ix = self.scaled_rows.partition_point(|s| s.rows.end <= row);
        match self.scaled_rows.get(ix) {
            Some(scaled) if scaled.rows.contains(&row) => {
                let mut height = scaled.row_height(line_height);
                if row == scaled.rows.start {
                    height += scaled.above;
                }
                if row + 1 == scaled.rows.end {
                    height += scaled.below;
                }
                height
            }
            _ => line_height,
        }
    }

    /// The display row under content-space `y` (rows past the end continue at
    /// `line_height`, so the caller clamps). A line's space maps to its nearest row.
    pub(crate) fn display_row_at_y(&self, y: Pixels, line_height: Pixels) -> usize {
        let y = y.max(px(0.));
        let uniform = |y: Pixels, base: usize| {
            base + (f32::from(y) / f32::from(line_height)).floor() as usize
        };
        let ix = self
            .scaled_rows
            .partition_point(|s| self.display_row_top(s.rows.start, line_height) <= y);
        let Some(scaled) = ix.checked_sub(1).map(|ix| &self.scaled_rows[ix]) else {
            return uniform(y, 0);
        };
        let top = self.display_row_top(scaled.rows.start, line_height);
        let row_height = scaled.row_height(line_height);
        let bottom = top + scaled.above + row_height * scaled.rows.len() as f32 + scaled.below;
        if y < bottom {
            let y = y - top - scaled.above;
            let local = if y <= px(0.) || row_height <= px(0.) {
                0
            } else {
                (f32::from(y) / f32::from(row_height)).floor() as usize
            };
            scaled.rows.start + local.min(scaled.rows.len() - 1)
        } else {
            uniform(y - bottom, scaled.rows.end)
        }
    }

    /// The height of every wrap row, for the scroll size: `line_height` each, plus the extra
    /// of the visible scaled rows, less the rows block widgets hide.
    pub(crate) fn wrap_rows_height(&self, line_height: Pixels) -> Pixels {
        let rows = self.wrap_map.wrap_row_count() as f32;
        if self.scaled_rows.is_empty() && self.block_hidden_rows == 0 {
            rows * line_height
        } else {
            (rows - self.block_hidden_rows as f32 + self.scaled_extra) * line_height
                + self.scaled_extra_px
        }
    }

    // ==================== Internal Helpers ====================

    /// Rebuild fold projection after wrap_map or fold state changes
    /// Only rebuilds if there are actually folded ranges
    fn rebuild_fold_projection(&mut self) {
        // A block widget hides its lines after the first the way a fold does.
        let hidden: Vec<Range<usize>> = self
            .presentation()
            .blocks
            .iter()
            .map(|b| b.lines.start + 1..b.lines.end)
            .filter(|lines| !lines.is_empty())
            .collect();
        self.block_hidden_rows = hidden
            .iter()
            .map(|lines| {
                self.wrap_map.buffer_line_to_first_wrap_row(lines.end)
                    - self.wrap_map.buffer_line_to_first_wrap_row(lines.start)
            })
            .sum();
        self.fold_map.set_hidden_lines(hidden);
        if self.fold_map.hides_rows() {
            self.fold_map.rebuild(&self.wrap_map);
        } else {
            // No active folds: identity mapping (wrap_row == display_row).
            // Just update cached count so query methods work without Vec allocation.
            self.fold_map
                .mark_dirty_with_wrap_count(self.wrap_map.wrap_row_count());
        }
        self.rebuild_scaled_rows();
    }

    /// Recompute the display rows of the lines that are not one line height a row. A folded
    /// line has none, so it adds no height.
    fn rebuild_scaled_rows(&mut self) {
        self.scaled_rows.clear();
        self.scaled_extra = 0.;
        self.scaled_extra_px = px(0.);
        self.rebuild_leaf_pads();
        let presentation = self.presentation().clone();
        if presentation.line_scales.is_empty()
            && presentation.line_space.is_empty()
            && presentation.blocks.is_empty()
            && presentation.line_heights.is_empty()
            && self.leaf_pads.is_empty()
        {
            return;
        }
        let mut lines: Vec<usize> = presentation
            .line_scales
            .iter()
            .map(|(line, _)| *line)
            .chain(presentation.line_space.iter().map(|(line, _, _)| *line))
            .chain(presentation.blocks.iter().map(|b| b.lines.start))
            .chain(presentation.line_heights.iter().map(|(line, _)| *line))
            .chain(self.leaf_pads.keys().copied())
            .collect();
        lines.sort_unstable();
        lines.dedup();
        let mut extra = 0.;
        let mut extra_px = px(0.);
        let mut scaled_rows = Vec::with_capacity(lines.len());
        for line in lines {
            let Some(rows) = self.buffer_line_to_display_row_range(line) else {
                continue;
            };
            let len = rows.len() as f32;
            let (scale, fixed, above, below) = match self.block_at(line) {
                // A widget's line is one row as tall as the widget: no line height, all
                // fixed pixels.
                Some(block) => (0., self.block_height(block), px(0.), px(0.)),
                None => {
                    let (above, below) = self.line_space(line);
                    (self.row_scale(line), px(0.), above, below)
                }
            };
            scaled_rows.push(ScaledRows {
                rows,
                scale,
                fixed,
                above,
                below,
                extra_before: extra,
                extra_before_px: extra_px,
            });
            extra += (scale - 1.) * len;
            extra_px += fixed * len + above + below;
        }
        self.scaled_rows = scaled_rows;
        self.scaled_extra = extra;
        self.scaled_extra_px = extra_px;
    }

    // ==================== Wrap Display Point Operations ====================

    /// Convert byte offset to wrap display point (with soft wrap info).
    #[inline]
    pub(crate) fn offset_to_wrap_display_point(&self, offset: usize) -> WrapDisplayPoint {
        self.wrap_map.wrapper().offset_to_display_point(offset)
    }

    /// Like [`Self::offset_to_wrap_display_point`], but honours the caret's line-end affinity so
    /// an offset on a soft wrap boundary resolves to the row the caret is drawn on.
    #[inline]
    pub(crate) fn offset_to_wrap_display_point_with_affinity(
        &self,
        offset: usize,
        line_end_affinity: bool,
    ) -> WrapDisplayPoint {
        self.wrap_map
            .wrapper()
            .offset_to_display_point_with_affinity(offset, line_end_affinity)
    }

    /// Convert wrap display point to byte offset.
    #[inline]
    pub(crate) fn wrap_display_point_to_offset(&self, point: WrapDisplayPoint) -> usize {
        self.wrap_map.wrapper().display_point_to_offset(point)
    }

    /// Convert wrap display point to TreeSitterPoint (buffer line/col).
    #[inline]
    pub(crate) fn wrap_display_point_to_point(&self, point: WrapDisplayPoint) -> TreeSitterPoint {
        self.wrap_map.wrapper().display_point_to_point(point)
    }

    /// Convert a wrap row to a display row (skipping folded rows).
    /// Returns None if the wrap row is folded.
    #[inline]
    pub fn wrap_row_to_display_row(&self, wrap_row: usize) -> Option<usize> {
        self.fold_map.wrap_row_to_display_row(wrap_row)
    }

    /// Find the nearest visible display row for a given wrap row.
    #[inline]
    pub fn nearest_visible_display_row(&self, wrap_row: usize) -> usize {
        self.fold_map.nearest_visible_display_row(wrap_row)
    }

    /// Convert a display row to a wrap row.
    #[inline]
    pub fn display_row_to_wrap_row(&self, display_row: usize) -> Option<usize> {
        self.fold_map.display_row_to_wrap_row(display_row)
    }

    pub(crate) fn display_row_column_to_offset(&self, display_row: usize, column: usize) -> usize {
        let wrap_row = self
            .fold_map
            .display_row_to_wrap_row(display_row)
            .unwrap_or(0);
        self.wrap_map
            .wrapper()
            .display_point_to_offset(WrapDisplayPoint::new(wrap_row, 0, column))
    }

    /// Get the longest row index (by byte length).
    #[inline]
    pub(crate) fn longest_row(&self) -> usize {
        self.wrap_map.wrapper().longest_row()
    }

    // ==================== Access Methods ====================

    /// Get the line item by buffer row index.
    #[inline]
    pub(crate) fn line(&self, row: usize) -> Option<&LineItem> {
        self.wrap_map.line(row)
    }

    /// Get the rope text
    #[inline]
    pub fn text(&self) -> &Rope {
        self.wrap_map.text()
    }

    /// Calculate how many wrap rows of a buffer line are visible (not folded)
    #[inline]
    pub fn visible_wrap_row_count_for_buffer_line(&self, line: usize) -> usize {
        self.wrap_map
            .visible_wrap_row_count_for_line(line, &self.fold_map)
    }

    /// Get the wrap row count (before folding)
    #[inline]
    pub fn wrap_row_count(&self) -> usize {
        self.wrap_map.wrap_row_count()
    }

    /// Get the buffer line count (logical lines)
    #[inline]
    pub fn buffer_line_count(&self) -> usize {
        self.wrap_map.buffer_line_count()
    }
}
