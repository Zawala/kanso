//! A hand-drawn line generator after rough.js (MIT), which Excalidraw draws
//! with: each line is a slightly bowed cubic curve, drawn twice with small
//! random offsets, and fills are hatched with rough lines. The randomness is
//! rough.js's own seeded generator, so a scene looks the same every time it
//! is drawn, and close to how Excalidraw draws it.

use std::f32::consts::PI;
use std::fmt::Write as _;

/// rough.js's generator: a Park–Miller step on a 32-bit seed.
#[derive(Clone, Debug)]
pub struct Random(i32);

impl Random {
    pub fn new(seed: i64) -> Self {
        // Seed 0 means "unseeded" to rough.js; a drawing must not change, so
        // it gets a fixed seed instead.
        let seed = (seed as i32).max(1);
        Random(seed)
    }

    pub fn next(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(48271);
        ((self.0 & 0x7FFF_FFFF) as f64 / 2_147_483_648.) as f32
    }
}

/// Points along a path.
type Points = Vec<(f32, f32)>;

/// How rough a shape is drawn.
#[derive(Clone, Debug)]
pub struct Options {
    /// 0: architect (clean), 1: artist, 2: cartoonist.
    pub roughness: f32,
    pub bowing: f32,
    pub max_offset: f32,
    pub curve_fitting: f32,
    pub curve_tightness: f32,
    pub curve_step_count: f32,
    /// Draw each line once (Excalidraw does for dashed and dotted lines).
    pub single_stroke: bool,
    /// Keep the corners of a path where they are.
    pub preserve_vertices: bool,
    pub hachure_angle: f32,
    pub hachure_gap: f32,
}

impl Options {
    pub fn new(roughness: f32, stroke_width: f32) -> Self {
        Options {
            roughness,
            bowing: 1.,
            max_offset: 2.,
            curve_fitting: 0.95,
            curve_tightness: 0.,
            curve_step_count: 9.,
            single_stroke: false,
            preserve_vertices: false,
            hachure_angle: -41.,
            hachure_gap: (stroke_width * 4.).max(1.),
        }
    }
}

/// One path command.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Op {
    Move(f32, f32),
    Line(f32, f32),
    Curve(f32, f32, f32, f32, f32, f32),
}

/// `ops` as SVG path data.
pub fn path_data(ops: &[Op]) -> String {
    let mut d = String::new();
    let n = |v: f32| crate::svg::number(v);
    for op in ops {
        let _ = match *op {
            Op::Move(x, y) => write!(d, "M{} {}", n(x), n(y)),
            Op::Line(x, y) => write!(d, "L{} {}", n(x), n(y)),
            Op::Curve(a, b, c, e, x, y) => {
                write!(d, "C{} {} {} {} {} {}", n(a), n(b), n(c), n(e), n(x), n(y))
            }
        };
    }
    d
}

/// The rough drawer: options and the generator.
pub struct Rough {
    pub o: Options,
    rng: Random,
}

impl Rough {
    pub fn new(o: Options, seed: i64) -> Self {
        Rough {
            o,
            rng: Random::new(seed),
        }
    }

    fn random(&mut self) -> f32 {
        self.rng.next()
    }

    fn offset(&mut self, min: f32, max: f32, gain: f32) -> f32 {
        self.o.roughness * gain * (self.random() * (max - min) + min)
    }

    fn offset_opt(&mut self, x: f32, gain: f32) -> f32 {
        self.offset(-x, x, gain)
    }

    /// A rough line from `a` to `b`: twice, unless single-stroked.
    pub fn line(&mut self, a: (f32, f32), b: (f32, f32)) -> Vec<Op> {
        let mut ops = self.single_line(a, b, false);
        if !self.o.single_stroke {
            ops.extend(self.single_line(a, b, true));
        }
        ops
    }

