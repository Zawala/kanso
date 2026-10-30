# Proposal: a render-only presentation for the gpui-base code editor

This is a change to offer to [GPUI Kit](https://github.com/longbridge/gpui-kit)
(`crates/base`), written against gpui-base 0.7.3. It hasn't been sent
anywhere.

## Summary

Let an `EditorState` be given a presentation of its text that changes only
how it is drawn: byte ranges laid out at zero width and not painted
("concealed"), and buffer lines drawn larger (a scale per line: the line's
text is shaped at `font_size * scale` and each of its visual rows is
`line_height * scale` tall, wrapped at `wrap_width / scale`). Offsets,
selections, editing, undo and search keep addressing the source text. With
no presentation set, nothing changes.

## Motivation

A Markdown editor that renders as you type (Typora, Obsidian's Live
Preview) keeps the Markdown source as the document but hides the marks
(`**`, `#`, link brackets and URLs) on the lines the caret is not on, and
draws headings larger. The editor already has everything such a view needs
except a way to lay text out at a width and height other than the uniform
row: inline tokens lay a range out as a fragment of its own width, but
always paint an element there and keep every row one line height tall.

An app cannot get this from outside. Overlaying its own text means
re-implementing hit testing, wrapping and caret placement; swapping the
text for a rendered one breaks offsets, undo and collaboration. Hiding text
needs only a zero-width fragment, which the row type already supports, and
taller rows need the editor's y arithmetic to stop assuming
`row * line_height`.

## API

```rust
/// A render-only presentation for an `EditorState`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct EditorPresentation {
    /// UTF-8 byte ranges laid out at zero width and not painted. Each lies
    /// within one line.
    pub concealed: Vec<Range<usize>>,
    /// `(buffer line, scale)`: the line is shaped `scale` times larger and
    /// each of its rows is `scale` line heights tall.
    pub line_scales: Vec<(usize, f32)>,
}

impl EditorState {
    pub fn set_presentation(&mut self, presentation: Option<EditorPresentation>, cx: &mut Context<Self>);
    pub fn editor_presentation(&self) -> EditorPresentation;
}
```

Input is validated against the current text when it is set: concealed
ranges that are empty, out of bounds, off a char boundary, cross a newline
or overlap are dropped, and so are scales that are not finite, not above
1.0, or for lines the text does not have. It never panics.

Edits carry the presentation along so it stays safe until the owner sets a
fresh one (an app is expected to set it again after every change): a
concealed range an edit touches is dropped and later ones shift, as inline
token metrics do; the scales of the lines an edit spans are dropped (an edit
within one line keeps that line's) and later lines' shift by the change in
line count. Replacing the whole text clears it. Layout checks concealed
ranges against the line once more before shaping.

Behaviour:

- A caret inside or at a concealed range is drawn at the point it was
  hidden at; a click on that point lands after the hidden text (a
  zero-width fragment resolves to its end).
- A concealed range wraps as a zero-width element (the same path inline
  tokens take). Lines with inline tokens ignore concealing.
- The display map keeps the display rows of the scaled lines (rebuilt with
  the fold projection, so a folded line adds no height) and maps rows to and
  from y. The visible range, scroll size, carets (the caret is 0.85 of its
  row's height), selections, search and hover highlights, range
  decorations, document colours, line numbers, the active line, fold icons,
  indent guides, hit testing, scroll to caret, IME and range bounds, touch
  handles and up/down moves all use it. `LineLayout` carries its scale, so
  every `line.size(line_height)` already measures the taller rows.
- Left uniform: page up/down move by `viewport / line_height` display rows;
  an auto-growing input counts rows; the scroll wheel's line delta, the
  popup anchor's line height, ghost (inline completion) lines and token
  elements' heights stay one line height.

## Compatibility

Additive. Every existing method keeps its signature and behaviour; with no
presentation the uniform paths run unchanged (each changed site keeps its
original expression when nothing is scaled, and `x * 1.0` elsewhere is
exact). `EditorPresentation` is a new export.

## Tests to add

Next to the existing ones in `src/input/base/element.rs` and the new
`src/input/editor/presentation.rs`:

- a concealed range has zero width, a caret inside it sits at the conceal
  point, a click there lands after it, and a selection over it spans the
  visible text;
- a scaled line's rows and caret are taller, its text is shaped larger, and
  points below it hit the right line;
- a scaled line wraps into more, narrower rows;
- up/down moves step display rows across a scaled line and keep x;
- the visible range follows mixed row heights when scrolled, and y ↔ row
  mapping round-trips;
- scales of folded lines add no height;
- edits shift, keep or drop the presentation as described, IME composition
  on a presented line lays out, and invalid or stale input never panics;
- with no presentation (or an empty one) the layout is identical to before.

## Diff

Against the published crate (`vendor/gpui-base` in this repository).

```diff
diff --git a/src/input/base/element.rs b/src/input/base/element.rs
index 2e23301e..7f868d18 100644
--- a/src/input/base/element.rs
+++ b/src/input/base/element.rs
@@ -558,7 +558,10 @@ impl<M: InputModeKind> TextElement<M> {
         let visible_buffer_lines = &last_layout.visible_buffer_lines;
         let caret_for = |row: usize, offset: usize, affinity: bool| -> Point<Pixels> {
             // y of the top of buffer line `row` in content space.
-            let top = line_height * state.display_map.buffer_line_to_display_row(row);
+            let top = state.display_map.display_row_top(
+                state.display_map.buffer_line_to_display_row(row),
+                line_height,
+            );
             let line_origin = point(px(0.), top);
 
             if let Some(vi) = visible_buffer_lines.iter().position(|&bl| bl == row) {
@@ -572,8 +575,6 @@ impl<M: InputModeKind> TextElement<M> {
             line_origin
         };
 
-        let cursor_height = 0.85 * line_height;
-
         for selection in state.selections.iter() {
             let is_active = selection.id == active_id;
 
@@ -608,6 +609,10 @@ impl<M: InputModeKind> TextElement<M> {
 
             let affinity = is_active && state.cursor_line_end_affinity;
             let cursor_pos = caret_for(cursor_row, cursor, affinity);
+            // A scaled line's rows are taller, and so is its caret.
+            let row_height = line_height * state.display_map.line_scale(cursor_row);
+            let cursor_height = 0.85 * row_height;
+            let below_row = row_height - line_height;
             let cursor_start = caret_for(sel_start_row, selected_range.start, false);
             let cursor_end = caret_for(sel_end_row, selected_range.end, false);
 
@@ -644,11 +649,11 @@ impl<M: InputModeKind> TextElement<M> {
                         // the frame the selection changed, so a one-line step
                         // would leave a far-off caret (e.g. after typing at the
                         // end of a long paste) outside the viewport.
-                        scroll_offset.y = if scroll_offset.y + cursor_pos.y
+                        scroll_offset.y = if scroll_offset.y + cursor_pos.y + below_row
                             > bounds.size.height - top_bottom_margin
                         {
                             // cursor is out of bottom
-                            bounds.size.height - top_bottom_margin - cursor_pos.y
+                            bounds.size.height - top_bottom_margin - cursor_pos.y - below_row
                         } else if scroll_offset.y + cursor_pos.y < top_bottom_margin {
                             // cursor is out of top
                             (top_bottom_margin - cursor_pos.y).min(px(0.))
@@ -689,7 +694,7 @@ impl<M: InputModeKind> TextElement<M> {
                 bounds: Bounds::new(
                     point(
                         cursor_x,
-                        bounds.top() + cursor_pos.y + ((line_height - cursor_height) / 2.),
+                        bounds.top() + cursor_pos.y + ((row_height - cursor_height) / 2.),
                     ),
                     size(CURSOR_WIDTH, cursor_height),
                 ),
@@ -765,6 +770,7 @@ impl<M: InputModeKind> TextElement<M> {
         {
             let mut offset = line_offset;
             let alignment = last_layout.alignment_offset(line.longest_width);
+            let row_height = line.row_height(last_layout.line_height);
             for (row, shaped) in line.wrapped_lines.iter().enumerate() {
                 let end = offset + shaped.len;
                 let start_ix = range.start.max(offset);
@@ -786,12 +792,12 @@ impl<M: InputModeKind> TextElement<M> {
                     corners.push(Corners {
                         top_left: point(left, y),
                         top_right: point(right, y),
-                        bottom_left: point(left, y + last_layout.line_height),
-                        bottom_right: point(right, y + last_layout.line_height),
+                        bottom_left: point(left, y + row_height),
+                        bottom_right: point(right, y + row_height),
                     });
                 }
                 offset = end;
-                y += last_layout.line_height;
+                y += row_height;
             }
         }
         (!corners.is_empty()).then_some(corners)
@@ -1044,22 +1050,40 @@ impl<M: InputModeKind> TextElement<M> {
         } else {
             state.scroll_handle.offset().y
         };
+        let content_height = if state.display_map.line_scales().is_empty() {
+            line_height * total_lines
+        } else {
+            state.display_map.wrap_rows_height(line_height)
+        };
         scroll_top = clamp_auto_grow_vertical_scroll_offset(
             &state.mode,
             scroll_top,
-            line_height * total_lines,
+            content_height,
             input_height,
         );
 
         // Display rows are uniformly `line_height` tall, so the visible window maps
-        // directly to a display-row range.
+        // directly to a display-row range. A presentation's scaled lines make some rows
+        // taller; the display map walks those instead.
         let viewport_top = (-scroll_top).max(px(0.));
         let viewport_bottom = viewport_top + input_height;
-        let line_height_f = f32::from(line_height);
-        let first_display =
-            ((f32::from(viewport_top) / line_height_f).floor() as usize).min(display_count - 1);
-        let last_display =
-            ((f32::from(viewport_bottom) / line_height_f).ceil() as usize).min(display_count - 1);
+        let (first_display, last_display) = if state.display_map.line_scales().is_empty() {
+            let line_height_f = f32::from(line_height);
+            (
+                ((f32::from(viewport_top) / line_height_f).floor() as usize).min(display_count - 1),
+                ((f32::from(viewport_bottom) / line_height_f).ceil() as usize)
+                    .min(display_count - 1),
+            )
+        } else {
+            let display_map = &state.display_map;
+            (
+                display_map
+                    .display_row_at_y(viewport_top, line_height)
+                    .min(display_count - 1),
+                (display_map.display_row_at_y(viewport_bottom, line_height) + 1)
+                    .min(display_count - 1),
+            )
+        };
 
         let start_line = state.display_map.display_row_to_buffer_line(first_display);
         let end_line = state.display_map.display_row_to_buffer_line(last_display);
@@ -1069,8 +1093,10 @@ impl<M: InputModeKind> TextElement<M> {
             .display_map
             .buffer_line_to_display_row_range(start_line)
         {
-            Some(range) => line_height * range.start,
-            None => line_height * first_display,
+            Some(range) => state.display_map.display_row_top(range.start, line_height),
+            None => state
+                .display_map
+                .display_row_top(first_display, line_height),
         };
 
         let visible_range = start_line..(end_line + 1 + extra_rows).min(buffer_line_count);
@@ -1288,6 +1314,7 @@ impl<M: InputModeKind> TextElement<M> {
             is_folded: bool,
             display_row: usize,
             offset_y: Pixels,
+            row_height: Pixels,
         }
 
         let line_number_hitbox = window.insert_hitbox(
@@ -1324,25 +1351,26 @@ impl<M: InputModeKind> TextElement<M> {
                         is_folded,
                         display_row: buffer_line,
                         offset_y,
+                        row_height: line.row_height(last_layout.line_height),
                     });
                 }
 
-                offset_y += line.wrapped_lines.len() * last_layout.line_height;
+                offset_y += line.size(last_layout.line_height).height;
             }
 
             infos
         }; // state is dropped here
 
         // Second pass: create and prepaint icons
-        let line_height = last_layout.line_height;
         let line_number_width =
             last_layout.line_number_width - LINE_NUMBER_RIGHT_MARGIN - FOLD_ICON_HITBOX_WIDTH;
-        let icon_relative_pos = point(
-            (FOLD_ICON_HITBOX_WIDTH - FOLD_ICON_WIDTH).half(),
-            (line_height - FOLD_ICON_WIDTH).half(),
-        );
 
         for (ix, info) in fold_infos.iter().enumerate() {
+            let line_height = info.row_height;
+            let icon_relative_pos = point(
+                (FOLD_ICON_HITBOX_WIDTH - FOLD_ICON_WIDTH).half(),
+                (line_height - FOLD_ICON_WIDTH).half(),
+            );
             // Position fold icon to the right of line numbers.
             // Use origin_x (unscrolled) so icons stay fixed in the gutter during horizontal scroll.
             let fold_icon_bounds = Bounds::new(
@@ -1838,6 +1866,8 @@ impl<M: InputModeKind> TextElement<M> {
                         .wrapped_lines
                         .clone()
                 };
+                let scale = state.display_map.line_scale(row);
+                let font_size = font_size * scale;
                 let mut lines: SmallVec<[InputLine; 1]> = SmallVec::new();
                 for range in ranges {
                     let mut fragments = Vec::new();
@@ -1909,6 +1939,7 @@ impl<M: InputModeKind> TextElement<M> {
                 LineLayout::new()
                     .inline_lines(lines)
                     .wrap_indent(wrap_indent)
+                    .with_scale(scale)
             })
             .collect()
     }
@@ -2043,9 +2074,12 @@ impl<M: InputModeKind> TextElement<M> {
             }
 
             // Keep placeholder lines in a single layout to stay parallel with visible_* metadata.
+            // A scaled first line keeps its row height (not its text size), so the caret
+            // and the rows below agree with the display map.
             let line_layout = LineLayout::new()
                 .lines(placeholder_lines)
                 .with_background(line_has_background)
+                .with_scale(state.display_map.line_scale(0))
                 .with_whitespaces(whitespace_indicators);
             return vec![line_layout];
         }
@@ -2065,6 +2099,36 @@ impl<M: InputModeKind> TextElement<M> {
 
             debug_assert_eq!(line_item.len(), line_text.len());
 
+            // A presentation scales the line's text and conceals ranges of it. Concealed
+            // text is laid out at zero width through inline fragments, like a token's.
+            let scale = state.display_map.line_scale(buffer_line);
+            let concealed = if state.masked {
+                Vec::new()
+            } else {
+                concealed_in_line(
+                    state.display_map.concealed(),
+                    state.text.line_start_offset(buffer_line),
+                    &line_text,
+                )
+            };
+            if scale != 1. || !concealed.is_empty() {
+                lines.push(Self::layout_presented_line(
+                    &line_text,
+                    &line_item.wrapped_lines,
+                    line_item.indent,
+                    &concealed,
+                    scale,
+                    font_size * scale,
+                    &runs_for_range(runs, run_offset, &(0..line_text.len())),
+                    last_layout.visible_line_byte_offsets[vi],
+                    bg_segments,
+                    whitespace_indicators.clone(),
+                    window,
+                ));
+                run_offset += line_text.len() + 1;
+                continue;
+            }
+
             let mut wrapped_lines: SmallVec<[ShapedLine; 1]> = SmallVec::with_capacity(1);
             let mut line_has_background = false;
 
@@ -2117,6 +2181,110 @@ impl<M: InputModeKind> TextElement<M> {
         lines
     }
 
+    /// Lay out one buffer line under a presentation: shaped at `font_size` (the scaled
+    /// size), with its rows `scale` line heights tall and `concealed` (line-local, sorted,
+    /// disjoint, on char boundaries) laid out at zero width and not painted.
+    ///
+    /// `runs` cover exactly this line's bytes.
+    #[allow(clippy::too_many_arguments)]
+    fn layout_presented_line(
+        line_text: &str,
+        wrapped_ranges: &[Range<usize>],
+        indent: u32,
+        concealed: &[Range<usize>],
+        scale: f32,
+        font_size: Pixels,
+        runs: &[TextRun],
+        line_offset: usize,
+        bg_segments: &[(Range<usize>, Hsla)],
+        whitespace_indicators: Option<WhitespaceIndicators>,
+        window: &mut Window,
+    ) -> LineLayout {
+        use crate::input::display_map::{InlineFragment, InputLine};
+
+        let mut lines: SmallVec<[InputLine; 1]> = SmallVec::with_capacity(1);
+        let mut line_has_background = false;
+        for range in wrapped_ranges {
+            let mut row_runs = runs_for_range(runs, 0, range);
+            if !bg_segments.is_empty() {
+                row_runs =
+                    split_runs_by_bg_segments(line_offset + range.start, &row_runs, bg_segments);
+            }
+            let row_text = &line_text[range.clone()];
+            let row_runs = align_runs_to_char_boundaries(row_text, &row_runs).unwrap_or(row_runs);
+            line_has_background |= has_background(&row_runs);
+
+            // Shape `part` (row-local) as one fragment at `x`.
+            let shape = |part: Range<usize>, x: Pixels, window: &mut Window| {
+                let text = &row_text[part.clone()];
+                let part_runs = runs_for_range(&row_runs, 0, &part);
+                let part_runs =
+                    align_runs_to_char_boundaries(text, &part_runs).unwrap_or(part_runs);
+                let shaped = window.text_system().shape_line(
+                    text.to_owned().into(),
+                    font_size,
+                    &part_runs,
+                    None,
+                );
+                InlineFragment {
+                    range: part,
+                    x,
+                    width: shaped.width,
+                    text: Some(shaped),
+                }
+            };
+
+            let first = concealed.partition_point(|c| c.end <= range.start);
+            let hidden = concealed[first..]
+                .iter()
+                .take_while(|c| c.start < range.end);
+            let mut fragments = Vec::new();
+            let mut offset = 0;
+            let mut x = px(0.);
+            for c in hidden {
+                let c = c.start.max(range.start) - range.start..c.end.min(range.end) - range.start;
+                if c.start < offset || c.is_empty() {
+                    continue;
+                }
+                if offset < c.start {
+                    let fragment = shape(offset..c.start, x, window);
+                    x += fragment.width;
+                    fragments.push(fragment);
+                }
+                fragments.push(InlineFragment {
+                    range: c.clone(),
+                    x,
+                    width: px(0.),
+                    text: None,
+                });
+                offset = c.end;
+            }
+            if offset < row_text.len() || fragments.is_empty() {
+                fragments.push(shape(offset..row_text.len(), x, window));
+            }
+            lines.push(InputLine::inline(row_text.to_owned().into(), fragments));
+        }
+
+        // Use the first visual line's indentation width for continuation lines.
+        let wrap_indent = if indent > 0 && lines.len() > 1 {
+            let indent_byte_len = line_text
+                .char_indices()
+                .nth(indent as usize)
+                .map(|(ix, _)| ix)
+                .unwrap_or(line_text.len());
+            lines[0].x_for_index(indent_byte_len)
+        } else {
+            px(0.)
+        };
+
+        LineLayout::new()
+            .inline_lines(lines)
+            .wrap_indent(wrap_indent)
+            .with_background(line_has_background)
+            .with_scale(scale)
+            .with_whitespaces(whitespace_indicators)
+    }
+
     /// First usize is the offset of skipped.
     fn highlight_lines(
         &mut self,
@@ -2777,6 +2945,14 @@ impl<M: InputModeKind> Element for TextElement<M> {
                     .map_or(px(0.), |cache| cache.unwrapped_width)
             };
         }
+        // The longest row by bytes no longer says which row is widest once rows are scaled
+        // or concealed, so the laid-out rows count too.
+        if !state.soft_wrap && state.display_map.has_presentation() {
+            longest_line_width = lines
+                .iter()
+                .map(|line| line.size(line_height).width)
+                .fold(longest_line_width, Pixels::max);
+        }
         last_layout.lines = Rc::new(lines);
 
         let (ghost_first_line, ghost_lines) = Self::layout_inline_completion(
@@ -2789,7 +2965,6 @@ impl<M: InputModeKind> Element for TextElement<M> {
         let ghost_line_count = ghost_lines.len();
         let ghost_lines_height = ghost_line_count as f32 * line_height;
 
-        let total_wrapped_lines = state.display_map.wrap_row_count();
         let empty_bottom_height = empty_bottom_height(
             state.is_code_editor(),
             state.scroll_beyond_last_line,
@@ -2806,7 +2981,7 @@ impl<M: InputModeKind> Element for TextElement<M> {
             } else {
                 longest_line_width
             },
-            (total_wrapped_lines as f32 * line_height
+            (state.display_map.wrap_rows_height(line_height)
                 + empty_bottom_height.max(ghost_lines_height))
             .max(bounds.size.height),
         );
@@ -3051,13 +3226,16 @@ impl<M: InputModeKind> Element for TextElement<M> {
             offset_y += invisible_top_padding;
 
             // Each item is the normal lines.
-            for (lines, &buffer_line) in line_numbers
+            for (line, &buffer_line) in prepaint
+                .last_layout
+                .lines
                 .iter()
                 .zip(prepaint.last_layout.visible_buffer_lines.iter())
+                .take(line_numbers.len())
             {
                 let is_active = prepaint.current_row == Some(buffer_line);
                 let p = point(input_bounds.origin.x, origin.y + offset_y);
-                let height = line_height * lines.len() as f32;
+                let height = line.size(line_height).height;
                 // Paint the current line background
                 if is_active {
                     if let Some(bg_color) = active_line_color {
@@ -3258,12 +3436,15 @@ impl<M: InputModeKind> Element for TextElement<M> {
             window.paint_quad(fill(gutter_bounds, gutter_bg));
 
             // Each item is the normal lines.
-            for (lines, &buffer_line) in line_numbers
+            for ((lines, &buffer_line), layout) in line_numbers
                 .iter()
                 .zip(prepaint.last_layout.visible_buffer_lines.iter())
+                .zip(prepaint.last_layout.lines.iter())
             {
                 let p = point(input_bounds.origin.x, origin.y + offset_y);
                 let is_active = prepaint.current_row == Some(buffer_line);
+                // A scaled line's number sits centred in its taller rows.
+                let line_height = layout.row_height(line_height);
 
                 let height = line_height * lines.len() as f32;
                 // paint active line number background
@@ -3355,6 +3536,35 @@ impl<M: InputModeKind> Element for TextElement<M> {
     }
 }
 
+/// The concealed ranges that lie within the line starting at `line_start`, made local to it.
+///
+/// Ranges are checked against the current text again here: a presentation is validated when
+/// it is set, but a range that is out of the line or off a char boundary must never reach
+/// shaping, whatever happened to the text since.
+fn concealed_in_line(
+    concealed: &[Range<usize>],
+    line_start: usize,
+    line_text: &str,
+) -> Vec<Range<usize>> {
+    let line_end = line_start + line_text.len();
+    let first = concealed.partition_point(|r| r.end <= line_start);
+    let mut result: Vec<Range<usize>> = Vec::new();
+    for range in concealed[first..].iter().take_while(|r| r.start < line_end) {
+        if range.start < line_start || range.end > line_end || range.is_empty() {
+            continue;
+        }
+        let local = range.start - line_start..range.end - line_start;
+        if !line_text.is_char_boundary(local.start)
+            || !line_text.is_char_boundary(local.end)
+            || result.last().is_some_and(|last| last.end > local.start)
+        {
+            continue;
+        }
+        result.push(local);
+    }
+    result
+}
+
 /// Split placeholder text into display lines and trim runs to each line.
 fn placeholder_line_runs<'a>(
     display_text: &'a str,
@@ -3592,7 +3802,9 @@ fn split_runs_by_bg_segments(
 #[cfg(test)]
 mod tests {
     use super::*;
-    use crate::input::{EditorMode, EditorState, FoldRange, RangeDecoration, Redo, Undo};
+    use crate::input::{
+        EditorMode, EditorPresentation, EditorState, FoldRange, RangeDecoration, Redo, Undo,
+    };
     use gpui::{
         AppContext as _, Context, EntityInputHandler as _, Render, TestAppContext,
         VisualTestContext, div,
@@ -4094,6 +4306,403 @@ mod tests {
         });
     }
 
+    fn present(
+        editor: &Entity<EditorState>,
+        concealed: Vec<Range<usize>>,
+        line_scales: Vec<(usize, f32)>,
+        cx: &mut App,
+    ) {
+        editor.update(cx, |state, cx| {
+            state.set_presentation(
+                Some(EditorPresentation {
+                    concealed,
+                    line_scales,
+                }),
+                cx,
+            )
+        });
+    }
+
+    fn assert_close(a: Pixels, b: Pixels) {
+        assert!((a - b).abs() < px(0.01), "{a:?} != {b:?}");
+    }
+
+    #[gpui::test]
+    fn presentation_conceals_ranges_at_zero_width_and_maps_carets_across_them(
+        cx: &mut TestAppContext,
+    ) {
+        let text = "**bold** text";
+        let (editor, window) = decoration_editor(cx, text, false);
+        let mut cx = VisualTestContext::from_window(window.into(), cx);
+        cx.update(|window, cx| {
+            window.draw(cx).clear(cx);
+            let plain_width =
+                editor.read(cx).last_layout.as_ref().unwrap().lines[0].wrapped_lines[0].width;
+            present(&editor, vec![0..2, 6..8], vec![], cx);
+            editor.update(cx, |state, cx| state.set_selected_range(7..7, cx));
+            window.draw(cx).clear(cx);
+            let state = editor.read(cx);
+            let layout = state.last_layout.as_ref().unwrap();
+            let row = &layout.lines[0].wrapped_lines[0];
+            // The hidden bytes take no room; offsets still address the source.
+            assert_eq!(row.len, text.len());
+            for ix in 0..=2 {
+                assert_eq!(row.x_for_index(ix), px(0.));
+            }
+            assert!(row.x_for_index(6) > px(0.));
+            assert_eq!(row.x_for_index(7), row.x_for_index(6));
+            assert_eq!(row.x_for_index(8), row.x_for_index(6));
+            assert!(row.width < plain_width);
+            // A click at a conceal point lands after the hidden source.
+            assert_eq!(row.closest_index_for_x(px(0.)), 2);
+            assert_eq!(row.closest_index_for_x(row.width + px(40.)), text.len());
+            // A caret inside a concealed range is drawn at its conceal point.
+            let caret = layout.cursor_bounds.unwrap();
+            assert_close(
+                caret.origin.x,
+                state.input_bounds.left() + layout.line_number_width + row.x_for_index(6),
+            );
+            // A selection over the hidden range spans the visible text only.
+            let corners = TextElement::<EditorMode>::layout_range_corners(&(0..8), layout).unwrap();
+            assert_eq!(corners[0].top_left.x, px(0.));
+            assert_eq!(corners[0].top_right.x, row.x_for_index(8));
+        });
+    }
+
+    #[gpui::test]
+    fn presentation_scales_a_line_its_caret_and_the_rows_below(cx: &mut TestAppContext) {
+        let text = "# Title\nbody\nmore";
+        let (editor, window) = decoration_editor(cx, text, false);
+        let mut cx = VisualTestContext::from_window(window.into(), cx);
+        cx.update(|window, cx| {
+            window.draw(cx).clear(cx);
+            let plain = {
+                let layout = editor.read(cx).last_layout.clone().unwrap();
+                let row = &layout.lines[0].wrapped_lines[0];
+                row.x_for_index(7) - row.x_for_index(2)
+            };
+            present(&editor, vec![0..2], vec![(0, 1.6)], cx);
+            editor.update(cx, |state, cx| state.set_selected_range(3..3, cx));
+            window.draw(cx).clear(cx);
+            let state = editor.read(cx);
+            let layout = state.last_layout.as_ref().unwrap();
+            let lh = layout.line_height;
+            assert_close(layout.lines[0].size(lh).height, lh * 1.6);
+            assert_eq!(layout.lines[1].size(lh).height, lh);
+            let row = &layout.lines[0].wrapped_lines[0];
+            let scaled = row.x_for_index(7) - row.x_for_index(2);
+            assert!(
+                (scaled - plain * 1.6).abs() < px(2.),
+                "{scaled:?} vs {plain:?}"
+            );
+            assert_eq!(row.x_for_index(2), px(0.));
+
+            let body = TextElement::<EditorMode>::layout_range_corners(&(8..12), layout).unwrap();
+            assert_close(body[0].top_left.y, layout.visible_top + lh * 1.6);
+            assert_close(body[0].bottom_left.y - body[0].top_left.y, lh);
+            let caret = layout.cursor_bounds.unwrap();
+            assert_close(caret.size.height, 0.85 * lh * 1.6);
+
+            // Hit testing walks the taller row: a point that a uniform layout would put
+            // on `body` is still on the heading, and the row below it is `body`.
+            let origin =
+                state.last_bounds.unwrap().origin + point(layout.line_number_width, px(0.));
+            let (heading, _, _) = state.resolve_mouse_position(origin + point(px(1.), lh * 1.3));
+            assert!(heading <= 7, "{heading}");
+            let (below, _, _) =
+                state.resolve_mouse_position(origin + point(px(1.), lh * 1.6 + lh * 0.5));
+            assert!((8..=12).contains(&below), "{below}");
+            let (more, _, _) =
+                state.resolve_mouse_position(origin + point(px(1.), lh * 2.6 + lh * 0.5));
+            assert!(more >= 13, "{more}");
+
+            editor.update(cx, |state, cx| state.set_selected_range(9..9, cx));
+            window.draw(cx).clear(cx);
+            let layout = editor.read(cx).last_layout.clone().unwrap();
+            let caret = layout.cursor_bounds.unwrap();
+            assert_close(caret.size.height, 0.85 * lh);
+            assert!(caret.origin.y >= state_top(&editor, cx) + lh * 1.6);
+        });
+    }
+
+    fn state_top(editor: &Entity<EditorState>, cx: &App) -> Pixels {
+        editor.read(cx).last_bounds.unwrap().origin.y
+    }
+
+    #[gpui::test]
+    fn presentation_scaled_line_wraps_narrower(cx: &mut TestAppContext) {
+        let text = "alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu";
+        let (editor, window) = decoration_editor(cx, text, true);
+        let mut cx = VisualTestContext::from_window(window.into(), cx);
+        cx.update(|window, cx| {
+            window.draw(cx).clear(cx);
+            let rows = editor.read(cx).last_layout.as_ref().unwrap().lines[0]
+                .wrapped_lines
+                .len();
+            present(&editor, vec![], vec![(0, 2.)], cx);
+            window.draw(cx).clear(cx);
+            let state = editor.read(cx);
+            let layout = state.last_layout.as_ref().unwrap();
+            let line = &layout.lines[0];
+            assert!(line.wrapped_lines.len() > rows);
+            let wrap_width = layout.wrap_width.unwrap();
+            for row in &line.wrapped_lines {
+                assert!(row.width <= wrap_width + px(4.), "{:?}", row.width);
+            }
+            assert_eq!(
+                line.size(layout.line_height).height,
+                layout.line_height * 2. * line.wrapped_lines.len() as f32
+            );
+        });
+    }
+
+    #[gpui::test]
+    fn presentation_visible_range_follows_mixed_row_heights(cx: &mut TestAppContext) {
+        let text = (0..80)
+            .map(|i| format!("line {i}"))
+            .collect::<Vec<_>>()
+            .join("\n");
+        let (editor, window) = decoration_editor(cx, &text, false);
+        let mut cx = VisualTestContext::from_window(window.into(), cx);
+        cx.update(|window, cx| {
+            let scales = (0..80).step_by(3).map(|line| (line, 2.)).collect();
+            present(&editor, vec![], scales, cx);
+            window.draw(cx).clear(cx);
+            let lh = editor.read(cx).last_layout.as_ref().unwrap().line_height;
+            {
+                let map = &editor.read(cx).display_map;
+                for row in 0..80 {
+                    let top = map.display_row_top(row, lh);
+                    let height = map.display_row_height(row, lh);
+                    assert_eq!(height, if row % 3 == 0 { lh * 2. } else { lh });
+                    assert_eq!(map.display_row_at_y(top, lh), row);
+                    assert_eq!(map.display_row_at_y(top + height - px(0.5), lh), row);
+                    assert_close(map.display_row_top(row + 1, lh), top + height);
+                }
+                assert_close(map.wrap_rows_height(lh), lh * (80. + 27.));
+            }
+
+            editor.update(cx, |state, cx| {
+                state.set_scroll_offset(point(px(0.), -lh * 50.), cx);
+            });
+            window.draw(cx).clear(cx);
+            window.draw(cx).clear(cx);
+            let state = editor.read(cx);
+            let layout = state.last_layout.as_ref().unwrap();
+            let top = -state.scroll_handle.offset().y;
+            assert!(top > lh * 40.);
+            assert!(layout.visible_top <= top);
+            assert!(layout.visible_top + layout.lines[0].size(lh).height > top);
+            let laid_out: Pixels = layout.lines.iter().map(|line| line.size(lh).height).sum();
+            assert!(layout.visible_top + laid_out >= top + state.input_bounds.size.height);
+            assert_eq!(
+                layout.visible_top,
+                state
+                    .display_map
+                    .display_row_top(layout.visible_buffer_lines[0], lh)
+            );
+            assert!(state.scroll_size.height >= state.display_map.wrap_rows_height(lh));
+        });
+    }
+
+    #[gpui::test]
+    fn presentation_scales_of_folded_lines_add_no_height(cx: &mut TestAppContext) {
+        let text = "top\nhidden one\nhidden two\nend\nlast";
+        let (editor, window) = decoration_editor(cx, text, false);
+        let mut cx = VisualTestContext::from_window(window.into(), cx);
+        cx.update(|window, cx| {
+            editor.update(cx, |state, cx| {
+                state.apply_highlighter_fold_candidates(vec![FoldRange::new(0, 3)], cx);
+                state.set_presentation(
+                    Some(EditorPresentation {
+                        concealed: vec![],
+                        line_scales: vec![(1, 2.), (3, 1.5)],
+                    }),
+                    cx,
+                );
+                state.display_map.set_folded(0, true);
+            });
+            window.draw(cx).clear(cx);
+            let state = editor.read(cx);
+            let layout = state.last_layout.as_ref().unwrap();
+            let lh = layout.line_height;
+            assert_eq!(layout.visible_buffer_lines, vec![0, 3, 4]);
+            assert_close(state.display_map.wrap_rows_height(lh), lh * 5.5);
+            let end = text.find("end").unwrap();
+            let corners =
+                TextElement::<EditorMode>::layout_range_corners(&(end..end + 3), layout).unwrap();
+            assert_close(corners[0].top_left.y, layout.visible_top + lh);
+            assert_close(corners[0].bottom_left.y - corners[0].top_left.y, lh * 1.5);
+        });
+    }
+
+    #[gpui::test]
+    fn presentation_survives_edits_and_invalid_input(cx: &mut TestAppContext) {
+        let text = "**a** b\n# H\ntail";
+        let (editor, window) = decoration_editor(cx, text, true);
+        let mut cx = VisualTestContext::from_window(window.into(), cx);
+        cx.update(|window, cx| {
+            window.draw(cx).clear(cx);
+            #[allow(clippy::reversed_empty_ranges)]
+            let reversed = 5..1;
+            present(
+                &editor,
+                vec![0..2, 3..5, 50..60, 9..20, reversed, 2..2],
+                vec![(1, 1.6), (7, 2.), (0, -1.), (2, f32::NAN), (2, 0.5)],
+                cx,
+            );
+            assert_eq!(
+                editor.read(cx).editor_presentation(),
+                EditorPresentation {
+                    concealed: vec![0..2, 3..5],
+                    line_scales: vec![(1, 1.6)],
+                }
+            );
+            window.draw(cx).clear(cx);
+
+            // An edit above shifts everything after it.
+            editor.update(cx, |state, cx| {
+                state.replace_text_in_range(Some(0..0), "xx\n", window, cx)
+            });
+            assert_eq!(
+                editor.read(cx).editor_presentation(),
+                EditorPresentation {
+                    concealed: vec![3..5, 6..8],
+                    line_scales: vec![(2, 1.6)],
+                }
+            );
+            window.draw(cx).clear(cx);
+
+            // An edit inside a concealed range drops it; typing within the heading keeps
+            // its scale.
+            editor.update(cx, |state, cx| {
+                state.replace_text_in_range(Some(4..4), "z", window, cx);
+                let heading = state.text().to_string().find("# H").unwrap() + 3;
+                state.replace_text_in_range(Some(heading..heading), "ey", window, cx);
+            });
+            assert_eq!(
+                editor.read(cx).editor_presentation(),
+                EditorPresentation {
+                    concealed: vec![7..9],
+                    line_scales: vec![(2, 1.6)],
+                }
+            );
+            window.draw(cx).clear(cx);
+
+            // IME composition on a concealed, scaled line lays out.
+            editor.update(cx, |state, cx| {
+                state.set_selected_range(8..8, cx);
+                state.replace_and_mark_text_in_range(None, "かな", None, window, cx);
+            });
+            window.draw(cx).clear(cx);
+            editor.update(cx, |state, cx| state.unmark_text(window, cx));
+
+            // Joining the heading into the line above drops the scale.
+            editor.update(cx, |state, cx| {
+                let heading = state.text().to_string().find("# H").unwrap();
+                let heading = state.text().offset_to_offset_utf16(heading);
+                state.replace_text_in_range(Some(heading - 1..heading), "", window, cx);
+            });
+            assert!(editor.read(cx).editor_presentation().line_scales.is_empty());
+            window.draw(cx).clear(cx);
+
+            // Replacing everything leaves nothing stale behind.
+            present(&editor, vec![0..1], vec![(0, 1.4), (1, 1.2)], cx);
+            editor.update(cx, |state, cx| state.set_value("", window, cx));
+            window.draw(cx).clear(cx);
+            assert!(editor.read(cx).editor_presentation().concealed.is_empty());
+            editor.update(cx, |state, cx| state.set_presentation(None, cx));
+            window.draw(cx).clear(cx);
+            assert!(editor.read(cx).editor_presentation().is_empty());
+        });
+    }
+
+    #[gpui::test]
+    fn presentation_vertical_moves_step_display_rows_and_keep_x(cx: &mut TestAppContext) {
+        let text = "body text here\n# Heading\nmore text here";
+        let (editor, window) = decoration_editor(cx, text, false);
+        let mut cx = VisualTestContext::from_window(window.into(), cx);
+        cx.update(|window, cx| {
+            present(&editor, vec![0..0, 15..17], vec![(1, 2.)], cx);
+            editor.update(cx, |state, cx| state.set_selected_range(5..5, cx));
+            window.draw(cx).clear(cx);
+            editor.update(cx, |state, cx| {
+                state.down(&crate::input::MoveDown, window, cx)
+            });
+            window.draw(cx).clear(cx);
+            let on_heading = editor.read(cx).cursor();
+            assert!((15..25).contains(&on_heading), "{on_heading}");
+            editor.update(cx, |state, cx| {
+                state.down(&crate::input::MoveDown, window, cx)
+            });
+            window.draw(cx).clear(cx);
+            let below = editor.read(cx).cursor();
+            assert_eq!(below, text.rfind("more").unwrap() + 5);
+            editor.update(cx, |state, cx| state.up(&crate::input::MoveUp, window, cx));
+            window.draw(cx).clear(cx);
+            assert_eq!(editor.read(cx).cursor(), on_heading);
+        });
+    }
+
+    #[test]
+    fn concealed_ranges_are_checked_against_the_line_at_layout() {
+        let line = "aé b";
+        // `é` is bytes 1..3.
+        assert_eq!(
+            concealed_in_line(&[0..1, 2..3, 3..4, 4..9, 10..11], 0, line),
+            vec![0..1, 3..4]
+        );
+        // Before the line, then `é` and the space as absolute offsets.
+        assert_eq!(
+            concealed_in_line(&[1..3, 6..8, 8..9], 5, line),
+            vec![1..3, 3..4]
+        );
+        assert!(concealed_in_line(&[], 0, line).is_empty());
+    }
+
+    #[gpui::test]
+    fn no_presentation_keeps_the_layout_unchanged(cx: &mut TestAppContext) {
+        let text = "alpha beta gamma delta epsilon zeta eta theta\n\n    short\nlast line";
+        let (editor, window) = decoration_editor(cx, text, true);
+        let mut cx = VisualTestContext::from_window(window.into(), cx);
+        cx.update(|window, cx| {
+            editor.update(cx, |state, cx| state.set_selected_range(20..20, cx));
+            let snapshot = |window: &mut Window, cx: &mut App| {
+                window.draw(cx).clear(cx);
+                let state = editor.read(cx);
+                let layout = state.last_layout.as_ref().unwrap();
+                (
+                    layout.visible_top,
+                    layout.visible_buffer_lines.clone(),
+                    layout
+                        .lines
+                        .iter()
+                        .map(|line| {
+                            (
+                                line.size(layout.line_height),
+                                line.wrapped_lines
+                                    .iter()
+                                    .map(|row| (row.len, row.width))
+                                    .collect::<Vec<_>>(),
+                            )
+                        })
+                        .collect::<Vec<_>>(),
+                    layout.cursor_bounds,
+                    state.scroll_size,
+                )
+            };
+            let before = snapshot(window, cx);
+            present(&editor, vec![0..6], vec![(0, 1.6), (2, 1.2)], cx);
+            assert_ne!(snapshot(window, cx), before);
+            editor.update(cx, |state, cx| state.set_presentation(None, cx));
+            assert_eq!(snapshot(window, cx), before);
+            editor.update(cx, |state, cx| {
+                state.set_presentation(Some(EditorPresentation::default()), cx)
+            });
+            assert_eq!(snapshot(window, cx), before);
+        });
+    }
+
     #[gpui::test]
     fn geometric_decorations_track_edits_history_replacement_and_owner_lifetime(
         cx: &mut TestAppContext,
diff --git a/src/input/base/movement.rs b/src/input/base/movement.rs
index 10304df2..ea8c7fe1 100644
--- a/src/input/base/movement.rs
+++ b/src/input/base/movement.rs
@@ -155,7 +155,7 @@ impl<M: InputModeKind> InputBaseState<M> {
                 if let Some((x, line_end_affinity)) = line.closest_index_for_position(
                     Point {
                         x: preferred_x,
-                        y: next_display_point.local_row * last_layout.line_height,
+                        y: next_display_point.local_row * line.row_height(last_layout.line_height),
                     },
                     last_layout,
                 ) {
diff --git a/src/input/base/state.rs b/src/input/base/state.rs
index d35f0834..174e8560 100644
--- a/src/input/base/state.rs
+++ b/src/input/base/state.rs
@@ -1017,7 +1017,7 @@ impl<M: InputModeKind> InputBaseState<M> {
             let prev_lines_offset = last_layout.visible_line_byte_offsets[vi];
             let local_offset = offset.saturating_sub(prev_lines_offset);
             if let Some(pos) = line.position_for_index(local_offset, last_layout, false) {
-                let sub_line_index = (pos.y / line_height) as usize;
+                let sub_line_index = (pos.y / line.row_height(line_height)) as usize;
                 let adjusted_pos = point(pos.x + last_layout.line_number_width, pos.y + y_offset);
                 return (vi, sub_line_index, Some(adjusted_pos));
             }
@@ -2675,7 +2675,14 @@ impl<M: InputModeKind> InputBaseState<M> {
         let display_pos = self
             .display_map
             .buffer_pos_to_display_pos(crate::input::BufferPoint::new(row, point.column));
-        let row_offset_y = line_height * display_pos.row;
+        let row_offset_y = self
+            .display_map
+            .display_row_top(display_pos.row, line_height);
+        // How far a scaled row reaches below a uniform one.
+        let below_row = self
+            .display_map
+            .display_row_height(display_pos.row, line_height)
+            - line_height;
 
         // For Right alignment use 0 margin: the cursor indicator is clamped inside bounds
         // in layout_cursors, so shifting the text here would cause a first-click visual jump.
@@ -2721,9 +2728,9 @@ impl<M: InputModeKind> InputBaseState<M> {
         if row_offset_y - edge_height + line_height < -scroll_offset.y {
             // Scroll up
             scroll_offset.y = -row_offset_y + edge_height - line_height;
-        } else if row_offset_y + edge_height > -scroll_offset.y + bounds.size.height {
+        } else if row_offset_y + below_row + edge_height > -scroll_offset.y + bounds.size.height {
             // Scroll down
-            scroll_offset.y = -(row_offset_y - bounds.size.height + edge_height);
+            scroll_offset.y = -(row_offset_y + below_row - bounds.size.height + edge_height);
         }
 
         // Avoid necessary scroll, when it was already in the correct position.
@@ -3310,7 +3317,9 @@ impl<M: InputModeKind> InputBaseState<M> {
             .zip(last_line_pos)
             .zip(last_layout.visible_line_byte_offsets.last())
             .map(|((line_layout, pos), line_start)| {
-                let last_row_top = (line_layout.size(line_height).height - line_height).max(px(0.));
+                let last_row_top = (line_layout.size(line_height).height
+                    - line_layout.row_height(line_height))
+                .max(px(0.));
                 let pos = point(pos.x, last_row_top);
                 (
                     self.resolve_index(line_start + line_layout.len()),
@@ -3814,7 +3823,15 @@ impl<M: InputModeKind> InputBaseState<M> {
 
         Some(Bounds::from_corners(
             last_bounds.origin + start_pos,
-            last_bounds.origin + end_pos + point(px(0.), last_layout.line_height),
+            last_bounds.origin
+                + end_pos
+                + point(
+                    px(0.),
+                    last_layout.line_height
+                        * self
+                            .display_map
+                            .line_scale(self.text.offset_to_point(range.end).row),
+                ),
         ))
     }
 
@@ -4563,7 +4580,16 @@ impl<M: InputModeKind> EntityInputHandler for InputBaseState<M> {
         Some(Bounds::from_corners(
             bounds.origin + line_number_origin + start_origin,
             // + line_height for show IME panel under the cursor line.
-            bounds.origin + line_number_origin + point(end_origin.x, end_origin.y + line_height),
+            bounds.origin
+                + line_number_origin
+                + point(
+                    end_origin.x,
+                    end_origin.y
+                        + line_height
+                            * self
+                                .display_map
+                                .line_scale(self.text.offset_to_point(range.start).row),
+                ),
         ))
     }
 
diff --git a/src/input/base/touch.rs b/src/input/base/touch.rs
index e3bd2d26..19413815 100644
--- a/src/input/base/touch.rs
+++ b/src/input/base/touch.rs
@@ -14,7 +14,7 @@
 
 use gpui::{Context, LongPressEvent, Pixels, Point, TouchPhase, Window, point};
 
-use super::{InputBaseState, InputModeKind};
+use super::{InputBaseState, InputModeKind, RopeExt as _};
 use crate::touch_selection::{
     EdgeDrag, SelectionEdge, TouchSelectionSnapshot, caret_in_view, caret_line_box,
 };
@@ -50,8 +50,11 @@ impl<M: InputModeKind> InputBaseState<M> {
         // side, which is all the other end's drag needs to know about it.
         let caret_box = |offset: usize, stand_in_y: Pixels| {
             let (_, _, position) = self.line_and_position_for_offset(offset);
+            // A presentation's scaled line has taller rows, and so a taller caret.
+            let row = self.text.offset_to_point(offset).row;
+            let row_height = line_height * self.display_map.line_scale(row);
             match position.filter(|_| laid_out.contains(&offset) || laid_out.end == offset) {
-                Some(position) => caret_line_box(origin + position, line_height),
+                Some(position) => caret_line_box(origin + position, row_height),
                 None => caret_line_box(point(viewport.left(), stand_in_y), line_height),
             }
         };
diff --git a/src/input/editor/display_map/display_map.rs b/src/input/editor/display_map/display_map.rs
index 650cd718..9facfe2f 100644
--- a/src/input/editor/display_map/display_map.rs
+++ b/src/input/editor/display_map/display_map.rs
@@ -6,7 +6,7 @@
 /// - Automatic projection updates on text/layout changes
 use std::ops::Range;
 
-use gpui::{App, Font, Pixels};
+use gpui::{App, Font, Pixels, px};
 use ropey::Rope;
 
 use super::fold_map::FoldMap;
@@ -29,6 +29,19 @@ use crate::input::rope_ext::RopeExt as _;
 pub struct DisplayMap {
     wrap_map: WrapMap,
     fold_map: FoldMap,
+    /// Visible display rows of the presentation's scaled lines, sorted, rebuilt with the
+    /// fold projection. Empty without a presentation, which keeps rows uniformly tall.
+    scaled_rows: Vec<ScaledRows>,
+    /// Sum over `scaled_rows` of `(scale - 1) * rows`, in line heights.
+    scaled_extra: f32,
+}
+
+/// The display rows of one scaled buffer line.
+struct ScaledRows {
+    rows: Range<usize>,
+    scale: f32,
+    /// Extra height, in line heights, of the scaled rows above these.
+    extra_before: f32,
 }
 
 impl DisplayMap {
@@ -36,6 +49,8 @@ impl DisplayMap {
         Self {
             wrap_map: WrapMap::new(font, font_size, wrap_width),
             fold_map: FoldMap::new(),
+            scaled_rows: Vec::new(),
+            scaled_extra: 0.,
         }
     }
 
@@ -266,6 +281,107 @@ impl DisplayMap {
         self.rebuild_fold_projection();
     }
 
+    // ==================== Presentation ====================
+
+    /// Replace the render-only presentation: byte ranges laid out at zero width and
+    /// per-buffer-line scales. Both must already be validated against the text (see
+    /// `EditorPresentation`). Returns whether anything changed.
+    pub(crate) fn set_presentation(
+        &mut self,
+        concealed: std::rc::Rc<[Range<usize>]>,
+        line_scales: std::rc::Rc<[(usize, f32)]>,
+        cx: &mut App,
+    ) -> bool {
+        let changed = self.wrap_map.set_presentation(concealed, line_scales, cx);
+        if changed {
+            self.rebuild_fold_projection();
+        }
+        changed
+    }
+
+    /// The concealed byte ranges, sorted and disjoint.
+    #[inline]
+    pub(crate) fn concealed(&self) -> &[Range<usize>] {
+        self.wrap_map.wrapper().concealed()
+    }
+
+    /// The `(buffer line, scale)` pairs of the presentation, sorted by line.
+    #[inline]
+    pub(crate) fn line_scales(&self) -> &[(usize, f32)] {
+        self.wrap_map.wrapper().line_scales()
+    }
+
+    /// Whether a presentation conceals or scales anything.
+    #[inline]
+    pub(crate) fn has_presentation(&self) -> bool {
+        !self.concealed().is_empty() || !self.line_scales().is_empty()
+    }
+
+    /// The scale of buffer line `line`: `1.0` unless the presentation scales it.
+    #[inline]
+    pub(crate) fn line_scale(&self, line: usize) -> f32 {
+        self.wrap_map.wrapper().line_scale(line)
+    }
+
+    /// The y of the top of display row `row`, in content space.
+    ///
+    /// Rows are `line_height` tall except those of scaled lines, so without a presentation
+    /// this is `line_height * row`.
+    pub(crate) fn display_row_top(&self, row: usize, line_height: Pixels) -> Pixels {
+        if self.scaled_rows.is_empty() {
+            return line_height * row;
+        }
+        let ix = self.scaled_rows.partition_point(|s| s.rows.start < row);
+        let Some(scaled) = ix.checked_sub(1).map(|ix| &self.scaled_rows[ix]) else {
+            return line_height * row;
+        };
+        let inside = row.min(scaled.rows.end) - scaled.rows.start;
+        line_height * (row as f32 + scaled.extra_before + (scaled.scale - 1.) * inside as f32)
+    }
+
+    /// The height of display row `row`.
+    pub(crate) fn display_row_height(&self, row: usize, line_height: Pixels) -> Pixels {
+        let ix = self.scaled_rows.partition_point(|s| s.rows.end <= row);
+        match self.scaled_rows.get(ix) {
+            Some(scaled) if scaled.rows.contains(&row) => line_height * scaled.scale,
+            _ => line_height,
+        }
+    }
+
+    /// The display row under content-space `y` (rows past the end continue at
+    /// `line_height`, so the caller clamps).
+    pub(crate) fn display_row_at_y(&self, y: Pixels, line_height: Pixels) -> usize {
+        let y = y.max(px(0.));
+        let uniform = |y: Pixels, base: usize| {
+            base + (f32::from(y) / f32::from(line_height)).floor() as usize
+        };
+        let ix = self
+            .scaled_rows
+            .partition_point(|s| self.display_row_top(s.rows.start, line_height) <= y);
+        let Some(scaled) = ix.checked_sub(1).map(|ix| &self.scaled_rows[ix]) else {
+            return uniform(y, 0);
+        };
+        let top = self.display_row_top(scaled.rows.start, line_height);
+        let row_height = line_height * scaled.scale;
+        let bottom = top + row_height * scaled.rows.len() as f32;
+        if y < bottom {
+            scaled.rows.start + (f32::from(y - top) / f32::from(row_height)).floor() as usize
+        } else {
+            uniform(y - bottom, scaled.rows.end)
+        }
+    }
+
+    /// The height of every wrap row, for the scroll size: `line_height` each, plus the extra
+    /// of the visible scaled rows.
+    pub(crate) fn wrap_rows_height(&self, line_height: Pixels) -> Pixels {
+        let rows = self.wrap_map.wrap_row_count() as f32;
+        if self.scaled_rows.is_empty() {
+            rows * line_height
+        } else {
+            (rows + self.scaled_extra) * line_height
+        }
+    }
+
     // ==================== Internal Helpers ====================
 
     /// Rebuild fold projection after wrap_map or fold state changes
@@ -279,6 +395,32 @@ impl DisplayMap {
             self.fold_map
                 .mark_dirty_with_wrap_count(self.wrap_map.wrap_row_count());
         }
+        self.rebuild_scaled_rows();
+    }
+
+    /// Recompute the display rows of the scaled lines. A folded line has none, so it adds no
+    /// height.
+    fn rebuild_scaled_rows(&mut self) {
+        self.scaled_rows.clear();
+        self.scaled_extra = 0.;
+        if self.line_scales().is_empty() {
+            return;
+        }
+        let mut extra = 0.;
+        let mut scaled_rows = Vec::with_capacity(self.line_scales().len());
+        for &(line, scale) in self.line_scales() {
+            if let Some(rows) = self.buffer_line_to_display_row_range(line) {
+                let len = rows.len() as f32;
+                scaled_rows.push(ScaledRows {
+                    rows,
+                    scale,
+                    extra_before: extra,
+                });
+                extra += (scale - 1.) * len;
+            }
+        }
+        self.scaled_rows = scaled_rows;
+        self.scaled_extra = extra;
     }
 
     // ==================== Wrap Display Point Operations ====================
diff --git a/src/input/editor/display_map/inline_line.rs b/src/input/editor/display_map/inline_line.rs
index b45fd3ba..ec0bceb9 100644
--- a/src/input/editor/display_map/inline_line.rs
+++ b/src/input/editor/display_map/inline_line.rs
@@ -66,7 +66,9 @@ impl InputLine {
                 for f in fragments {
                     if x <= f.x + f.width {
                         return f.text.as_ref().map_or(
-                            if x - f.x < f.width / 2. {
+                            // A concealed range has no width to split, so a click at
+                            // its point lands after it, past the hidden source.
+                            if f.width > px(0.) && x - f.x < f.width / 2. {
                                 f.range.start
                             } else {
                                 f.range.end
@@ -79,6 +81,15 @@ impl InputLine {
             }
         }
     }
+    /// Whether the byte at `ix` is concealed: inside a fragment drawn at zero width.
+    pub(crate) fn is_concealed(&self, ix: usize) -> bool {
+        match &self.content {
+            Content::Text(_) => false,
+            Content::Inline(fragments) => fragments
+                .iter()
+                .any(|f| f.text.is_none() && f.width <= px(0.) && f.range.contains(&ix)),
+        }
+    }
     pub(crate) fn index_for_x(&self, x: Pixels) -> Option<usize> {
         match &self.content {
             Content::Text(line) => line.index_for_x(x),
diff --git a/src/input/editor/display_map/text_wrapper.rs b/src/input/editor/display_map/text_wrapper.rs
index 4c59bf27..f511ed9a 100644
--- a/src/input/editor/display_map/text_wrapper.rs
+++ b/src/input/editor/display_map/text_wrapper.rs
@@ -228,6 +228,12 @@ pub(crate) struct TextWrapper {
     pub(crate) lines: SumTree<LineItem>,
 
     inline_metrics: Rc<[(Range<usize>, Pixels)]>,
+    /// Byte ranges laid out at zero width (render-only presentation): sorted, disjoint, each
+    /// within one line. Ignored for lines that have inline token metrics.
+    concealed: Rc<[Range<usize>]>,
+    /// `(buffer line, scale)` sorted by line, every scale finite and above 1. A scaled line is
+    /// wrapped at `wrap_width / scale`, so its text fits once it is shaped `scale` times larger.
+    line_scales: Rc<[(usize, f32)]>,
     _initialized: bool,
 }
 
@@ -242,6 +248,8 @@ impl TextWrapper {
             wrapping_indent: WrappingIndent::default(),
             lines: SumTree::new(&()),
             inline_metrics: Rc::from([]),
+            concealed: Rc::from([]),
+            line_scales: Rc::from([]),
             _initialized: false,
         }
     }
@@ -373,6 +381,7 @@ impl TextWrapper {
             .text_system()
             .line_wrapper(self.font.clone(), self.font_size);
         let metrics = self.inline_metrics.clone();
+        let concealed = self.concealed.clone();
         let text_system = gpui::WindowTextSystem::new(cx.text_system().clone());
         let font = self.font.clone();
         let font_size = self.font_size;
@@ -404,6 +413,29 @@ impl TextWrapper {
                     fragments.push(LineFragment::element(*width, range.len()));
                     offset = range.end;
                 }
+                // Concealed text takes no room, so it wraps as a zero-width element. A line
+                // with inline tokens keeps them alone.
+                if fragments.is_empty() {
+                    let line_end = line_start + line_str.len();
+                    let first = concealed.partition_point(|r| r.end <= line_start);
+                    for range in concealed[first..].iter().take_while(|r| r.start < line_end) {
+                        if range.start < line_start || range.end > line_end {
+                            continue;
+                        }
+                        let range = range.start - line_start..range.end - line_start;
+                        if range.start < offset
+                            || !line_str.is_char_boundary(range.start)
+                            || !line_str.is_char_boundary(range.end)
+                        {
+                            continue;
+                        }
+                        if offset < range.start {
+                            fragments.push(LineFragment::text(&line_str[offset..range.start]));
+                        }
+                        fragments.push(LineFragment::element(px(0.), range.len()));
+                        offset = range.end;
+                    }
+                }
                 if fragments.is_empty() {
                     return measured_wrap_boundaries(
                         line_str,
@@ -515,6 +547,155 @@ impl TextWrapper {
         }
     }
 
+    /// The concealed byte ranges of the presentation.
+    #[inline]
+    pub(crate) fn concealed(&self) -> &[Range<usize>] {
+        &self.concealed
+    }
+
+    /// The `(buffer line, scale)` pairs of the presentation, sorted by line.
+    #[inline]
+    pub(crate) fn line_scales(&self) -> &[(usize, f32)] {
+        &self.line_scales
+    }
+
+    /// The scale of buffer line `row`, `1.0` unless the presentation scales it.
+    #[inline]
+    pub(crate) fn line_scale(&self, row: usize) -> f32 {
+        if self.line_scales.is_empty() {
+            return 1.;
+        }
+        self.line_scales
+            .binary_search_by_key(&row, |(line, _)| *line)
+            .map_or(1., |ix| self.line_scales[ix].1)
+    }
+
+    /// Replace the render-only presentation, already validated against the text, and rewrap
+    /// the lines whose conceal or scale changed. Returns whether anything changed.
+    pub(crate) fn set_presentation(
+        &mut self,
+        concealed: Rc<[Range<usize>]>,
+        line_scales: Rc<[(usize, f32)]>,
+        cx: &mut App,
+    ) -> bool {
+        if self.concealed == concealed && self.line_scales == line_scales {
+            return false;
+        }
+        let text = self.text.clone();
+        let row_of = |offset: usize| text.offset_to_point(offset.min(text.len())).row;
+        let mut rows: Vec<usize> = Vec::new();
+        // Ranges in only one of the two lists, found the way `set_inline_metrics` does.
+        let (mut old, mut new) = (
+            self.concealed.iter().peekable(),
+            concealed.iter().peekable(),
+        );
+        loop {
+            match (old.peek(), new.peek()) {
+                (Some(a), Some(b)) if a == b => {
+                    old.next();
+                    new.next();
+                }
+                (Some(a), Some(b)) if a.start <= b.start => {
+                    rows.push(row_of(a.start));
+                    old.next();
+                }
+                (_, Some(b)) => {
+                    rows.push(row_of(b.start));
+                    new.next();
+                }
+                (Some(a), None) => {
+                    rows.push(row_of(a.start));
+                    old.next();
+                }
+                (None, None) => break,
+            }
+        }
+        for &(line, _) in self.line_scales.iter().chain(line_scales.iter()) {
+            let new_scale = line_scales
+                .binary_search_by_key(&line, |(line, _)| *line)
+                .map_or(1., |ix| line_scales[ix].1);
+            if self.line_scale(line) != new_scale {
+                rows.push(line);
+            }
+        }
+        self.concealed = concealed;
+        self.line_scales = line_scales;
+        if !self._initialized {
+            // The first `prepare_if_need` wraps everything with the new presentation.
+            return true;
+        }
+        rows.sort_unstable();
+        rows.dedup();
+        let line_count = self.lines_count();
+        for row in rows.into_iter().filter(|&row| row < line_count) {
+            let start = text.line_start_offset(row);
+            let end = text.line_end_offset(row);
+            self.update(
+                &text,
+                &(start..end),
+                &Rope::from(text.slice(start..end).to_string()),
+                cx,
+            );
+        }
+        true
+    }
+
+    /// Carry the presentation across an edit of the current text, before the wrapped lines
+    /// are updated for it.
+    ///
+    /// Concealed ranges the edit touches are dropped and later ones shift with it, the way
+    /// inline metrics do. Scales of the lines the edit spans are dropped and later lines'
+    /// shift by the change in line count, except that an edit within one line keeps that
+    /// line's scale (typing in a heading leaves it a heading). The owner is expected to set a
+    /// fresh presentation after the edit; this only keeps the stale one safe and close.
+    pub(crate) fn adjust_presentation_for_edit(&mut self, range: &Range<usize>, new_text: &Rope) {
+        if !self.concealed.is_empty() {
+            let new_len = new_text.len();
+            let shift = new_len as isize - range.len() as isize;
+            self.concealed = self
+                .concealed
+                .iter()
+                .filter_map(|r| {
+                    if r.start < range.end && range.start < r.end {
+                        return None;
+                    }
+                    if r.start >= range.end {
+                        Some(r.start.checked_add_signed(shift)?..r.end.checked_add_signed(shift)?)
+                    } else {
+                        Some(r.clone())
+                    }
+                })
+                .collect();
+        }
+        if !self.line_scales.is_empty() {
+            let len = self.text.len();
+            let start_line = self.text.offset_to_point(range.start.min(len)).row;
+            let end_line = self.text.offset_to_point(range.end.min(len)).row;
+            let new_lines = new_text.lines_len().saturating_sub(1);
+            let delta = new_lines as isize - (end_line - start_line) as isize;
+            let in_place = start_line == end_line && new_lines == 0;
+            self.line_scales = self
+                .line_scales
+                .iter()
+                .filter_map(|&(line, scale)| {
+                    if line < start_line || (in_place && line == start_line) {
+                        Some((line, scale))
+                    } else if line > end_line {
+                        Some((line.checked_add_signed(delta)?, scale))
+                    } else {
+                        None
+                    }
+                })
+                .collect();
+        }
+    }
+
+    /// Drop the presentation without rewrapping, for a text that replaces the whole buffer.
+    pub(crate) fn clear_presentation(&mut self) {
+        self.concealed = Rc::from([]);
+        self.line_scales = Rc::from([]);
+    }
+
     fn _update<F>(
         &mut self,
         changed_text: &Rope,
@@ -549,6 +730,13 @@ impl TextWrapper {
 
             // If wrap_width is Pixels::MAX, skip wrapping to disable word wrap
             if let Some(wrap_width) = wrap_width {
+                // A scaled line is shaped larger, so it wraps narrower.
+                let scale = self.line_scale(row);
+                let wrap_width = if scale != 1. {
+                    wrap_width / scale
+                } else {
+                    wrap_width
+                };
                 // Borrowed for lines within a single rope chunk.
                 let line_str: Cow<str> = line.into();
                 match self.wrapping_indent {
@@ -749,6 +937,9 @@ pub(crate) struct LineLayout {
     /// Whether any run of this line carries a background color, so [`Self::paint_background`]
     /// can skip the glyph walk for the common case of a line without highlights.
     has_background: bool,
+    /// Height of each visual line as a multiple of the editor's line height, `1.0` unless
+    /// the presentation scales this line.
+    scale: f32,
 }
 
 impl LineLayout {
@@ -761,9 +952,22 @@ impl LineLayout {
             whitespace_chars: Vec::new(),
             whitespace_indicators: None,
             has_background: false,
+            scale: 1.,
         }
     }
 
+    /// Set the line's scale: each visual line is `line_height * scale` tall.
+    pub(crate) fn with_scale(mut self, scale: f32) -> Self {
+        self.scale = scale;
+        self
+    }
+
+    /// The height of one visual line of this layout.
+    #[inline]
+    pub(crate) fn row_height(&self, line_height: Pixels) -> Pixels {
+        line_height * self.scale
+    }
+
     /// Record whether any run of this line carries a background color.
     pub(crate) fn with_background(mut self, has_background: bool) -> Self {
         self.has_background = has_background;
@@ -824,7 +1028,7 @@ impl LineLayout {
 
         for (line_index, wrapped_line) in self.wrapped_lines.iter().enumerate() {
             for (relative_offset, c) in wrapped_line.text.char_indices() {
-                if matches!(c, ' ' | '\t') {
+                if matches!(c, ' ' | '\t') && !wrapped_line.is_concealed(relative_offset) {
                     let is_tab = c == '\t';
                     let start_x = wrapped_line.x_for_index(relative_offset);
                     let end_x = wrapped_line.x_for_index(relative_offset + c.len_utf8());
@@ -888,7 +1092,7 @@ impl LineLayout {
             // Always advance by actual line length. The last line gets +1 so the
             // cursor can be placed after the final character.
             acc_len += if is_last { line.len + 1 } else { line.len };
-            offset_y += last_layout.line_height;
+            offset_y += self.row_height(last_layout.line_height);
         }
 
         None
@@ -929,7 +1133,7 @@ impl LineLayout {
         let x_offset = last_layout.alignment_offset(self.longest_width);
 
         for (i, line) in self.wrapped_lines.iter().enumerate() {
-            let line_bottom = line_top + last_layout.line_height;
+            let line_bottom = line_top + self.row_height(last_layout.line_height);
             if pos.y >= line_top && pos.y < line_bottom {
                 return Some((i, offset, pos.x - x_offset - self.line_indent(i)));
             }
@@ -1013,7 +1217,10 @@ impl LineLayout {
             .map(|(ix, line)| line.width + self.line_indent(ix))
             .max()
             .unwrap_or(self.longest_width);
-        size(width, self.wrapped_lines.len() * line_height)
+        size(
+            width,
+            self.wrapped_lines.len() * self.row_height(line_height),
+        )
     }
 
     /// Paint only the glyph background quads of this line.
@@ -1035,6 +1242,7 @@ impl LineLayout {
             return;
         }
 
+        let line_height = self.row_height(line_height);
         for (ix, line) in self.wrapped_lines.iter().enumerate() {
             _ = line.paint_background(
                 pos + point(self.line_indent(ix), ix * line_height),
@@ -1056,6 +1264,7 @@ impl LineLayout {
         window: &mut Window,
         cx: &mut App,
     ) {
+        let line_height = self.row_height(line_height);
         for (ix, line) in self.wrapped_lines.iter().enumerate() {
             _ = line.paint(
                 pos + point(self.line_indent(ix), ix * line_height),
diff --git a/src/input/editor/display_map/wrap_map.rs b/src/input/editor/display_map/wrap_map.rs
index 0cbb3d04..eccd6e7f 100644
--- a/src/input/editor/display_map/wrap_map.rs
+++ b/src/input/editor/display_map/wrap_map.rs
@@ -108,9 +108,20 @@ impl WrapMap {
         cx: &mut App,
     ) {
         self.wrapper.adjust_inline_metrics(range, new_text.len());
+        self.wrapper.adjust_presentation_for_edit(range, new_text);
         self.wrapper.update(changed_text, range, new_text, cx);
     }
 
+    /// Replace the render-only presentation. Returns whether anything changed.
+    pub(super) fn set_presentation(
+        &mut self,
+        concealed: std::rc::Rc<[Range<usize>]>,
+        line_scales: std::rc::Rc<[(usize, f32)]>,
+        cx: &mut App,
+    ) -> bool {
+        self.wrapper.set_presentation(concealed, line_scales, cx)
+    }
+
     /// Update layout parameters (wrap width or font)
     pub(super) fn set_inline_metrics(
         &mut self,
@@ -141,6 +152,7 @@ impl WrapMap {
 
     /// Initialize with text
     pub(super) fn set_text(&mut self, text: &Rope, cx: &mut App) {
+        self.wrapper.clear_presentation();
         self.wrapper.set_default_text(text);
         self.wrapper.prepare_if_need(text, cx);
     }
diff --git a/src/input/editor/indent.rs b/src/input/editor/indent.rs
index d50bff1c..34573f37 100644
--- a/src/input/editor/indent.rs
+++ b/src/input/editor/indent.rs
@@ -128,6 +128,7 @@ impl<M: InputModeKind> TextElement<M> {
             .zip(last_layout.lines.iter())
         {
             let line = state.text.slice_line(buffer_line);
+            let line_height = line_layout.row_height(line_height);
             let mut current_indents = vec![];
             if line.len() > 0 {
                 let indent_count = tab_size.indent_count(&line);
@@ -153,7 +154,7 @@ impl<M: InputModeKind> TextElement<M> {
                 current_indents = last_indents.clone();
             }
 
-            offset_y += line_layout.wrapped_lines.len() * line_height;
+            offset_y += line_layout.size(last_layout.line_height).height;
             last_indents = current_indents;
         }
 
diff --git a/src/input/mod.rs b/src/input/mod.rs
index 13f64102..671ee46a 100644
--- a/src/input/mod.rs
+++ b/src/input/mod.rs
@@ -61,6 +61,8 @@ mod mode;
 mod movement;
 #[path = "base/native.rs"]
 mod native;
+#[path = "editor/presentation.rs"]
+mod presentation;
 #[path = "base/rope_ext.rs"]
 mod rope_ext;
 #[path = "editor/search.rs"]
@@ -117,6 +119,7 @@ pub use mask_pattern::MaskPattern;
 #[doc(hidden)]
 pub use native::set_text_content_type;
 pub use native::{NativeMenu, NativeMenuItem};
+pub use presentation::EditorPresentation;
 pub use rope_ext::{InputEdit, Point, RopeExt, RopeLines};
 pub use ropey::Rope;
 pub use search::{SearchMatcher, SearchSession};
diff --git a/src/input/editor/presentation.rs b/src/input/editor/presentation.rs
new file mode 100644
index 00000000..34bf1734
--- /dev/null
+++ b/src/input/editor/presentation.rs
@@ -0,0 +1,172 @@
+//! A render-only presentation of the code editor's text: ranges laid out at zero
+//! width, and lines drawn larger.
+//!
+//! Nothing here changes the text, the selections, or editing. Every offset keeps
+//! addressing the source, so a caret inside a concealed range sits at the point
+//! the range was hidden at, and a selection over one still selects its bytes.
+use std::ops::Range;
+use std::rc::Rc;
+
+use gpui::Context;
+use ropey::Rope;
+
+use super::{EditorState, RopeExt as _};
+
+/// A render-only presentation for an [`EditorState`], set with
+/// [`EditorState::set_presentation`].
+///
+/// Input is checked against the current text when it is set: a concealed range
+/// that is empty, out of bounds, off a char boundary, crosses a newline, or
+/// overlaps an earlier one is dropped, and so is a scale that is not finite,
+/// not above `1.0`, or for a line the text does not have.
+#[derive(Debug, Clone, Default, PartialEq)]
+pub struct EditorPresentation {
+    /// UTF-8 byte ranges laid out at zero width and not painted. Each lies
+    /// within one line (it may end at the line's end, not past its newline).
+    pub concealed: Vec<Range<usize>>,
+    /// `(buffer line, scale)`: the line's text is shaped `scale` times larger,
+    /// and each of its visual rows is `scale` line heights tall.
+    pub line_scales: Vec<(usize, f32)>,
+}
+
+impl EditorPresentation {
+    /// Whether the presentation conceals or scales nothing.
+    pub fn is_empty(&self) -> bool {
+        self.concealed.is_empty() && self.line_scales.is_empty()
+    }
+
+    /// The valid part of this presentation for `text`: sorted, disjoint
+    /// concealed ranges, and scales sorted by line (the first of duplicates
+    /// wins).
+    pub(crate) fn validate(&self, text: &Rope) -> (Rc<[Range<usize>]>, Rc<[(usize, f32)]>) {
+        let mut concealed: Vec<Range<usize>> = self
+            .concealed
+            .iter()
+            .filter(|range| {
+                range.start < range.end
+                    && range.end <= text.len()
+                    && text.is_char_boundary(range.start)
+                    && text.is_char_boundary(range.end)
+                    && text.offset_to_point(range.start).row == text.offset_to_point(range.end).row
+            })
+            .cloned()
+            .collect();
+        concealed.sort_by_key(|range| range.start);
+        let mut end = 0;
+        concealed.retain(|range| {
+            let keep = range.start >= end;
+            if keep {
+                end = range.end;
+            }
+            keep
+        });
+
+        let lines = text.lines_len();
+        let mut line_scales: Vec<(usize, f32)> = self
+            .line_scales
+            .iter()
+            .copied()
+            .filter(|&(line, scale)| line < lines && scale.is_finite() && scale > 1.)
+            .collect();
+        line_scales.sort_by_key(|&(line, _)| line);
+        line_scales.dedup_by_key(|&mut (line, _)| line);
+
+        (concealed.into(), line_scales.into())
+    }
+}
+
+impl EditorState {
+    /// Set, or with `None` clear, the render-only presentation of the text.
+    ///
+    /// Off by default; without one the editor lays out exactly as before. The
+    /// presentation is validated against the current text (see
+    /// [`EditorPresentation`]). Edits carry it along: a concealed range an edit
+    /// touches is dropped, and later ones shift; the scales of the lines an edit
+    /// spans are dropped (an edit within one line keeps that line's), and later
+    /// lines' shift with the line count. Replacing the whole text clears it.
+    /// It is meant to be set again after every change, so this only keeps a
+    /// stale presentation safe until then.
+    ///
+    /// Concealing is ignored while inline tokens are shown.
+    pub fn set_presentation(
+        &mut self,
+        presentation: Option<EditorPresentation>,
+        cx: &mut Context<Self>,
+    ) {
+        let (concealed, line_scales) = presentation
+            .map(|presentation| presentation.validate(&self.text))
+            .unwrap_or_else(|| (Rc::from([]), Rc::from([])));
+        if self
+            .display_map
+            .set_presentation(concealed, line_scales, cx)
+        {
+            cx.notify();
+        }
+    }
+
+    /// The presentation in effect: what [`Self::set_presentation`] kept,
+    /// carried through any edits since.
+    pub fn editor_presentation(&self) -> EditorPresentation {
+        EditorPresentation {
+            concealed: self.display_map.concealed().to_vec(),
+            line_scales: self.display_map.line_scales().to_vec(),
+        }
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    fn valid(presentation: EditorPresentation, text: &str) -> EditorPresentation {
+        let (concealed, line_scales) = presentation.validate(&Rope::from(text));
+        EditorPresentation {
+            concealed: concealed.to_vec(),
+            line_scales: line_scales.to_vec(),
+        }
+    }
+
+    #[test]
+    fn invalid_concealed_ranges_are_dropped() {
+        let text = "# héllo\n**b**\n";
+        let presentation = EditorPresentation {
+            concealed: vec![
+                9..11,
+                0..2,
+                1..2, // overlaps 0..2
+                3..4, // ends inside `é`
+                5..5, // empty
+                6..9, // crosses the newline
+                12..14,
+                40..42, // out of bounds
+                Range { start: 4, end: 3 },
+            ],
+            line_scales: vec![],
+        };
+        assert_eq!(
+            valid(presentation, text).concealed,
+            vec![0..2, 9..11, 12..14]
+        );
+    }
+
+    #[test]
+    fn invalid_scales_are_dropped() {
+        let presentation = EditorPresentation {
+            concealed: vec![],
+            line_scales: vec![
+                (2, 1.2),
+                (0, 1.6),
+                (0, 1.4),
+                (1, 0.5),
+                (1, 1.),
+                (1, f32::NAN),
+                (1, f32::INFINITY),
+                (9, 2.),
+            ],
+        };
+        assert_eq!(
+            valid(presentation, "a\nb\nc").line_scales,
+            vec![(0, 1.6), (2, 1.2)]
+        );
+    }
+}
```
