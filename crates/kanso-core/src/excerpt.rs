//! The plain-text excerpt shown under a note's title in the note list.
//!
//! Most notes start with a heading that repeats their title, and Markdown
//! markers read as noise in a two-line preview, so the excerpt skips front
//! matter, code blocks, table rules and a leading title heading, and keeps
//! only the words of everything else.

/// The longest excerpt, in characters.
pub const EXCERPT_CHARS: usize = 200;

/// The note's body as one line of plain text, at most [`EXCERPT_CHARS`]
/// characters, without a first heading that repeats `title`.
pub fn excerpt(title: &str, body: &str) -> String {
    let mut out = String::new();
    let mut lines = body.lines().peekable();

    // YAML front matter.
    if lines.peek().is_some_and(|l| l.trim_end() == "---") {
        lines.next();
        for line in lines.by_ref() {
            if matches!(line.trim_end(), "---" | "...") {
                break;
            }
        }
    }

    let mut fence: Option<&str> = None;
    let mut seen_text = false;
    for line in lines {
        let trimmed = line.trim();
        if let Some(marker) = fence {
            if trimmed.starts_with(marker) {
                fence = None;
            }
            continue;
        }
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            fence = Some(&trimmed[..3]);
            continue;
        }
        if trimmed.is_empty() || is_rule(trimmed) {
            continue;
        }
        let (is_heading, text) = strip_block_markers(trimmed);
        let plain = strip_inline(text);
        if plain.is_empty() {
            continue;
        }
        if !seen_text && is_heading && same_words(&plain, title) {
            seen_text = true;
            continue;
        }
        seen_text = true;
        for word in plain.split_whitespace() {
            if !out.is_empty() {
                out.push(' ');
            }
            out.push_str(word);
        }
        if out.chars().count() >= EXCERPT_CHARS {
            break;
        }
    }
    truncate(&out, EXCERPT_CHARS)
}

/// A thematic break or a table's header rule, such as `---` or `|:--|--:|`.
fn is_rule(line: &str) -> bool {
    let marks = line
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>();
    marks.len() >= 3
        && (marks.chars().all(|c| c == '-')
            || marks.chars().all(|c| c == '*')
            || marks.chars().all(|c| c == '_')
            || (marks.contains('-') && marks.chars().all(|c| matches!(c, '|' | '-' | ':'))))
}

/// Removes heading, quote, list and task markers. Returns whether the line
/// was a heading.
fn strip_block_markers(line: &str) -> (bool, &str) {
    let mut rest = line;
    let mut heading = false;
    loop {
        let before = rest;
        if let Some(after) = rest.strip_prefix('>') {
            rest = after.trim_start();
        }
        let hashes = rest.chars().take_while(|&c| c == '#').count();
        if (1..=6).contains(&hashes) && rest[hashes..].starts_with([' ', '\t']) {
            heading = true;
            rest = rest[hashes..].trim_start();
        }
        for marker in ["- ", "* ", "+ "] {
            if let Some(after) = rest.strip_prefix(marker) {
                rest = after.trim_start();
            }
        }
        let digits = rest.chars().take_while(char::is_ascii_digit).count();
        if digits > 0
            && digits <= 9
            && (rest[digits..].starts_with(". ") || rest[digits..].starts_with(") "))
        {
            rest = rest[digits + 2..].trim_start();
        }
        for task in ["[ ] ", "[x] ", "[X] "] {
            if let Some(after) = rest.strip_prefix(task) {
                rest = after.trim_start();
            }
        }
        if rest == before {
            break;
        }
    }
    // Closing hashes of an ATX heading.
    if heading {
        rest = rest.trim_end_matches('#').trim_end();
    }
    (heading, rest)
}

