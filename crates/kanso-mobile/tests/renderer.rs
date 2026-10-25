//! Drawing: the preview of a note, highlighting, themes, math and diagrams.

mod common;

use std::path::PathBuf;

use common::*;
use kanso_mobile::{DiagramKind, KansoError, PaletteDto, Renderer};
use proptest::prelude::*;

/// Where the repository's themes are: the crate's folder at run time (the
/// phone test runners set it), two levels up.
fn themes_dir() -> PathBuf {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    PathBuf::from(manifest).join("../../themes")
}

fn theme_files() -> Vec<PathBuf> {
    let mut files: Vec<_> = std::fs::read_dir(themes_dir())
        .unwrap()
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        // The catalogue lists themes; it isn't one.
        .filter(|path| path.file_name().is_some_and(|name| name != "index.json"))
        .collect();
    files.sort();
    files
}

fn palette() -> PaletteDto {
    let text = std::fs::read_to_string(themes_dir().join("ayu-dark.json")).unwrap();
    Renderer::theme_tokens(text).unwrap().palette
}

// no-prop-test: every theme file the repository ships, all of them.
#[test]
fn every_shipped_theme_fits() {
    let files = theme_files();
    assert!(!files.is_empty());
    let mut real: Vec<_> = kanso_engine::highlight::Token::ALL
        .iter()
        .map(|t| t.key())
        .collect();
    real.push("text");
    for file in files {
        let text = std::fs::read_to_string(&file).unwrap();
        let tokens = Renderer::theme_tokens(text).unwrap_or_else(|err| panic!("{file:?}: {err}"));
        assert!(!tokens.name.is_empty(), "{file:?}");
        assert_eq!(tokens.palette.dark, tokens.dark, "{file:?}");
        assert!(tokens.ui.iter().all(|c| c.rgb <= 0xFF_FFFF), "{file:?}");
        assert!(tokens.ui.iter().any(|c| c.name == "editor"), "{file:?}");
        assert!(tokens.palette.background <= 0xFF_FFFF && tokens.palette.text <= 0xFF_FFFF);
        assert!(tokens.caret <= 0xFF_FFFF && tokens.selection <= 0xFF_FFFF);
        assert!(
            tokens
                .syntax
                .iter()
                .all(|style| real.contains(&style.token.as_str())),
            "{file:?}"
        );
    }
}

