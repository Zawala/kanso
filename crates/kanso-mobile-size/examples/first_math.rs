//! Times the first maths drawing (which starts QuickJS and loads MathJax) and
//! the second, for the size report (#48). Run on a phone's simulator or
//! device: `scripts/mobile-size.sh` shows how.

use std::time::Instant;

use kanso_render::{Job, Kind, Palette};

fn main() {
    for (label, source) in [("first", "x^2 + y^2 = z^2"), ("second", "\\frac{a}{b}")] {
        let started = Instant::now();
        let drawn = kanso_render::render(&Job::new(Kind::DisplayMath, source, Palette::LIGHT));
        println!(
            "math {label}: {} ms ({})",
            started.elapsed().as_millis(),
            if drawn.is_ok() { "drawn" } else { "failed" }
        );
    }
}