/// Keeps the readable text of inline Markdown: link and image text, code
/// contents, words without emphasis markers, no HTML tags or table pipes.
fn strip_inline(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        match c {
            // An image keeps nothing; a link keeps its text.
            '!' if chars.get(i + 1) == Some(&'[') => {
                if let Some(end) = link_end(&chars, i + 1) {
                    i = end;
                    continue;
                }
                out.push(c);
            }
            '[' => {
                // [[wiki link]]
                if chars.get(i + 1) == Some(&'[')
                    && let Some(close) = find(&chars, i + 2, &[']', ']'])
                {
                    out.extend(&chars[i + 2..close]);
                    out.push(' ');
                    i = close + 2;
                    continue;
                }
                if let Some(end) = link_end(&chars, i) {
                    let close = find(&chars, i + 1, &[']']).unwrap_or(i);
                    out.push_str(&strip_inline(
                        &chars[i + 1..close].iter().collect::<String>(),
                    ));
                    i = end;
                    continue;
                }
                out.push(c);
            }
            '<' => match chars[i + 1..].iter().position(|&c| c == '>') {
                Some(len) if looks_like_tag(&chars[i + 1..i + 1 + len]) => {
                    out.push(' ');
                    i += len + 2;
                    continue;
                }
                _ => out.push(c),
            },
            '`' | '|' => out.push(' '),
            '*' | '_' | '~' => {
                // Emphasis markers sit at a word's edge; inside a word
                // (snake_case, 2*3) they are text.
                let prev = i.checked_sub(1).map(|p| chars[p]);
                let next = chars.get(i + 1).copied();
                let inside = prev.is_some_and(char::is_alphanumeric)
                    && next.is_some_and(char::is_alphanumeric);
                if inside {
                    out.push(c);
                } else {
                    out.push(' ');
                }
            }
            '\\' if chars.get(i + 1).is_some_and(|n| n.is_ascii_punctuation()) => {
                // An escaped marker is still noise in a preview.
                let escaped = chars[i + 1];
                out.push(if "`|*_~".contains(escaped) {
                    ' '
                } else {
                    escaped
                });
                i += 2;
                continue;
            }
            c if c.is_control() => out.push(' '),
            c => out.push(c),
        }
        i += 1;
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The index after `[text](target)` starting at `open`, if it is a link.
fn link_end(chars: &[char], open: usize) -> Option<usize> {
    let close = find(chars, open + 1, &[']'])?;
    if chars.get(close + 1) != Some(&'(') {
        return None;
    }
    let end = find(chars, close + 2, &[')'])?;
    Some(end + 1)
}

fn find(chars: &[char], from: usize, needle: &[char]) -> Option<usize> {
    (from..chars.len().saturating_sub(needle.len() - 1))
        .find(|&i| chars[i..i + needle.len()] == *needle)
}

/// `<b>`, `</div>`, `<img src="…">`, `<br/>`: a name right after the bracket.
fn looks_like_tag(inner: &[char]) -> bool {
    let inner = inner.strip_prefix(&['/']).unwrap_or(inner);
    inner.first().is_some_and(char::is_ascii_alphabetic)
        && inner
            .iter()
            .take_while(|c| !c.is_whitespace() && **c != '/')
            .all(|c| c.is_ascii_alphanumeric() || *c == '-')
}

/// Whether two texts have the same words, ignoring case and punctuation.
fn same_words(a: &str, b: &str) -> bool {
    let words = |s: &str| -> Vec<String> {
        s.split(|c: char| !c.is_alphanumeric())
            .filter(|w| !w.is_empty())
            .map(str::to_lowercase)
            .collect()
    };
    let (a, b) = (words(a), words(b));
    !a.is_empty() && a == b
}

fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_owned();
    }
    let cut: String = text.chars().take(max).collect();
    // End on a word boundary when one is reasonably close.
    match cut.rfind(' ') {
        Some(at) if at > max / 2 => cut[..at].to_owned(),
        _ => cut,
    }
}

