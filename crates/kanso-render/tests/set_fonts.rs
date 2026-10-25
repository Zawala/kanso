//! The app hands its fonts over once, before anything is drawn. This file is
//! its own test binary, so no other test has built the font database first.

// no-prop-test: process-wide state set once; there is nothing to vary.
#[test]
fn the_fonts_are_taken_once() {
    assert!(kanso_render::set_fonts(vec![]));
    assert!(
        !kanso_render::set_fonts(vec![]),
        "a second hand-over is refused"
    );
}
