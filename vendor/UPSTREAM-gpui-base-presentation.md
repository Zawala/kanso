# Proposal: a render-only presentation for the gpui-base code editor

This is a change to offer to [GPUI Kit](https://github.com/longbridge/gpui-kit)
(`crates/base`), written against gpui-base 0.7.3. It hasn't been sent
anywhere.

## Summary

Let an `EditorState` be given a presentation of its text that changes only
how it is drawn:

- byte ranges laid out at zero width and not painted ("concealed");
- byte ranges drawn as other text ("replacements", such as a bullet for a
  list marker), atomic for the caret, optionally clickable; in a font
  family and at a size of their own (a code span), right-aligned in a
  column that can hang into the line's inset (an outside list marker), on a
  rounded box with an icon and a dashed underline (a code span, a date
  chip, a link to a missing page), or drawn as an image on the baseline (a
  typeset formula);
- buffer lines drawn larger or smaller (a scale per line: the line's text is
  shaped at `font_size * scale` and each of its visual rows is
  `line_height * scale` tall, wrapped at `wrap_width / scale`), with a row
  height factor of their own (a heading's tighter line height), and a base
  row height that is not rounded to whole pixels;
- empty space above and below a line, and a left inset for every row of a
  line (wrapped that much narrower);
- ornaments painted with a line: a vertical bar over its full height, a
  horizontal rule across the text area, or a task's check box;
- runs of whole lines drawn as an element of the app's ("block widgets"),
  in one row as tall as the element, the other lines hidden like a fold's.

Offsets, selections, editing, undo and search keep addressing the source
text. A click on a clickable replacement or on a block widget is reported
as an event. With no presentation set, nothing changes.

## Motivation

A Markdown editor that renders as you type (Typora, Obsidian's Live
Preview) keeps the Markdown source as the document but hides the marks
(`**`, `#`, link brackets and URLs) on the lines the caret is not on, draws
headings larger with the gaps a rendered page has around them, list markers
as bullets and task boxes, quotes with a bar and an indent, rules as rules,
and tables, images or diagrams as rendered blocks. The editor already has
everything such a view needs except a way to lay text out at a width,
height and position other than the uniform row: inline tokens lay a range
out as a fragment of its own width, but always paint an element there and
keep every row one line height tall, and folds hide lines but leave no room
for anything in their place.

An app cannot get this from outside. Overlaying its own text means
re-implementing hit testing, wrapping and caret placement; swapping the
text for a rendered one breaks offsets, undo and collaboration. Hiding text
needs only a zero-width fragment, which the row type already supports,
replacing it a fragment with other text, and taller, spaced or inset rows
need the editor's y and x arithmetic to stop assuming `row * line_height`
and a zero left edge.

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
    /// Source ranges drawn as other text.
    pub replacements: Vec<Replacement>,
    /// `(buffer line, above, below)`: space above the first row and below the last.
    pub line_space: Vec<(usize, Pixels, Pixels)>,
    /// `(buffer line, inset)`: every row starts `inset` further right.
    pub line_insets: Vec<(usize, Pixels)>,
    /// `(buffer line, ornament)`: drawn with the line, beneath its text.
    pub line_ornaments: Vec<(usize, Ornament)>,
    /// Runs of whole lines drawn as an element.
    pub block_widgets: Vec<BlockWidget>,
    /// `(buffer line, factor)`: its rows are `factor` times as tall as its scale makes them.
    pub line_heights: Vec<(usize, f32)>,
    /// The height of a row at scale 1, instead of the window's (rounded) line height.
    pub line_height: Option<Pixels>,
    /// Runs of lines laid out as one box (plus extra), rounded up to whole device pixels.
    pub leaves: Vec<(Range<usize>, Pixels)>,
}

pub struct Replacement {
    pub range: Range<usize>,             // within one line
    pub text: SharedString,              // one line
    pub style: HighlightStyle,           // over the editor's text style
    pub clickable: bool,                 // a click is reported, the caret stays
    pub family: Option<SharedString>,    // its own font family
    pub size: f32,                       // relative to the line's; keeps the row's baseline
    pub column: Option<ReplacementColumn>, // right-aligned in a column, maybe hanging
    pub chip: Option<ReplacementChip>,   // a rounded box behind it, with room either side
    pub image: Option<ReplacementImage>, // drawn as an image on the row's baseline
}

pub struct ReplacementColumn { pub width: Pixels, pub gap: Pixels, pub hanging: bool }
pub struct ReplacementChip {
    pub padding: Pixels,
    pub background: Hsla,
    pub radius: Pixels,
    pub height: ChipHeight,          // CapHeight, Fixed(h) or Baseline { ascent, descent }
    pub icon: Option<ChipIcon>,      // an SVG asset before the text, in its colour
    pub dashed_bottom: Option<Hsla>, // a dashed line along the box's bottom
    pub whole_width: bool,           // an element's: text width rounded up, box on device pixels
    pub text_baseline: Option<Pixels>, // the text's baseline below the box's top
}
pub struct ReplacementImage { pub image: Arc<Image>, pub size: Size<Pixels>, pub baseline: Pixels }

pub enum Ornament {
    /// Full line height (its space included), `x` from the text area's left edge.
    Bar { x: Pixels, width: Pixels, color: Hsla },
    /// Across the text area, centred on the line's rows.
    Rule { thickness: Pixels, color: Hsla },
    /// A check box `size` square at `x`, centred on the line's first row.
    Check { x: Pixels, size: Pixels, radius: Pixels, border: Pixels, border_color: Hsla,
            fill: Option<Hsla>, check: Option<Hsla>, row: Option<Pixels> },
}

pub type BlockWidgetRenderer = Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>;

#[derive(Clone)] // Debug and PartialEq by lines, height and key
pub struct BlockWidget {
    pub lines: Range<usize>, // buffer lines, start..end
    pub height: Pixels,      // until the element is measured
    pub key: u64,            // keeps the measured height
    pub render: BlockWidgetRenderer,
}

/// Emitted by the editor's state.
pub enum EditorPresentationEvent {
    ReplacementClicked { range: Range<usize> },
    BlockClicked { lines: Range<usize> },
}

impl EditorState {
    pub fn set_presentation(&mut self, presentation: Option<EditorPresentation>, cx: &mut Context<Self>);
    pub fn editor_presentation(&self) -> EditorPresentation;
}
```

Input is validated against the current text when it is set, and invalid
items are dropped: concealed ranges and replacements that are empty, out of
bounds, off a char boundary, cross a newline or overlap (a replacement must
not overlap a concealed range either, and its text must be one line);
scales and row height factors that are not finite, not above zero, 1.0, or
for lines the text does not have; a base row height that is not finite and
positive; space that is not finite, negative or zero; insets that are not
finite or not positive; ornaments with a non-finite or non-positive size;
block widgets that are empty, out of the text, overlap or have a
non-finite or non-positive height. Duplicates per line keep the first
(ornaments keep all). It never panics.

Edits carry the presentation along so it stays safe until the owner sets a
fresh one (an app is expected to set it again after every change): a
concealed range or replacement an edit touches is dropped and later ones
shift, as inline token metrics do; the per-line items of the lines an edit
spans are dropped (an edit within one line keeps that line's) and later
lines' shift by the change in line count; a block widget whose lines an
edit touches is dropped and later ones shift. Replacing the whole text
clears it. Layout checks concealed ranges and replacements against the line
once more before shaping.

Behaviour:

- A caret inside or at a concealed range is drawn at the point it was
  hidden at; a click on that point lands after the hidden text (a
  zero-width fragment resolves to its end).
- A replacement is shaped in the editor's text style refined by its style,
  at the line's (scaled) font size, and wraps as an element of its width. A
  caret inside or before it is at its start, after it at its end; a click on
  its left half lands at its start, on its right half at its end. A press on
  a clickable one emits `ReplacementClicked` and leaves the caret.
- A concealed range wraps as a zero-width element (the same path inline
  tokens take). Lines with inline tokens ignore concealing and replacing.
- Space, scale and block heights all go through one model in the display
  map: the rows that are not one line height (rebuilt with the fold
  projection, so a folded line adds no height), with the space above a line
  belonging to its first row and the space below to its last. It maps rows
  to and from y; the visible range, scroll size, carets (the caret is 0.85
  of its text row's height and sits in the row, inside the space),
  selections, search and hover highlights, range decorations, document
  colours, line numbers, the active line, fold icons, indent guides, hit
  testing (a point in a line's space lands on its nearest row), scroll to
  caret, IME and range bounds, touch handles and up/down moves all use it.
  `LineLayout` carries its scale and a frame (space, inset, block height),
  so every `line.size(line_height)` already measures the taller rows.
- An inset shifts every row of its line (continuation rows keep their wrap
  indent past it), and positions, hit testing and selections follow it.
- Ornaments are painted under the line's glyph backgrounds and change no
  layout.
- A block widget's first line is never wrapped and becomes one row as tall
  as the widget; its other lines are hidden the way a fold's inside is, so a
  vertical move steps from the line above onto the widget's row and then to
  the line below. The element is laid out at the text area's width and
  painted in that row each frame it is visible; its measured height
  replaces the given one from the next frame on (kept by key). Mouse events
  inside reach the element; a press it does not handle is kept from placing
  the caret, and a click it does not stop is reported as `BlockClicked`.
- A replacement in a hanging column takes no room: its text is drawn left
  of where it is, in the line's inset, and a click in the column hits it. A
  replacement at another size is moved to keep the row's baseline; one with
  a chip is drawn on the box (the backgrounds pass) with its icon; one with
  an image draws the image (loaded through the asset cache) instead of its
  text.
- Without a highlighter, decorations are clipped to the visible (unfolded)
  lines' bytes as the highlighter path already clips its styles; before,
  a decoration over a folded or block-hidden line shifted every later run.
- Left uniform: page up/down move by `viewport / line_height` display rows;
  an auto-growing input counts rows; the scroll wheel's line delta, the
  popup anchor's line height, ghost (inline completion) lines and token
  elements' heights stay one line height.

## Compatibility

Additive. Every existing method keeps its signature and behaviour; with no
presentation the uniform paths run unchanged (each changed site keeps its
original expression when nothing is scaled or spaced, and `x * 1.0` or
`x + 0.0` elsewhere is exact). The presentation types and
`EditorPresentationEvent` (a second event type the input states emit, so
existing `InputEvent` matches stay exhaustive) are new exports.

## Tests to add

Next to the existing ones in `src/input/base/element.rs` and the new
`src/input/editor/presentation.rs`:

- a concealed range has zero width, a caret inside it sits at the conceal
  point, a click there lands after it, and a selection over it spans the
  visible text;
- a replacement is as wide as its text, atomic for carets and clicks, and a
  click on a clickable one is reported without moving the caret;
- a scaled line's rows and caret are taller, its text is shaped larger, and
  points below it hit the right line;
- a scaled line wraps into more, narrower rows, and an inset line is shifted
  and wraps narrower;
- space around a line moves the rows below, keeps the text and caret in the
  row, maps points in the space to the line, and scrolling keeps the caret
  in view;
- up/down moves step display rows across a scaled or spaced line and keep x;
- ornaments change no layout;
- a block widget takes its measured height, hides its other lines, is
  stepped over by vertical moves, reports clicks its element leaves alone
  and not those it handles, and is dropped by an edit of its lines;
- the visible range follows mixed row heights when scrolled, and y ↔ row
  mapping round-trips;
- scales of folded lines add no height;
- edits shift, keep or drop the presentation as described, IME composition
  on a presented line lays out, and invalid or stale input never panics;
- with no presentation (or an empty one, or after clearing one) the layout
  is identical to before.

## Diff

Against the published crate (`vendor/gpui-base` in this repository).

```diff
diff --git a/src/input/base/element.rs b/src/input/base/element.rs
index 2e23301e..ccdb4f12 100644
--- a/src/input/base/element.rs
+++ b/src/input/base/element.rs
@@ -17,7 +17,12 @@ use std::{ops::Range, rc::Rc};
 
 use crate::{
     Scrollbar,
-    input::{RopeExt as _, blink_cursor::CURSOR_WIDTH, display_map::LineLayout},
+    input::{
+        RopeExt as _,
+        blink_cursor::CURSOR_WIDTH,
+        display_map::LineLayout,
+        presentation::{EditorPresentationEvent, Ornament, Replacement},
+    },
 };
 
 use super::{
@@ -48,6 +53,8 @@ fn diagnostic_highlight_style(
 
 const BOTTOM_MARGIN_ROWS: usize = 3;
 pub(super) const RIGHT_MARGIN: Pixels = px(10.);
+/// A check box's mark, as the text view draws a ticked task's (`text/node.rs`).
+const CHECK_MARK_SVG: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" fill="none"><path d="m3.25 8.25 3 3 6.5-7" stroke="white" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/></svg>"#;
 pub(super) const LINE_NUMBER_RIGHT_MARGIN: Pixels = px(6.);
 const FOLD_ICON_WIDTH: Pixels = px(14.);
 const FOLD_ICON_HITBOX_WIDTH: Pixels = px(18.);
@@ -558,7 +565,10 @@ impl<M: InputModeKind> TextElement<M> {
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
@@ -572,8 +582,6 @@ impl<M: InputModeKind> TextElement<M> {
             line_origin
         };
 
-        let cursor_height = 0.85 * line_height;
-
         for selection in state.selections.iter() {
             let is_active = selection.id == active_id;
 
@@ -608,6 +616,10 @@ impl<M: InputModeKind> TextElement<M> {
 
             let affinity = is_active && state.cursor_line_end_affinity;
             let cursor_pos = caret_for(cursor_row, cursor, affinity);
+            // A scaled line's rows are taller, and so is its caret.
+            let row_height = line_height * state.display_map.row_scale(cursor_row);
+            let cursor_height = 0.85 * row_height;
+            let below_row = row_height - line_height;
             let cursor_start = caret_for(sel_start_row, selected_range.start, false);
             let cursor_end = caret_for(sel_end_row, selected_range.end, false);
 
@@ -644,11 +656,11 @@ impl<M: InputModeKind> TextElement<M> {
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
@@ -689,7 +701,7 @@ impl<M: InputModeKind> TextElement<M> {
                 bounds: Bounds::new(
                     point(
                         cursor_x,
-                        bounds.top() + cursor_pos.y + ((line_height - cursor_height) / 2.),
+                        bounds.top() + cursor_pos.y + ((row_height - cursor_height) / 2.),
                     ),
                     size(CURSOR_WIDTH, cursor_height),
                 ),
@@ -765,6 +777,8 @@ impl<M: InputModeKind> TextElement<M> {
         {
             let mut offset = line_offset;
             let alignment = last_layout.alignment_offset(line.longest_width);
+            let row_height = line.row_height(last_layout.line_height);
+            y += line.frame().above;
             for (row, shaped) in line.wrapped_lines.iter().enumerate() {
                 let end = offset + shaped.len;
                 let start_ix = range.start.max(offset);
@@ -774,7 +788,7 @@ impl<M: InputModeKind> TextElement<M> {
                 // A wrap boundary is not a newline and must not get such a cell.
                 let newline = is_last && range.start <= end && range.end > end;
                 if start_ix < end_ix || newline {
-                    let indent = if row == 0 { px(0.) } else { line.wrap_indent };
+                    let indent = line.line_indent(row);
                     let left = alignment + indent + shaped.x_for_index(start_ix.min(end) - offset);
                     let right = alignment
                         + indent
@@ -786,13 +800,14 @@ impl<M: InputModeKind> TextElement<M> {
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
+            y += line.frame().below;
         }
         (!corners.is_empty()).then_some(corners)
     }
@@ -1044,22 +1059,40 @@ impl<M: InputModeKind> TextElement<M> {
         } else {
             state.scroll_handle.offset().y
         };
+        let content_height = if !state.display_map.has_uneven_rows() {
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
+        let (first_display, last_display) = if !state.display_map.has_uneven_rows() {
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
@@ -1069,8 +1102,10 @@ impl<M: InputModeKind> TextElement<M> {
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
@@ -1288,6 +1323,7 @@ impl<M: InputModeKind> TextElement<M> {
             is_folded: bool,
             display_row: usize,
             offset_y: Pixels,
+            row_height: Pixels,
         }
 
         let line_number_hitbox = window.insert_hitbox(
@@ -1323,26 +1359,27 @@ impl<M: InputModeKind> TextElement<M> {
                         buffer_line,
                         is_folded,
                         display_row: buffer_line,
-                        offset_y,
+                        offset_y: offset_y + line.frame().above,
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
@@ -1838,6 +1875,8 @@ impl<M: InputModeKind> TextElement<M> {
                         .wrapped_lines
                         .clone()
                 };
+                let scale = state.display_map.line_scale(row);
+                let font_size = font_size * scale;
                 let mut lines: SmallVec<[InputLine; 1]> = SmallVec::new();
                 for range in ranges {
                     let mut fragments = Vec::new();
@@ -1864,6 +1903,12 @@ impl<M: InputModeKind> TextElement<M> {
                                 x,
                                 width,
                                 text: Some(shaped),
+                                replaced: false,
+                                text_x: px(0.),
+                                hang: px(0.),
+                                text_y: px(0.),
+                                chip: None,
+                                image: None,
                             });
                             x += width;
                         }
@@ -1873,6 +1918,12 @@ impl<M: InputModeKind> TextElement<M> {
                             x,
                             width,
                             text: None,
+                            replaced: false,
+                            text_x: px(0.),
+                            hang: px(0.),
+                            text_y: px(0.),
+                            chip: None,
+                            image: None,
                         });
                         x += width;
                         offset = local.end;
@@ -1891,6 +1942,12 @@ impl<M: InputModeKind> TextElement<M> {
                             x,
                             width,
                             text: Some(shaped),
+                            replaced: false,
+                            text_x: px(0.),
+                            hang: px(0.),
+                            text_y: px(0.),
+                            chip: None,
+                            image: None,
                         });
                     }
                     lines.push(InputLine::inline(text[range].to_owned().into(), fragments));
@@ -1909,6 +1966,8 @@ impl<M: InputModeKind> TextElement<M> {
                 LineLayout::new()
                     .inline_lines(lines)
                     .wrap_indent(wrap_indent)
+                    .with_scale(state.display_map.row_scale(row))
+                    .with_frame(state.display_map.line_frame(row))
             })
             .collect()
     }
@@ -1994,6 +2053,85 @@ impl<M: InputModeKind> TextElement<M> {
         out
     }
 
+    /// Lay out the elements of the visible block widgets at the rows of their first lines,
+    /// the width of the text area, and record their measured heights for the next frame.
+    fn prepaint_blocks(
+        &self,
+        layout: &LastLayout,
+        bounds: Bounds<Pixels>,
+        window: &mut Window,
+        cx: &mut App,
+    ) -> Vec<AnyElement> {
+        use gpui::{InteractiveElement as _, StatefulInteractiveElement as _};
+
+        let placements: Vec<(crate::input::BlockWidget, Pixels)> = {
+            let state = self.state.read(cx);
+            if state.display_map.presentation().blocks.is_empty() {
+                return vec![];
+            }
+            let mut y = layout.visible_top;
+            let mut placements = Vec::new();
+            for (line, &row) in layout.lines.iter().zip(layout.visible_buffer_lines.iter()) {
+                if let Some(block) = state.display_map.block_at(row) {
+                    placements.push((block.clone(), y));
+                }
+                y += line.size(layout.line_height).height;
+            }
+            placements
+        };
+        let width = layout
+            .wrap_width
+            .unwrap_or(layout.content_width - layout.line_number_width)
+            .max(px(1.));
+        let mut elements = Vec::with_capacity(placements.len());
+        let mut measured = Vec::with_capacity(placements.len());
+        for (block, y) in placements {
+            let lines = block.lines.clone();
+            let state = self.state.clone();
+            let mut element = gpui::div()
+                .id(ElementId::from(("presentation-block", block.key as usize)))
+                .w(width)
+                .cursor_default()
+                // A press on the widget that its element leaves alone does not place the
+                // caret; a click is reported instead.
+                .on_mouse_down(MouseButton::Left, |_, window, _| window.prevent_default())
+                .on_click(move |_, _, cx| {
+                    let lines = lines.clone();
+                    state.update(cx, |_, cx| {
+                        cx.emit(EditorPresentationEvent::BlockClicked { lines })
+                    });
+                })
+                .child((block.render)(window, cx))
+                .into_any_element();
+            let element_size = element.layout_as_root(
+                size(
+                    gpui::AvailableSpace::Definite(width),
+                    gpui::AvailableSpace::MinContent,
+                ),
+                window,
+                cx,
+            );
+            element.prepaint_at(
+                bounds.origin + point(layout.line_number_width, y),
+                window,
+                cx,
+            );
+            measured.push((block.key, element_size.height));
+            elements.push(element);
+        }
+        self.state.update(cx, |state, cx| {
+            let mut changed = false;
+            for (key, height) in measured {
+                changed |= state.display_map.set_block_height(key, height);
+            }
+            // The rows below follow the measured height from the next frame.
+            if changed {
+                cx.notify();
+            }
+        });
+        elements
+    }
+
     #[allow(clippy::too_many_arguments)]
     fn layout_lines(
         state: &InputBaseState<M>,
@@ -2043,14 +2181,26 @@ impl<M: InputModeKind> TextElement<M> {
             }
 
             // Keep placeholder lines in a single layout to stay parallel with visible_* metadata.
+            // A scaled first line keeps its row height (not its text size), so the caret
+            // and the rows below agree with the display map.
             let line_layout = LineLayout::new()
                 .lines(placeholder_lines)
                 .with_background(line_has_background)
+                .with_scale(state.display_map.row_scale(0))
+                .with_frame(state.display_map.line_frame(0))
                 .with_whitespaces(whitespace_indicators);
             return vec![line_layout];
         }
 
         let mut lines = Vec::with_capacity(last_layout.visible_buffer_lines.len());
+        // Replacement text is drawn in the editor's text style, refined by its own.
+        let replacement_style = {
+            let mut style = window.text_style();
+            if state.disabled {
+                style.color = style.color.opacity(0.5);
+            }
+            style
+        };
         // run_offset tracks position in the runs vec coordinate space (only visible line bytes).
         // This is separate from the visible_text offset because runs from highlight_lines
         // only cover visible (non-folded) lines.
@@ -2065,6 +2215,71 @@ impl<M: InputModeKind> TextElement<M> {
 
             debug_assert_eq!(line_item.len(), line_text.len());
 
+            // A presentation scales the line's text, conceals ranges of it and draws others
+            // as replacement text. Concealed text is laid out at zero width through inline
+            // fragments, like a token's, and a replacement as a fragment of its own text.
+            let scale = state.display_map.line_scale(buffer_line);
+            let frame = state.display_map.line_frame(buffer_line);
+            let hidden = if state.masked {
+                Vec::new()
+            } else {
+                let line_start = state.text.line_start_offset(buffer_line);
+                let mut hidden: Vec<(Range<usize>, Option<&Replacement>)> =
+                    concealed_in_line(state.display_map.concealed(), line_start, &line_text)
+                        .into_iter()
+                        .map(|range| (range, None))
+                        .collect();
+                let replacements = &state.display_map.presentation().replacements;
+                if !replacements.is_empty() {
+                    let ranges: Vec<Range<usize>> =
+                        replacements.iter().map(|r| r.range.clone()).collect();
+                    let first = replacements.partition_point(|r| r.range.end <= line_start);
+                    let local = concealed_in_line(&ranges[first..], line_start, &line_text);
+                    let mut found = replacements[first..].iter();
+                    for range in local {
+                        // `concealed_in_line` keeps them in order, so each is found ahead.
+                        if let Some(replacement) = found
+                            .by_ref()
+                            .find(|r| r.range.start == range.start + line_start)
+                        {
+                            hidden.push((range, Some(replacement)));
+                        }
+                    }
+                    hidden.sort_by_key(|(range, _)| range.start);
+                    let mut end = 0;
+                    hidden.retain(|(range, _)| {
+                        let keep = range.start >= end;
+                        if keep {
+                            end = range.end;
+                        }
+                        keep
+                    });
+                }
+                hidden
+            };
+            let row_scale = state.display_map.row_scale(buffer_line);
+            if scale != 1. || row_scale != scale || !hidden.is_empty() {
+                lines.push(
+                    Self::layout_presented_line(
+                        &line_text,
+                        &line_item.wrapped_lines,
+                        line_item.indent,
+                        &hidden,
+                        row_scale,
+                        font_size * scale,
+                        &runs_for_range(runs, run_offset, &(0..line_text.len())),
+                        last_layout.visible_line_byte_offsets[vi],
+                        bg_segments,
+                        whitespace_indicators.clone(),
+                        &replacement_style,
+                        window,
+                    )
+                    .with_frame(frame),
+                );
+                run_offset += line_text.len() + 1;
+                continue;
+            }
+
             let mut wrapped_lines: SmallVec<[ShapedLine; 1]> = SmallVec::with_capacity(1);
             let mut line_has_background = false;
 
@@ -2107,6 +2322,7 @@ impl<M: InputModeKind> TextElement<M> {
                 .lines(wrapped_lines)
                 .wrap_indent(wrap_indent)
                 .with_background(line_has_background)
+                .with_frame(frame)
                 .with_whitespaces(whitespace_indicators.clone());
             lines.push(line_layout);
 
@@ -2117,6 +2333,243 @@ impl<M: InputModeKind> TextElement<M> {
         lines
     }
 
+    /// Lay out one buffer line under a presentation: shaped at `font_size` (the scaled
+    /// size), with its rows `scale` line heights tall and the ranges in `hidden` (line-local,
+    /// sorted, disjoint, on char boundaries) laid out at zero width and not painted, or, for a
+    /// replacement, as its text shaped in `replacement_style` refined by its own style.
+    ///
+    /// `runs` cover exactly this line's bytes.
+    #[allow(clippy::too_many_arguments)]
+    fn layout_presented_line(
+        line_text: &str,
+        wrapped_ranges: &[Range<usize>],
+        indent: u32,
+        hidden: &[(Range<usize>, Option<&Replacement>)],
+        scale: f32,
+        font_size: Pixels,
+        runs: &[TextRun],
+        line_offset: usize,
+        bg_segments: &[(Range<usize>, Hsla)],
+        whitespace_indicators: Option<WhitespaceIndicators>,
+        replacement_style: &TextStyle,
+        window: &mut Window,
+    ) -> LineLayout {
+        use crate::input::display_map::{FragmentChip, InlineFragment, InputLine};
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
+                    replaced: false,
+                    text_x: px(0.),
+                    hang: px(0.),
+                    text_y: px(0.),
+                    chip: None,
+                    image: None,
+                }
+            };
+
+            let first = hidden.partition_point(|(c, _)| c.end <= range.start);
+            let in_row = hidden[first..]
+                .iter()
+                .take_while(|(c, _)| c.start < range.end);
+            let mut fragments = Vec::new();
+            let mut offset = 0;
+            let mut x = px(0.);
+            for (c, replacement) in in_row {
+                // A replacement split by a wrap (which the wrapper never makes) is concealed.
+                let whole = c.start >= range.start && c.end <= range.end;
+                let c = c.start.max(range.start) - range.start..c.end.min(range.end) - range.start;
+                if c.start < offset || c.is_empty() {
+                    continue;
+                }
+                if offset < c.start {
+                    let fragment = shape(offset..c.start, x, window);
+                    x += fragment.width;
+                    fragments.push(fragment);
+                }
+                match replacement.filter(|_| whole) {
+                    Some(replacement) => {
+                        let text = replacement.text.clone();
+                        let mut text_style = replacement_style.clone();
+                        if let Some(family) = &replacement.family {
+                            text_style.font_family = family.clone();
+                        }
+                        let run = text_style
+                            .clone()
+                            .highlight(replacement.style)
+                            .to_run(text.len());
+                        let shaped = window.text_system().shape_line(
+                            text,
+                            replacement.font_size(font_size),
+                            &[run],
+                            None,
+                        );
+                        let (width, text_x, hang) = replacement.placement(shaped.width);
+                        // Another size keeps the row's baseline: the text moves by half
+                        // the difference of what is above and below it.
+                        let base = window.text_system().resolve_font(&replacement_style.font());
+                        let (ascent, descent) = (
+                            window.text_system().ascent(base, font_size),
+                            // The text system gives the descent below the baseline as
+                            // a negative number; the shaped line, as a positive one.
+                            window.text_system().descent(base, font_size).abs(),
+                        );
+                        let centred = matches!(
+                            replacement.chip,
+                            Some(crate::input::ReplacementChip {
+                                height: crate::input::ChipHeight::Fixed(_),
+                                ..
+                            })
+                        );
+                        let placed = replacement
+                            .chip
+                            .as_ref()
+                            .is_some_and(|chip| chip.text_baseline.is_some());
+                        let text_y =
+                            if placed || centred || replacement.font_size(font_size) == font_size {
+                                px(0.)
+                            } else {
+                                (ascent - descent - shaped.ascent + shaped.descent) / 2.
+                            };
+                        let chip = replacement.chip.as_ref().map(|chip| {
+                            let font = window.text_system().resolve_font(&text_style.font());
+                            let size = replacement.font_size(font_size);
+                            let color = replacement.style.color.unwrap_or(text_style.color);
+                            FragmentChip {
+                                icon: chip.icon.as_ref().map(|icon| {
+                                    (icon.path.clone(), icon.size, color, chip.padding)
+                                }),
+                                dashed_bottom: chip.dashed_bottom,
+                                whole_width: chip.whole_width,
+                                text_baseline: chip.text_baseline,
+                                background: chip.background,
+                                radius: chip.radius,
+                                height: chip.height,
+                                cap_height: window.text_system().cap_height(font, size),
+                                font_size: size,
+                            }
+                        });
+                        fragments.push(InlineFragment {
+                            range: c.clone(),
+                            x,
+                            width,
+                            text: Some(shaped),
+                            replaced: true,
+                            text_x,
+                            hang,
+                            text_y,
+                            chip,
+                            image: replacement
+                                .image
+                                .clone()
+                                .map(|image| (image, ascent, descent)),
+                        });
+                        x += width;
+                    }
+                    None => fragments.push(InlineFragment {
+                        range: c.clone(),
+                        x,
+                        width: px(0.),
+                        text: None,
+                        replaced: false,
+                        text_x: px(0.),
+                        hang: px(0.),
+                        text_y: px(0.),
+                        chip: None,
+                        image: None,
+                    }),
+                }
+                offset = c.end;
+            }
+            if offset < row_text.len() || fragments.is_empty() {
+                fragments.push(shape(offset..row_text.len(), x, window));
+            }
+            // A replacement's box is drawn with the backgrounds.
+            line_has_background |= fragments.iter().any(|f| f.chip.is_some());
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
+    /// The decorations alone, without a highlighter, clipped to the visible lines' bytes:
+    /// the runs cover only lines that are not folded (or hidden by a block widget), so a
+    /// style over a hidden line would shift every later run.
+    fn decorations_only(
+        state: &InputBaseState<M>,
+        text: &Rope,
+        visible_buffer_lines: &[usize],
+        visible_byte_range: Range<usize>,
+    ) -> Option<Vec<(Range<usize>, HighlightStyle)>> {
+        if state.masked {
+            return None;
+        }
+        let styles = compose_decoration_collections(
+            Vec::new(),
+            state.extras.decoration_layers().into_iter(),
+            visible_byte_range,
+        )?;
+        let is_multi_line = state.is_multi_line();
+        let mut groups: Vec<Range<usize>> = Vec::new();
+        let mut lines = visible_buffer_lines.iter().peekable();
+        while let Some(&first) = lines.next() {
+            let mut last = first;
+            while lines.peek().is_some_and(|&&next| next == last + 1) {
+                last = *lines.next().expect("peeked");
+            }
+            let end = if is_multi_line {
+                text.line_start_offset(last + 1)
+            } else {
+                text.line_end_offset(last)
+            };
+            groups.push(text.line_start_offset(first)..end);
+        }
+        Some(clip_styles_to_ranges(styles, &groups))
+    }
+
     /// First usize is the offset of skipped.
     fn highlight_lines(
         &mut self,
@@ -2136,27 +2589,16 @@ impl<M: InputModeKind> TextElement<M> {
                 ..
             } => (highlighter.borrow_mut(), diagnostics),
             _ => {
-                return (!state.masked)
-                    .then(|| {
-                        compose_decoration_collections(
-                            Vec::new(),
-                            state.extras.decoration_layers().into_iter(),
-                            visible_byte_range,
-                        )
-                    })
-                    .flatten();
+                return Self::decorations_only(
+                    state,
+                    text,
+                    visible_buffer_lines,
+                    visible_byte_range,
+                );
             }
         };
         let Some(highlighter) = highlighter.as_mut() else {
-            return (!state.masked)
-                .then(|| {
-                    compose_decoration_collections(
-                        Vec::new(),
-                        state.extras.decoration_layers().into_iter(),
-                        visible_byte_range,
-                    )
-                })
-                .flatten();
+            return Self::decorations_only(state, text, visible_buffer_lines, visible_byte_range);
         };
 
         let mut styles = Vec::with_capacity(visible_buffer_lines.len());
@@ -2281,6 +2723,8 @@ pub(super) struct PrepaintState {
     /// The lines of entire lines.
     last_layout: LastLayout,
     token_elements: Vec<AnyElement>,
+    /// The elements of the presentation's visible block widgets, laid out in place.
+    block_elements: Vec<AnyElement>,
     /// The lines only contains the visible lines in the viewport, based on `visible_range`.
     ///
     /// The child is the soft lines.
@@ -2487,7 +2931,7 @@ impl<M: InputModeKind> Element for TextElement<M> {
         cx: &mut App,
     ) -> (LayoutId, Self::RequestLayoutState) {
         let state = self.state.read(cx);
-        let line_height = window.line_height();
+        let line_height = state.presented_line_height(window);
 
         let mut style = Style::default();
         style.size.width = relative(1.).into();
@@ -2584,8 +3028,13 @@ impl<M: InputModeKind> Element for TextElement<M> {
                 state.display_map.set_wrapping_indent(wrapping_indent, cx);
             });
         }
+        // A presentation's leaves round to the window's device pixels.
+        let scale_factor = window.scale_factor();
+        self.state.update(cx, |state, _| {
+            state.display_map.set_device_scale(scale_factor);
+        });
 
-        let line_height = window.line_height();
+        let line_height = self.state.read(cx).presented_line_height(window);
         let token_elements = self.measure_tokens(
             (bounds.size.width - line_number_width - RIGHT_MARGIN).max(px(1.)),
             line_height,
@@ -2777,6 +3226,14 @@ impl<M: InputModeKind> Element for TextElement<M> {
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
@@ -2789,7 +3246,6 @@ impl<M: InputModeKind> Element for TextElement<M> {
         let ghost_line_count = ghost_lines.len();
         let ghost_lines_height = ghost_line_count as f32 * line_height;
 
-        let total_wrapped_lines = state.display_map.wrap_row_count();
         let empty_bottom_height = empty_bottom_height(
             state.is_code_editor(),
             state.scroll_beyond_last_line,
@@ -2806,7 +3262,7 @@ impl<M: InputModeKind> Element for TextElement<M> {
             } else {
                 longest_line_width
             },
-            (total_wrapped_lines as f32 * line_height
+            (state.display_map.wrap_rows_height(line_height)
                 + empty_bottom_height.max(ghost_lines_height))
             .max(bounds.size.height),
         );
@@ -2975,8 +3431,10 @@ impl<M: InputModeKind> Element for TextElement<M> {
         let hitbox = window.insert_hitbox(input_bounds, HitboxBehavior::Normal);
 
         let token_elements = self.prepaint_tokens(&last_layout, bounds, token_elements, window, cx);
+        let block_elements = self.prepaint_blocks(&last_layout, bounds, window, cx);
         PrepaintState {
             token_elements,
+            block_elements,
             hitbox,
             bounds,
             last_layout,
@@ -3032,7 +3490,7 @@ impl<M: InputModeKind> Element for TextElement<M> {
         );
 
         // Paint multi line text
-        let line_height = window.line_height();
+        let line_height = self.state.read(cx).presented_line_height(window);
         let origin = bounds.origin;
 
         let invisible_top_padding = prepaint.last_layout.visible_top;
@@ -3051,13 +3509,16 @@ impl<M: InputModeKind> Element for TextElement<M> {
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
@@ -3088,6 +3549,16 @@ impl<M: InputModeKind> Element for TextElement<M> {
         // backgrounds need their own pass. It runs ahead of the indent guides, selections,
         // and text so a highlighted range sits under them instead of covering them.
         let mut offset_y = invisible_top_padding;
+        // A presentation's ornaments are drawn with their lines, beneath everything else.
+        let ornaments = {
+            let state = self.state.read(cx);
+            let presentation = state.display_map.presentation();
+            (!presentation.line_ornaments.is_empty()).then(|| presentation.line_ornaments.clone())
+        };
+        let text_width = prepaint.last_layout.wrap_width.unwrap_or(
+            (prepaint.last_layout.content_width - prepaint.last_layout.line_number_width)
+                .max(px(0.)),
+        );
         for (line, &buffer_line) in prepaint
             .last_layout
             .lines
@@ -3099,6 +3570,81 @@ impl<M: InputModeKind> Element for TextElement<M> {
                 origin.y + offset_y,
             );
 
+            if let Some(ornaments) = &ornaments {
+                let first = ornaments.partition_point(|(l, _)| *l < buffer_line);
+                let height = line.size(line_height).height;
+                let frame = line.frame();
+                for (_, ornament) in ornaments[first..]
+                    .iter()
+                    .take_while(|(l, _)| *l == buffer_line)
+                {
+                    let quad = match *ornament {
+                        Ornament::Check {
+                            x,
+                            size: side,
+                            radius,
+                            border,
+                            border_color,
+                            fill: background,
+                            check,
+                            row,
+                        } => {
+                            // Centred in its row's box, or in the box given, on whole device
+                            // pixels as a laid-out box is.
+                            let row = row.unwrap_or_else(|| line.row_height(line_height));
+                            let origin = p + point(x, frame.above + (row - side).half());
+                            let scale = window.scale_factor();
+                            // Within a rounding error of a half is a half.
+                            let snap = |v: Pixels| {
+                                let device = f32::from(v) * scale;
+                                px((device.abs() - 0.5 - 1e-3).ceil().copysign(device) / scale)
+                            };
+                            let origin = point(snap(origin.x), snap(origin.y));
+                            let bounds = Bounds::new(origin, size(side, side));
+                            window.paint_quad(gpui::quad(
+                                bounds,
+                                radius,
+                                background.unwrap_or(gpui::transparent_black()),
+                                border.max(px(0.)),
+                                border_color,
+                                gpui::BorderStyle::default(),
+                            ));
+                            if let Some(color) = check {
+                                // The text view's check mark, in the middle `0.8` of the box.
+                                let mark = Bounds::new(
+                                    origin + point(side * 0.1, side * 0.1),
+                                    size(side * 0.8, side * 0.8),
+                                );
+                                window
+                                    .paint_svg(
+                                        mark,
+                                        "gpui-base-presentation-check.svg".into(),
+                                        Some(CHECK_MARK_SVG),
+                                        gpui::TransformationMatrix::unit(),
+                                        color,
+                                        cx,
+                                    )
+                                    .ok();
+                            }
+                            continue;
+                        }
+                        Ornament::Bar { x, width, color } => fill(
+                            Bounds::new(p + point(x, px(0.)), size(width, height)),
+                            color,
+                        ),
+                        Ornament::Rule { thickness, color } => {
+                            let rows = height - frame.above - frame.below;
+                            let y = frame.above + (rows - thickness).half();
+                            fill(
+                                Bounds::new(p + point(px(0.), y), size(text_width, thickness)),
+                                color,
+                            )
+                        }
+                    };
+                    window.paint_quad(quad);
+                }
+            }
+
             line.paint_background(
                 p,
                 line_height,
@@ -3229,6 +3775,9 @@ impl<M: InputModeKind> Element for TextElement<M> {
         for element in &mut prepaint.token_elements {
             element.paint(window, cx);
         }
+        for element in &mut prepaint.block_elements {
+            element.paint(window, cx);
+        }
 
         // Paint blinking cursors (shared blink state for all carets)
         if focused && show_cursor {
@@ -3258,12 +3807,17 @@ impl<M: InputModeKind> Element for TextElement<M> {
             window.paint_quad(fill(gutter_bounds, gutter_bg));
 
             // Each item is the normal lines.
-            for (lines, &buffer_line) in line_numbers
+            for ((lines, &buffer_line), layout) in line_numbers
                 .iter()
                 .zip(prepaint.last_layout.visible_buffer_lines.iter())
+                .zip(prepaint.last_layout.lines.iter())
             {
+                // A line's number sits beside its rows, past the space above them.
+                offset_y += layout.frame().above;
                 let p = point(input_bounds.origin.x, origin.y + offset_y);
                 let is_active = prepaint.current_row == Some(buffer_line);
+                // A scaled line's number sits centred in its taller rows.
+                let line_height = layout.row_height(line_height);
 
                 let height = line_height * lines.len() as f32;
                 // paint active line number background
@@ -3283,6 +3837,7 @@ impl<M: InputModeKind> Element for TextElement<M> {
                     _ = line.paint(p, line_height, TextAlign::Left, None, window, cx);
                     offset_y += line_height;
                 }
+                offset_y += layout.frame().below;
 
                 // Add ghost line height after cursor row for line numbers alignment
                 if !prepaint.ghost_lines.is_empty() && prepaint.current_row == Some(buffer_line) {
@@ -3355,6 +3910,35 @@ impl<M: InputModeKind> Element for TextElement<M> {
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
@@ -3592,7 +4176,10 @@ fn split_runs_by_bg_segments(
 #[cfg(test)]
 mod tests {
     use super::*;
-    use crate::input::{EditorMode, EditorState, FoldRange, RangeDecoration, Redo, Undo};
+    use crate::input::{
+        BlockWidget, EditorMode, EditorPresentation, EditorPresentationEvent, EditorState,
+        FoldRange, Ornament, RangeDecoration, Redo, Replacement, Undo,
+    };
     use gpui::{
         AppContext as _, Context, EntityInputHandler as _, Render, TestAppContext,
         VisualTestContext, div,
@@ -4094,6 +4681,1167 @@ mod tests {
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
+                    ..Default::default()
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
+                        ..Default::default()
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
+                vec![(1, 1.6), (7, 2.), (0, -1.), (2, f32::NAN), (2, 0.), (2, 1.)],
+                cx,
+            );
+            assert_eq!(
+                editor.read(cx).editor_presentation(),
+                EditorPresentation {
+                    concealed: vec![0..2, 3..5],
+                    line_scales: vec![(1, 1.6)],
+                    ..Default::default()
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
+                    ..Default::default()
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
+                    ..Default::default()
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
+    fn present_with(editor: &Entity<EditorState>, presentation: EditorPresentation, cx: &mut App) {
+        editor.update(cx, |state, cx| {
+            state.set_presentation(Some(presentation), cx)
+        });
+    }
+
+    fn replacement(range: Range<usize>, text: &str, clickable: bool) -> Replacement {
+        Replacement {
+            clickable,
+            ..Replacement::new(range, SharedString::from(text.to_string()))
+        }
+    }
+
+    /// Record the presentation events `editor` emits.
+    fn record_events(
+        editor: &Entity<EditorState>,
+        cx: &mut VisualTestContext,
+    ) -> (
+        Rc<std::cell::RefCell<Vec<EditorPresentationEvent>>>,
+        gpui::Subscription,
+    ) {
+        let events = Rc::new(std::cell::RefCell::new(Vec::new()));
+        let subscription = cx.update(|_, cx| {
+            let events = events.clone();
+            cx.subscribe(editor, move |_, event: &EditorPresentationEvent, _| {
+                events.borrow_mut().push(event.clone())
+            })
+        });
+        (events, subscription)
+    }
+
+    #[gpui::test]
+    fn presentation_replacement_takes_its_text_width_and_is_atomic(cx: &mut TestAppContext) {
+        let text = "- item\n••••";
+        let (editor, window) = decoration_editor(cx, text, false);
+        let mut cx = VisualTestContext::from_window(window.into(), cx);
+        cx.update(|window, cx| {
+            present_with(
+                &editor,
+                EditorPresentation {
+                    replacements: vec![replacement(0..2, "••••", false)],
+                    ..Default::default()
+                },
+                cx,
+            );
+            editor.update(cx, |state, cx| state.set_selected_range(2..2, cx));
+            window.draw(cx).clear(cx);
+            let state = editor.read(cx);
+            let layout = state.last_layout.as_ref().unwrap();
+            let row = &layout.lines[0].wrapped_lines[0];
+            // The replacement is as wide as its text, shaped like the same text in the line
+            // below, and offsets still address the source.
+            let bullets = layout.lines[1].wrapped_lines[0].width;
+            assert_eq!(row.len, 6);
+            assert_close(row.x_for_index(2), bullets);
+            // Atomic: a caret inside it sits at its start, one after it at its end.
+            assert_eq!(row.x_for_index(0), px(0.));
+            assert_eq!(row.x_for_index(1), px(0.));
+            assert_eq!(row.closest_index_for_x(px(1.)), 0);
+            assert_eq!(row.closest_index_for_x(bullets - px(1.)), 2);
+            assert!(row.closest_index_for_x(bullets + px(40.)) > 2);
+            let caret = layout.cursor_bounds.unwrap();
+            assert_close(
+                caret.origin.x,
+                state.input_bounds.left() + layout.line_number_width + bullets,
+            );
+            // A selection over it covers its text.
+            let corners = TextElement::<EditorMode>::layout_range_corners(&(0..2), layout).unwrap();
+            assert_close(corners[0].top_right.x - corners[0].top_left.x, bullets);
+        });
+    }
+
+    #[gpui::test]
+    fn presentation_line_heights_and_line_height_set_row_heights(cx: &mut TestAppContext) {
+        let text = "# Title\nbody\nmore";
+        let (editor, window) = decoration_editor(cx, text, false);
+        let mut cx = VisualTestContext::from_window(window.into(), cx);
+        cx.update(|window, cx| {
+            present_with(
+                &editor,
+                EditorPresentation {
+                    line_scales: vec![(0, 2.)],
+                    line_heights: vec![(0, 0.75), (2, 0.5)],
+                    line_height: Some(px(20.5)),
+                    ..Default::default()
+                },
+                cx,
+            );
+            window.draw(cx).clear(cx);
+            let state = editor.read(cx);
+            let layout = state.last_layout.as_ref().unwrap();
+            // The rows are as tall as asked, not rounded to whole pixels.
+            let lh = layout.line_height;
+            assert_close(lh, px(20.5));
+            let map = &state.display_map;
+            assert_close(map.display_row_height(0, lh), px(20.5 * 2. * 0.75));
+            assert_close(map.display_row_height(1, lh), px(20.5));
+            assert_close(map.display_row_height(2, lh), px(20.5 * 0.5));
+            assert_close(layout.lines[0].size(lh).height, px(20.5 * 1.5));
+            assert_close(map.wrap_rows_height(lh), px(20.5 * (1.5 + 1. + 0.5)));
+        });
+    }
+
+    #[gpui::test]
+    fn presentation_leaves_round_up_to_device_pixels(cx: &mut TestAppContext) {
+        let text = "one\ntwo\nthree";
+        let (editor, window) = decoration_editor(cx, text, false);
+        let mut cx = VisualTestContext::from_window(window.into(), cx);
+        cx.update(|window, cx| {
+            present_with(
+                &editor,
+                EditorPresentation {
+                    line_height: Some(px(20.3)),
+                    leaves: vec![(0..2, px(0.25)), (2..3, px(0.))],
+                    ..Default::default()
+                },
+                cx,
+            );
+            window.draw(cx).clear(cx);
+            window.draw(cx).clear(cx);
+            let scale = window.scale_factor();
+            let state = editor.read(cx);
+            let lh = state.last_layout.as_ref().unwrap().line_height;
+            let map = &state.display_map;
+            let ceil = |v: f32| (v * scale).ceil() / scale;
+            // Two rows and the extra, rounded up below the second.
+            assert_close(map.display_row_height(0, lh), px(20.3));
+            assert_close(map.display_row_height(1, lh), px(ceil(40.85) - 20.55));
+            assert_close(map.display_row_height(2, lh), px(ceil(20.3)));
+        });
+    }
+
+    #[gpui::test]
+    fn presentation_hanging_column_takes_no_room_and_takes_clicks(cx: &mut TestAppContext) {
+        let text = "- item";
+        let (editor, window) = decoration_editor(cx, text, false);
+        let mut cx = VisualTestContext::from_window(window.into(), cx);
+        cx.update(|window, cx| {
+            present_with(
+                &editor,
+                EditorPresentation {
+                    replacements: vec![Replacement {
+                        clickable: true,
+                        column: Some(crate::input::ReplacementColumn {
+                            width: px(20.),
+                            gap: px(6.),
+                            hanging: true,
+                        }),
+                        ..Replacement::new(0..2, "•")
+                    }],
+                    line_insets: vec![(0, px(20.))],
+                    ..Default::default()
+                },
+                cx,
+            );
+            window.draw(cx).clear(cx);
+            let state = editor.read(cx);
+            let layout = state.last_layout.as_ref().unwrap();
+            let row = &layout.lines[0].wrapped_lines[0];
+            // The text starts at the inset; the marker hangs in it.
+            assert_eq!(row.x_for_index(2), px(0.));
+            assert_eq!(row.replacement_at_x(px(-10.)), Some(0..2));
+            assert_eq!(row.replacement_at_x(px(-21.)), None);
+            assert_eq!(row.replacement_at_x(px(1.)), None);
+        });
+    }
+
+    #[test]
+    fn decorations_without_a_highlighter_keep_to_the_visible_lines() {
+        let text = Rope::from("hidden\nalso\nbold here");
+        let styles = vec![(0..text.len(), HighlightStyle::default())];
+        // Lines 0 and 1 are hidden (folded, or under a block widget): the
+        // styles keep only line 2's bytes, so the runs line up with it.
+        let groups = [12..text.len()];
+        let clipped = clip_styles_to_ranges(styles, &groups);
+        assert_eq!(clipped, vec![(12..text.len(), HighlightStyle::default())]);
+    }
+
+    #[gpui::test]
+    fn presentation_click_on_a_clickable_replacement_is_reported(cx: &mut TestAppContext) {
+        let text = "- [ ] task\nnext";
+        let (editor, window) = decoration_editor(cx, text, false);
+        let mut cx = VisualTestContext::from_window(window.into(), cx);
+        let (events, _subscription) = record_events(&editor, &mut cx);
+        let set = |clickable: bool, cx: &mut VisualTestContext| {
+            cx.update(|window, cx| {
+                present_with(
+                    &editor,
+                    EditorPresentation {
+                        replacements: vec![replacement(0..6, "☐ ", clickable)],
+                        ..Default::default()
+                    },
+                    cx,
+                );
+                editor.update(cx, |state, cx| state.set_selected_range(13..13, cx));
+                window.draw(cx).clear(cx);
+            });
+            editor.read_with(cx, |state, _| state.range_to_bounds(&(0..6)).unwrap())
+        };
+
+        // A plain replacement places the caret at one of its ends.
+        let bounds = set(false, &mut cx);
+        cx.simulate_click(bounds.center(), gpui::Modifiers::default());
+        cx.run_until_parked();
+        let cursor = editor.read_with(&cx, |state, _| state.cursor());
+        assert!(cursor == 0 || cursor == 6, "{cursor}");
+        assert!(events.borrow().is_empty());
+
+        // A clickable one is reported, and the caret stays.
+        let bounds = set(true, &mut cx);
+        cx.simulate_click(bounds.center(), gpui::Modifiers::default());
+        cx.run_until_parked();
+        assert_eq!(editor.read_with(&cx, |state, _| state.cursor()), 13);
+        assert_eq!(
+            *events.borrow(),
+            vec![EditorPresentationEvent::ReplacementClicked { range: 0..6 }]
+        );
+
+        // Past it, a click places the caret as usual.
+        let past = bounds.center() + point(bounds.size.width, px(0.)) + point(px(20.), px(0.));
+        cx.simulate_click(past, gpui::Modifiers::default());
+        cx.run_until_parked();
+        assert!(editor.read_with(&cx, |state, _| state.cursor()) > 6);
+        assert_eq!(events.borrow().len(), 1);
+    }
+
+    #[gpui::test]
+    fn presentation_space_around_a_line_moves_rows_and_maps_points_in_it(cx: &mut TestAppContext) {
+        let text = "alpha\nbeta\ngamma";
+        let (editor, window) = decoration_editor(cx, text, false);
+        let mut cx = VisualTestContext::from_window(window.into(), cx);
+        cx.update(|window, cx| {
+            present_with(
+                &editor,
+                EditorPresentation {
+                    line_space: vec![(1, px(10.), px(6.))],
+                    ..Default::default()
+                },
+                cx,
+            );
+            editor.update(cx, |state, cx| state.set_selected_range(8..8, cx));
+            window.draw(cx).clear(cx);
+            let state = editor.read(cx);
+            let layout = state.last_layout.as_ref().unwrap();
+            let lh = layout.line_height;
+            let map = &state.display_map;
+            assert_close(map.display_row_top(1, lh), lh);
+            assert_close(map.display_row_height(1, lh), lh + px(16.));
+            assert_close(map.display_row_top(2, lh), lh * 2. + px(16.));
+            assert_eq!(map.display_row_at_y(lh + px(3.), lh), 1);
+            assert_eq!(map.display_row_at_y(lh * 2. + px(14.), lh), 1);
+            assert_eq!(map.display_row_at_y(lh * 2. + px(17.), lh), 2);
+            assert_close(map.wrap_rows_height(lh), lh * 3. + px(16.));
+            assert_close(layout.lines[1].size(lh).height, lh + px(16.));
+
+            // The text, the caret and selections sit in the row, inside the space.
+            let beta = TextElement::<EditorMode>::layout_range_corners(&(6..10), layout).unwrap();
+            assert_close(beta[0].top_left.y, layout.visible_top + lh + px(10.));
+            assert_close(beta[0].bottom_left.y - beta[0].top_left.y, lh);
+            let gamma = TextElement::<EditorMode>::layout_range_corners(&(11..16), layout).unwrap();
+            assert_close(gamma[0].top_left.y, layout.visible_top + lh * 2. + px(16.));
+            let caret = layout.cursor_bounds.unwrap();
+            assert_close(
+                caret.origin.y,
+                state.input_bounds.top() + lh + px(10.) + (lh - caret.size.height) / 2.,
+            );
+
+            // Points in the space land on the line's row.
+            let origin =
+                state.last_bounds.unwrap().origin + point(layout.line_number_width, px(0.));
+            let (above, _, _) = state.resolve_mouse_position(origin + point(px(1.), lh + px(4.)));
+            assert!((6..=10).contains(&above), "{above}");
+            let (below, _, _) =
+                state.resolve_mouse_position(origin + point(px(1.), lh * 2. + px(13.)));
+            assert!((6..=10).contains(&below), "{below}");
+            let (next, _, _) =
+                state.resolve_mouse_position(origin + point(px(1.), lh * 2.5 + px(16.)));
+            assert!(next >= 11, "{next}");
+        });
+
+        // Up and down step over the space a row at a time.
+        cx.update(|window, cx| {
+            editor.update(cx, |state, cx| {
+                state.set_selected_range(2..2, cx);
+                state.down(&crate::input::MoveDown, window, cx);
+            });
+            window.draw(cx).clear(cx);
+            assert_eq!(editor.read(cx).cursor(), 8);
+            editor.update(cx, |state, cx| {
+                state.down(&crate::input::MoveDown, window, cx)
+            });
+            assert_eq!(editor.read(cx).cursor(), 13);
+        });
+    }
+
+    #[gpui::test]
+    fn presentation_space_keeps_the_caret_in_view(cx: &mut TestAppContext) {
+        let text = (0..40)
+            .map(|i| format!("line {i}"))
+            .collect::<Vec<_>>()
+            .join("\n");
+        let (editor, window) = decoration_editor(cx, &text, false);
+        let mut cx = VisualTestContext::from_window(window.into(), cx);
+        cx.update(|window, cx| {
+            let space = (0..40).map(|line| (line, px(12.), px(12.))).collect();
+            present_with(
+                &editor,
+                EditorPresentation {
+                    line_space: space,
+                    ..Default::default()
+                },
+                cx,
+            );
+            window.draw(cx).clear(cx);
+            let target = text.find("line 30").unwrap() + 2;
+            editor.update(cx, |state, cx| {
+                state.set_selected_range(target..target, cx);
+                state.scroll_to(target, None, cx);
+            });
+            window.draw(cx).clear(cx);
+            window.draw(cx).clear(cx);
+            let state = editor.read(cx);
+            let layout = state.last_layout.as_ref().unwrap();
+            let lh = layout.line_height;
+            assert!(layout.visible_buffer_lines.contains(&30));
+            // The caret's text row, inside the line's space, is within the viewport.
+            let row_top = state.display_map.display_row_top(30, lh) + px(12.);
+            let view_top = -state.scroll_handle.offset().y;
+            let view_bottom = view_top + state.input_bounds.size.height;
+            assert!(
+                row_top >= view_top && row_top + lh <= view_bottom,
+                "{row_top:?} outside {view_top:?}..{view_bottom:?}"
+            );
+            assert!(state.scroll_size.height >= lh * 40. + px(24. * 40.));
+        });
+    }
+
+    #[gpui::test]
+    fn presentation_inset_shifts_a_line_and_narrows_its_wrapping(cx: &mut TestAppContext) {
+        let text = "alpha beta gamma delta epsilon zeta eta theta iota kappa\nshort";
+        let (editor, window) = decoration_editor(cx, text, true);
+        let mut cx = VisualTestContext::from_window(window.into(), cx);
+        cx.update(|window, cx| {
+            window.draw(cx).clear(cx);
+            let rows = editor.read(cx).last_layout.as_ref().unwrap().lines[0]
+                .wrapped_lines
+                .len();
+            present_with(
+                &editor,
+                EditorPresentation {
+                    line_insets: vec![(0, px(60.))],
+                    ..Default::default()
+                },
+                cx,
+            );
+            window.draw(cx).clear(cx);
+            let state = editor.read(cx);
+            let layout = state.last_layout.as_ref().unwrap();
+            let line = &layout.lines[0];
+            assert!(line.wrapped_lines.len() > rows);
+            let wrap_width = layout.wrap_width.unwrap();
+            for row in &line.wrapped_lines {
+                assert!(
+                    row.width <= wrap_width - px(60.) + px(4.),
+                    "{:?}",
+                    row.width
+                );
+            }
+            for (ix, offset) in [(0, 0), (1, line.wrapped_lines[0].len)] {
+                let position = line.position_for_index(offset, layout, false).unwrap();
+                assert!(position.x >= px(60.), "row {ix}: {position:?}");
+            }
+            assert_close(
+                line.position_for_index(0, layout, false).unwrap().x,
+                px(60.),
+            );
+            let corners = TextElement::<EditorMode>::layout_range_corners(&(0..5), layout).unwrap();
+            assert_close(corners[0].top_left.x, px(60.));
+            // The next line is not inset.
+            assert_eq!(
+                layout.lines[1]
+                    .position_for_index(0, layout, false)
+                    .unwrap()
+                    .x,
+                px(0.)
+            );
+            // A click left of the inset text lands at its start; one on it, inside it.
+            let origin =
+                state.last_bounds.unwrap().origin + point(layout.line_number_width, px(0.));
+            let half = layout.line_height / 2.;
+            let (start, _, _) = state.resolve_mouse_position(origin + point(px(10.), half));
+            assert_eq!(start, 0);
+            let beta = line.position_for_index(7, layout, false).unwrap().x;
+            let (inside, _, _) = state.resolve_mouse_position(origin + point(beta, half));
+            assert!((6..=8).contains(&inside), "{inside}");
+        });
+    }
+
+    #[gpui::test]
+    fn presentation_ornaments_change_no_layout(cx: &mut TestAppContext) {
+        let text = "> quote\n---\nbody";
+        let (editor, window) = decoration_editor(cx, text, true);
+        let mut cx = VisualTestContext::from_window(window.into(), cx);
+        cx.update(|window, cx| {
+            editor.update(cx, |state, cx| state.set_selected_range(3..3, cx));
+            let snapshot = |window: &mut Window, cx: &mut App| {
+                window.draw(cx).clear(cx);
+                let state = editor.read(cx);
+                let layout = state.last_layout.as_ref().unwrap();
+                (
+                    layout
+                        .lines
+                        .iter()
+                        .map(|line| line.size(layout.line_height))
+                        .collect::<Vec<_>>(),
+                    layout.cursor_bounds,
+                    state.scroll_size,
+                )
+            };
+            let before = snapshot(window, cx);
+            let color = gpui::hsla(0., 0., 0.5, 1.);
+            present_with(
+                &editor,
+                EditorPresentation {
+                    line_ornaments: vec![
+                        (
+                            0,
+                            Ornament::Bar {
+                                x: px(2.),
+                                width: px(3.),
+                                color,
+                            },
+                        ),
+                        (
+                            1,
+                            Ornament::Rule {
+                                thickness: px(1.),
+                                color,
+                            },
+                        ),
+                    ],
+                    ..Default::default()
+                },
+                cx,
+            );
+            assert_eq!(snapshot(window, cx), before);
+            assert_eq!(
+                editor.read(cx).editor_presentation().line_ornaments.len(),
+                2
+            );
+        });
+    }
+
+    fn block(lines: Range<usize>, key: u64, height: f32) -> BlockWidget {
+        BlockWidget {
+            lines,
+            height: px(50.),
+            key,
+            render: Rc::new(move |_, _| div().h(px(height)).w_full().into_any_element()),
+        }
+    }
+
+    #[gpui::test]
+    fn presentation_block_widget_replaces_its_lines_with_its_element(cx: &mut TestAppContext) {
+        let text = "top\n| a |\n| b |\nbottom";
+        let (editor, window) = decoration_editor(cx, text, false);
+        let mut cx = VisualTestContext::from_window(window.into(), cx);
+        let (events, _subscription) = record_events(&editor, &mut cx);
+        cx.update(|window, cx| {
+            present_with(
+                &editor,
+                EditorPresentation {
+                    block_widgets: vec![block(1..3, 7, 30.)],
+                    ..Default::default()
+                },
+                cx,
+            );
+            window.draw(cx).clear(cx);
+            {
+                // Laid out at the height it was given until measured.
+                let state = editor.read(cx);
+                let layout = state.last_layout.as_ref().unwrap();
+                assert_eq!(layout.visible_buffer_lines, vec![0, 1, 3]);
+                assert_close(layout.lines[1].size(layout.line_height).height, px(50.));
+            }
+            window.draw(cx).clear(cx);
+            let state = editor.read(cx);
+            let layout = state.last_layout.as_ref().unwrap();
+            let lh = layout.line_height;
+            assert_eq!(layout.visible_buffer_lines, vec![0, 1, 3]);
+            assert_close(layout.lines[1].size(lh).height, px(30.));
+            assert_close(state.display_map.display_row_top(2, lh), lh + px(30.));
+            assert_close(state.display_map.wrap_rows_height(lh), lh * 2. + px(30.));
+            let bottom = text.find("bottom").unwrap();
+            let corners =
+                TextElement::<EditorMode>::layout_range_corners(&(bottom..bottom + 2), layout)
+                    .unwrap();
+            assert_close(corners[0].top_left.y, layout.visible_top + lh + px(30.));
+        });
+
+        // Vertical moves step over the widget's lines in one row.
+        cx.update(|window, cx| {
+            editor.update(cx, |state, cx| {
+                state.set_selected_range(1..1, cx);
+                state.down(&crate::input::MoveDown, window, cx);
+            });
+            let on_block = editor.read(cx).cursor();
+            assert!((4..10).contains(&on_block), "{on_block}");
+            editor.update(cx, |state, cx| {
+                state.down(&crate::input::MoveDown, window, cx)
+            });
+            assert!(editor.read(cx).cursor() >= text.find("bottom").unwrap());
+            editor.update(cx, |state, cx| state.set_selected_range(1..1, cx));
+            window.draw(cx).clear(cx);
+        });
+
+        // A click on the widget is reported and leaves the caret.
+        let (origin, lh) = editor.read_with(&cx, |state, _| {
+            let layout = state.last_layout.as_ref().unwrap();
+            (
+                state.last_bounds.unwrap().origin + point(layout.line_number_width, px(0.)),
+                layout.line_height,
+            )
+        });
+        cx.simulate_click(
+            origin + point(px(20.), lh + px(15.)),
+            gpui::Modifiers::default(),
+        );
+        cx.run_until_parked();
+        assert_eq!(editor.read_with(&cx, |state, _| state.cursor()), 1);
+        assert_eq!(
+            *events.borrow(),
+            vec![EditorPresentationEvent::BlockClicked { lines: 1..3 }]
+        );
+
+        // An edit in its lines drops it, and the lines come back.
+        cx.update(|window, cx| {
+            editor.update(cx, |state, cx| {
+                state.replace_text_in_range(Some(12..12), "z", window, cx)
+            });
+            assert!(
+                editor
+                    .read(cx)
+                    .editor_presentation()
+                    .block_widgets
+                    .is_empty()
+            );
+            window.draw(cx).clear(cx);
+            let layout = editor.read(cx).last_layout.clone().unwrap();
+            assert_eq!(layout.visible_buffer_lines, vec![0, 1, 2, 3]);
+        });
+    }
+
+    #[gpui::test]
+    fn presentation_block_widget_element_gets_its_own_clicks(cx: &mut TestAppContext) {
+        use gpui::StatefulInteractiveElement as _;
+        let text = "top\n| a |\nbottom";
+        let (editor, window) = decoration_editor(cx, text, false);
+        let mut cx = VisualTestContext::from_window(window.into(), cx);
+        let (events, _subscription) = record_events(&editor, &mut cx);
+        let clicks = Rc::new(std::cell::Cell::new(0));
+        cx.update(|window, cx| {
+            let clicks = clicks.clone();
+            present_with(
+                &editor,
+                EditorPresentation {
+                    block_widgets: vec![BlockWidget {
+                        lines: 1..2,
+                        height: px(40.),
+                        key: 1,
+                        render: Rc::new(move |_, _| {
+                            let clicks = clicks.clone();
+                            div()
+                                .id("inner")
+                                .size(px(40.))
+                                .on_click(move |_, _, cx| {
+                                    clicks.set(clicks.get() + 1);
+                                    cx.stop_propagation();
+                                })
+                                .into_any_element()
+                        }),
+                    }],
+                    ..Default::default()
+                },
+                cx,
+            );
+            editor.update(cx, |state, cx| state.set_selected_range(0..0, cx));
+            window.draw(cx).clear(cx);
+            window.draw(cx).clear(cx);
+        });
+        let (origin, lh) = editor.read_with(&cx, |state, _| {
+            let layout = state.last_layout.as_ref().unwrap();
+            (
+                state.last_bounds.unwrap().origin + point(layout.line_number_width, px(0.)),
+                layout.line_height,
+            )
+        });
+        cx.simulate_click(
+            origin + point(px(10.), lh + px(10.)),
+            gpui::Modifiers::default(),
+        );
+        cx.run_until_parked();
+        assert_eq!(clicks.get(), 1);
+        assert!(events.borrow().is_empty());
+        assert_eq!(editor.read_with(&cx, |state, _| state.cursor()), 0);
+    }
+
+    #[gpui::test]
+    fn presentation_extras_survive_edits_and_invalid_input(cx: &mut TestAppContext) {
+        let text = "- a\n> b\n---\n| t |\n| u |\nend";
+        let (editor, window) = decoration_editor(cx, text, true);
+        let mut cx = VisualTestContext::from_window(window.into(), cx);
+        cx.update(|window, cx| {
+            let color = gpui::hsla(0., 0., 0., 1.);
+            present_with(
+                &editor,
+                EditorPresentation {
+                    concealed: vec![4..6],
+                    replacements: vec![
+                        replacement(0..2, "•", true),
+                        replacement(1..3, "x", false),
+                        replacement(4..5, "x", false),
+                        replacement(90..95, "x", false),
+                        replacement(2..3, "a\nb", false),
+                    ],
+                    line_space: vec![
+                        (2, px(8.), px(8.)),
+                        (40, px(1.), px(1.)),
+                        (1, px(f32::NAN), px(1.)),
+                    ],
+                    line_insets: vec![(1, px(20.)), (1, px(30.)), (2, px(-4.))],
+                    line_ornaments: vec![
+                        (
+                            1,
+                            Ornament::Bar {
+                                x: px(0.),
+                                width: px(3.),
+                                color,
+                            },
+                        ),
+                        (
+                            2,
+                            Ornament::Rule {
+                                thickness: px(1.),
+                                color,
+                            },
+                        ),
+                        (
+                            9,
+                            Ornament::Rule {
+                                thickness: px(1.),
+                                color,
+                            },
+                        ),
+                    ],
+                    block_widgets: vec![block(3..5, 1, 20.), block(4..9, 2, 20.)],
+                    ..Default::default()
+                },
+                cx,
+            );
+            let kept = editor.read(cx).editor_presentation();
+            assert_eq!(kept.replacements.len(), 1);
+            assert_eq!(kept.line_space, vec![(2, px(8.), px(8.))]);
+            assert_eq!(kept.line_insets, vec![(1, px(20.))]);
+            assert_eq!(kept.line_ornaments.len(), 2);
+            assert_eq!(kept.block_widgets.len(), 1);
+            window.draw(cx).clear(cx);
+
+            // A new line at the top shifts everything down a line.
+            editor.update(cx, |state, cx| {
+                state.replace_text_in_range(Some(0..0), "x\n", window, cx)
+            });
+            let shifted = editor.read(cx).editor_presentation();
+            assert_eq!(shifted.replacements[0].range, 2..4);
+            assert_eq!(shifted.line_space, vec![(3, px(8.), px(8.))]);
+            assert_eq!(shifted.line_insets, vec![(2, px(20.))]);
+            assert_eq!(
+                shifted
+                    .line_ornaments
+                    .iter()
+                    .map(|(line, _)| *line)
+                    .collect::<Vec<_>>(),
+                vec![2, 3]
+            );
+            assert_eq!(shifted.block_widgets[0].lines, 4..6);
+            window.draw(cx).clear(cx);
+
+            // Typing in the replaced bullet drops it; joining the quote into the list line
+            // drops the quote's items.
+            editor.update(cx, |state, cx| {
+                state.replace_text_in_range(Some(3..3), "y", window, cx);
+                let quote = state.text().to_string().find("> b").unwrap();
+                state.replace_text_in_range(Some(quote - 1..quote), "", window, cx);
+            });
+            let edited = editor.read(cx).editor_presentation();
+            assert!(edited.replacements.is_empty());
+            assert!(edited.line_insets.is_empty());
+            assert_eq!(edited.line_space, vec![(2, px(8.), px(8.))]);
+            assert_eq!(edited.block_widgets[0].lines, 3..5);
+            window.draw(cx).clear(cx);
+
+            // Replacing everything leaves nothing stale behind.
+            editor.update(cx, |state, cx| state.set_value("", window, cx));
+            window.draw(cx).clear(cx);
+            assert!(editor.read(cx).editor_presentation().is_empty());
+            editor.update(cx, |state, cx| state.set_presentation(None, cx));
+            window.draw(cx).clear(cx);
+        });
+    }
+
+    #[gpui::test]
+    fn presentation_extras_unset_keep_the_layout_unchanged(cx: &mut TestAppContext) {
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
+            present_with(
+                &editor,
+                EditorPresentation {
+                    replacements: vec![replacement(0..5, "•", true)],
+                    line_space: vec![(1, px(9.), px(9.))],
+                    line_insets: vec![(0, px(30.))],
+                    block_widgets: vec![block(2..4, 3, 20.)],
+                    ..Default::default()
+                },
+                cx,
+            );
+            assert_ne!(snapshot(window, cx), before);
+            editor.update(cx, |state, cx| state.set_presentation(None, cx));
+            assert_eq!(snapshot(window, cx), before);
+        });
+    }
+
     #[gpui::test]
     fn geometric_decorations_track_edits_history_replacement_and_owner_lifetime(
         cx: &mut TestAppContext,
diff --git a/src/input/base/movement.rs b/src/input/base/movement.rs
index 10304df2..509e09eb 100644
--- a/src/input/base/movement.rs
+++ b/src/input/base/movement.rs
@@ -155,7 +155,7 @@ impl<M: InputModeKind> InputBaseState<M> {
                 if let Some((x, line_end_affinity)) = line.closest_index_for_position(
                     Point {
                         x: preferred_x,
-                        y: next_display_point.local_row * last_layout.line_height,
+                        y: line.row_top(next_display_point.local_row, last_layout.line_height),
                     },
                     last_layout,
                 ) {
diff --git a/src/input/base/state.rs b/src/input/base/state.rs
index d35f0834..65944c44 100644
--- a/src/input/base/state.rs
+++ b/src/input/base/state.rs
@@ -1017,7 +1017,8 @@ impl<M: InputModeKind> InputBaseState<M> {
             let prev_lines_offset = last_layout.visible_line_byte_offsets[vi];
             let local_offset = offset.saturating_sub(prev_lines_offset);
             if let Some(pos) = line.position_for_index(local_offset, last_layout, false) {
-                let sub_line_index = (pos.y / line_height) as usize;
+                let sub_line_index =
+                    ((pos.y - line.frame().above) / line.row_height(line_height)) as usize;
                 let adjusted_pos = point(pos.x + last_layout.line_number_width, pos.y + y_offset);
                 return (vi, sub_line_index, Some(adjusted_pos));
             }
@@ -2419,6 +2420,17 @@ impl<M: InputModeKind> InputBaseState<M> {
         if window.default_prevented() {
             return;
         }
+        // A press on a clickable replacement of the presentation is reported, and leaves
+        // the caret where it is.
+        if event.button == MouseButton::Left
+            && event.click_count == 1
+            && !event.modifiers.shift
+            && !event.modifiers.alt
+            && let Some(range) = self.clickable_replacement_at(event.position)
+        {
+            cx.emit(crate::input::EditorPresentationEvent::ReplacementClicked { range });
+            return;
+        }
         self.undo_manager.break_transaction_coalescing();
         // Input has its own text selection; suppress the window-level text
         // selection (Root) so it does not start a drag from here.
@@ -2675,7 +2687,14 @@ impl<M: InputModeKind> InputBaseState<M> {
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
@@ -2721,9 +2740,9 @@ impl<M: InputModeKind> InputBaseState<M> {
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
@@ -3310,7 +3329,10 @@ impl<M: InputModeKind> InputBaseState<M> {
             .zip(last_line_pos)
             .zip(last_layout.visible_line_byte_offsets.last())
             .map(|((line_layout, pos), line_start)| {
-                let last_row_top = (line_layout.size(line_height).height - line_height).max(px(0.));
+                let last_row_top = (line_layout.size(line_height).height
+                    - line_layout.frame().below
+                    - line_layout.row_height(line_height))
+                .max(px(0.));
                 let pos = point(pos.x, last_row_top);
                 (
                     self.resolve_index(line_start + line_layout.len()),
@@ -3814,7 +3836,15 @@ impl<M: InputModeKind> InputBaseState<M> {
 
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
+                            .row_scale(self.text.offset_to_point(range.end).row),
+                ),
         ))
     }
 
@@ -4563,7 +4593,16 @@ impl<M: InputModeKind> EntityInputHandler for InputBaseState<M> {
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
+                                .row_scale(self.text.offset_to_point(range.start).row),
+                ),
         ))
     }
 
diff --git a/src/input/base/touch.rs b/src/input/base/touch.rs
index e3bd2d26..a9fc83aa 100644
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
+            let row_height = line_height * self.display_map.row_scale(row);
             match position.filter(|_| laid_out.contains(&offset) || laid_out.end == offset) {
-                Some(position) => caret_line_box(origin + position, line_height),
+                Some(position) => caret_line_box(origin + position, row_height),
                 None => caret_line_box(point(viewport.left(), stand_in_y), line_height),
             }
         };
diff --git a/src/input/editor/display_map/display_map.rs b/src/input/editor/display_map/display_map.rs
index 650cd718..d85a9dd0 100644
--- a/src/input/editor/display_map/display_map.rs
+++ b/src/input/editor/display_map/display_map.rs
@@ -4,19 +4,21 @@
 /// - BufferPoint ↔ DisplayPoint conversion
 /// - Fold management (candidates, toggle, query)
 /// - Automatic projection updates on text/layout changes
+use std::collections::HashMap;
 use std::ops::Range;
 
-use gpui::{App, Font, Pixels};
+use gpui::{App, Font, Pixels, px};
 use ropey::Rope;
 
 use super::fold_map::FoldMap;
 use super::folding::FoldRange;
 pub use super::text_wrapper::WrappingIndent;
-use super::text_wrapper::{LineItem, WrapDisplayPoint};
+use super::text_wrapper::{LineFrame, LineItem, WrapDisplayPoint};
 use super::wrap_map::WrapMap;
 use super::{BufferPoint, DisplayPoint};
 use crate::input::Point as TreeSitterPoint;
 use crate::input::display_map::WrapPoint;
+use crate::input::presentation::{BlockWidget, ValidPresentation, line_value};
 use crate::input::rope_ext::RopeExt as _;
 
 /// DisplayMap is the main interface for Editor/Input coordinate mapping.
@@ -29,6 +31,46 @@ use crate::input::rope_ext::RopeExt as _;
 pub struct DisplayMap {
     wrap_map: WrapMap,
     fold_map: FoldMap,
+    /// Visible display rows of the presentation's lines that are not one line height a row
+    /// (scaled lines, lines with space, block widgets), sorted, rebuilt with the fold
+    /// projection. Empty without such a presentation, which keeps rows uniformly tall.
+    scaled_rows: Vec<ScaledRows>,
+    /// Sum over `scaled_rows` of `(scale - 1) * rows`, in line heights.
+    scaled_extra: f32,
+    /// Sum over `scaled_rows` of their extra height in pixels: their space, and how much
+    /// taller than a line height a widget's row is.
+    scaled_extra_px: Pixels,
+    /// Wrap rows hidden by block widgets, which the scroll height leaves out.
+    block_hidden_rows: usize,
+    /// Measured heights of block widgets by key, which replace the heights they were set with.
+    block_heights: HashMap<u64, Pixels>,
+    /// The window's device pixels per logical pixel, which leaves round to; `0.` until known.
+    device_scale: f32,
+    /// The room below the last line of each of the presentation's leaves that rounds the
+    /// leaf up to whole device pixels, by that line.
+    leaf_pads: HashMap<usize, Pixels>,
+}
+
+/// The display rows of one buffer line that are not one line height each.
+struct ScaledRows {
+    rows: Range<usize>,
+    /// Each row is `scale` line heights plus `fixed` pixels tall.
+    scale: f32,
+    fixed: Pixels,
+    /// Space above the first row and below the last.
+    above: Pixels,
+    below: Pixels,
+    /// Extra height, in line heights, of the scaled rows above these.
+    extra_before: f32,
+    /// Extra height, in pixels, of the rows above these.
+    extra_before_px: Pixels,
+}
+
+impl ScaledRows {
+    /// The height of each row, without the space.
+    fn row_height(&self, line_height: Pixels) -> Pixels {
+        line_height * self.scale + self.fixed
+    }
 }
 
 impl DisplayMap {
@@ -36,6 +78,13 @@ impl DisplayMap {
         Self {
             wrap_map: WrapMap::new(font, font_size, wrap_width),
             fold_map: FoldMap::new(),
+            scaled_rows: Vec::new(),
+            scaled_extra: 0.,
+            scaled_extra_px: px(0.),
+            block_hidden_rows: 0,
+            block_heights: HashMap::new(),
+            device_scale: 0.,
+            leaf_pads: HashMap::new(),
         }
     }
 
@@ -266,12 +315,291 @@ impl DisplayMap {
         self.rebuild_fold_projection();
     }
 
+    // ==================== Presentation ====================
+
+    /// Replace the render-only presentation, already validated against the text (see
+    /// `EditorPresentation`). Returns whether anything changed.
+    pub(crate) fn set_presentation(
+        &mut self,
+        presentation: ValidPresentation,
+        cx: &mut App,
+    ) -> bool {
+        // Measured heights are kept only for the widgets still set.
+        if !self.block_heights.is_empty() {
+            self.block_heights
+                .retain(|key, _| presentation.blocks.iter().any(|b| b.key == *key));
+        }
+        let changed = self.wrap_map.set_presentation(presentation, cx);
+        if changed {
+            self.rebuild_fold_projection();
+        }
+        changed
+    }
+
+    /// The validated presentation, carried through any edits since it was set.
+    #[inline]
+    pub(crate) fn presentation(&self) -> &ValidPresentation {
+        self.wrap_map.wrapper().presentation()
+    }
+
+    /// The concealed byte ranges, sorted and disjoint.
+    #[inline]
+    pub(crate) fn concealed(&self) -> &[Range<usize>] {
+        self.wrap_map.wrapper().concealed()
+    }
+
+    /// Whether a presentation changes anything.
+    #[inline]
+    pub(crate) fn has_presentation(&self) -> bool {
+        !self.presentation().is_empty()
+    }
+
+    /// Whether some display rows are not one line height tall.
+    #[inline]
+    pub(crate) fn has_uneven_rows(&self) -> bool {
+        !self.scaled_rows.is_empty()
+    }
+
+    /// The scale of buffer line `line`: `1.0` unless the presentation scales it.
+    #[inline]
+    pub(crate) fn line_scale(&self, line: usize) -> f32 {
+        self.wrap_map.wrapper().line_scale(line)
+    }
+
+    /// How many line heights tall each row of buffer line `line` is: its scale times its
+    /// line height factor.
+    #[inline]
+    pub(crate) fn row_scale(&self, line: usize) -> f32 {
+        self.wrap_map.wrapper().row_scale(line)
+    }
+
+    /// The block widget whose first line is `line`.
+    pub(crate) fn block_at(&self, line: usize) -> Option<&BlockWidget> {
+        let blocks = &self.presentation().blocks;
+        blocks
+            .binary_search_by_key(&line, |b| b.lines.start)
+            .ok()
+            .map(|ix| &blocks[ix])
+    }
+
+    /// The height a block widget is laid out at: its measured one once it has been drawn.
+    pub(crate) fn block_height(&self, block: &BlockWidget) -> Pixels {
+        self.block_heights
+            .get(&block.key)
+            .copied()
+            .unwrap_or(block.height)
+    }
+
+    /// Record the measured height of the widget with `key`. Returns whether the layout
+    /// changed.
+    pub(crate) fn set_block_height(&mut self, key: u64, height: Pixels) -> bool {
+        if !f32::from(height).is_finite() || height <= px(0.) {
+            return false;
+        }
+        let Some(block) = self.presentation().blocks.iter().find(|b| b.key == key) else {
+            return false;
+        };
+        if (self.block_height(block) - height).abs() < px(0.5) {
+            return false;
+        }
+        self.block_heights.insert(key, height);
+        self.rebuild_scaled_rows();
+        true
+    }
+
+    /// Set the window's device pixels per logical pixel, which the presentation's leaves
+    /// round to. Returns whether the layout changed.
+    pub(crate) fn set_device_scale(&mut self, scale: f32) -> bool {
+        if self.device_scale == scale || !scale.is_finite() || scale <= 0. {
+            return false;
+        }
+        self.device_scale = scale;
+        if self.presentation().leaves.is_empty() {
+            return false;
+        }
+        self.rebuild_scaled_rows();
+        true
+    }
+
+    /// The room the presentation gives buffer line `line` around and before its rows.
+    pub(crate) fn line_frame(&self, line: usize) -> LineFrame {
+        let presentation = self.presentation();
+        if presentation.line_space.is_empty()
+            && presentation.line_insets.is_empty()
+            && presentation.blocks.is_empty()
+            && self.leaf_pads.is_empty()
+        {
+            return LineFrame::default();
+        }
+        if let Some(block) = self.block_at(line) {
+            return LineFrame {
+                block: Some(self.block_height(block)),
+                ..LineFrame::default()
+            };
+        }
+        let (above, below) = self.line_space(line);
+        LineFrame {
+            above,
+            below,
+            inset: line_value(&presentation.line_insets, line).unwrap_or(px(0.)),
+            block: None,
+        }
+    }
+
+    /// The space above and below buffer line `line`.
+    fn line_space(&self, line: usize) -> (Pixels, Pixels) {
+        let space = &self.presentation().line_space;
+        let (above, below) = space
+            .binary_search_by_key(&line, |(l, _, _)| *l)
+            .map_or((px(0.), px(0.)), |ix| (space[ix].1, space[ix].2));
+        let pad = self.leaf_pads.get(&line).copied().unwrap_or(px(0.));
+        (above, below + pad)
+    }
+
+    /// Work out the room that rounds each leaf up to whole device pixels: its rows (as
+    /// many as it wraps to now), the space between them and its extra.
+    fn rebuild_leaf_pads(&mut self) {
+        self.leaf_pads.clear();
+        let presentation = self.presentation().clone();
+        let (Some(line_height), scale) = (presentation.line_height, self.device_scale) else {
+            return;
+        };
+        if scale <= 0. {
+            return;
+        }
+        for (lines, extra) in presentation.leaves.iter() {
+            let mut height = *extra;
+            for line in lines.clone() {
+                let Some(rows) = self.buffer_line_to_display_row_range(line) else {
+                    continue;
+                };
+                height += line_height * self.row_scale(line) * rows.len() as f32;
+                let space = &presentation.line_space;
+                if let Ok(ix) = space.binary_search_by_key(&line, |(l, _, _)| *l) {
+                    if line != lines.start {
+                        height += space[ix].1;
+                    }
+                    if line + 1 != lines.end {
+                        height += space[ix].2;
+                    }
+                }
+            }
+            let device = f32::from(height) * scale;
+            // Within a rounding error of a whole device pixel is on it: a scaled row's
+            // height is a product of floats.
+            let pad = px(((device - 1e-3).ceil() - device).max(0.) / scale);
+            if pad > px(0.) {
+                self.leaf_pads.insert(lines.end - 1, pad);
+            }
+        }
+    }
+
+    /// The y of the top of display row `row`, in content space.
+    ///
+    /// Rows are `line_height` tall except those of scaled lines, so without a presentation
+    /// this is `line_height * row`. The space above a line belongs to its first row, and the
+    /// space below to its last.
+    pub(crate) fn display_row_top(&self, row: usize, line_height: Pixels) -> Pixels {
+        if self.scaled_rows.is_empty() {
+            return line_height * row;
+        }
+        let ix = self.scaled_rows.partition_point(|s| s.rows.start < row);
+        let Some(scaled) = ix.checked_sub(1).map(|ix| &self.scaled_rows[ix]) else {
+            return line_height * row;
+        };
+        let inside = row.min(scaled.rows.end) - scaled.rows.start;
+        let space = if row >= scaled.rows.end {
+            scaled.above + scaled.below
+        } else {
+            scaled.above
+        };
+        line_height * (row as f32 + scaled.extra_before + (scaled.scale - 1.) * inside as f32)
+            + scaled.extra_before_px
+            + scaled.fixed * inside as f32
+            + space
+    }
+
+    /// The height of display row `row`, its share of its line's space included.
+    pub(crate) fn display_row_height(&self, row: usize, line_height: Pixels) -> Pixels {
+        let ix = self.scaled_rows.partition_point(|s| s.rows.end <= row);
+        match self.scaled_rows.get(ix) {
+            Some(scaled) if scaled.rows.contains(&row) => {
+                let mut height = scaled.row_height(line_height);
+                if row == scaled.rows.start {
+                    height += scaled.above;
+                }
+                if row + 1 == scaled.rows.end {
+                    height += scaled.below;
+                }
+                height
+            }
+            _ => line_height,
+        }
+    }
+
+    /// The display row under content-space `y` (rows past the end continue at
+    /// `line_height`, so the caller clamps). A line's space maps to its nearest row.
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
+        let row_height = scaled.row_height(line_height);
+        let bottom = top + scaled.above + row_height * scaled.rows.len() as f32 + scaled.below;
+        if y < bottom {
+            let y = y - top - scaled.above;
+            let local = if y <= px(0.) || row_height <= px(0.) {
+                0
+            } else {
+                (f32::from(y) / f32::from(row_height)).floor() as usize
+            };
+            scaled.rows.start + local.min(scaled.rows.len() - 1)
+        } else {
+            uniform(y - bottom, scaled.rows.end)
+        }
+    }
+
+    /// The height of every wrap row, for the scroll size: `line_height` each, plus the extra
+    /// of the visible scaled rows, less the rows block widgets hide.
+    pub(crate) fn wrap_rows_height(&self, line_height: Pixels) -> Pixels {
+        let rows = self.wrap_map.wrap_row_count() as f32;
+        if self.scaled_rows.is_empty() && self.block_hidden_rows == 0 {
+            rows * line_height
+        } else {
+            (rows - self.block_hidden_rows as f32 + self.scaled_extra) * line_height
+                + self.scaled_extra_px
+        }
+    }
+
     // ==================== Internal Helpers ====================
 
     /// Rebuild fold projection after wrap_map or fold state changes
     /// Only rebuilds if there are actually folded ranges
     fn rebuild_fold_projection(&mut self) {
-        if !self.fold_map.folded_ranges().is_empty() {
+        // A block widget hides its lines after the first the way a fold does.
+        let hidden: Vec<Range<usize>> = self
+            .presentation()
+            .blocks
+            .iter()
+            .map(|b| b.lines.start + 1..b.lines.end)
+            .filter(|lines| !lines.is_empty())
+            .collect();
+        self.block_hidden_rows = hidden
+            .iter()
+            .map(|lines| {
+                self.wrap_map.buffer_line_to_first_wrap_row(lines.end)
+                    - self.wrap_map.buffer_line_to_first_wrap_row(lines.start)
+            })
+            .sum();
+        self.fold_map.set_hidden_lines(hidden);
+        if self.fold_map.hides_rows() {
             self.fold_map.rebuild(&self.wrap_map);
         } else {
             // No active folds: identity mapping (wrap_row == display_row).
@@ -279,6 +607,68 @@ impl DisplayMap {
             self.fold_map
                 .mark_dirty_with_wrap_count(self.wrap_map.wrap_row_count());
         }
+        self.rebuild_scaled_rows();
+    }
+
+    /// Recompute the display rows of the lines that are not one line height a row. A folded
+    /// line has none, so it adds no height.
+    fn rebuild_scaled_rows(&mut self) {
+        self.scaled_rows.clear();
+        self.scaled_extra = 0.;
+        self.scaled_extra_px = px(0.);
+        self.rebuild_leaf_pads();
+        let presentation = self.presentation().clone();
+        if presentation.line_scales.is_empty()
+            && presentation.line_space.is_empty()
+            && presentation.blocks.is_empty()
+            && presentation.line_heights.is_empty()
+            && self.leaf_pads.is_empty()
+        {
+            return;
+        }
+        let mut lines: Vec<usize> = presentation
+            .line_scales
+            .iter()
+            .map(|(line, _)| *line)
+            .chain(presentation.line_space.iter().map(|(line, _, _)| *line))
+            .chain(presentation.blocks.iter().map(|b| b.lines.start))
+            .chain(presentation.line_heights.iter().map(|(line, _)| *line))
+            .chain(self.leaf_pads.keys().copied())
+            .collect();
+        lines.sort_unstable();
+        lines.dedup();
+        let mut extra = 0.;
+        let mut extra_px = px(0.);
+        let mut scaled_rows = Vec::with_capacity(lines.len());
+        for line in lines {
+            let Some(rows) = self.buffer_line_to_display_row_range(line) else {
+                continue;
+            };
+            let len = rows.len() as f32;
+            let (scale, fixed, above, below) = match self.block_at(line) {
+                // A widget's line is one row as tall as the widget: no line height, all
+                // fixed pixels.
+                Some(block) => (0., self.block_height(block), px(0.), px(0.)),
+                None => {
+                    let (above, below) = self.line_space(line);
+                    (self.row_scale(line), px(0.), above, below)
+                }
+            };
+            scaled_rows.push(ScaledRows {
+                rows,
+                scale,
+                fixed,
+                above,
+                below,
+                extra_before: extra,
+                extra_before_px: extra_px,
+            });
+            extra += (scale - 1.) * len;
+            extra_px += fixed * len + above + below;
+        }
+        self.scaled_rows = scaled_rows;
+        self.scaled_extra = extra;
+        self.scaled_extra_px = extra_px;
     }
 
     // ==================== Wrap Display Point Operations ====================
diff --git a/src/input/editor/display_map/fold_map.rs b/src/input/editor/display_map/fold_map.rs
index 32c6b0a2..cbbf0318 100644
--- a/src/input/editor/display_map/fold_map.rs
+++ b/src/input/editor/display_map/fold_map.rs
@@ -54,6 +54,10 @@ pub(super) struct FoldMap {
     /// Cached wrap_row_count from last rebuild
     /// Used to detect if WrapMap changed and rebuild is needed
     cached_wrap_row_count: usize,
+
+    /// Buffer lines a presentation hides (those of block widgets after the first), sorted
+    /// and disjoint. Hidden like the inside of a fold.
+    hidden_lines: Vec<Range<usize>>,
 }
 
 impl FoldMap {
@@ -66,9 +70,24 @@ impl FoldMap {
             folded: Vec::new(),
             needs_rebuild: true,
             cached_wrap_row_count: 0,
+            hidden_lines: Vec::new(),
+        }
+    }
+
+    /// Replace the buffer lines a presentation hides.
+    pub(super) fn set_hidden_lines(&mut self, hidden_lines: Vec<Range<usize>>) {
+        if self.hidden_lines != hidden_lines {
+            self.hidden_lines = hidden_lines;
+            self.needs_rebuild = true;
         }
     }
 
+    /// Whether any wrap row is hidden, by a fold or a presentation.
+    #[inline]
+    pub(super) fn hides_rows(&self) -> bool {
+        !self.folded.is_empty() || !self.hidden_lines.is_empty()
+    }
+
     /// Update cached wrap_row_count without full rebuild.
     /// Used when no folds are active (identity mapping assumed).
     pub(super) fn mark_dirty_with_wrap_count(&mut self, wrap_row_count: usize) {
@@ -78,7 +97,7 @@ impl FoldMap {
 
     /// Get total number of visible display rows
     pub(super) fn display_row_count(&self) -> usize {
-        if self.folded.is_empty() {
+        if !self.hides_rows() {
             return self.cached_wrap_row_count;
         }
         self.projected_wrap_row_count - self.total_hidden
@@ -94,7 +113,7 @@ impl FoldMap {
     /// Convert wrap_row to display_row
     /// Returns None if the wrap_row is hidden by folding
     pub(super) fn wrap_row_to_display_row(&self, wrap_row: usize) -> Option<usize> {
-        if self.folded.is_empty() {
+        if !self.hides_rows() {
             return if wrap_row < self.cached_wrap_row_count {
                 Some(wrap_row)
             } else {
@@ -119,7 +138,7 @@ impl FoldMap {
 
     /// Convert display_row to wrap_row
     pub(super) fn display_row_to_wrap_row(&self, display_row: usize) -> Option<usize> {
-        if self.folded.is_empty() {
+        if !self.hides_rows() {
             return if display_row < self.cached_wrap_row_count {
                 Some(display_row)
             } else {
@@ -137,7 +156,7 @@ impl FoldMap {
 
     /// Find the nearest visible display_row for a given wrap_row
     pub(super) fn nearest_visible_display_row(&self, wrap_row: usize) -> usize {
-        if self.folded.is_empty() {
+        if !self.hides_rows() {
             return wrap_row.min(self.cached_wrap_row_count.saturating_sub(1));
         }
 
@@ -307,7 +326,7 @@ impl FoldMap {
 
         self.cached_wrap_row_count = wrap_row_count;
 
-        if self.folded.is_empty() {
+        if !self.hides_rows() {
             // Fast path: no folds, all wrap rows are visible
             self.set_hidden_rows(wrap_row_count, Vec::new());
             self.needs_rebuild = false;
@@ -338,6 +357,17 @@ impl FoldMap {
                 hidden_ranges.push(start_wrap_row..end_wrap_row);
             }
         }
+        for lines in &self.hidden_lines {
+            let start_wrap_row = wrap_map.buffer_line_to_first_wrap_row(lines.start);
+            let end_wrap_row = if lines.end < wrap_map.buffer_line_count() {
+                wrap_map.buffer_line_to_first_wrap_row(lines.end)
+            } else {
+                wrap_row_count
+            };
+            if start_wrap_row < end_wrap_row {
+                hidden_ranges.push(start_wrap_row..end_wrap_row);
+            }
+        }
 
         self.set_hidden_rows(wrap_row_count, hidden_ranges);
         self.needs_rebuild = false;
diff --git a/src/input/editor/display_map/inline_line.rs b/src/input/editor/display_map/inline_line.rs
index b45fd3ba..b4b8435a 100644
--- a/src/input/editor/display_map/inline_line.rs
+++ b/src/input/editor/display_map/inline_line.rs
@@ -1,5 +1,7 @@
 //! A visual row whose offsets remain in source UTF-8 bytes.
-use gpui::{App, Pixels, Point, ShapedLine, SharedString, TextAlign, Window, point, px};
+use gpui::{
+    App, Bounds, Hsla, Pixels, Point, ShapedLine, SharedString, TextAlign, Window, point, px, size,
+};
 use std::ops::Range;
 
 pub(crate) struct InlineFragment {
@@ -7,6 +9,41 @@ pub(crate) struct InlineFragment {
     pub(crate) x: Pixels,
     pub(crate) width: Pixels,
     pub(crate) text: Option<ShapedLine>,
+    /// A presentation's replacement: `text` is drawn in place of the source in `range`, which
+    /// the caret treats as one unit, like an element.
+    pub(crate) replaced: bool,
+    /// Where `text` starts within the fragment: a replacement set in a column is
+    /// right-aligned in it.
+    pub(crate) text_x: Pixels,
+    /// How far a hanging replacement's column reaches left of `x`, where a click
+    /// still lands on it.
+    pub(crate) hang: Pixels,
+    /// How far `text` is drawn below where the row would centre it: a replacement at
+    /// another size keeps the row's baseline.
+    pub(crate) text_y: Pixels,
+    /// A replacement's box, drawn behind `text`.
+    pub(crate) chip: Option<FragmentChip>,
+    /// A replacement's image, drawn instead of `text`, with the row font's ascent and
+    /// descent to put its baseline on the row's.
+    pub(crate) image: Option<(crate::input::ReplacementImage, Pixels, Pixels)>,
+}
+
+/// A replacement's box, as laid out: what to draw, and its text's metrics to place it.
+#[derive(Clone)]
+pub(crate) struct FragmentChip {
+    /// The icon's asset, size and colour, and where it starts in the fragment.
+    pub(crate) icon: Option<(SharedString, Pixels, Hsla, Pixels)>,
+    pub(crate) dashed_bottom: Option<Hsla>,
+    /// Whether it stands for a laid-out element, which sits on whole device pixels.
+    pub(crate) whole_width: bool,
+    /// Where its text's baseline is, below the box's top.
+    pub(crate) text_baseline: Option<Pixels>,
+    pub(crate) background: Hsla,
+    pub(crate) radius: Pixels,
+    /// Its top and height within the row, from the row's height.
+    pub(crate) height: crate::input::ChipHeight,
+    pub(crate) cap_height: Pixels,
+    pub(crate) font_size: Pixels,
 }
 
 pub(crate) struct InputLine {
@@ -49,10 +86,13 @@ impl InputLine {
                 for f in fragments {
                     if ix < f.range.end {
                         return f.x
-                            + f.text.as_ref().map_or(
-                                if ix <= f.range.start { px(0.) } else { f.width },
-                                |line| line.x_for_index(ix.saturating_sub(f.range.start)),
-                            );
+                            + match &f.text {
+                                // A caret inside a replacement is drawn at its start.
+                                Some(_) if f.replaced => px(0.),
+                                Some(line) => line.x_for_index(ix.saturating_sub(f.range.start)),
+                                None if ix <= f.range.start => px(0.),
+                                None => f.width,
+                            };
                     }
                 }
                 self.width
@@ -65,20 +105,41 @@ impl InputLine {
             Content::Inline(fragments) => {
                 for f in fragments {
                     if x <= f.x + f.width {
-                        return f.text.as_ref().map_or(
-                            if x - f.x < f.width / 2. {
-                                f.range.start
-                            } else {
-                                f.range.end
-                            },
-                            |line| f.range.start + line.closest_index_for_x(x - f.x),
-                        );
+                        return match &f.text {
+                            Some(line) if !f.replaced => {
+                                f.range.start + line.closest_index_for_x(x - f.x)
+                            }
+                            // A concealed range has no width to split, so a click at
+                            // its point lands after it, past the hidden source.
+                            _ if f.width > px(0.) && x - f.x < f.width / 2. => f.range.start,
+                            _ => f.range.end,
+                        };
                     }
                 }
                 self.len
             }
         }
     }
+    /// Whether the byte at `ix` is concealed: inside a fragment drawn at zero width, or
+    /// drawn as a replacement's text.
+    pub(crate) fn is_concealed(&self, ix: usize) -> bool {
+        match &self.content {
+            Content::Text(_) => false,
+            Content::Inline(fragments) => fragments.iter().any(|f| {
+                ((f.text.is_none() && f.width <= px(0.)) || f.replaced) && f.range.contains(&ix)
+            }),
+        }
+    }
+    /// The row-local source range of the replacement drawn under `x`, if any.
+    pub(crate) fn replacement_at_x(&self, x: Pixels) -> Option<Range<usize>> {
+        match &self.content {
+            Content::Text(_) => None,
+            Content::Inline(fragments) => fragments
+                .iter()
+                .find(|f| f.replaced && x >= f.x - f.hang && x < f.x + f.width.max(px(0.)))
+                .map(|f| f.range.clone()),
+        }
+    }
     pub(crate) fn index_for_x(&self, x: Pixels) -> Option<usize> {
         match &self.content {
             Content::Text(line) => line.index_for_x(x),
@@ -138,11 +199,125 @@ impl InputLine {
                     TextAlign::Center => remaining / 2.,
                     _ => px(0.),
                 };
+                let scale = window.scale_factor();
+                // As the layout snaps a box: to whole device pixels, halves toward zero.
+                // Within a rounding error of a half is a half.
+                let snap = |v: Pixels| {
+                    let device = f32::from(v) * scale;
+                    px((device.abs() - 0.5 - 1e-3).ceil().copysign(device) / scale)
+                };
                 for f in fragments {
+                    // A chip's box: its top and height in the row.
+                    let geometry = match (&f.chip, &f.text) {
+                        (Some(chip), Some(line)) => Some(match chip.height {
+                            crate::input::ChipHeight::Fixed(h) => ((height - h) / 2., h),
+                            crate::input::ChipHeight::Baseline { ascent, descent } => {
+                                let baseline =
+                                    (height - line.ascent - line.descent) / 2. + line.ascent;
+                                (baseline - ascent, ascent + descent)
+                            }
+                            crate::input::ChipHeight::CapHeight => {
+                                let baseline = (height - line.ascent - line.descent) / 2.
+                                    + line.ascent
+                                    + f.text_y;
+                                let pad = chip.font_size * 0.125 + line.descent / 2.;
+                                (baseline - chip.cap_height - pad, chip.cap_height + pad * 2.)
+                            }
+                        }),
+                        _ => None,
+                    };
+                    // A chip standing for a laid-out element sits on whole device pixels,
+                    // as the element's box does, and so does everything in it.
+                    let shift = match (&f.chip, geometry) {
+                        (Some(chip), Some((top, _))) if chip.whole_width => {
+                            let origin = pos + point(offset + f.x, top);
+                            point(snap(origin.x) - origin.x, snap(origin.y) - origin.y)
+                        }
+                        _ => point(px(0.), px(0.)),
+                    };
+                    if background
+                        && let (Some(chip), Some((top, chip_height))) = (&f.chip, geometry)
+                    {
+                        window.paint_quad(
+                            gpui::fill(
+                                Bounds::new(
+                                    pos + shift + point(offset + f.x, top),
+                                    size(f.width, chip_height),
+                                ),
+                                chip.background,
+                            )
+                            .corner_radii(chip.radius),
+                        );
+                        if let Some(color) = chip.dashed_bottom {
+                            // On whole device pixels, as a laid-out box's border is, so
+                            // its dashes fall the same way.
+                            let origin = pos + shift + point(offset + f.x, top);
+                            let (left, top) = (snap(origin.x), snap(origin.y));
+                            let right = snap(origin.x + f.width);
+                            let bottom = snap(origin.y + chip_height);
+                            window.paint_quad(gpui::quad(
+                                Bounds::new(point(left, top), size(right - left, bottom - top)),
+                                px(0.),
+                                gpui::transparent_black(),
+                                gpui::Edges {
+                                    bottom: px(1.),
+                                    ..Default::default()
+                                },
+                                color,
+                                gpui::BorderStyle::Dashed,
+                            ));
+                        }
+                    }
+                    if !background
+                        && let Some(FragmentChip {
+                            icon: Some((path, side, color, x)),
+                            ..
+                        }) = &f.chip
+                    {
+                        let bounds = Bounds::new(
+                            pos + shift + point(offset + f.x + *x, (height - *side) / 2.),
+                            size(*side, *side),
+                        );
+                        window
+                            .paint_svg(
+                                bounds,
+                                path.clone(),
+                                None,
+                                gpui::TransformationMatrix::unit(),
+                                *color,
+                                cx,
+                            )
+                            .ok();
+                    }
+                    if let Some((image, ascent, descent)) = &f.image {
+                        if !background {
+                            // An image is a laid-out element: on whole device pixels.
+                            let baseline = (height - *ascent - *descent) / 2. + *ascent;
+                            let origin = pos + point(offset + f.x, baseline - image.baseline);
+                            let bounds =
+                                Bounds::new(point(snap(origin.x), snap(origin.y)), image.size);
+                            if let Some(data) = image.image.clone().use_render_image(window, cx) {
+                                window
+                                    .paint_image(bounds, bounds, Default::default(), data, 0, false)
+                                    .ok();
+                            }
+                        }
+                        continue;
+                    }
+                    // Text set in the box, as the element sets it.
+                    let text_y = match (&f.chip, geometry, &f.text) {
+                        (Some(chip), Some((top, _)), Some(line))
+                            if chip.text_baseline.is_some() =>
+                        {
+                            let centred = (height - line.ascent - line.descent) / 2. + line.ascent;
+                            top + chip.text_baseline.unwrap_or_default() - centred
+                        }
+                        _ => f.text_y,
+                    };
                     if let Some(line) = &f.text {
                         paint(
                             line,
-                            pos + point(offset + f.x, px(0.)),
+                            pos + shift + point(offset + f.x + f.text_x, text_y),
                             TextAlign::Left,
                             None,
                             window,
diff --git a/src/input/editor/display_map/mod.rs b/src/input/editor/display_map/mod.rs
index 0ebaf622..3214ddca 100644
--- a/src/input/editor/display_map/mod.rs
+++ b/src/input/editor/display_map/mod.rs
@@ -9,7 +9,7 @@
 /// about `BufferPoint ↔ DisplayPoint` mapping, without worrying about internal wrap/fold complexity.
 mod display_map;
 mod inline_line;
-pub(crate) use inline_line::{InlineFragment, InputLine};
+pub(crate) use inline_line::{FragmentChip, InlineFragment, InputLine};
 mod fold_map;
 mod folding;
 mod text_wrapper;
diff --git a/src/input/editor/display_map/text_wrapper.rs b/src/input/editor/display_map/text_wrapper.rs
index 4c59bf27..30be8312 100644
--- a/src/input/editor/display_map/text_wrapper.rs
+++ b/src/input/editor/display_map/text_wrapper.rs
@@ -15,6 +15,7 @@ use unicode_segmentation::UnicodeSegmentation as _;
 use crate::input::{
     Point as TreeSitterPoint, RopeExt,
     layout::{LastLayout, WhitespaceIndicators},
+    presentation::{BlockWidget, Replacement, ValidPresentation, line_value},
 };
 
 /// Controls how soft-wrapped continuation lines are indented.
@@ -228,6 +229,12 @@ pub(crate) struct TextWrapper {
     pub(crate) lines: SumTree<LineItem>,
 
     inline_metrics: Rc<[(Range<usize>, Pixels)]>,
+    /// The render-only presentation, validated. Its concealed ranges wrap at zero width and
+    /// its replacements at the width of their text (both ignored for lines that have inline
+    /// token metrics). A scaled line is wrapped at `wrap_width / scale`, so its text fits once
+    /// it is shaped `scale` times larger, an inset line that much narrower first, and the
+    /// first line of a block widget is never wrapped (it is drawn as one row).
+    presentation: ValidPresentation,
     _initialized: bool,
 }
 
@@ -242,6 +249,7 @@ impl TextWrapper {
             wrapping_indent: WrappingIndent::default(),
             lines: SumTree::new(&()),
             inline_metrics: Rc::from([]),
+            presentation: ValidPresentation::default(),
             _initialized: false,
         }
     }
@@ -373,6 +381,8 @@ impl TextWrapper {
             .text_system()
             .line_wrapper(self.font.clone(), self.font_size);
         let metrics = self.inline_metrics.clone();
+        let concealed = self.presentation.concealed.clone();
+        let replacements = self.presentation.replacements.clone();
         let text_system = gpui::WindowTextSystem::new(cx.text_system().clone());
         let font = self.font.clone();
         let font_size = self.font_size;
@@ -404,6 +414,71 @@ impl TextWrapper {
                     fragments.push(LineFragment::element(*width, range.len()));
                     offset = range.end;
                 }
+                // Concealed text takes no room, so it wraps as a zero-width element, and a
+                // replacement as an element as wide as its text. A line with inline tokens
+                // keeps them alone.
+                if fragments.is_empty() {
+                    let line_end = line_start + line_str.len();
+                    let first = concealed.partition_point(|r| r.end <= line_start);
+                    let hidden = concealed[first..]
+                        .iter()
+                        .take_while(|r| r.start < line_end)
+                        .map(|range| (range.clone(), None));
+                    let first = replacements.partition_point(|r| r.range.end <= line_start);
+                    let replaced = replacements[first..]
+                        .iter()
+                        .take_while(|r| r.range.start < line_end)
+                        .map(|r| (r.range.clone(), Some(r)));
+                    let mut items: Vec<_> = hidden.chain(replaced).collect();
+                    if !items.is_empty() {
+                        items.sort_by_key(|(range, _)| range.start);
+                    }
+                    for (range, replacement) in items {
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
+                        let width = replacement.map_or(px(0.), |replacement| {
+                            let text = replacement.text.clone();
+                            let font = gpui::Font {
+                                family: replacement
+                                    .family
+                                    .clone()
+                                    .unwrap_or_else(|| font.family.clone()),
+                                weight: replacement.style.font_weight.unwrap_or(font.weight),
+                                style: replacement.style.font_style.unwrap_or(font.style),
+                                ..font.clone()
+                            };
+                            let width = text_system
+                                .layout_line(
+                                    &text,
+                                    replacement.font_size(font_size),
+                                    &[gpui::TextRun {
+                                        len: text.len(),
+                                        font,
+                                        color: gpui::black(),
+                                        background_color: None,
+                                        underline: None,
+                                        strikethrough: None,
+                                    }],
+                                    None,
+                                )
+                                .width;
+                            replacement.placement(width).0
+                        });
+                        fragments.push(LineFragment::element(width, range.len()));
+                        offset = range.end;
+                    }
+                }
                 if fragments.is_empty() {
                     return measured_wrap_boundaries(
                         line_str,
@@ -515,6 +590,280 @@ impl TextWrapper {
         }
     }
 
+    /// The validated presentation.
+    #[inline]
+    pub(crate) fn presentation(&self) -> &ValidPresentation {
+        &self.presentation
+    }
+
+    /// The concealed byte ranges of the presentation.
+    #[inline]
+    pub(crate) fn concealed(&self) -> &[Range<usize>] {
+        &self.presentation.concealed
+    }
+
+    /// The `(buffer line, scale)` pairs of the presentation, sorted by line.
+    #[inline]
+    pub(crate) fn line_scales(&self) -> &[(usize, f32)] {
+        &self.presentation.line_scales
+    }
+
+    /// The scale of buffer line `row`, `1.0` unless the presentation scales it.
+    #[inline]
+    pub(crate) fn line_scale(&self, row: usize) -> f32 {
+        if self.presentation.line_scales.is_empty() {
+            return 1.;
+        }
+        line_value(&self.presentation.line_scales, row).unwrap_or(1.)
+    }
+
+    /// How many line heights tall each visual row of buffer line `row` is: its scale,
+    /// times its line height factor.
+    #[inline]
+    pub(crate) fn row_scale(&self, row: usize) -> f32 {
+        let scale = self.line_scale(row);
+        if self.presentation.line_heights.is_empty() {
+            return scale;
+        }
+        scale * line_value(&self.presentation.line_heights, row).unwrap_or(1.)
+    }
+
+    /// How much the wrap width of buffer line `row` shrinks, and whether it is wrapped at all.
+    fn line_wrap(presentation: &ValidPresentation, row: usize) -> (f32, Pixels, bool) {
+        let scale = line_value(&presentation.line_scales, row).unwrap_or(1.);
+        let inset = line_value(&presentation.line_insets, row).unwrap_or(px(0.));
+        let block = presentation
+            .blocks
+            .binary_search_by_key(&row, |b| b.lines.start)
+            .is_ok();
+        (scale, inset, block)
+    }
+
+    /// Replace the render-only presentation, already validated against the text, and rewrap
+    /// the lines whose wrapping it changes. Returns whether anything changed.
+    pub(crate) fn set_presentation(
+        &mut self,
+        presentation: ValidPresentation,
+        cx: &mut App,
+    ) -> bool {
+        if self.presentation == presentation {
+            return false;
+        }
+        let text = self.text.clone();
+        let row_of = |offset: usize| text.offset_to_point(offset.min(text.len())).row;
+        let mut rows: Vec<usize> = Vec::new();
+        // Ranges in only one of the two lists, found the way `set_inline_metrics` does.
+        fn diff<T: PartialEq>(
+            old: &[T],
+            new: &[T],
+            start: impl Fn(&T) -> usize,
+            mut changed: impl FnMut(usize),
+        ) {
+            let (mut old, mut new) = (old.iter().peekable(), new.iter().peekable());
+            loop {
+                match (old.peek(), new.peek()) {
+                    (Some(a), Some(b)) if a == b => {
+                        old.next();
+                        new.next();
+                    }
+                    (Some(a), Some(b)) if start(a) <= start(b) => {
+                        changed(start(a));
+                        old.next();
+                    }
+                    (_, Some(b)) => {
+                        changed(start(b));
+                        new.next();
+                    }
+                    (Some(a), None) => {
+                        changed(start(a));
+                        old.next();
+                    }
+                    (None, None) => break,
+                }
+            }
+        }
+        let old = &self.presentation;
+        diff(
+            &old.concealed,
+            &presentation.concealed,
+            |r| r.start,
+            |o| rows.push(row_of(o)),
+        );
+        diff(
+            &old.replacements,
+            &presentation.replacements,
+            |r| r.range.start,
+            |o| rows.push(row_of(o)),
+        );
+        let lines = old
+            .line_scales
+            .iter()
+            .chain(presentation.line_scales.iter())
+            .map(|(line, _)| *line)
+            .chain(
+                old.line_insets
+                    .iter()
+                    .chain(presentation.line_insets.iter())
+                    .map(|(line, _)| *line),
+            )
+            .chain(
+                old.blocks
+                    .iter()
+                    .chain(presentation.blocks.iter())
+                    .map(|b| b.lines.start),
+            );
+        for line in lines {
+            if Self::line_wrap(old, line) != Self::line_wrap(&presentation, line) {
+                rows.push(line);
+            }
+        }
+        self.presentation = presentation;
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
+    /// Concealed ranges and replacements the edit touches are dropped and later ones shift
+    /// with it, the way inline metrics do. Per-line items (scales, space, insets, ornaments)
+    /// of the lines the edit spans are dropped and later lines' shift by the change in line
+    /// count, except that an edit within one line keeps that line's (typing in a heading
+    /// leaves it a heading). A block widget whose lines the edit touches is dropped, and later
+    /// ones shift. The owner is expected to set a fresh presentation after the edit; this only
+    /// keeps the stale one safe and close.
+    pub(crate) fn adjust_presentation_for_edit(&mut self, range: &Range<usize>, new_text: &Rope) {
+        if self.presentation.is_empty() {
+            return;
+        }
+        let new_len = new_text.len();
+        let shift = new_len as isize - range.len() as isize;
+        let shift_range = |r: &Range<usize>| -> Option<Range<usize>> {
+            if r.start < range.end && range.start < r.end {
+                return None;
+            }
+            if r.start >= range.end {
+                Some(r.start.checked_add_signed(shift)?..r.end.checked_add_signed(shift)?)
+            } else {
+                Some(r.clone())
+            }
+        };
+        let p = &mut self.presentation;
+        if !p.concealed.is_empty() {
+            p.concealed = p.concealed.iter().filter_map(shift_range).collect();
+        }
+        if !p.replacements.is_empty() {
+            p.replacements = p
+                .replacements
+                .iter()
+                .filter_map(|r| {
+                    Some(Replacement {
+                        range: shift_range(&r.range)?,
+                        ..r.clone()
+                    })
+                })
+                .collect();
+        }
+
+        let len = self.text.len();
+        let start_line = self.text.offset_to_point(range.start.min(len)).row;
+        let end_line = self.text.offset_to_point(range.end.min(len)).row;
+        let new_lines = new_text.lines_len().saturating_sub(1);
+        let delta = new_lines as isize - (end_line - start_line) as isize;
+        let in_place = start_line == end_line && new_lines == 0;
+        let shift_line = |line: usize| -> Option<usize> {
+            if line < start_line || (in_place && line == start_line) {
+                Some(line)
+            } else if line > end_line {
+                line.checked_add_signed(delta)
+            } else {
+                None
+            }
+        };
+        fn shift_lines<T: Clone>(
+            items: &Rc<[(usize, T)]>,
+            shift_line: impl Fn(usize) -> Option<usize>,
+        ) -> Rc<[(usize, T)]> {
+            if items.is_empty() {
+                return items.clone();
+            }
+            items
+                .iter()
+                .filter_map(|(line, value)| Some((shift_line(*line)?, value.clone())))
+                .collect()
+        }
+        p.line_scales = shift_lines(&p.line_scales, shift_line);
+        p.line_insets = shift_lines(&p.line_insets, shift_line);
+        p.line_heights = shift_lines(&p.line_heights, shift_line);
+        p.line_ornaments = shift_lines(&p.line_ornaments, shift_line);
+        if !p.line_space.is_empty() {
+            p.line_space = p
+                .line_space
+                .iter()
+                .filter_map(|&(line, above, below)| Some((shift_line(line)?, above, below)))
+                .collect();
+        }
+        if !p.leaves.is_empty() {
+            p.leaves = p
+                .leaves
+                .iter()
+                .filter_map(|(lines, extra)| {
+                    if lines.start <= end_line && start_line < lines.end {
+                        None
+                    } else if lines.start > end_line {
+                        Some((
+                            lines.start.checked_add_signed(delta)?
+                                ..lines.end.checked_add_signed(delta)?,
+                            *extra,
+                        ))
+                    } else {
+                        Some((lines.clone(), *extra))
+                    }
+                })
+                .collect();
+        }
+        if !p.blocks.is_empty() {
+            p.blocks = p
+                .blocks
+                .iter()
+                .filter_map(|b| {
+                    if b.lines.start <= end_line && start_line < b.lines.end {
+                        None
+                    } else if b.lines.start > end_line {
+                        Some(BlockWidget {
+                            lines: b.lines.start.checked_add_signed(delta)?
+                                ..b.lines.end.checked_add_signed(delta)?,
+                            ..b.clone()
+                        })
+                    } else {
+                        Some(b.clone())
+                    }
+                })
+                .collect();
+        }
+    }
+
+    /// Drop the presentation without rewrapping, for a text that replaces the whole buffer.
+    pub(crate) fn clear_presentation(&mut self) {
+        self.presentation = ValidPresentation::default();
+    }
+
     fn _update<F>(
         &mut self,
         changed_text: &Rope,
@@ -548,7 +897,21 @@ impl TextWrapper {
             let mut indent_chars = 0;
 
             // If wrap_width is Pixels::MAX, skip wrapping to disable word wrap
-            if let Some(wrap_width) = wrap_width {
+            // The first line of a block widget is drawn as one row, so it is not wrapped.
+            let (scale, inset, block) = Self::line_wrap(&self.presentation, row);
+            if let Some(wrap_width) = wrap_width.filter(|_| !block) {
+                // A scaled line is shaped larger, so it wraps narrower, and an inset line
+                // has less room.
+                let wrap_width = if inset > px(0.) {
+                    (wrap_width - inset).max(px(1.))
+                } else {
+                    wrap_width
+                };
+                let wrap_width = if scale != 1. {
+                    wrap_width / scale
+                } else {
+                    wrap_width
+                };
                 // Borrowed for lines within a single rope chunk.
                 let line_str: Cow<str> = line.into();
                 match self.wrapping_indent {
@@ -749,6 +1112,24 @@ pub(crate) struct LineLayout {
     /// Whether any run of this line carries a background color, so [`Self::paint_background`]
     /// can skip the glyph walk for the common case of a line without highlights.
     has_background: bool,
+    /// Height of each visual line as a multiple of the editor's line height, `1.0` unless
+    /// the presentation scales this line.
+    scale: f32,
+    /// What the presentation adds around the line, zero without one.
+    frame: LineFrame,
+}
+
+/// The room a presentation gives one buffer line beyond its rows.
+#[derive(Debug, Clone, Copy, Default, PartialEq)]
+pub(crate) struct LineFrame {
+    /// Space above the first visual row.
+    pub(crate) above: Pixels,
+    /// Space below the last visual row.
+    pub(crate) below: Pixels,
+    /// How far right every visual row starts.
+    pub(crate) inset: Pixels,
+    /// For the first line of a block widget: the height of its one row, which is not painted.
+    pub(crate) block: Option<Pixels>,
 }
 
 impl LineLayout {
@@ -761,6 +1142,41 @@ impl LineLayout {
             whitespace_chars: Vec::new(),
             whitespace_indicators: None,
             has_background: false,
+            scale: 1.,
+            frame: LineFrame::default(),
+        }
+    }
+
+    /// Set the room the presentation gives the line around and before its rows.
+    pub(crate) fn with_frame(mut self, frame: LineFrame) -> Self {
+        self.frame = frame;
+        self
+    }
+
+    /// The room the presentation gives the line.
+    #[inline]
+    pub(crate) fn frame(&self) -> &LineFrame {
+        &self.frame
+    }
+
+    /// The top of visual row `ix`, relative to the top of the line (its space included).
+    #[inline]
+    pub(crate) fn row_top(&self, ix: usize, line_height: Pixels) -> Pixels {
+        self.frame.above + ix * self.row_height(line_height)
+    }
+
+    /// Set the line's scale: each visual line is `line_height * scale` tall.
+    pub(crate) fn with_scale(mut self, scale: f32) -> Self {
+        self.scale = scale;
+        self
+    }
+
+    /// The height of one visual line of this layout.
+    #[inline]
+    pub(crate) fn row_height(&self, line_height: Pixels) -> Pixels {
+        match self.frame.block {
+            Some(height) => height,
+            None => line_height * self.scale,
         }
     }
 
@@ -777,13 +1193,14 @@ impl LineLayout {
     }
 
     /// The pixel indent applied to the given visual line, relative to the line's
-    /// leading text. Only continuation lines (index > 0) are indented.
+    /// leading text. Only continuation lines (index > 0) are indented, past the inset that a
+    /// presentation gives every row.
     #[inline]
-    fn line_indent(&self, line_index: usize) -> Pixels {
+    pub(crate) fn line_indent(&self, line_index: usize) -> Pixels {
         if line_index == 0 {
-            px(0.)
+            self.frame.inset
         } else {
-            self.wrap_indent
+            self.frame.inset + self.wrap_indent
         }
     }
 
@@ -824,7 +1241,7 @@ impl LineLayout {
 
         for (line_index, wrapped_line) in self.wrapped_lines.iter().enumerate() {
             for (relative_offset, c) in wrapped_line.text.char_indices() {
-                if matches!(c, ' ' | '\t') {
+                if matches!(c, ' ' | '\t') && !wrapped_line.is_concealed(relative_offset) {
                     let is_tab = c == '\t';
                     let start_x = wrapped_line.x_for_index(relative_offset);
                     let end_x = wrapped_line.x_for_index(relative_offset + c.len_utf8());
@@ -860,7 +1277,7 @@ impl LineLayout {
         line_end_affinity: bool,
     ) -> Option<Point<Pixels>> {
         let mut acc_len = 0;
-        let mut offset_y = px(0.);
+        let mut offset_y = self.frame.above;
 
         let x_offset = last_layout.alignment_offset(self.longest_width);
 
@@ -888,7 +1305,7 @@ impl LineLayout {
             // Always advance by actual line length. The last line gets +1 so the
             // cursor can be placed after the final character.
             acc_len += if is_last { line.len + 1 } else { line.len };
-            offset_y += last_layout.line_height;
+            offset_y += self.row_height(last_layout.line_height);
         }
 
         None
@@ -925,11 +1342,11 @@ impl LineLayout {
         last_layout: &LastLayout,
     ) -> Option<(usize, usize, Pixels)> {
         let mut offset = 0;
-        let mut line_top = px(0.);
+        let mut line_top = self.frame.above;
         let x_offset = last_layout.alignment_offset(self.longest_width);
 
         for (i, line) in self.wrapped_lines.iter().enumerate() {
-            let line_bottom = line_top + last_layout.line_height;
+            let line_bottom = line_top + self.row_height(last_layout.line_height);
             if pos.y >= line_top && pos.y < line_bottom {
                 return Some((i, offset, pos.x - x_offset - self.line_indent(i)));
             }
@@ -938,7 +1355,18 @@ impl LineLayout {
             line_top = line_bottom;
         }
 
-        None
+        // The space a presentation gives the line belongs to its nearest row.
+        let in_space_above = pos.y >= px(0.) && pos.y < self.frame.above;
+        let in_space_below = pos.y >= line_top && pos.y < line_top + self.frame.below;
+        let last = self.wrapped_lines.len().checked_sub(1)?;
+        if in_space_above {
+            Some((0, 0, pos.x - x_offset - self.line_indent(0)))
+        } else if in_space_below {
+            let offset = offset - self.wrapped_lines[last].len;
+            Some((last, offset, pos.x - x_offset - self.line_indent(last)))
+        } else {
+            None
+        }
     }
 
     /// Get the index for the given position (x, y) in this line layout.
@@ -995,6 +1423,24 @@ impl LineLayout {
         (past_end / last_layout.space_width).round() as usize
     }
 
+    /// The source range, local to this line layout, of the replacement drawn under `pos`.
+    ///
+    /// The `pos` is relative to the top-left corner of this line layout, start from (0, 0).
+    pub(crate) fn replacement_at_position(
+        &self,
+        pos: Point<Pixels>,
+        last_layout: &LastLayout,
+    ) -> Option<Range<usize>> {
+        // Only the rows, not the space around them.
+        let rows_bottom = self.row_top(self.wrapped_lines.len(), last_layout.line_height);
+        if pos.y < self.frame.above || pos.y >= rows_bottom {
+            return None;
+        }
+        let (i, offset, x) = self.wrapped_line_at(pos, last_layout)?;
+        let range = self.wrapped_lines[i].replacement_at_x(x)?;
+        Some(offset + range.start..offset + range.end)
+    }
+
     pub(crate) fn index_for_position(
         &self,
         pos: Point<Pixels>,
@@ -1013,7 +1459,12 @@ impl LineLayout {
             .map(|(ix, line)| line.width + self.line_indent(ix))
             .max()
             .unwrap_or(self.longest_width);
-        size(width, self.wrapped_lines.len() * line_height)
+        size(
+            width,
+            self.frame.above
+                + self.wrapped_lines.len() * self.row_height(line_height)
+                + self.frame.below,
+        )
     }
 
     /// Paint only the glyph background quads of this line.
@@ -1031,10 +1482,12 @@ impl LineLayout {
     ) {
         // Painting a background walks every glyph and pushes a scene layer, so skip the
         // whole pass for lines that have no background color to paint.
-        if !self.has_background {
+        if !self.has_background || self.frame.block.is_some() {
             return;
         }
 
+        let line_height = self.row_height(line_height);
+        let pos = pos + point(px(0.), self.frame.above);
         for (ix, line) in self.wrapped_lines.iter().enumerate() {
             _ = line.paint_background(
                 pos + point(self.line_indent(ix), ix * line_height),
@@ -1056,6 +1509,12 @@ impl LineLayout {
         window: &mut Window,
         cx: &mut App,
     ) {
+        // A block widget's element is drawn in place of its first line.
+        if self.frame.block.is_some() {
+            return;
+        }
+        let line_height = self.row_height(line_height);
+        let pos = pos + point(px(0.), self.frame.above);
         for (ix, line) in self.wrapped_lines.iter().enumerate() {
             _ = line.paint(
                 pos + point(self.line_indent(ix), ix * line_height),
diff --git a/src/input/editor/display_map/wrap_map.rs b/src/input/editor/display_map/wrap_map.rs
index 0cbb3d04..29d2e7fd 100644
--- a/src/input/editor/display_map/wrap_map.rs
+++ b/src/input/editor/display_map/wrap_map.rs
@@ -108,9 +108,19 @@ impl WrapMap {
         cx: &mut App,
     ) {
         self.wrapper.adjust_inline_metrics(range, new_text.len());
+        self.wrapper.adjust_presentation_for_edit(range, new_text);
         self.wrapper.update(changed_text, range, new_text, cx);
     }
 
+    /// Replace the render-only presentation. Returns whether anything changed.
+    pub(super) fn set_presentation(
+        &mut self,
+        presentation: crate::input::presentation::ValidPresentation,
+        cx: &mut App,
+    ) -> bool {
+        self.wrapper.set_presentation(presentation, cx)
+    }
+
     /// Update layout parameters (wrap width or font)
     pub(super) fn set_inline_metrics(
         &mut self,
@@ -141,6 +151,7 @@ impl WrapMap {
 
     /// Initialize with text
     pub(super) fn set_text(&mut self, text: &Rope, cx: &mut App) {
+        self.wrapper.clear_presentation();
         self.wrapper.set_default_text(text);
         self.wrapper.prepare_if_need(text, cx);
     }
diff --git a/src/input/editor/indent.rs b/src/input/editor/indent.rs
index d50bff1c..ea3c45f3 100644
--- a/src/input/editor/indent.rs
+++ b/src/input/editor/indent.rs
@@ -128,6 +128,9 @@ impl<M: InputModeKind> TextElement<M> {
             .zip(last_layout.lines.iter())
         {
             let line = state.text.slice_line(buffer_line);
+            let line_height = line_layout.row_height(line_height);
+            // The guides run beside the rows, not through the space around them.
+            let rows_top = offset_y + line_layout.frame().above;
             let mut current_indents = vec![];
             if line.len() > 0 {
                 let indent_count = tab_size.indent_count(&line);
@@ -138,7 +141,7 @@ impl<M: InputModeKind> TextElement<M> {
                         px(0.)
                     };
 
-                    let pos = point(x + last_layout.line_number_width, offset_y);
+                    let pos = point(x + last_layout.line_number_width, rows_top);
 
                     builder.move_to(pos);
                     builder.line_to(point(pos.x, pos.y + line_height));
@@ -146,14 +149,14 @@ impl<M: InputModeKind> TextElement<M> {
                 }
             } else if last_indents.len() > 0 {
                 for x in &last_indents {
-                    let pos = point(*x, offset_y);
+                    let pos = point(*x, rows_top);
                     builder.move_to(pos);
                     builder.line_to(point(pos.x, pos.y + line_height));
                 }
                 current_indents = last_indents.clone();
             }
 
-            offset_y += line_layout.wrapped_lines.len() * line_height;
+            offset_y += line_layout.size(last_layout.line_height).height;
             last_indents = current_indents;
         }
 
diff --git a/src/input/editor/presentation.rs b/src/input/editor/presentation.rs
new file mode 100644
index 00000000..866ce58c
--- /dev/null
+++ b/src/input/editor/presentation.rs
@@ -0,0 +1,950 @@
+//! A render-only presentation of the code editor's text: ranges laid out at zero
+//! width or drawn as other text, lines drawn larger, with space around them, inset
+//! or ornamented, and runs of lines drawn as an element of the app's.
+//!
+//! Nothing here changes the text, the selections, or editing. Every offset keeps
+//! addressing the source, so a caret inside a concealed range sits at the point
+//! the range was hidden at, and a selection over one still selects its bytes.
+use std::fmt;
+use std::ops::Range;
+use std::rc::Rc;
+
+use gpui::{
+    AnyElement, App, Context, EventEmitter, HighlightStyle, Hsla, Pixels, SharedString, Window, px,
+};
+use ropey::Rope;
+
+use super::{EditorState, InputBaseState, InputModeKind, RopeExt as _};
+
+/// A render-only presentation for an [`EditorState`], set with
+/// [`EditorState::set_presentation`].
+///
+/// Input is checked against the current text when it is set: a concealed range
+/// that is empty, out of bounds, off a char boundary, crosses a newline, or
+/// overlaps an earlier one is dropped, and so is a scale that is not finite,
+/// not above zero, `1.0`, or for a line the text does not have. The other fields are
+/// checked the same way (see each), and nothing in them can make it panic.
+#[derive(Debug, Clone, Default, PartialEq)]
+pub struct EditorPresentation {
+    /// UTF-8 byte ranges laid out at zero width and not painted. Each lies
+    /// within one line (it may end at the line's end, not past its newline).
+    pub concealed: Vec<Range<usize>>,
+    /// `(buffer line, scale)`: the line's text is shaped `scale` times larger
+    /// (or smaller, below `1.0`), and each of its visual rows is `scale` line
+    /// heights tall.
+    pub line_scales: Vec<(usize, f32)>,
+    /// Source ranges drawn as other text. Each is checked like a concealed
+    /// range, must not overlap a concealed range or an earlier replacement, and
+    /// its text must be one line.
+    pub replacements: Vec<Replacement>,
+    /// `(buffer line, above, below)`: empty space added above the line's first
+    /// visual row and below its last. Both must be finite and not negative.
+    pub line_space: Vec<(usize, Pixels, Pixels)>,
+    /// `(buffer line, inset)`: every visual row of the line starts `inset`
+    /// further right, and the line wraps that much narrower. Must be finite
+    /// and above zero.
+    pub line_insets: Vec<(usize, Pixels)>,
+    /// `(buffer line, ornament)`: drawn with the line, beneath its text. They
+    /// change no layout. A line may have several.
+    pub line_ornaments: Vec<(usize, Ornament)>,
+    /// Runs of whole lines drawn as an element instead of their text.
+    pub block_widgets: Vec<BlockWidget>,
+    /// `(buffer line, factor)`: each visual row of the line is `factor` times as
+    /// tall as its scale alone makes it (the editor's line height times the
+    /// line's scale), its text centred in it as before. Must be finite, above
+    /// zero and not `1.0`.
+    pub line_heights: Vec<(usize, f32)>,
+    /// The height of a row at scale `1.0`, instead of the window's line height
+    /// (which is rounded to whole pixels), so rows can follow another
+    /// renderer's exactly. Must be finite and above zero.
+    pub line_height: Option<Pixels>,
+    /// `(buffer lines, extra)`: runs of lines laid out as one box, as a laid-out
+    /// page's text box is: its rows plus `extra` are rounded up to whole device
+    /// pixels, the difference added below its last row. Only with a
+    /// `line_height`; the lines must be in the text, must not overlap an earlier
+    /// run or a block widget, and `extra` must be finite and not negative.
+    pub leaves: Vec<(Range<usize>, Pixels)>,
+}
+
+/// A source range drawn as `text` instead, see [`EditorPresentation::replacements`].
+///
+/// The text is shaped at its line's font size (times the line's scale) in the
+/// editor's text style refined by `style`, and takes the room of the source.
+/// It is atomic for the caret: a caret before it is at its start, after it at
+/// its end, and one inside the source is drawn at its start. A click on its
+/// left half lands at the start and on its right half at the end, unless it is
+/// `clickable`: a click on a clickable replacement leaves the caret where it is
+/// and is reported as [`EditorPresentationEvent::ReplacementClicked`].
+#[derive(Debug, Clone, PartialEq)]
+pub struct Replacement {
+    /// UTF-8 byte range of the source, within one line.
+    pub range: Range<usize>,
+    /// What is drawn in its place.
+    pub text: SharedString,
+    /// Applied over the editor's text style for `text`.
+    pub style: HighlightStyle,
+    /// Whether a click on it is reported instead of placing the caret.
+    pub clickable: bool,
+    /// The font family `text` is shaped in, instead of the editor's.
+    pub family: Option<SharedString>,
+    /// `text`'s size relative to its line's. Must be finite and above zero;
+    /// anything else is taken as `1.0`.
+    pub size: f32,
+    /// A column `text` is set in, right-aligned, instead of its own width.
+    pub column: Option<ReplacementColumn>,
+    /// A box drawn behind `text`, with room either side of it.
+    pub chip: Option<ReplacementChip>,
+    /// An image drawn in place of `text` (which is then not drawn).
+    pub image: Option<ReplacementImage>,
+}
+
+/// An image a [`Replacement`] is drawn as, like a typeset formula: `size`,
+/// with its baseline `baseline` from its top on the row's baseline.
+#[derive(Debug, Clone, PartialEq)]
+pub struct ReplacementImage {
+    pub image: std::sync::Arc<gpui::Image>,
+    pub size: gpui::Size<Pixels>,
+    pub baseline: Pixels,
+}
+
+impl Replacement {
+    /// `range` drawn as `text`, in the editor's style, not clickable.
+    pub fn new(range: Range<usize>, text: impl Into<SharedString>) -> Self {
+        Self {
+            range,
+            text: text.into(),
+            style: HighlightStyle::default(),
+            clickable: false,
+            family: None,
+            size: 1.,
+            column: None,
+            chip: None,
+            image: None,
+        }
+    }
+
+    /// The size its text is shaped at, for a line shaped at `font_size`.
+    pub(crate) fn font_size(&self, font_size: Pixels) -> Pixels {
+        if self.size.is_finite() && self.size > 0. {
+            font_size * self.size
+        } else {
+            font_size
+        }
+    }
+
+    /// How it is placed for text `text_width` wide: how wide it is laid out,
+    /// where its text starts (from where it is laid out), and how far its
+    /// column hangs to the left of where it is laid out.
+    pub(crate) fn placement(&self, text_width: Pixels) -> (Pixels, Pixels, Pixels) {
+        if let Some(image) = &self.image {
+            return (image.size.width, px(0.), px(0.));
+        }
+        if let Some(chip) = &self.chip {
+            let text_width = if chip.whole_width {
+                text_width.ceil()
+            } else {
+                text_width
+            };
+            let padding = chip.padding.max(px(0.));
+            let icon = chip
+                .icon
+                .as_ref()
+                .map_or(px(0.), |icon| icon.size + icon.gap);
+            return (text_width + icon + padding * 2., padding + icon, px(0.));
+        }
+        match self.column {
+            Some(column) => {
+                let gap = column.gap.max(px(0.));
+                let width = column.width.max(text_width + gap);
+                if column.hanging {
+                    (px(0.), -(gap + text_width), width)
+                } else {
+                    (width, width - gap - text_width, px(0.))
+                }
+            }
+            None => (text_width, px(0.), px(0.)),
+        }
+    }
+}
+
+/// A box behind a [`Replacement`]'s text, like a code span's or a tag's: the
+/// replacement is `padding` wider either side, and the box spans it.
+#[derive(Debug, Clone, PartialEq)]
+pub struct ReplacementChip {
+    pub padding: Pixels,
+    pub background: Hsla,
+    pub radius: Pixels,
+    pub height: ChipHeight,
+    /// An icon before the text, in the text's colour.
+    pub icon: Option<ChipIcon>,
+    /// A dashed line this colour along the bottom of the box, 1px thick.
+    pub dashed_bottom: Option<Hsla>,
+    /// Whether it stands for a laid-out element: its text's room is its width
+    /// rounded up to a whole pixel, and the box sits on whole device pixels.
+    pub whole_width: bool,
+    /// Where its text's baseline is, below the box's top; by default the
+    /// text keeps the row's baseline.
+    pub text_baseline: Option<Pixels>,
+}
+
+/// An icon in a [`ReplacementChip`]: the SVG asset at `path`, `size` square,
+/// centred on the row, `gap` before the text.
+#[derive(Debug, Clone, PartialEq)]
+pub struct ChipIcon {
+    pub path: SharedString,
+    pub size: Pixels,
+    pub gap: Pixels,
+}
+
+/// How tall a [`ReplacementChip`] is.
+#[derive(Debug, Clone, Copy, PartialEq)]
+pub enum ChipHeight {
+    /// Around its text's capitals: their height and `size * 0.125 +
+    /// descent / 2` either side, as a code span's. Its text keeps the row's
+    /// baseline.
+    CapHeight,
+    /// This tall, centred on the row, as is its text.
+    Fixed(Pixels),
+    /// From `ascent` above the row's baseline to `descent` below it, like an
+    /// inline element set on the line's baseline.
+    Baseline { ascent: Pixels, descent: Pixels },
+}
+
+/// A column a [`Replacement`]'s text is set in, like a list's marker column:
+/// at least `width` wide, its text right-aligned `gap` short of its right edge.
+/// A wider text widens the column.
+///
+/// A `hanging` column takes no room: it hangs to the left of where the
+/// replacement is (into the line's inset, like an outside list marker), so
+/// the text after it starts where the line's wrapped rows do. A click on it
+/// is a click on the replacement.
+#[derive(Debug, Clone, Copy, PartialEq)]
+pub struct ReplacementColumn {
+    pub width: Pixels,
+    pub gap: Pixels,
+    pub hanging: bool,
+}
+
+/// Something drawn with a line, see [`EditorPresentation::line_ornaments`].
+#[derive(Debug, Clone, Copy, PartialEq)]
+pub enum Ornament {
+    /// A vertical bar the full height of the line, its space included, `x`
+    /// from the left edge of the text area (whatever the line's inset).
+    Bar {
+        x: Pixels,
+        width: Pixels,
+        color: Hsla,
+    },
+    /// A horizontal line across the text area, centred on the line's rows.
+    Rule { thickness: Pixels, color: Hsla },
+    /// A task's box: a square `size` wide with corners of `radius`, `x` from
+    /// the left edge of the text area, centred on the line's first row. Its
+    /// border is `border` wide in `border_color`; a `fill` fills it, and a
+    /// `check` draws a check mark in it (a stroke `size * 0.1` wide).
+    Check {
+        x: Pixels,
+        size: Pixels,
+        radius: Pixels,
+        border: Pixels,
+        border_color: Hsla,
+        fill: Option<Hsla>,
+        check: Option<Hsla>,
+        /// The height of the box from the row's top it is centred in, when not
+        /// the row's.
+        row: Option<Pixels>,
+    },
+}
+
+/// Builds the element of a [`BlockWidget`].
+pub type BlockWidgetRenderer = Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>;
+
+/// Whole buffer lines drawn as an element, see
+/// [`EditorPresentation::block_widgets`].
+///
+/// The lines' text is not drawn: the first line becomes one row as tall as the
+/// element, and the others are hidden the way folded lines are, so vertical
+/// moves step over them in one row. The element is laid out at the width of
+/// the text area; its measured height replaces `height` from the next frame
+/// on, remembered by `key` until a widget with that key is no longer set.
+///
+/// Mouse events inside the element reach it. A press it leaves alone does not
+/// place the caret, and a click it leaves alone is reported as
+/// [`EditorPresentationEvent::BlockClicked`]; an element that handles a click
+/// itself should stop its propagation. Selections and edits address the
+/// source as usual: an edit touching the lines drops the widget.
+///
+/// `lines` must be in the text and must not overlap an earlier widget, and
+/// `height` must be finite and above zero.
+#[derive(Clone)]
+pub struct BlockWidget {
+    /// The buffer lines, `start..end`.
+    pub lines: Range<usize>,
+    /// The height to lay out before the element is measured.
+    pub height: Pixels,
+    /// Identifies the element's content, so its measured height is kept.
+    pub key: u64,
+    /// Builds the element, each frame it is visible.
+    pub render: BlockWidgetRenderer,
+}
+
+impl fmt::Debug for BlockWidget {
+    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
+        f.debug_struct("BlockWidget")
+            .field("lines", &self.lines)
+            .field("height", &self.height)
+            .field("key", &self.key)
+            .finish_non_exhaustive()
+    }
+}
+
+/// Widgets are equal when their lines, height and key are: the key stands for
+/// what the renderer draws.
+impl PartialEq for BlockWidget {
+    fn eq(&self, other: &Self) -> bool {
+        self.lines == other.lines && self.height == other.height && self.key == other.key
+    }
+}
+
+/// What a click on a presentation reports, emitted by the editor's state.
+#[derive(Debug, Clone, PartialEq)]
+pub enum EditorPresentationEvent {
+    /// A [`Replacement`] with `clickable` set was clicked; the caret did not move.
+    ReplacementClicked { range: Range<usize> },
+    /// A [`BlockWidget`] was clicked where its element did not handle it.
+    BlockClicked { lines: Range<usize> },
+}
+
+impl<M: InputModeKind> EventEmitter<EditorPresentationEvent> for InputBaseState<M> {}
+
+/// A presentation checked against the text: every list sorted (by start or by
+/// line) and free of the items validation drops.
+#[derive(Clone, Default)]
+pub(crate) struct ValidPresentation {
+    pub(crate) concealed: Rc<[Range<usize>]>,
+    pub(crate) line_scales: Rc<[(usize, f32)]>,
+    pub(crate) replacements: Rc<[Replacement]>,
+    pub(crate) line_space: Rc<[(usize, Pixels, Pixels)]>,
+    pub(crate) line_insets: Rc<[(usize, Pixels)]>,
+    pub(crate) line_ornaments: Rc<[(usize, Ornament)]>,
+    pub(crate) blocks: Rc<[BlockWidget]>,
+    pub(crate) line_heights: Rc<[(usize, f32)]>,
+    pub(crate) line_height: Option<Pixels>,
+    pub(crate) leaves: Rc<[(Range<usize>, Pixels)]>,
+}
+
+impl PartialEq for ValidPresentation {
+    fn eq(&self, other: &Self) -> bool {
+        self.concealed == other.concealed
+            && self.line_scales == other.line_scales
+            && self.replacements == other.replacements
+            && self.line_space == other.line_space
+            && self.line_insets == other.line_insets
+            && self.line_ornaments == other.line_ornaments
+            && self.blocks == other.blocks
+            && self.line_heights == other.line_heights
+            && self.line_height == other.line_height
+            && self.leaves == other.leaves
+    }
+}
+
+impl ValidPresentation {
+    pub(crate) fn is_empty(&self) -> bool {
+        self.concealed.is_empty()
+            && self.line_scales.is_empty()
+            && self.replacements.is_empty()
+            && self.line_space.is_empty()
+            && self.line_insets.is_empty()
+            && self.line_ornaments.is_empty()
+            && self.blocks.is_empty()
+            && self.line_heights.is_empty()
+            && self.line_height.is_none()
+            && self.leaves.is_empty()
+    }
+
+    fn to_presentation(&self) -> EditorPresentation {
+        EditorPresentation {
+            concealed: self.concealed.to_vec(),
+            line_scales: self.line_scales.to_vec(),
+            replacements: self.replacements.to_vec(),
+            line_space: self.line_space.to_vec(),
+            line_insets: self.line_insets.to_vec(),
+            line_ornaments: self.line_ornaments.to_vec(),
+            block_widgets: self.blocks.to_vec(),
+            line_heights: self.line_heights.to_vec(),
+            line_height: self.line_height,
+            leaves: self.leaves.to_vec(),
+        }
+    }
+}
+
+/// Look up the value of `line` in a list sorted by line with one entry per line.
+pub(crate) fn line_value<T: Copy>(items: &[(usize, T)], line: usize) -> Option<T> {
+    items
+        .binary_search_by_key(&line, |(l, _)| *l)
+        .ok()
+        .map(|ix| items[ix].1)
+}
+
+/// Whether `range` lies within one line of `text`, on char boundaries, and is not empty.
+fn is_line_range(text: &Rope, range: &Range<usize>) -> bool {
+    range.start < range.end
+        && range.end <= text.len()
+        && text.is_char_boundary(range.start)
+        && text.is_char_boundary(range.end)
+        && text.offset_to_point(range.start).row == text.offset_to_point(range.end).row
+}
+
+/// Sort `items` by line and keep the first of each line.
+fn first_per_line<T>(mut items: Vec<(usize, T)>) -> Vec<(usize, T)> {
+    items.sort_by_key(|(line, _)| *line);
+    items.dedup_by_key(|(line, _)| *line);
+    items
+}
+
+fn finite_px(value: Pixels) -> bool {
+    f32::from(value).is_finite()
+}
+
+impl EditorPresentation {
+    /// Whether the presentation changes nothing.
+    pub fn is_empty(&self) -> bool {
+        self.concealed.is_empty()
+            && self.line_scales.is_empty()
+            && self.replacements.is_empty()
+            && self.line_space.is_empty()
+            && self.line_insets.is_empty()
+            && self.line_ornaments.is_empty()
+            && self.block_widgets.is_empty()
+            && self.line_heights.is_empty()
+            && self.line_height.is_none()
+            && self.leaves.is_empty()
+    }
+
+    /// The valid part of this presentation for `text`: sorted, disjoint ranges,
+    /// per-line items sorted by line (the first of duplicates wins, except
+    /// ornaments, which keep every one in order), and disjoint widgets.
+    pub(crate) fn validate(&self, text: &Rope) -> ValidPresentation {
+        let mut concealed: Vec<Range<usize>> = self
+            .concealed
+            .iter()
+            .filter(|range| is_line_range(text, range))
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
+        let mut replacements: Vec<Replacement> = self
+            .replacements
+            .iter()
+            .filter(|r| is_line_range(text, &r.range) && !r.text.contains(['\n', '\r']))
+            .cloned()
+            .collect();
+        replacements.sort_by_key(|r| r.range.start);
+        let mut end = 0;
+        replacements.retain(|r| {
+            let overlaps_concealed = {
+                let ix = concealed.partition_point(|c| c.end <= r.range.start);
+                concealed.get(ix).is_some_and(|c| c.start < r.range.end)
+            };
+            let keep = r.range.start >= end && !overlaps_concealed;
+            if keep {
+                end = r.range.end;
+            }
+            keep
+        });
+
+        let lines = text.lines_len();
+        let lines_count = lines;
+        let line_scales = first_per_line(
+            self.line_scales
+                .iter()
+                .copied()
+                .filter(|&(line, scale)| {
+                    line < lines && scale.is_finite() && scale > 0. && scale != 1.
+                })
+                .collect(),
+        );
+        let line_space = {
+            let mut items: Vec<(usize, Pixels, Pixels)> = self
+                .line_space
+                .iter()
+                .copied()
+                .filter(|&(line, above, below)| {
+                    line < lines
+                        && finite_px(above)
+                        && finite_px(below)
+                        && above >= px(0.)
+                        && below >= px(0.)
+                        && (above > px(0.) || below > px(0.))
+                })
+                .collect();
+            items.sort_by_key(|(line, _, _)| *line);
+            items.dedup_by_key(|(line, _, _)| *line);
+            items
+        };
+        let line_insets = first_per_line(
+            self.line_insets
+                .iter()
+                .copied()
+                .filter(|&(line, inset)| line < lines && finite_px(inset) && inset > px(0.))
+                .collect(),
+        );
+        let mut line_ornaments: Vec<(usize, Ornament)> = self
+            .line_ornaments
+            .iter()
+            .copied()
+            .filter(|(line, ornament)| {
+                *line < lines
+                    && match *ornament {
+                        Ornament::Bar { x, width, .. } => {
+                            finite_px(x) && finite_px(width) && width > px(0.)
+                        }
+                        Ornament::Rule { thickness, .. } => {
+                            finite_px(thickness) && thickness > px(0.)
+                        }
+                        Ornament::Check {
+                            x,
+                            size,
+                            radius,
+                            border,
+                            ..
+                        } => {
+                            finite_px(x)
+                                && finite_px(size)
+                                && size > px(0.)
+                                && finite_px(radius)
+                                && finite_px(border)
+                        }
+                    }
+            })
+            .collect();
+        line_ornaments.sort_by_key(|(line, _)| *line);
+
+        let mut blocks: Vec<BlockWidget> = self
+            .block_widgets
+            .iter()
+            .filter(|b| {
+                b.lines.start < b.lines.end
+                    && b.lines.end <= lines
+                    && finite_px(b.height)
+                    && b.height > px(0.)
+            })
+            .cloned()
+            .collect();
+        blocks.sort_by_key(|b| b.lines.start);
+        let mut end = 0;
+        blocks.retain(|b| {
+            let keep = b.lines.start >= end;
+            if keep {
+                end = b.lines.end;
+            }
+            keep
+        });
+
+        let line_heights = first_per_line(
+            self.line_heights
+                .iter()
+                .copied()
+                .filter(|&(line, factor)| {
+                    line < lines && factor.is_finite() && factor > 0. && factor != 1.
+                })
+                .collect(),
+        );
+
+        let mut leaves: Vec<(Range<usize>, Pixels)> = self
+            .leaves
+            .iter()
+            .filter(|(lines, extra)| {
+                lines.start < lines.end
+                    && lines.end <= lines_count
+                    && finite_px(*extra)
+                    && *extra >= px(0.)
+                    && !blocks
+                        .iter()
+                        .any(|b| b.lines.start < lines.end && lines.start < b.lines.end)
+            })
+            .cloned()
+            .collect();
+        leaves.sort_by_key(|(lines, _)| lines.start);
+        let mut end = 0;
+        leaves.retain(|(lines, _)| {
+            let keep = lines.start >= end;
+            if keep {
+                end = lines.end;
+            }
+            keep
+        });
+
+        ValidPresentation {
+            leaves: leaves.into(),
+            line_height: self
+                .line_height
+                .filter(|&height| finite_px(height) && height > px(0.)),
+            line_heights: line_heights.into(),
+            concealed: concealed.into(),
+            line_scales: line_scales.into(),
+            replacements: replacements.into(),
+            line_space: line_space.into(),
+            line_insets: line_insets.into(),
+            line_ornaments: line_ornaments.into(),
+            blocks: blocks.into(),
+        }
+    }
+}
+
+impl EditorState {
+    /// Set, or with `None` clear, the render-only presentation of the text.
+    ///
+    /// Off by default; without one the editor lays out exactly as before. The
+    /// presentation is validated against the current text (see
+    /// [`EditorPresentation`]). Edits carry it along: a concealed range or
+    /// replacement an edit touches is dropped, and later ones shift; the
+    /// per-line items of the lines an edit spans are dropped (an edit within
+    /// one line keeps that line's), and later lines' shift with the line count;
+    /// a block widget whose lines an edit touches is dropped. Replacing the
+    /// whole text clears it. It is meant to be set again after every change, so
+    /// this only keeps a stale presentation safe until then.
+    ///
+    /// Concealing and replacing are ignored while inline tokens are shown.
+    pub fn set_presentation(
+        &mut self,
+        presentation: Option<EditorPresentation>,
+        cx: &mut Context<Self>,
+    ) {
+        let presentation = presentation
+            .map(|presentation| presentation.validate(&self.text))
+            .unwrap_or_default();
+        // A widget's renderer may change under the same key, so a frame is due
+        // whenever widgets are set.
+        let has_blocks = !presentation.blocks.is_empty();
+        if self.display_map.set_presentation(presentation, cx) || has_blocks {
+            cx.notify();
+        }
+    }
+
+    /// The presentation in effect: what [`Self::set_presentation`] kept,
+    /// carried through any edits since.
+    pub fn editor_presentation(&self) -> EditorPresentation {
+        self.display_map.presentation().to_presentation()
+    }
+}
+
+impl<M: InputModeKind> InputBaseState<M> {
+    /// The height of a row at scale `1.0`: the presentation's, or the window's line
+    /// height.
+    pub(crate) fn presented_line_height(&self, window: &Window) -> Pixels {
+        self.display_map
+            .presentation()
+            .line_height
+            .unwrap_or_else(|| window.line_height())
+    }
+
+    /// The source range of the clickable replacement drawn under window `position`, if any.
+    pub(crate) fn clickable_replacement_at(
+        &self,
+        position: gpui::Point<Pixels>,
+    ) -> Option<Range<usize>> {
+        let replacements = &self.display_map.presentation().replacements;
+        if self.masked || !replacements.iter().any(|r| r.clickable) {
+            return None;
+        }
+        let (bounds, last_layout) = (self.last_bounds.as_ref()?, self.last_layout.as_ref()?);
+        let inner = position - bounds.origin - gpui::point(last_layout.line_number_width, px(0.));
+        let mut y = last_layout.visible_top;
+        for (vi, line) in last_layout.lines.iter().enumerate() {
+            let height = line.size(last_layout.line_height).height;
+            if inner.y >= y && inner.y < y + height {
+                let local =
+                    line.replacement_at_position(inner - gpui::point(px(0.), y), last_layout)?;
+                let start = last_layout.visible_line_byte_offsets.get(vi)?;
+                let range = start + local.start..start + local.end;
+                return replacements
+                    .iter()
+                    .find(|r| r.clickable && r.range == range)
+                    .map(|r| r.range.clone());
+            }
+            y += height;
+        }
+        None
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use gpui::{IntoElement as _, hsla};
+
+    fn valid(presentation: EditorPresentation, text: &str) -> EditorPresentation {
+        presentation.validate(&Rope::from(text)).to_presentation()
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
+            ..Default::default()
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
+            ..Default::default()
+        };
+        assert_eq!(
+            valid(presentation, "a\nb\nc").line_scales,
+            vec![(0, 1.6), (1, 0.5), (2, 1.2)]
+        );
+    }
+
+    fn replacement(range: Range<usize>, text: &str) -> Replacement {
+        Replacement::new(range, SharedString::from(text.to_string()))
+    }
+
+    #[test]
+    fn invalid_replacements_are_dropped() {
+        let text = "- [ ] é task\n- b";
+        let presentation = EditorPresentation {
+            concealed: vec![6..8],
+            replacements: vec![
+                replacement(14..16, "•"),
+                replacement(0..2, "•"),
+                replacement(1..3, "x"), // overlaps 0..2
+                replacement(2..5, "☐"),
+                replacement(6..7, "x"),     // inside a concealed range
+                replacement(7..8, "x"),     // ends inside `é` and is concealed
+                replacement(9..10, "a\nb"), // two lines
+                replacement(11..14, "x"),   // crosses the newline
+                replacement(30..31, "x"),   // out of bounds
+                replacement(10..10, "x"),   // empty
+            ],
+            ..Default::default()
+        };
+        let valid = valid(presentation, text);
+        assert_eq!(
+            valid
+                .replacements
+                .iter()
+                .map(|r| r.range.clone())
+                .collect::<Vec<_>>(),
+            vec![0..2, 2..5, 14..16]
+        );
+    }
+
+    #[test]
+    fn replacements_are_placed_in_columns_chips_and_images() {
+        let mut r = Replacement::new(0..2, "1.");
+        assert_eq!(r.placement(px(10.)), (px(10.), px(0.), px(0.)));
+        r.column = Some(ReplacementColumn {
+            width: px(22.),
+            gap: px(6.),
+            hanging: false,
+        });
+        // Right-aligned in the column, `gap` short of its edge.
+        assert_eq!(r.placement(px(10.)), (px(22.), px(6.), px(0.)));
+        // A wider text widens the column.
+        assert_eq!(r.placement(px(20.)), (px(26.), px(0.), px(0.)));
+        r.column = Some(ReplacementColumn {
+            width: px(22.),
+            gap: px(6.),
+            hanging: true,
+        });
+        // Hanging: no room, the text left of where it is, the column to hit.
+        assert_eq!(r.placement(px(10.)), (px(0.), px(-16.), px(22.)));
+        let chip = Replacement {
+            chip: Some(ReplacementChip {
+                padding: px(4.),
+                background: hsla(0., 0., 0., 1.),
+                radius: px(2.),
+                height: ChipHeight::CapHeight,
+                icon: Some(ChipIcon {
+                    path: "icons/calendar.svg".into(),
+                    size: px(12.),
+                    gap: px(4.),
+                }),
+                dashed_bottom: None,
+                whole_width: false,
+                text_baseline: None,
+            }),
+            ..Replacement::new(0..2, "x")
+        };
+        assert_eq!(
+            chip.placement(px(10.)),
+            (px(10. + 16. + 8.), px(20.), px(0.))
+        );
+        let image = Replacement {
+            image: Some(ReplacementImage {
+                image: std::sync::Arc::new(gpui::Image::from_bytes(gpui::ImageFormat::Png, vec![])),
+                size: gpui::size(px(30.), px(14.)),
+                baseline: px(11.),
+            }),
+            ..Replacement::new(0..2, "")
+        };
+        assert_eq!(image.placement(px(0.)), (px(30.), px(0.), px(0.)));
+        // A size that is not one is taken as the line's.
+        let sized = Replacement {
+            size: f32::NAN,
+            ..Replacement::new(0..1, "x")
+        };
+        assert_eq!(sized.font_size(px(16.)), px(16.));
+    }
+
+    #[test]
+    fn invalid_line_heights_and_line_height_are_dropped() {
+        let presentation = EditorPresentation {
+            line_heights: vec![(1, 0.5), (0, 1.), (0, 0.), (2, f32::NAN), (9, 2.)],
+            line_height: Some(px(-1.)),
+            ..Default::default()
+        };
+        let valid = valid(presentation, "a\nb\nc");
+        assert_eq!(valid.line_heights, vec![(1, 0.5)]);
+        assert_eq!(valid.line_height, None);
+        let kept = EditorPresentation {
+            line_height: Some(px(26.4)),
+            ..Default::default()
+        };
+        assert_eq!(valid_height(kept), Some(px(26.4)));
+    }
+
+    fn valid_height(presentation: EditorPresentation) -> Option<Pixels> {
+        valid(presentation, "a").line_height
+    }
+
+    #[test]
+    fn invalid_line_items_and_widgets_are_dropped() {
+        let color = hsla(0., 0., 0., 1.);
+        let render: BlockWidgetRenderer = Rc::new(|_, _| gpui::Empty.into_any_element());
+        let block = |lines: Range<usize>, height: f32| BlockWidget {
+            lines,
+            height: px(height),
+            key: 1,
+            render: render.clone(),
+        };
+        let presentation = EditorPresentation {
+            line_space: vec![
+                (2, px(4.), px(0.)),
+                (0, px(1.), px(2.)),
+                (0, px(9.), px(9.)),
+                (1, px(0.), px(0.)),
+                (1, px(-1.), px(3.)),
+                (1, px(f32::NAN), px(3.)),
+                (7, px(1.), px(1.)),
+            ],
+            line_insets: vec![
+                (1, px(12.)),
+                (0, px(0.)),
+                (2, px(f32::INFINITY)),
+                (5, px(3.)),
+            ],
+            line_ornaments: vec![
+                (
+                    1,
+                    Ornament::Rule {
+                        thickness: px(1.),
+                        color,
+                    },
+                ),
+                (
+                    0,
+                    Ornament::Bar {
+                        x: px(0.),
+                        width: px(3.),
+                        color,
+                    },
+                ),
+                (
+                    0,
+                    Ornament::Bar {
+                        x: px(8.),
+                        width: px(3.),
+                        color,
+                    },
+                ),
+                (
+                    0,
+                    Ornament::Bar {
+                        x: px(8.),
+                        width: px(0.),
+                        color,
+                    },
+                ),
+                (
+                    2,
+                    Ornament::Rule {
+                        thickness: px(f32::NAN),
+                        color,
+                    },
+                ),
+                (
+                    4,
+                    Ornament::Rule {
+                        thickness: px(1.),
+                        color,
+                    },
+                ),
+            ],
+            block_widgets: vec![
+                block(1..3, 40.),
+                block(0..2, 40.), // sorts first, so 1..3 overlaps it
+                block(2..2, 40.),
+                block(2..4, 40.),
+                block(0..1, f32::NAN),
+            ],
+            ..Default::default()
+        };
+        let valid = valid(presentation, "a\nb\nc");
+        assert_eq!(
+            valid.line_space,
+            vec![(0, px(1.), px(2.)), (2, px(4.), px(0.))]
+        );
+        assert_eq!(valid.line_insets, vec![(1, px(12.))]);
+        assert_eq!(
+            valid
+                .line_ornaments
+                .iter()
+                .map(|(line, _)| *line)
+                .collect::<Vec<_>>(),
+            vec![0, 0, 1]
+        );
+        assert_eq!(
+            valid
+                .block_widgets
+                .iter()
+                .map(|b| b.lines.clone())
+                .collect::<Vec<_>>(),
+            vec![0..2]
+        );
+    }
+}
diff --git a/src/input/mod.rs b/src/input/mod.rs
index 13f64102..35339bb2 100644
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
@@ -117,6 +119,11 @@ pub use mask_pattern::MaskPattern;
 #[doc(hidden)]
 pub use native::set_text_content_type;
 pub use native::{NativeMenu, NativeMenuItem};
+pub use presentation::{
+    BlockWidget, BlockWidgetRenderer, ChipHeight, ChipIcon, EditorPresentation,
+    EditorPresentationEvent, Ornament, Replacement, ReplacementChip, ReplacementColumn,
+    ReplacementImage,
+};
 pub use rope_ext::{InputEdit, Point, RopeExt, RopeLines};
 pub use ropey::Rope;
 pub use search::{SearchMatcher, SearchSession};
```