    fn single_line(
        &mut self,
        (x1, y1): (f32, f32),
        (x2, y2): (f32, f32),
        overlay: bool,
    ) -> Vec<Op> {
        let length_sq = (x1 - x2).powi(2) + (y1 - y2).powi(2);
        let length = length_sq.sqrt();
        let gain = if length < 200. {
            1.
        } else if length > 500. {
            0.4
        } else {
            -0.0016668 * length + 1.233334
        };
        let mut offset = self.o.max_offset;
        if offset * offset * 100. > length_sq {
            offset = length / 10.;
        }
        let half = offset / 2.;
        let diverge = 0.2 + self.random() * 0.2;
        let mid_x = self.o.bowing * self.o.max_offset * (y2 - y1) / 200.;
        let mid_y = self.o.bowing * self.o.max_offset * (x1 - x2) / 200.;
        let mid_x = self.offset_opt(mid_x, gain);
        let mid_y = self.offset_opt(mid_y, gain);
        let jitter = if overlay { half } else { offset };
        let keep = self.o.preserve_vertices;
        let r = |this: &mut Self| this.offset_opt(jitter, gain);
        let (mx, my) = if keep { (0., 0.) } else { (r(self), r(self)) };
        let c1 = (
            mid_x + x1 + (x2 - x1) * diverge + r(self),
            mid_y + y1 + (y2 - y1) * diverge + r(self),
        );
        let c2 = (
            mid_x + x1 + 2. * (x2 - x1) * diverge + r(self),
            mid_y + y1 + 2. * (y2 - y1) * diverge + r(self),
        );
        let (ex, ey) = if keep { (0., 0.) } else { (r(self), r(self)) };
        vec![
            Op::Move(x1 + mx, y1 + my),
            Op::Curve(c1.0, c1.1, c2.0, c2.1, x2 + ex, y2 + ey),
        ]
    }

    /// Straight rough lines through `points`, closed back to the first if
    /// `close`.
    pub fn linear_path(&mut self, points: &[(f32, f32)], close: bool) -> Vec<Op> {
        let mut ops = Vec::new();
        for pair in points.windows(2) {
            ops.extend(self.line(pair[0], pair[1]));
        }
        if close && points.len() > 2 {
            ops.extend(self.line(points[points.len() - 1], points[0]));
        }
        ops
    }

    /// A smooth curve through `points` (Catmull–Rom), drawn twice.
    pub fn curve(&mut self, points: &[(f32, f32)]) -> Vec<Op> {
        if points.len() < 2 {
            return Vec::new();
        }
        let first = 1. + self.o.roughness * 0.2;
        let mut ops = self.curve_with_offset(points, first);
        if !self.o.single_stroke {
            let second = 1.5 * (1. + self.o.roughness * 0.22);
            ops.extend(self.curve_with_offset(points, second));
        }
        ops
    }

    fn curve_with_offset(&mut self, points: &[(f32, f32)], offset: f32) -> Vec<Op> {
        let mut ps = Vec::with_capacity(points.len() + 2);
        let jitter = |this: &mut Self, (x, y): (f32, f32)| {
            (
                x + this.offset_opt(offset, 1.),
                y + this.offset_opt(offset, 1.),
            )
        };
        ps.push(jitter(self, points[0]));
        ps.push(jitter(self, points[0]));
        for (i, p) in points.iter().enumerate().skip(1) {
            ps.push(jitter(self, *p));
            if i == points.len() - 1 {
                ps.push(jitter(self, *p));
            }
        }
        self.spline(&ps)
    }

    /// A Catmull–Rom spline through `points` (the first and last are only
    /// guides), as cubic curves.
    fn spline(&mut self, points: &[(f32, f32)]) -> Vec<Op> {
        let len = points.len();
        let mut ops = Vec::new();
        if len > 3 {
            let s = 1. - self.o.curve_tightness;
            ops.push(Op::Move(points[1].0, points[1].1));
            let mut i = 1;
            while i + 2 < len {
                let p = points[i];
                let b1 = (
                    p.0 + (s * points[i + 1].0 - s * points[i - 1].0) / 6.,
                    p.1 + (s * points[i + 1].1 - s * points[i - 1].1) / 6.,
                );
                let b2 = (
                    points[i + 1].0 + (s * points[i].0 - s * points[i + 2].0) / 6.,
                    points[i + 1].1 + (s * points[i].1 - s * points[i + 2].1) / 6.,
                );
                let b3 = points[i + 1];
                ops.push(Op::Curve(b1.0, b1.1, b2.0, b2.1, b3.0, b3.1));
                i += 1;
            }
        } else if len == 3 {
            ops.push(Op::Move(points[1].0, points[1].1));
            ops.push(Op::Curve(
                points[1].0,
                points[1].1,
                points[2].0,
                points[2].1,
                points[2].0,
                points[2].1,
            ));
        } else if len == 2 {
            ops.extend(self.single_line(points[0], points[1], true));
        }
        ops
    }