// no-prop-test: refusals.
#[test]
fn a_theme_that_is_not_one_is_refused_in_words() {
    for text in ["", "[]", "{\"name\":1}", "not json"] {
        let err = Renderer::theme_tokens(text.into()).unwrap_err();
        assert!(matches!(err, KansoError::Invalid { .. }), "{text}: {err:?}");
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// Highlighting any text over any range gives runs inside the range that
    /// are sorted, apart, not empty and on whole characters, each styled by
    /// real tokens.
    #[test]
    fn prop_highlight_runs_are_inside_sorted_and_styled(
        text in "[#*`\\[\\]()a-z0-9 \n_>-]{0,80}|\\PC{0,30}",
        a in 0u32..100,
        b in 0u32..100,
    ) {
        let units = text.encode_utf16().count() as u32;
        let (start, end) = (a.min(b), a.max(b));
        let spans = Renderer::highlight(text.clone(), start, end).unwrap();
        let real: Vec<_> = kanso_engine::highlight::Token::ALL.iter().map(|t| t.key()).collect();
        let mut at = 0;
        for span in &spans {
            prop_assert!(span.range.start < span.range.end);
            prop_assert!(span.range.end <= units);
            prop_assert!(span.range.start >= at);
            at = span.range.end;
            prop_assert!(!span.tokens.is_empty());
            prop_assert!(span.tokens.iter().all(|t| real.contains(&t.as_str())));
        }
        // Inside what was asked for, widened to whole characters.
        if let (Some(first), Some(last)) = (spans.first(), spans.last()) {
            prop_assert!(first.range.start + 1 >= start.min(units));
            prop_assert!(last.range.end <= end.min(units) + 1);
        }
    }
}

// no-prop-test: a heading and a link, in UTF-16 units after wide characters.
#[test]
fn highlighting_counts_in_utf16_units() {
    let text = "🙂 é\n# Heading\n[link](https://example.com)\n";
    let spans = Renderer::highlight(text.into(), 0, text.encode_utf16().count() as u32).unwrap();
    let units: Vec<u16> = text.encode_utf16().collect();
    let words = |tokens: &str| -> Vec<String> {
        spans
            .iter()
            .filter(|s| s.tokens.iter().any(|t| t == tokens))
            .map(|s: &kanso_mobile::SpanRow| {
                String::from_utf16(&units[s.range.start as usize..s.range.end as usize]).unwrap()
            })
            .collect()
    };
    assert!(
        words("heading").iter().any(|w| w.contains("Heading")),
        "{spans:?}"
    );
    assert!(
        words("link").iter().any(|w| w.contains("link"))
            || words("url").iter().any(|w| w.contains("example"))
    );
    // Nothing before the heading is a run: the emoji and the é are plain,
    // and they take five UTF-16 units with the line break.
    assert!(spans.first().unwrap().range.start >= 5);
}

// no-prop-test: drawings of fixed sources.
#[test]
fn math_and_diagrams_are_drawn_and_a_bad_diagram_is_refused() {
    let palette = palette();
    let inline = Renderer::math_svg("x^2 + y^2".into(), false, palette).unwrap();
    assert!(inline.markup.contains("<svg"));
    assert!(inline.width > 0. && inline.height > 0.);
    assert!(inline.baseline.is_some());
    let display = Renderer::math_svg("\\int_0^1 x\\,dx".into(), true, palette).unwrap();
    assert!(display.markup.contains("<svg"));

    let flow = Renderer::diagram_svg(
        DiagramKind::Mermaid,
        "graph TD\n  A[Start] --> B[End]".into(),
        palette,
    )
    .unwrap();
    assert!(flow.markup.contains("<svg") && flow.width > 0.);
    let scene = r##"{"type":"excalidraw","version":2,"elements":[{"id":"a","type":"rectangle","x":0,"y":0,"width":120,"height":60,"strokeColor":"#1e1e1e","backgroundColor":"transparent","fillStyle":"solid","strokeWidth":1,"roughness":0,"seed":1}],"appState":{}}"##;
    let drawing = Renderer::diagram_svg(DiagramKind::Excalidraw, scene.into(), palette).unwrap();
    assert!(drawing.markup.contains("<svg"));

    let err = Renderer::diagram_svg(
        DiagramKind::Mermaid,
        "this is not a diagram".into(),
        palette,
    )
    .unwrap_err();
    assert!(matches!(err, KansoError::Invalid { .. }), "{err:?}");
    let err = Renderer::diagram_svg(DiagramKind::Excalidraw, "{".into(), palette).unwrap_err();
    assert!(matches!(err, KansoError::Invalid { .. }), "{err:?}");
}

// no-prop-test: a note with math, rendered, and a locked one refused.
#[test]
fn a_preview_is_a_page_and_a_closed_note_has_none() {
    let phone = lone_phone();
    let id = note(
        &phone.library,
        "Notes on <angles> & more",
        "# Heading\n\nSome *emphasis* and $a^2$.\n\n```mermaid\ngraph TD\n  A --> B\n```\n\n- [x] done\n",
    );
    let page = phone
        .library
        .renderer()
        .preview_html(&id, palette())
        .unwrap();
    assert!(page.contains("<html") || page.contains("<!DOCTYPE") || page.contains("<!doctype"));
    assert!(page.contains("Heading"));
    assert!(page.contains("<em>emphasis</em>"));
    assert!(page.contains("<svg"), "the math and the diagram are drawn");
    // The title is escaped.
    assert!(page.contains("&lt;angles&gt;"));
    assert!(!page.contains("<angles>"));

    phone
        .library
        .locks()
        .lock_note(&id, LOCK_PASSWORD.into())
        .unwrap();
    assert!(
        phone
            .library
            .renderer()
            .preview_html(&id, palette())
            .unwrap()
            .contains("Heading")
    );
    phone.library.locks().lock().unwrap();
    assert_eq!(
        phone.library.renderer().preview_html(&id, palette()),
        Err(KansoError::Locked)
    );
    assert_eq!(
        phone
            .library
            .renderer()
            .preview_html("note:none", palette()),
        Err(KansoError::NotFound)
    );
}

// no-prop-test: the fonts are taken by the first caller in a process; the
// render crate's own tests cover the order.
#[test]
fn fonts_are_handed_over_to_the_render_crate() {
    // Whether this call is the first is up to the other tests in this
    // process; it never fails.
    let _ = Renderer::set_fonts(vec![]);
    assert!(!Renderer::set_fonts(vec![]));
}
