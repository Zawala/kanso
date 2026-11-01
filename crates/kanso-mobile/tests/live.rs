//! Live mode on a phone: the marks to hide, the spans to style and the
//! heading lines to draw larger, in UTF-16 units of the field's window.

mod common;

use common::*;
use kanso_mobile::{LiveKind, Range16};
use proptest::prelude::*;

fn r(start: u32, end: u32) -> Range16 {
    Range16 { start, end }
}

fn units(text: &str) -> usize {
    text.encode_utf16().count()
}

#[test]
fn marks_styles_and_headings_are_in_utf16_units() {
    let phone = lone_phone();
    // "é" and "🙂" are one and two units, and two and four bytes.
    let body = "# Héading\n\n🙂 **bold** and `code`\n";
    let id = note(&phone.library, "Title", body);
    let session = phone.library.edit(&id).unwrap();
    let text = session.snapshot().unwrap().text;
    let batch = session.live_layout(0, units(&text) as u32).unwrap();
    assert_eq!(batch.version, session.snapshot().unwrap().version);
    let line = units("# Héading\n\n");
    let bold = line as u32 + units("🙂 ") as u32;
    let code = bold + units("**bold** and ") as u32;
    assert_eq!(
        batch.conceal,
        vec![
            r(0, 2),
            r(bold, bold + 2),
            r(bold + 6, bold + 8),
            r(code, code + 1),
            r(code + 5, code + 6),
        ]
    );
    let runs: Vec<_> = batch.runs.iter().map(|run| (run.range, run.kind)).collect();
    assert_eq!(
        runs,
        vec![
            (r(2, 9), LiveKind::Heading),
            (r(bold + 2, bold + 6), LiveKind::Strong),
            (r(code + 1, code + 5), LiveKind::Code),
        ]
    );
    assert_eq!(batch.lines.len(), 1);
    assert_eq!(batch.lines[0].range, r(0, 9));
    assert_eq!(batch.lines[0].level, 1);
    assert_eq!(batch.lines[0].scale, 1.6);
}

/// markdown-rs panics ("Cannot push to non-parent") on a setext heading
/// followed by `---` lines and a paragraph; the layout must still come back
/// with the text untouched.
#[test]
fn a_setext_heading_then_dashes_does_not_crash_the_layout() {
    let phone = lone_phone();
    let body = "Setext\n===\n---\npara **b** _i_ ~~s~~ `c` 🙂\n---\n";
    let id = note(&phone.library, "Title", body);
    let session = phone.library.edit(&id).unwrap();
    let text = session.snapshot().unwrap().text;
    assert!(text.contains(body));
    session.live_document(Vec::new()).unwrap();
    session.live_layout(0, units(&text) as u32).unwrap();
}

#[test]
fn a_window_counts_from_its_start() {
    let phone = lone_phone();
    let body = "first line\n## Second _it_\nlast";
    let id = note(&phone.library, "Title", body);
    let session = phone.library.edit(&id).unwrap();
    let start = units("first line\n") as u32;
    let end = start + units("## Second _it_") as u32;
    let batch = session.live_layout(start, end).unwrap();
    assert_eq!(batch.conceal, vec![r(0, 3), r(10, 11), r(13, 14)]);
    let runs: Vec<_> = batch.runs.iter().map(|run| (run.range, run.kind)).collect();
    assert_eq!(
        runs,
        vec![
            (r(3, 14), LiveKind::Heading),
            (r(11, 13), LiveKind::Emphasis)
        ]
    );
    assert_eq!(batch.lines.len(), 1);
    assert_eq!(batch.lines[0].range, r(0, 14));
    assert_eq!((batch.lines[0].level, batch.lines[0].scale), (2, 1.4));
}