    /// A rough ellipse centred on `(cx, cy)`; also the points its outline
    /// follows, for filling it.
    pub fn ellipse(
        &mut self,
        cx: f32,
        cy: f32,
        width: f32,
        height: f32,
    ) -> (Vec<Op>, Vec<(f32, f32)>) {
        let psq = (PI * 2. * (((width / 2.).powi(2) + (height / 2.).powi(2)) / 2.).sqrt()).sqrt();
        let steps = self
            .o
            .curve_step_count
            .max(self.o.curve_step_count / 200f32.sqrt() * psq)
            .ceil()
            .min(400.);
        let increment = PI * 2. / steps;
        let fit = 1. - self.o.curve_fitting;
        let mut rx = (width / 2.).abs();
        let mut ry = (height / 2.).abs();
        rx += self.offset_opt(rx * fit, 1.);
        ry += self.offset_opt(ry * fit, 1.);
        let overlap_scale = self.offset(0.4, 1., 1.);
        let overlap = increment * self.offset(0.1, overlap_scale, 1.);
        let (all, core) = self.ellipse_points(increment, cx, cy, rx, ry, 1., overlap);
        let mut ops = self.spline(&all);
        if !self.o.single_stroke && self.o.roughness != 0. {
            let (all, _) = self.ellipse_points(increment, cx, cy, rx, ry, 1.5, 0.);
            ops.extend(self.spline(&all));
        }
        (ops, core)
    }

    #[allow(clippy::too_many_arguments)]
    fn ellipse_points(
        &mut self,
        increment: f32,
        cx: f32,
        cy: f32,
        rx: f32,
        ry: f32,
        offset: f32,
        overlap: f32,
    ) -> (Points, Points) {
        let mut core = Vec::new();
        let mut all = Vec::new();
        if self.o.roughness == 0. {
            let increment = increment / 4.;
            all.push((cx + rx * (-increment).cos(), cy + ry * (-increment).sin()));
            let mut angle = 0.;
            while angle <= PI * 2. {
                let p = (cx + rx * f32::cos(angle), cy + ry * f32::sin(angle));
                core.push(p);
                all.push(p);
                angle += increment;
            }
            all.push((cx + rx, cy));
            all.push((cx + rx * increment.cos(), cy + ry * increment.sin()));
        } else {
            let rad = self.offset_opt(0.5, 1.) - PI / 2.;
            let j = |this: &mut Self| this.offset_opt(offset, 1.);
            all.push((
                j(self) + cx + 0.9 * rx * (rad - increment).cos(),
                j(self) + cy + 0.9 * ry * (rad - increment).sin(),
            ));
            let end = PI * 2. + rad - 0.01;
            let mut angle = rad;
            while angle < end {
                let p = (
                    j(self) + cx + rx * angle.cos(),
                    j(self) + cy + ry * angle.sin(),
                );
                core.push(p);
                all.push(p);
                angle += increment;
            }
            for (scale, at) in [
                (1., rad + PI * 2. + overlap * 0.5),
                (0.98, rad + overlap),
                (0.9, rad + overlap * 0.5),
            ] {
                all.push((
                    j(self) + cx + scale * rx * at.cos(),
                    j(self) + cy + scale * ry * at.sin(),
                ));
            }
        }
        (all, core)
    }

    /// A solid fill of `polygon`, its corners nudged as rough.js does.
    pub fn solid_fill(&mut self, polygon: &[(f32, f32)]) -> Vec<Op> {
        let mut ops = Vec::with_capacity(polygon.len());
        // A dense outline (a curve) is nudged less, or its edge would fray.
        let offset = if polygon.len() > 12 {
            0.4
        } else {
            self.o.max_offset
        };
        for (i, &(x, y)) in polygon.iter().enumerate() {
            let p = (
                x + self.offset_opt(offset, 1.),
                y + self.offset_opt(offset, 1.),
            );
            ops.push(if i == 0 {
                Op::Move(p.0, p.1)
            } else {
                Op::Line(p.0, p.1)
            });
        }
        ops
    }

    /// Hatching across `polygon`: rough lines `gap` apart at the hachure
    /// angle, and across it again if `cross`.
    pub fn hachure(&mut self, polygon: &[(f32, f32)], cross: bool) -> Vec<Op> {
        let mut ops = Vec::new();
        let angles: &[f32] = if cross { &[0., 90.] } else { &[0.] };
        for extra in angles {
            let angle = self.o.hachure_angle + extra;
            for (a, b) in hatch_lines(polygon, self.o.hachure_gap, angle) {
                ops.extend(self.line(a, b));
            }
        }
        ops
    }
}

