//! Mermaid diagrams, drawn by the pure-Rust `mermaid-svg` in the preview
//! theme's colors.

use std::borrow::Cow;

use mermaid_svg::{ParseError, RenderError, Theme};

use crate::{Failure, Outcome, Palette, Rgb, Svg, svg};

/// The longest diagram source drawn.
const MAX_SOURCE: usize = 64 * 1024;
/// Room around a diagram, in points.
const PAD: f32 = 8.;

/// The diagram keywords `mermaid-svg` draws.
const KEYWORDS: &[&str] = &[
    "pie",
    "sequenceDiagram",
    "flowchart",
    "graph",
    "flowchart-elk",
    "stateDiagram",
    "stateDiagram-v2",
    "classDiagram",
    "classDiagram-v2",
    "erDiagram",
    "gantt",
    "journey",
    "timeline",
    "sankey",
    "sankey-beta",
    "quadrantChart",
    "xychart",
    "xychart-beta",
    "radar",
    "radar-beta",
    "packet",
    "packet-beta",
    "mindmap",
    "gitGraph",
    "requirementDiagram",
    "C4Context",
    "C4Container",
    "C4Component",
    "C4Dynamic",
    "C4Deployment",
    "block",
    "block-beta",
    "architecture",
    "architecture-beta",
    "kanban",
    "treemap",
    "treemap-beta",
    "zenuml",
];

/// The diagram's keyword: the first word after any front matter, `%%`
/// comments and `%%{init}%%` directives.
fn keyword(source: &str) -> &str {
    let mut lines = source.lines().map(str::trim);
    let mut in_front_matter = false;
    for line in lines.by_ref() {
        if line.is_empty() || line.starts_with("%%") {
            continue;
        }
        if line == "---" {
            in_front_matter = !in_front_matter;
            continue;
        }
        if in_front_matter {
            continue;
        }
        return line
            .split(|c: char| c.is_whitespace() || c == ';')
            .next()
            .unwrap_or("")
            .trim_end_matches(':');
    }
    ""
}

/// Whether Kanso draws diagrams like `source`.
pub fn is_supported(source: &str) -> bool {
    KEYWORDS.contains(&keyword(source))
}

/// The theme: the page's colors, nodes tinted with its accent.
fn theme(p: &Palette) -> Theme {
    let base = if p.dark {
        Theme::dark()
    } else {
        Theme::default_theme()
    };
    let hex = |c: Rgb| Cow::Owned(c.hex());
    let node_fill = p
        .accent
        .over(p.background, if p.dark { 0.16 } else { 0.10 });
    let node_stroke = p.accent.over(p.background, 0.6);
    let line = p.muted.over(p.text, 0.5);
    let note = Rgb(0xE9C46A).over(p.background, if p.dark { 0.22 } else { 0.28 });
    Theme {
        fg: hex(p.text),
        fg_muted: hex(p.muted),
        bg: hex(p.background),
        actor_fill: hex(node_fill),
        actor_stroke: hex(node_stroke),
        lifeline: hex(p.border.over(p.muted, 0.5)),
        arrow_stroke: hex(line),
        note_fill: hex(note),
        note_stroke: hex(Rgb(0xC9A227).over(p.background, 0.6)),
        activation_fill: hex(node_fill),
        activation_stroke: hex(node_stroke),
        frame_label_fill: hex(node_fill),
        flow_node_fill: hex(node_fill),
        flow_node_stroke: hex(node_stroke),
        flow_edge_stroke: hex(line),
        flow_label_bg: hex(p.surface),
        flow_cluster_fill: hex(p.surface),
        flow_cluster_stroke: hex(p.border),
        pie_stroke: Some(hex(p.background)),
        font_family: Cow::Borrowed(
            "'Helvetica Neue', 'Segoe UI', Inter, 'Noto Sans', 'DejaVu Sans', Arial, sans-serif",
        ),
        font_size: 14.,
        responsive: false,
        ..base
    }
}