#[test]
fn a_range_past_the_end_is_cut_to_the_text() {
    let phone = lone_phone();
    let id = note(&phone.library, "Title", "~~gone~~");
    let session = phone.library.edit(&id).unwrap();
    let batch = session.live_layout(0, 1_000).unwrap();
    assert_eq!(batch.conceal, vec![r(0, 2), r(6, 8)]);
    let empty = session.live_layout(500, 1_000).unwrap();
    assert!(empty.conceal.is_empty() && empty.runs.is_empty() && empty.lines.is_empty());
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// Whatever the text and the window, every range is in the window, in
    /// order, never empty, never across a line break and never inside a
    /// surrogate pair.
    #[test]
    fn prop_ranges_stay_in_the_window(
        body in "[#*`_~\\[\\]()a-z \n|é🙂-]{0,80}",
        window in (0u32..100, 0u32..100),
    ) {
        let phone = lone_phone();
        let id = note(&phone.library, "Title", &body);
        let session = phone.library.edit(&id).unwrap();
        let text: Vec<u16> = session.snapshot().unwrap().text.encode_utf16().collect();
        let total = text.len() as u32;
        let (a, b) = (window.0.min(total), window.1.min(total));
        let (a, b) = (a.min(b), a.max(b));
        let batch = session.live_layout(a, b).unwrap();
        let len = b - a;
        let pair = |at: u32| {
            let at = (a + at) as usize;
            at > 0 && at < text.len() && (0xDC00..0xE000).contains(&text[at])
        };
        let mut last = 0;
        for range in &batch.conceal {
            prop_assert!(range.start < range.end && range.end <= len && range.start >= last);
            last = range.end;
            let inside = &text[(a + range.start) as usize..(a + range.end) as usize];
            prop_assert!(!inside.contains(&u16::from(b'\n')));
            prop_assert!(!pair(range.start) && !pair(range.end));
        }
        for run in &batch.runs {
            prop_assert!(run.range.start < run.range.end && run.range.end <= len);
        }
        for line in &batch.lines {
            prop_assert!(line.range.start <= line.range.end && line.range.end <= len);
            prop_assert!((1..=6).contains(&line.level) && line.scale >= 1.0);
        }
    }
}

// MARK: The whole document

mod document {
    use super::*;
    use kanso_mobile::{
        LiveDocBlockKind, LiveDocLinkKind, LiveDocMarkerKind, LiveDocStyleKind, LiveDocument,
        TextChange,
    };

    fn at(text: &str, needle: &str) -> u32 {
        units(&text[..text.find(needle).unwrap()]) as u32
    }

    fn markers(doc: &LiveDocument, kind: LiveDocMarkerKind) -> Vec<(u32, String)> {
        doc.markers
            .iter()
            .filter(|marker| marker.kind == kind)
            .map(|marker| (marker.depth, marker.text.clone()))
            .collect()
    }

    #[test]
    fn bullets_and_numbers_are_the_phone_previews() {
        let phone = lone_phone();
        let body = "- a\n  - b\n    - c\n      - d\n\n1. one\n   1. sub\n   1. sub\n2. two\n\npara\n\n7. seven\n7. eight\n";
        let id = note(&phone.library, "Title", body);
        let session = phone.library.edit(&id).unwrap();
        let doc = session.live_document(Vec::new()).unwrap();
        assert_eq!(
            markers(&doc, LiveDocMarkerKind::Bullet),
            vec![
                (0, "•".into()),
                (1, "◦".into()),
                (2, "▪".into()),
                (3, "•".into())
            ]
        );
        assert_eq!(
            markers(&doc, LiveDocMarkerKind::Number),
            vec![
                (0, "1.".into()),
                (1, "1.".into()),
                (1, "2.".into()),
                (0, "2.".into()),
                (0, "7.".into()),
                (0, "8.".into())
            ]
        );
    }

