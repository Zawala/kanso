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

/// The ids `themes/index.json` lists, as the desktop's Themes page orders
/// them: Kanso's own first, then the rest as the index lists them.
fn indexed_files() -> Vec<String> {
    let text = std::fs::read_to_string(themes_dir().join("index.json")).unwrap();
    let index: serde_json::Value = serde_json::from_str(&text).unwrap();
    index["themes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|theme| theme["file"].as_str().unwrap().to_owned())
        .collect()
}

// no-prop-test: every built-in theme, all of them, against the files and the
// engine's own fit.
#[test]
fn the_built_in_themes_are_the_desktops_in_its_order_fitted_as_it_fits_them() {
    use kanso_engine::theme::Tokens;
    use kanso_engine::theme::builtin::BUILTIN;
    use kanso_engine::theme::file::ThemeFile;

    let themes = Renderer::builtin_themes().unwrap();
    let ids: Vec<&str> = themes.iter().map(|t| t.id.as_str()).collect();
    let order: Vec<&str> = BUILTIN
        .iter()
        .map(|(file, _)| file.trim_end_matches(".json"))
        .collect();
    assert_eq!(ids, order, "the engine's table, in its order");
    assert_eq!(&ids[..2], ["kanso-light", "kanso-dark"]);
    // The same themes as the index lists, none missing and none extra.
    let mut listed: Vec<String> = indexed_files()
        .into_iter()
        .map(|file| file.trim_end_matches(".json").to_owned())
        .collect();
    let mut have: Vec<String> = ids.iter().map(|id| (*id).to_owned()).collect();
    listed.sort();
    have.sort();
    assert_eq!(have, listed);

    for theme in &themes {
        let file = themes_dir().join(format!("{}.json", theme.id));
        let text = std::fs::read_to_string(&file).unwrap();
        // The same as fitting the file by hand.
        assert_eq!(
            Renderer::builtin_theme_tokens(theme.id.clone()).unwrap(),
            theme.tokens,
            "{}",
            theme.id
        );
        assert_eq!(
            Renderer::theme_tokens(text.clone()).unwrap(),
            theme.tokens,
            "{}",
            theme.id
        );
        // And as the desktop fits a theme for its own panes.
        let parsed = ThemeFile::parse(&text).unwrap();
        let resolved = parsed.resolve();
        let fit = Tokens::fit(&resolved, &resolved, &resolved);
        let row = |name: &str| theme.tokens.ui.iter().find(|c| c.name == name).unwrap().rgb;
        assert_eq!(row("editor"), fit.ui.editor.0, "{}", theme.id);
        assert_eq!(row("text"), fit.ui.text.0, "{}", theme.id);
        assert_eq!(row("accent"), fit.ui.accent.0, "{}", theme.id);
        assert_eq!(row("link"), fit.ui.link.0, "{}", theme.id);
        assert_eq!(row("danger"), fit.ui.danger.0, "{}", theme.id);
        assert_eq!(theme.tokens.caret, fit.caret.0);
        assert_eq!(theme.tokens.selection, fit.selection.rgb.0);
        assert_eq!(theme.tokens.palette.surface, fit.preview.code_background.0);
        assert_eq!(theme.tokens.palette.text, fit.preview.text.0);
        assert_eq!(theme.tokens.dark, fit.dark);
        assert_eq!(theme.dark, parsed.resolve().dark);
        assert_eq!(theme.tokens.syntax.len(), fit.syntax.len());
        for style in &theme.tokens.syntax {
            let want = fit.syntax[style.token.as_str()];
            assert_eq!(
                style.color,
                want.color.map(|c| c.0),
                "{} {}",
                theme.id,
                style.token
            );
            assert_eq!((style.italic, style.weight), (want.italic, want.weight));
        }
        // Who made it.
        assert_eq!(theme.name, parsed.name);
        assert_eq!(theme.author, parsed.author);
        assert_eq!(theme.license, parsed.license);
        assert_eq!(theme.source, parsed.source);
    }
}

// no-prop-test: a refusal.
#[test]
fn a_theme_that_is_not_built_in_is_not_found() {
    for id in ["", "nope", "nord.json", "../nord"] {
        assert_eq!(
            Renderer::builtin_theme_tokens(id.into()),
            Err(KansoError::NotFound)
        );
        assert_eq!(
            Renderer::styled_runs("# a".into(), 0, 3, id.into()),
            Err(KansoError::NotFound)
        );
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// Model: the styled runs are the highlighter's runs with each one's
    /// tokens folded through the theme's syntax rows, outermost first, a
    /// later token's attributes winning and unset ones showing what is under
    /// them; the runs the theme leaves unstyled are left out.
    #[test]
    fn prop_styled_runs_are_the_highlights_folded_through_the_theme(
        text in "[#*`\\[\\]()a-z0-9 \n_>-]{0,80}|\\PC{0,30}",
        a in 0u32..100,
        b in 0u32..100,
        theme in prop::sample::select(vec!["kanso-light", "kanso-dark", "nord", "github-light", "dracula"]),
    ) {
        let (start, end) = (a.min(b), a.max(b));
        let rows = Renderer::builtin_theme_tokens(theme.into()).unwrap().syntax;
        let fold = |tokens: &[String]| {
            tokens.iter().fold((None, None, None), |under: (Option<u32>, Option<bool>, Option<u16>), token| {
                let row = rows.iter().find(|r| &r.token == token);
                match row {
                    Some(r) => (r.color.or(under.0), r.italic.or(under.1), r.weight.or(under.2)),
                    None => under,
                }
            })
        };
        let expected: Vec<_> = Renderer::highlight(text.clone(), start, end).unwrap()
            .into_iter()
            .filter_map(|span| {
                let style = fold(&span.tokens);
                (style != (None, None, None)).then_some((span.range, style))
            })
            .collect();
        let runs = Renderer::styled_runs(text.clone(), start, end, theme.into()).unwrap();
        let got: Vec<_> = runs.iter().map(|r| (r.range, (r.color, r.italic, r.weight))).collect();
        prop_assert_eq!(got, expected);
        let units = text.encode_utf16().count() as u32;
        let mut at = 0;
        for run in &runs {
            prop_assert!(run.range.start < run.range.end && run.range.end <= units);
            prop_assert!(run.range.start >= at);
            at = run.range.end;
            prop_assert!(run.color.is_none_or(|c| c <= 0xFF_FFFF));
        }
    }
}

// no-prop-test: a heading in Nord's own color.
#[test]
fn a_heading_is_in_the_themes_heading_color() {
    let nord = Renderer::builtin_theme_tokens("nord".into()).unwrap();
    let heading = nord.syntax.iter().find(|s| s.token == "heading").unwrap();
    let runs = Renderer::styled_runs("# Title\n".into(), 0, 8, "nord".into()).unwrap();
    assert!(
        runs.iter()
            .any(|run| run.color == heading.color && run.color.is_some()),
        "{runs:?}"
    );
}
