//! A visual row whose offsets remain in source UTF-8 bytes.
use gpui::{
    App, Bounds, Hsla, Pixels, Point, ShapedLine, SharedString, TextAlign, Window, point, px, size,
};
use std::ops::Range;

pub(crate) struct InlineFragment {
    pub(crate) range: Range<usize>,
    pub(crate) x: Pixels,
    pub(crate) width: Pixels,
    pub(crate) text: Option<ShapedLine>,
    /// A presentation's replacement: `text` is drawn in place of the source in `range`, which
    /// the caret treats as one unit, like an element.
    pub(crate) replaced: bool,
    /// Where `text` starts within the fragment: a replacement set in a column is
    /// right-aligned in it.
    pub(crate) text_x: Pixels,
    /// How far a hanging replacement's column reaches left of `x`, where a click
    /// still lands on it.
    pub(crate) hang: Pixels,
    /// How far `text` is drawn below where the row would centre it: a replacement at
    /// another size keeps the row's baseline.
    pub(crate) text_y: Pixels,
    /// A replacement's box, drawn behind `text`.
    pub(crate) chip: Option<FragmentChip>,
    /// A replacement's image, drawn instead of `text`, with the row font's ascent and
    /// descent to put its baseline on the row's.
    pub(crate) image: Option<(crate::input::ReplacementImage, Pixels, Pixels)>,
}

/// A replacement's box, as laid out: what to draw, and its text's metrics to place it.
#[derive(Clone)]
pub(crate) struct FragmentChip {
    /// The icon's asset, size and colour, and where it starts in the fragment.
    pub(crate) icon: Option<(SharedString, Pixels, Hsla, Pixels)>,
    pub(crate) dashed_bottom: Option<Hsla>,
    /// Whether it stands for a laid-out element, which sits on whole device pixels.
    pub(crate) whole_width: bool,
    /// Where its text's baseline is, below the box's top.
    pub(crate) text_baseline: Option<Pixels>,
    pub(crate) background: Hsla,
    pub(crate) radius: Pixels,
    /// Its top and height within the row, from the row's height.
    pub(crate) height: crate::input::ChipHeight,
    pub(crate) cap_height: Pixels,
    pub(crate) font_size: Pixels,
}

