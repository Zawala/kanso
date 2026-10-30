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