    #[test]
    fn the_layout_is_in_units_of_the_whole_text() {
        let phone = lone_phone();
        let library = &phone.library;
        note(library, "Other", "x");
        let body = "# Héad 🙂\n\n> quote **b**\n\n- [x] done\n- [ ] todo\n\n```rust\nfn x() {}\n```\n\n[[Other]] [[Nowhere#Part]] [w](https://e.com) $x$\n";
        let id = note(library, "Title", body);
        let session = library.edit(&id).unwrap();
        let snapshot = session.snapshot().unwrap();
        let text = snapshot.text;
        let doc = session.live_document(Vec::new()).unwrap();
        assert_eq!(doc.version, snapshot.version);

        // The heading: its `# ` hidden, its text styled at its level.
        assert_eq!(doc.conceal[0], r(0, 2));
        let heading = doc
            .styles
            .iter()
            .find(|style| style.kind == LiveDocStyleKind::Heading)
            .unwrap();
        assert_eq!(heading.range, r(2, units("# Héad 🙂") as u32));
        assert_eq!(heading.level, 1);
        let block = &doc.blocks[0];
        assert_eq!(
            (block.kind, block.level, block.first_line, block.end_line),
            (LiveDocBlockKind::Heading, 1, 0, 1)
        );

        // The quote: a bar, and the quote's span at depth 1.
        let quote = at(&text, "> quote");
        assert!(
            doc.markers
                .iter()
                .any(|marker| marker.kind == LiveDocMarkerKind::Quote
                    && marker.range == r(quote, quote + 2)
                    && marker.depth == 1)
        );
        assert!(
            doc.styles
                .iter()
                .any(|style| style.kind == LiveDocStyleKind::Quote
                    && style.level == 1
                    && style.range.start == quote)
        );

        // Tasks: boxes, the done one's text muted.
        let tasks: Vec<_> = doc
            .markers
            .iter()
            .filter(|marker| marker.kind == LiveDocMarkerKind::Task)
            .map(|marker| (marker.range, marker.checked))
            .collect();
        let done = at(&text, "- [x]");
        let todo = at(&text, "- [ ]");
        assert_eq!(
            tasks,
            vec![(r(done, done + 6), true), (r(todo, todo + 6), false)]
        );
        assert!(
            doc.styles
                .iter()
                .any(|style| style.kind == LiveDocStyleKind::TaskDone
                    && style.range == r(done + 6, done + 10))
        );

        // The fence: a widget over its whole lines.
        assert_eq!(doc.widgets.len(), 1);
        let widget = &doc.widgets[0];
        let fence = at(&text, "```rust");
        let fence_end = at(&text, "```\n\n[[") + 3;
        assert_eq!(widget.kind, LiveDocBlockKind::Code);
        assert_eq!(widget.language.as_deref(), Some("rust"));
        assert_eq!((widget.first_line, widget.end_line), (7, 10));
        assert_eq!(widget.lines, r(fence, fence_end));
        assert_eq!(widget.source, r(fence, fence_end));

        // Links: a wiki-link to nothing is missing, one to a note isn't.
        let links: Vec<_> = doc
            .links
            .iter()
            .map(|link| {
                (
                    link.kind,
                    link.target.as_str(),
                    link.heading.as_deref(),
                    link.missing,
                )
            })
            .collect();
        assert_eq!(
            links,
            vec![
                (LiveDocLinkKind::Wiki, "Other", None, false),
                (LiveDocLinkKind::Wiki, "Nowhere", Some("Part"), true),
                (LiveDocLinkKind::Url, "https://e.com", None, false),
            ]
        );
        let other = at(&text, "[[Other]]");
        assert_eq!(doc.links[0].range, r(other, other + 9));
        assert_eq!(doc.links[0].text, r(other + 2, other + 7));
        let math = at(&text, "$x$");
        assert!(
            doc.styles
                .iter()
                .any(|style| style.kind == LiveDocStyleKind::InlineMath
                    && style.range == r(math + 1, math + 2))
        );
    }