#[cfg(test)]
mod tests {
    use super::{EXCERPT_CHARS, excerpt, strip_inline};
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 512,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    #[test]
    fn examples() {
        assert_eq!(
            excerpt(
                "Reading list",
                "# Reading list\n\n1. *The Design of Everyday Things*\n2. **Shape Up**\n"
            ),
            "The Design of Everyday Things Shape Up"
        );
        assert_eq!(
            excerpt(
                "Weekly sync",
                "# Weekly sync\n\n| Topic | Owner |\n|---|---|\n| Release | Ngoni |"
            ),
            "Topic Owner Release Ngoni"
        );
        assert_eq!(
            excerpt(
                "x",
                "---\ntitle: x\n---\nSee [the docs](https://e.com/a_b) and ![shot](inkdrop://file:abc)."
            ),
            "See the docs and ."
        );
        assert_eq!(
            excerpt(
                "Plan",
                "## Goals\n```rust\nfn main() {}\n```\n- [x] Ship `v1`\n> quoted <b>bold</b>"
            ),
            "Goals Ship v1 quoted bold"
        );
        // A heading that isn't the title stays; snake_case and 2*3 stay.
        assert_eq!(
            excerpt("Other", "# Heading\nuse snake_case and 2*3"),
            "Heading use snake_case and 2*3"
        );
        assert_eq!(excerpt("T", ""), "");
        assert_eq!(excerpt("T", "# T"), "");
        assert_eq!(excerpt("a < b", "a < b > c"), "a < b > c");
    }

    fn gen_word() -> impl Strategy<Value = String> {
        "[A-Za-z0-9]{1,8}"
    }

    fn gen_markdown() -> impl Strategy<Value = String> {
        let line = prop_oneof![
            prop::collection::vec(gen_word(), 1..6).prop_map(|w| w.join(" ")),
            prop::collection::vec(gen_word(), 1..4).prop_map(|w| format!("# {}", w.join(" "))),
            gen_word().prop_map(|w| format!("- [ ] **{w}**")),
            (gen_word(), gen_word()).prop_map(|(a, b)| format!("[{a}](https://x.y/{b})")),
            gen_word().prop_map(|w| format!("> _{w}_ `code`")),
            Just("|---|:-:|".to_owned()),
            Just("```\nfn hidden() {}\n```".to_owned()),
            "\\PC{0,30}",
        ];
        prop::collection::vec(line, 0..12).prop_map(|lines| lines.join("\n"))
    }

    proptest! {
        #![proptest_config(config())]

        /// Refinement: an excerpt is one trimmed line, short, and free of
        /// the Markdown markers that read as noise.
        #[test]
        fn prop_excerpts_are_short_plain_lines(title in "\\PC{0,20}", body in gen_markdown()) {
            let text = excerpt(&title, &body);
            prop_assert!(text.chars().count() <= EXCERPT_CHARS);
            prop_assert!(!text.contains(['\n', '\r', '`', '|']));
            prop_assert!(!text.contains("**") && !text.contains("]("));
            prop_assert_eq!(text.trim(), text.as_str());
            prop_assert!(!text.contains("  "));
        }

        /// Metamorphic: a first heading repeating the title adds nothing.
        #[test]
        fn prop_a_title_heading_is_skipped(words in prop::collection::vec(gen_word(), 1..4), body in gen_markdown()) {
            let title = words.join(" ");
            let with_heading = format!("# {title}\n\n{body}");
            prop_assume!(!body.trim_start().starts_with('#') && !body.trim_start().starts_with("---"));
            prop_assert_eq!(excerpt(&title, &with_heading), excerpt(&title, &body));
        }

        /// Invariant: the excerpt invents nothing; every word in it is in
        /// the body.
        #[test]
        fn prop_excerpt_words_come_from_the_body(body in gen_markdown()) {
            let text = excerpt("", &body);
            for word in text.split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()) {
                prop_assert!(body.contains(word), "{} not in body", word);
            }
            prop_assert!(!text.contains("hidden"), "code block content leaked");
        }

        /// Oracle: plain words come through as they are, joined by single
        /// spaces.
        #[test]
        fn prop_plain_text_is_kept(words in prop::collection::vec(gen_word(), 1..20)) {
            let body = words.join("  \n ");
            let expected = words.join(" ");
            prop_assume!(expected.chars().count() <= EXCERPT_CHARS);
            prop_assert_eq!(excerpt("", &body), expected);
        }

        /// Idempotence: stripping inline Markdown twice changes nothing more.
        /// Backslash escapes are decoded once by design (`\\{` becomes `\{`), so
        /// they are left out.
        #[test]
        fn prop_strip_inline_is_idempotent(text in "[^\\\\]{0,60}") {
            let once = strip_inline(&text);
            prop_assert_eq!(strip_inline(&once), once);
        }
    }
}
