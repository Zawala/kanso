//! Counts the editor's footer shows: words in a note and numbers with
//! thousands separators.

/// The words in Markdown text: runs between whitespace with at least one
/// letter or digit in them. Markup on its own, like `#`, `-`, `|` or a
/// table's `---`, isn't a word; `**bold**` and `[a](b)` are one each.
pub fn word_count(text: &str) -> usize {
    text.split_whitespace()
        .filter(|token| token.chars().any(char::is_alphanumeric))
        .count()
}

/// `n` with a comma between each group of three digits: 2,841.
pub fn format_count(n: usize) -> String {
    let digits = n.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (ix, digit) in digits.chars().enumerate() {
        if ix > 0 && (digits.len() - ix).is_multiple_of(3) {
            out.push(',');
        }
        out.push(digit);
    }
    out
}

/// "1 word" or "2,841 words".
pub fn plural(n: usize, one: &str, many: &str) -> String {
    format!("{} {}", format_count(n), if n == 1 { one } else { many })
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 512,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    /// Text that mixes words, markup and every kind of whitespace.
    fn text() -> impl Strategy<Value = String> {
        "([a-zé0-9#*|>`\\[\\]()-]{0,6}[ \t\n\u{a0}]{0,3}){0,12}"
    }

    proptest! {
        #![proptest_config(config())]

        /// Additivity: two texts joined by a space have the words of both.
        #[test]
        fn prop_word_count_adds_up(a in text(), b in text()) {
            prop_assert_eq!(word_count(&format!("{a} {b}")), word_count(&a) + word_count(&b));
        }

        /// Whitespace doesn't count: any run of it can stand in for any
        /// other without changing the count.
        #[test]
        fn prop_word_count_ignores_the_kind_of_whitespace(
            tokens in prop::collection::vec("[a-z#|*-]{1,5}", 0..12),
            gaps in prop::collection::vec("[ \t\n\r]{1,4}", 12),
        ) {
            let spaced = tokens.join(" ");
            let mixed: String = tokens.iter().zip(&gaps).map(|(t, g)| format!("{t}{g}")).collect();
            prop_assert_eq!(word_count(&spaced), word_count(&mixed));
        }

        /// Bound: words are a subset of the whitespace-separated runs, and
        /// runs of markup alone never count.
        #[test]
        fn prop_word_count_skips_markup(text in text(), markup in prop::collection::vec("[#|*>`-]{1,4}", 0..10)) {
            prop_assert!(word_count(&text) <= text.split_whitespace().count());
            prop_assert_eq!(word_count(&markup.join(" ")), 0);
            prop_assert_eq!(word_count(&format!("{} {text}", markup.join("\n"))), word_count(&text));
        }

        /// Round trip and shape: without its commas the text is the number,
        /// and the commas split it into groups of three after a first group
        /// of one to three digits.
        #[test]
        fn prop_format_count_groups_by_three(n in any::<usize>()) {
            let shown = format_count(n);
            prop_assert_eq!(shown.replace(',', "").parse::<usize>().unwrap(), n);
            let groups: Vec<&str> = shown.split(',').collect();
            prop_assert!((1..=3).contains(&groups[0].len()), "{}", shown);
            prop_assert!(groups[1..].iter().all(|g| g.len() == 3), "{}", shown);
            prop_assert!(groups[0] == "0" || !groups[0].starts_with('0'), "{}", shown);
        }
    }

    #[test]
    fn word_count_examples() {
        assert_eq!(word_count(""), 0);
        assert_eq!(
            word_count("# MCP support model\n\n- Enable **secure** access."),
            6
        );
        assert_eq!(word_count("| a | b |\n|---|---|"), 2);
        assert_eq!(
            word_count("[protocol overview](https://example.com) here"),
            3
        );
    }

    #[test]
    fn format_count_examples() {
        assert_eq!(format_count(0), "0");
        assert_eq!(format_count(999), "999");
        assert_eq!(format_count(2841), "2,841");
        assert_eq!(format_count(1_234_567), "1,234,567");
        assert_eq!(plural(1, "word", "words"), "1 word");
        assert_eq!(plural(2841, "char", "chars"), "2,841 chars");
    }
}
