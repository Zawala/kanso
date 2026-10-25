//! Fonts handed over after the first drawing are refused. Its own test
//! binary, so it is the only thing in its process.

// no-prop-test: process-wide state set once; there is nothing to vary.
#[test]
fn fonts_handed_over_after_the_database_is_built_are_refused() {
    // Flattening a drawing builds the font database.
    let flat = kanso_render::svg::flatten(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"10\" height=\"10\"><rect width=\"10\" height=\"10\"/></svg>",
        0.,
    );
    assert!(flat.is_some());
    assert!(!kanso_render::set_fonts(vec![]));
}