pub(crate) struct InputLine {
    pub(crate) len: usize,
    pub(crate) width: Pixels,
    pub(crate) text: SharedString,
    content: Content,
}
// Keep the ordinary shaped row inline: boxing it would add an allocation to
// every existing plain-text row. Only token rows allocate fragment storage.
#[allow(clippy::large_enum_variant)]
enum Content {
    Text(ShapedLine),
    Inline(Vec<InlineFragment>),
}
impl From<ShapedLine> for InputLine {
    fn from(line: ShapedLine) -> Self {
        Self {
            len: line.len,
            width: line.width,
            text: line.text.clone(),
            content: Content::Text(line),
        }
    }
}
impl InputLine {
    pub(crate) fn inline(text: SharedString, fragments: Vec<InlineFragment>) -> Self {
        let width = fragments.last().map_or(px(0.), |f| f.x + f.width);
        Self {
            len: text.len(),
            text,
            width,
            content: Content::Inline(fragments),
        }
    }
    pub(crate) fn x_for_index(&self, ix: usize) -> Pixels {
        match &self.content {
            Content::Text(line) => line.x_for_index(ix),
            Content::Inline(fragments) => {
                for f in fragments {
                    if ix < f.range.end {
                        return f.x
                            + match &f.text {
                                // A caret inside a replacement is drawn at its start.
                                Some(_) if f.replaced => px(0.),
                                Some(line) => line.x_for_index(ix.saturating_sub(f.range.start)),
                                None if ix <= f.range.start => px(0.),
                                None => f.width,
                            };
                    }
                }
                self.width
            }
        }
    }
    pub(crate) fn closest_index_for_x(&self, x: Pixels) -> usize {
        match &self.content {
            Content::Text(line) => line.closest_index_for_x(x),
            Content::Inline(fragments) => {
                for f in fragments {
                    if x <= f.x + f.width {
                        return match &f.text {
                            Some(line) if !f.replaced => {
                                f.range.start + line.closest_index_for_x(x - f.x)
                            }
                            // A concealed range has no width to split, so a click at
                            // its point lands after it, past the hidden source.
                            _ if f.width > px(0.) && x - f.x < f.width / 2. => f.range.start,
                            _ => f.range.end,
                        };
                    }
                }
                self.len
            }
        }
    }
    /// Whether the byte at `ix` is concealed: inside a fragment drawn at zero width, or
    /// drawn as a replacement's text.
    pub(crate) fn is_concealed(&self, ix: usize) -> bool {
        match &self.content {
            Content::Text(_) => false,
            Content::Inline(fragments) => fragments.iter().any(|f| {
                ((f.text.is_none() && f.width <= px(0.)) || f.replaced) && f.range.contains(&ix)
            }),
        }
    }
    /// The row-local source range of the replacement drawn under `x`, if any.
    pub(crate) fn replacement_at_x(&self, x: Pixels) -> Option<Range<usize>> {
        match &self.content {
            Content::Text(_) => None,
            Content::Inline(fragments) => fragments
                .iter()
                .find(|f| f.replaced && x >= f.x - f.hang && x < f.x + f.width.max(px(0.)))
                .map(|f| f.range.clone()),
        }
    }
    pub(crate) fn index_for_x(&self, x: Pixels) -> Option<usize> {
        match &self.content {
            Content::Text(line) => line.index_for_x(x),
            Content::Inline(_) => {
                (x >= px(0.) && x <= self.width).then(|| self.closest_index_for_x(x))
            }
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn paint(
        &self,
        pos: Point<Pixels>,
        height: Pixels,
        align: TextAlign,
        width: Option<Pixels>,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.paint_pass(pos, height, align, width, false, window, cx);
    }
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn paint_background(
        &self,
        pos: Point<Pixels>,
        height: Pixels,
        align: TextAlign,
        width: Option<Pixels>,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.paint_pass(pos, height, align, width, true, window, cx);
    }
    #[allow(clippy::too_many_arguments)]
    fn paint_pass(
        &self,
        pos: Point<Pixels>,
        height: Pixels,
        align: TextAlign,
        width: Option<Pixels>,
        background: bool,
        window: &mut Window,
        cx: &mut App,
    ) {
        let paint = |line: &ShapedLine, pos, align, width, window: &mut Window, cx: &mut App| {
            if background {
                let _ = line.paint_background(pos, height, align, width, window, cx);
            } else {
                let _ = line.paint(pos, height, align, width, window, cx);
            }
        };
        match &self.content {
            Content::Text(line) => paint(line, pos, align, width, window, cx),
            Content::Inline(fragments) => {
                let remaining = (width.unwrap_or(self.width) - self.width).max(px(0.));
                let offset = match align {
                    TextAlign::Right => remaining,
                    TextAlign::Center => remaining / 2.,
                    _ => px(0.),
                };
                let scale = window.scale_factor();
                // As the layout snaps a box: to whole device pixels, halves toward zero.
                // Within a rounding error of a half is a half.
                let snap = |v: Pixels| {
                    let device = f32::from(v) * scale;
                    px((device.abs() - 0.5 - 1e-3).ceil().copysign(device) / scale)
                };
                for f in fragments {
                    // A chip's box: its top and height in the row.
                    let geometry = match (&f.chip, &f.text) {
                        (Some(chip), Some(line)) => Some(match chip.height {
                            crate::input::ChipHeight::Fixed(h) => ((height - h) / 2., h),
                            crate::input::ChipHeight::Baseline { ascent, descent } => {
                                let baseline =
                                    (height - line.ascent - line.descent) / 2. + line.ascent;
                                (baseline - ascent, ascent + descent)
                            }
                            crate::input::ChipHeight::CapHeight => {
                                let baseline = (height - line.ascent - line.descent) / 2.
                                    + line.ascent
                                    + f.text_y;
                                let pad = chip.font_size * 0.125 + line.descent / 2.;
                                (baseline - chip.cap_height - pad, chip.cap_height + pad * 2.)
                            }
                        }),
                        _ => None,
                    };
                    // A chip standing for a laid-out element sits on whole device pixels,
                    // as the element's box does, and so does everything in it.
                    let shift = match (&f.chip, geometry) {
                        (Some(chip), Some((top, _))) if chip.whole_width => {
                            let origin = pos + point(offset + f.x, top);
                            point(snap(origin.x) - origin.x, snap(origin.y) - origin.y)
                        }
                        _ => point(px(0.), px(0.)),
                    };
                    if background
                        && let (Some(chip), Some((top, chip_height))) = (&f.chip, geometry)
                    {
                        window.paint_quad(
                            gpui::fill(
                                Bounds::new(
                                    pos + shift + point(offset + f.x, top),
                                    size(f.width, chip_height),
                                ),
                                chip.background,
                            )
                            .corner_radii(chip.radius),
                        );
                        if let Some(color) = chip.dashed_bottom {
                            // On whole device pixels, as a laid-out box's border is, so
                            // its dashes fall the same way.
                            let origin = pos + shift + point(offset + f.x, top);
                            let (left, top) = (snap(origin.x), snap(origin.y));
                            let right = snap(origin.x + f.width);
                            let bottom = snap(origin.y + chip_height);
                            window.paint_quad(gpui::quad(
                                Bounds::new(point(left, top), size(right - left, bottom - top)),
                                px(0.),
                                gpui::transparent_black(),
                                gpui::Edges {
                                    bottom: px(1.),
                                    ..Default::default()
                                },
                                color,
                                gpui::BorderStyle::Dashed,
                            ));
                        }
                    }
                    if !background
                        && let Some(FragmentChip {
                            icon: Some((path, side, color, x)),
                            ..
                        }) = &f.chip
                    {
                        let bounds = Bounds::new(
                            pos + shift + point(offset + f.x + *x, (height - *side) / 2.),
                            size(*side, *side),
                        );
                        window
                            .paint_svg(
                                bounds,
                                path.clone(),
                                None,
                                gpui::TransformationMatrix::unit(),
                                *color,
                                cx,
                            )
                            .ok();
                    }
                    if let Some((image, ascent, descent)) = &f.image {
                        if !background {
                            // An image is a laid-out element: on whole device pixels.
                            let baseline = (height - *ascent - *descent) / 2. + *ascent;
                            let origin = pos + point(offset + f.x, baseline - image.baseline);
                            let bounds =
                                Bounds::new(point(snap(origin.x), snap(origin.y)), image.size);
                            if let Some(data) = image.image.clone().use_render_image(window, cx) {
                                window
                                    .paint_image(bounds, bounds, Default::default(), data, 0, false)
                                    .ok();
                            }
                        }
                        continue;
                    }
                    // Text set in the box, as the element sets it.
                    let text_y = match (&f.chip, geometry, &f.text) {
                        (Some(chip), Some((top, _)), Some(line))
                            if chip.text_baseline.is_some() =>
                        {
                            let centred = (height - line.ascent - line.descent) / 2. + line.ascent;
                            top + chip.text_baseline.unwrap_or_default() - centred
                        }
                        _ => f.text_y,
                    };
                    if let Some(line) = &f.text {
                        paint(
                            line,
                            pos + shift + point(offset + f.x + f.text_x, text_y),
                            TextAlign::Left,
                            None,
                            window,
                            cx,
                        );
                    }
                }
            }
        }
    }
}