/// Draws a Mermaid diagram.
pub(crate) fn render(source: &str, palette: &Palette) -> Outcome {
    if source.len() > MAX_SOURCE {
        return Err(Failure::Invalid("The diagram is too long to draw.".into()));
    }
    let theme = theme(palette);
    // A third-party layout engine: a bug in it shows the source, not a crash.
    let drawn = std::panic::catch_unwind(|| mermaid_svg::render_with(source, &theme))
        .map_err(|_| Failure::Invalid("The diagram couldn't be laid out.".into()))?;
    let markup = drawn.map_err(|err| match err {
        RenderError::Parse(ParseError::UnknownDiagramType(kind)) => Failure::Unsupported(kind),
        RenderError::Parse(ParseError::Empty) => Failure::Invalid("The diagram is empty.".into()),
        RenderError::Parse(err) => Failure::Invalid(err.to_string()),
        #[allow(unreachable_patterns)]
        err => Failure::Invalid(err.to_string()),
    })?;
    let flat = svg::flatten(&markup, PAD)
        .ok_or_else(|| Failure::Invalid("The diagram drew nothing.".into()))?;
    Ok(Svg {
        markup: flat.markup,
        width: flat.width,
        height: flat.height,
        baseline: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    const SAMPLES: &[&str] = &[
        "flowchart TD\n  A[Start] --> B{Working?}\n  B -->|Yes| C[Ship]\n  B -->|No| D[Debug] --> B\n",
        "graph LR\n  subgraph Client\n    a[Editor] --> b[Preview]\n  end\n  b --> c[(Store)]\n",
        "sequenceDiagram\n  Alice->>Bob: Hello\n  Bob-->>Alice: Hi\n  Note over Alice,Bob: A note\n",
        "classDiagram\n  Animal <|-- Dog\n  class Animal { +String name }\n",
        "stateDiagram-v2\n  [*] --> Still\n  Still --> Moving\n  Moving --> [*]\n",
        "erDiagram\n  CUSTOMER ||--o{ ORDER : places\n",
        "pie title Pets\n  \"Dogs\" : 386\n  \"Cats\" : 85\n",
        "gantt\n  dateFormat YYYY-MM-DD\n  section A\n  Task :a1, 2024-01-01, 3d\n",
        "mindmap\n  root((Kanso))\n    Notes\n    Sync\n",
        "---\ntitle: T\n---\n%% a comment\njourney\n  title Day\n  section Work\n    Write: 5: Me\n",
    ];

    #[test]
    fn common_diagrams_draw_in_the_theme() {
        for source in SAMPLES {
            for palette in [Palette::LIGHT, Palette::DARK] {
                let svg = render(source, &palette).unwrap_or_else(|e| panic!("{source}: {e:?}"));
                assert!(svg.width > 2. * PAD && svg.height > 2. * PAD, "{source}");
                assert!(!svg.markup.contains("<text"), "text is drawn as outlines");
                assert!(!svg.markup.contains("foreignObject"));
                let tree = usvg::Tree::from_str(&svg.markup, &usvg::Options::default()).unwrap();
                assert!((tree.size().width() - svg.width).abs() < 0.01);
            }
            assert!(is_supported(source), "{source}");
        }
        let dark = render(SAMPLES[0], &Palette::DARK).unwrap();
        assert!(
            dark.markup
                .contains(&Palette::DARK.text.over(Palette::DARK.muted, 0.5).hex())
                || dark
                    .markup
                    .contains(&Palette::DARK.muted.over(Palette::DARK.text, 0.5).hex())
        );
    }

    #[test]
    fn unknown_kinds_and_mistakes_are_told_apart() {
        assert_eq!(
            render("sparkleDiagram\n  a --> b\n", &Palette::LIGHT),
            Err(Failure::Unsupported("sparkleDiagram".into()))
        );
        assert!(!is_supported("sparkleDiagram\n"));
        assert!(matches!(
            render("", &Palette::LIGHT),
            Err(Failure::Invalid(_))
        ));
        assert_eq!(keyword("%%{init: {}}%%\n\n  graph TD;A-->B"), "graph");
        assert_eq!(keyword("gitGraph:\n commit"), "gitGraph");
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        /// Invariant: whatever the text, drawing answers without panicking,
        /// and a drawing is a sized, parseable SVG.
        #[test]
        fn prop_drawing_answers(body in prop::collection::vec("[A-Za-z0-9 \\[\\]{}()|>-]{0,16}", 0..8), head in prop::sample::select(vec!["flowchart TD", "graph LR", "sequenceDiagram", "pie", "classDiagram", "stateDiagram-v2", "nonsense"])) {
            let source = format!("{head}\n{}", body.join("\n"));
            match render(&source, &Palette::LIGHT) {
                Ok(svg) => {
                    prop_assert!(svg.width > 0. && svg.height > 0.);
                    prop_assert!(usvg::Tree::from_str(&svg.markup, &usvg::Options::default()).is_ok());
                }
                Err(Failure::Unsupported(kind)) => prop_assert_eq!(kind, "nonsense"),
                Err(Failure::Invalid(_)) => {}
            }
        }
    }
}