/// The hatching segments across `polygon`: parallel lines `gap` apart at
/// `angle` degrees, clipped to its inside (even–odd). Pure.
pub fn hatch_lines(polygon: &[(f32, f32)], gap: f32, angle: f32) -> Vec<((f32, f32), (f32, f32))> {
    if polygon.len() < 3 {
        return Vec::new();
    }
    let gap = gap.max(0.5);
    let (sin, cos) = (angle.to_radians()).sin_cos();
    // Into a frame where the lines are horizontal, and back.
    let rotate = |(x, y): (f32, f32)| (x * cos + y * sin, -x * sin + y * cos);
    let unrotate = |(x, y): (f32, f32)| (x * cos - y * sin, x * sin + y * cos);
    let points: Vec<(f32, f32)> = polygon.iter().copied().map(rotate).collect();
    let (min_y, max_y) = points.iter().fold((f32::MAX, f32::MIN), |(lo, hi), p| {
        (lo.min(p.1), hi.max(p.1))
    });
    if !(min_y.is_finite() && max_y.is_finite()) || (max_y - min_y) / gap > 5000. {
        return Vec::new();
    }
    let mut lines = Vec::new();
    let mut y = min_y + gap / 2.;
    while y < max_y {
        let mut xs: Vec<f32> = Vec::new();
        for i in 0..points.len() {
            let (a, b) = (points[i], points[(i + 1) % points.len()]);
            if (a.1 <= y && b.1 > y) || (b.1 <= y && a.1 > y) {
                xs.push(a.0 + (y - a.1) / (b.1 - a.1) * (b.0 - a.0));
            }
        }
        xs.sort_by(f32::total_cmp);
        for pair in xs.as_chunks::<2>().0 {
            if pair[1] - pair[0] > 0.01 {
                lines.push((unrotate((pair[0], y)), unrotate((pair[1], y))));
            }
        }
        y += gap;
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    /// rough.js's own first values for seed 1, from its generator.
    #[test]
    fn random_matches_rough_js() {
        let mut r = Random::new(1);
        let a = r.next();
        let b = r.next();
        assert!((a - 48271. / 2_147_483_648.).abs() < 1e-9);
        assert!((b - ((48271i64 * 48271) & 0x7FFF_FFFF) as f32 / 2_147_483_648.).abs() < 1e-6);
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Invariant: the generator stays in [0, 1).
        #[test]
        fn prop_random_in_unit_interval(seed in any::<i64>()) {
            let mut r = Random::new(seed);
            for _ in 0..64 {
                let v = r.next();
                prop_assert!((0. ..1.).contains(&v));
            }
        }

        /// Determinism: the same seed draws the same line.
        #[test]
        fn prop_same_seed_same_drawing(seed in any::<i64>(), x in -500f32..500., y in -500f32..500.) {
            let draw = || Rough::new(Options::new(1., 2.), seed).line((0., 0.), (x, y));
            prop_assert_eq!(draw(), draw());
        }

        /// Invariant: a rough line stays close to the straight one: its
        /// ends within the jitter of the true ends.
        #[test]
        fn prop_lines_stay_near_their_ends(seed in any::<i64>(), x1 in -300f32..300., y1 in -300f32..300., x2 in -300f32..300., y2 in -300f32..300., roughness in 0f32..=2.) {
            let ops = Rough::new(Options::new(roughness, 2.), seed).line((x1, y1), (x2, y2));
            let slack = 2. * roughness * 2. + 0.01;
            for op in ops {
                match op {
                    Op::Move(x, y) => prop_assert!((x - x1).abs() <= slack && (y - y1).abs() <= slack),
                    Op::Curve(.., x, y) => prop_assert!((x - x2).abs() <= slack && (y - y2).abs() <= slack),
                    Op::Line(..) => {}
                }
            }
        }

        /// Invariant: roughness 0 draws a line exactly between its ends.
        #[test]
        fn prop_architect_lines_are_exact(seed in any::<i64>(), x in -300f32..300., y in -300f32..300.) {
            let ops = Rough::new(Options::new(0., 2.), seed).line((1., 2.), (x, y));
            prop_assert_eq!(ops[0], Op::Move(1., 2.));
            let Op::Curve(.., ex, ey) = ops[1] else { panic!() };
            prop_assert_eq!((ex, ey), (x, y));
        }

        /// Invariant: hatching stays inside a rectangle's bounds, and a
        /// wider gap never draws more lines.
        #[test]
        fn prop_hatching_stays_inside(w in 1f32..400., h in 1f32..400., gap in 1f32..20., angle in -90f32..90.) {
            let rect = [(0., 0.), (w, 0.), (w, h), (0., h)];
            let lines = hatch_lines(&rect, gap, angle);
            for (a, b) in &lines {
                for (x, y) in [a, b] {
                    prop_assert!(*x >= -0.01 && *x <= w + 0.01 && *y >= -0.01 && *y <= h + 0.01);
                }
            }
            prop_assert!(hatch_lines(&rect, gap * 2., angle).len() <= lines.len() + 1);
        }
    }
}
