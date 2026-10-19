//! A note's outline: its headings, outside code, as the preview shows them.

use crate::links::plain_line;

/// A heading in a body.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Heading {
    /// 1 to 6.
    pub level: u8,
    /// The heading's words, without Markdown markers.
    pub text: String,
    /// The line (0-based) it is on; for a setext heading, its text's line.
    pub line: u32,
}

/// The headings of a body in order: ATX (`## Text`) and setext (text over
/// `===` or `---`), outside fenced code and front matter. Empty headings are
/// left out.
pub fn headings(body: &str) -> Vec<Heading> {
    let mut found = Vec::new();
    let mut fence: Option<(u8, usize)> = None;
    let lines: Vec<&str> = body.split('\n').collect();
    let mut start = 0;
    // Front matter: a `---` block at the very top.
    if lines.first().is_some_and(|l| l.trim_end() == "---")
        && let Some(end) = lines.iter().skip(1).position(|l| {
            let l = l.trim_end();
            l == "---" || l == "..."
        })
    {
        start = end + 2;
    }
    let mut previous_paragraph: Option<(usize, &str)> = None;
    for (ix, raw) in lines.iter().enumerate().skip(start) {
        let line = raw.trim_end_matches('\r');
        let indent = line.len() - line.trim_start_matches(' ').len();
        let trimmed = line.trim_start();
        if let Some(open) = fence_marker(trimmed) {
            fence = match fence {
                None => Some(open),
                Some((ch, len)) if open.0 == ch && open.1 >= len => None,
                other => other,
            };
            previous_paragraph = None;
            continue;
        }
        if fence.is_some() {
            continue;
        }
        if indent > 3 || trimmed.is_empty() {
            previous_paragraph = None;
            continue;
        }
        if let Some((level, text)) = atx(trimmed) {
            if !text.is_empty() {
                found.push(Heading {
                    level,
                    text,
                    line: ix as u32,
                });
            }
            previous_paragraph = None;
            continue;
        }
        if let Some(level) = setext_level(trimmed)
            && let Some((at, text)) = previous_paragraph
        {
            let text = plain_line(text);
            if !text.is_empty() {
                found.push(Heading {
                    level,
                    text,
                    line: at as u32,
                });
            }
            previous_paragraph = None;
            continue;
        }
        previous_paragraph = paragraph_start(trimmed).then_some((ix, trimmed));
    }
    found
}

/// `(level, text)` of an ATX heading line, already left-trimmed.
fn atx(trimmed: &str) -> Option<(u8, String)> {
    let hashes = trimmed.bytes().take_while(|&b| b == b'#').count();
    if !(1..=6).contains(&hashes) {
        return None;
    }
    let rest = &trimmed[hashes..];
    if !rest.is_empty() && !rest.starts_with([' ', '\t']) {
        return None;
    }
    let mut text = rest.trim();
    // A closing run of `#`s after a space.
    let closing = text.trim_end_matches('#');
    if closing.is_empty() {
        text = "";
    } else if closing.ends_with([' ', '\t']) {
        text = closing.trim_end();
    }
    Some((hashes as u8, plain_line(text)))
}

/// The level a setext underline gives the line above it.
fn setext_level(trimmed: &str) -> Option<u8> {
    let line = trimmed.trim_end();
    if !line.is_empty() && line.bytes().all(|b| b == b'=') {
        Some(1)
    } else if !line.is_empty() && line.bytes().all(|b| b == b'-') {
        Some(2)
    } else {
        None
    }
}

/// Whether a line can be the text of a setext heading: ordinary prose,
/// not a list item, quote, table row or rule.
fn paragraph_start(trimmed: &str) -> bool {
    let first = trimmed.as_bytes()[0];
    if matches!(first, b'>' | b'|' | b'<') {
        return false;
    }
    if matches!(first, b'-' | b'*' | b'+') && trimmed[1..].starts_with([' ', '\t']) {
        return false;
    }
    if setext_level(trimmed).is_some() {
        return false;
    }
    let digits = trimmed.bytes().take_while(u8::is_ascii_digit).count();
    !(digits > 0 && trimmed[digits..].starts_with(['.', ')']))
}

fn fence_marker(trimmed: &str) -> Option<(u8, usize)> {
    let first = *trimmed.as_bytes().first()?;
    if first != b'`' && first != b'~' {
        return None;
    }
    let run = trimmed.bytes().take_while(|&b| b == first).count();
    (run >= 3).then_some((first, run))
}

/// The heading of `headings` that `wanted` names, by its words without
/// regard to case or spacing.
pub fn find_heading<'a>(headings: &'a [Heading], wanted: &str) -> Option<&'a Heading> {
    let wanted = crate::links::normalize_title(wanted);
    headings
        .iter()
        .find(|h| crate::links::normalize_title(&h.text) == wanted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn h(level: u8, text: &str, line: u32) -> Heading {
        Heading {
            level,
            text: text.into(),
            line,
        }
    }

    #[test]
    fn finds_atx_and_setext_headings_outside_code() {
        let body = "---\ntitle: x\n---\n# One\ntext\n## **Two** ##\n```\n# not\n```\nThree\n=====\n- item\n---\n####### seven\n#tag\n   ### Four\n";
        assert_eq!(
            headings(body),
            vec![
                h(1, "One", 3),
                h(2, "Two", 5),
                h(1, "Three", 9),
                h(3, "Four", 15)
            ]
        );
    }

    #[test]
    fn finds_headings_by_name() {
        let list = headings("# Getting Started\n## Set up\n");
        assert_eq!(find_heading(&list, "set  UP").map(|h| h.line), Some(1));
        assert_eq!(find_heading(&list, "nope"), None);
    }

    proptest! {
        /// Model-based: a body built from known headings and prose lines has
        /// exactly those headings, at their lines, in order.
        #[test]
        fn prop_written_headings_are_found(blocks in prop::collection::vec((prop::option::of(1u8..=6), "[A-Za-z][A-Za-z ]{0,10}[a-z]"), 0..12)) {
            let mut body = String::new();
            let mut expected = Vec::new();
            for (ix, (level, text)) in blocks.iter().enumerate() {
                match level {
                    Some(level) => {
                        expected.push(h(*level, &text.split_whitespace().collect::<Vec<_>>().join(" "), (ix * 2) as u32));
                        body.push_str(&format!("{} {text}\n\n", "#".repeat(*level as usize)));
                    }
                    None => body.push_str(&format!("{text}\n\n")),
                }
            }
            prop_assert_eq!(headings(&body), expected);
        }

        /// Invariants: levels are 1 to 6, lines increase, text is never
        /// empty, whatever the text.
        #[test]
        fn prop_headings_are_well_formed(body in "[#=\\-`a-z \n*]{0,120}") {
            let found = headings(&body);
            let mut last = None;
            for heading in &found {
                prop_assert!((1..=6).contains(&heading.level));
                prop_assert!(!heading.text.is_empty());
                prop_assert!(last.is_none_or(|l| heading.line > l));
                last = Some(heading.line);
            }
        }
    }
}