    #[test]
    fn what_a_caret_is_on_is_shown() {
        let phone = lone_phone();
        let body = "- item **bold**\n\n```\ncode\n```";
        let id = note(&phone.library, "Title", body);
        let session = phone.library.edit(&id).unwrap();
        let text = session.snapshot().unwrap().text;
        let hidden = session.live_document(Vec::new()).unwrap();
        assert_eq!(hidden.widgets.len(), 1);
        assert_eq!(hidden.markers[0].kind, LiveDocMarkerKind::Bullet);
        assert_eq!(hidden.conceal.len(), 2);

        // A caret in the bold: its line's bullet and the bold's marks show.
        let bold = at(&text, "bold");
        let shown = session.live_document(vec![r(bold, bold)]).unwrap();
        assert!(shown.markers.iter().all(|marker| marker.line != 0));
        assert!(shown.conceal.is_empty());
        assert!(shown.blocks[0].revealed);
        assert_eq!(shown.widgets.len(), 1);

        // A selection run backwards into the fence shows its source.
        let code = at(&text, "code");
        let into = session.live_document(vec![r(code + 2, bold)]).unwrap();
        assert!(into.widgets.is_empty());
        // Out of range is cut to the text: the end, on the fence.
        let past = session.live_document(vec![r(9_999, 9_999)]).unwrap();
        assert!(past.widgets.is_empty());
    }

    #[test]
    fn the_version_follows_the_text() {
        let phone = lone_phone();
        let id = note(&phone.library, "Title", "**a**");
        let session = phone.library.edit(&id).unwrap();
        let before = session.live_document(Vec::new()).unwrap();
        let version = session.snapshot().unwrap().version;
        session
            .apply(TextChange {
                base_version: version,
                start: 0,
                end: 0,
                text: "# ".into(),
                selection: r(2, 2),
                composing: None,
            })
            .unwrap();
        let after = session.live_document(Vec::new()).unwrap();
        assert!(after.version > before.version);
        assert_eq!(after.version, session.snapshot().unwrap().version);
        assert_eq!(after.blocks[0].kind, LiveDocBlockKind::Heading);
    }

    /// Lines of the text, as UTF-16 unit ranges without their line breaks.
    fn line_ranges(text: &[u16]) -> Vec<(u32, u32)> {
        let mut lines = Vec::new();
        let mut start = 0;
        for (i, &unit) in text.iter().enumerate() {
            if unit == u16::from(b'\n') {
                lines.push((start as u32, i as u32));
                start = i + 1;
            }
        }
        lines.push((start as u32, text.len() as u32));
        lines
    }

    fn piece() -> impl Strategy<Value = &'static str> {
        prop::sample::select(vec![
            "# Head é\n",
            "para **b** _i_ ~~s~~ `c` 🙂\n",
            "- a\n  - b\n",
            "1. x\n   1. y\n",
            "- [ ] t\n- [x] d\n",
            "> q [[Note]]\n> > deep\n",
            "```js\nlet a;\n```\n",
            "| a | b |\n| - | - |\n| 1 | 2 |\n",
            "$$\nx^2\n$$\n",
            "---\n",
            "Setext\n===\n",
            "[w](https://e.com) $m$\n",
            "\n",
        ])
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(32))]

        /// Each change the text takes gives a later version, and the
        /// layout says the version it was made from.
        #[test]
        fn prop_the_version_only_grows(edits in prop::collection::vec((0u32..20, "[a-z#*\n -]{1,3}"), 1..6)) {
            let phone = lone_phone();
            let id = note(&phone.library, "Title", "- a **b**");
            let session = phone.library.edit(&id).unwrap();
            let mut last = session.live_document(Vec::new()).unwrap().version;
            for (at, insert) in edits {
                let snapshot = session.snapshot().unwrap();
                let at = at.min(units(&snapshot.text) as u32);
                let end = at + units(&insert) as u32;
                session.apply(TextChange {
                    base_version: snapshot.version,
                    start: at,
                    end: at,
                    text: insert,
                    selection: r(end, end),
                    composing: None,
                }).unwrap();
                let doc = session.live_document(vec![r(end, end)]).unwrap();
                prop_assert!(doc.version > last);
                prop_assert_eq!(doc.version, session.snapshot().unwrap().version);
                last = doc.version;
            }
        }

        /// Whatever the text and the carets: every range is in the text,
        /// the hidden ones in order, apart and each on one line; the
        /// blocks are the lines in order; a widget is whole lines; and
        /// nothing is hidden on a line a caret or selection is on.
        #[test]
        fn prop_the_document_layout_holds_together(
            pieces in prop::collection::vec(piece(), 0..10),
            reveal in prop::collection::vec((0u32..300, 0u32..300), 0..3),
        ) {
            let body: String = pieces.concat();
            let phone = lone_phone();
            let id = note(&phone.library, "Title", &body);
            let session = phone.library.edit(&id).unwrap();
            let text16: Vec<u16> = session.snapshot().unwrap().text.encode_utf16().collect();
            let total = text16.len() as u32;
            let lines = line_ranges(&text16);
            let line_of = |at: u32| lines.iter().position(|&(s, e)| at >= s && at <= e).unwrap();
            let reveal: Vec<Range16> = reveal.into_iter().map(|(a, b)| r(a, b)).collect();
            let doc = session.live_document(reveal.clone()).unwrap();
            let pair = |at: u32| {
                let at = at as usize;
                at > 0 && at < text16.len() && (0xDC00..0xE000).contains(&text16[at])
            };
            let on_line = |range: &Range16| {
                range.start < range.end
                    && range.end <= total
                    && !text16[range.start as usize..range.end as usize].contains(&u16::from(b'\n'))
                    && !pair(range.start)
                    && !pair(range.end)
            };
            // The lines a caret or selection is on.
            let mut shown = vec![false; lines.len()];
            for range in &reveal {
                let (a, b) = (range.start.min(total), range.end.min(total));
                let (a, b) = (a.min(b), a.max(b));
                shown[line_of(a)..=line_of(b)].fill(true);
            }
            let mut last = 0;
            for range in &doc.conceal {
                prop_assert!(on_line(range) && range.start >= last, "{:?}", range);
                last = range.end;
                // Never touching a caret or selection.
                for reveal in &reveal {
                    let (a, b) = (reveal.start.min(total), reveal.end.min(total));
                    let (a, b) = (a.min(b), a.max(b));
                    prop_assert!(range.end < a || range.start > b, "{:?} at {:?}", range, reveal);
                }
            }
            let mut last = 0;
            for marker in &doc.markers {
                prop_assert!(on_line(&marker.range) && marker.range.start >= last);
                last = marker.range.end;
                prop_assert_eq!(line_of(marker.range.start), marker.line as usize);
                prop_assert!(!shown[marker.line as usize]);
                match marker.kind {
                    LiveDocMarkerKind::Bullet => prop_assert_eq!(
                        marker.text.as_str(),
                        ["•", "◦", "▪"][marker.depth as usize % 3]
                    ),
                    LiveDocMarkerKind::Number => prop_assert!(marker.text.ends_with('.')
                        && marker.text[..marker.text.len() - 1].parse::<u32>().is_ok()),
                    _ => prop_assert!(marker.text.is_empty()),
                }
            }
            let mut next_line = 0;
            for block in &doc.blocks {
                prop_assert_eq!(block.first_line, next_line);
                prop_assert!(block.end_line > block.first_line);
                next_line = block.end_line;
                prop_assert!(block.source.start <= block.source.end && block.source.end <= total);
                prop_assert_eq!(
                    block.revealed,
                    (block.first_line..block.end_line).any(|line| shown[line as usize])
                );
            }
            prop_assert_eq!(next_line as usize, lines.len());
            for style in &doc.styles {
                prop_assert!(style.range.start <= style.range.end && style.range.end <= total);
            }
            prop_assert!(doc.styles.windows(2).all(|w| w[0].range.start <= w[1].range.start));
            for link in &doc.links {
                prop_assert!(link.range.start <= link.text.start && link.text.end <= link.range.end);
                prop_assert!(link.range.end <= total);
            }
            let mut last_line = 0;
            for widget in &doc.widgets {
                prop_assert!(widget.first_line >= last_line && widget.end_line > widget.first_line);
                last_line = widget.end_line;
                let first = lines[widget.first_line as usize];
                let end = lines[widget.end_line as usize - 1];
                prop_assert_eq!(widget.lines, r(first.0, end.1));
                prop_assert!(widget.lines.start <= widget.source.start
                    && widget.source.end <= widget.lines.end);
                prop_assert!((widget.first_line..widget.end_line).all(|line| !shown[line as usize]));
            }
        }
    }
}
