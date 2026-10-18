//! Where a search's words occur in a note's title or excerpt, so the list
//! can highlight them. Matching ignores case the way the search does.

use std::ops::Range;

use kanso_core::{Field, Query};

/// The words of `query` that can match `field` (the title or the body):
/// free text and that field's own clauses, but not negated ones.
pub fn highlight_terms(query: &Query, field: Field) -> Vec<String> {
    query.terms_for(field).map(str::to_owned).collect()
}

/// The byte ranges of `text` where any of `terms` occurs, ignoring case.
/// Ranges are sorted, never touch or overlap (overlapping and adjacent
/// matches are joined), and start and end on character boundaries. Empty
/// terms match nothing.
pub fn match_ranges(text: &str, terms: &[impl AsRef<str>]) -> Vec<Range<usize>> {
    let needles: Vec<Vec<char>> = terms
        .iter()
        .map(|t| fold(t.as_ref()).collect::<Vec<_>>())
        .filter(|n| !n.is_empty())
        .collect();
    if needles.is_empty() || text.is_empty() {
        return Vec::new();
    }
    // The text with each character case-folded, and for each character where
    // its folded form starts. A match must start and end at one of those.
    let mut folded = Vec::with_capacity(text.len());
    let mut starts = Vec::with_capacity(text.len() + 1);
    let mut bytes = Vec::with_capacity(text.len() + 1);
    for (byte, c) in text.char_indices() {
        starts.push(folded.len());
        bytes.push(byte);
        folded.extend(c.to_lowercase());
    }
    starts.push(folded.len());
    bytes.push(text.len());

    let mut found = Vec::new();
    for needle in &needles {
        for (ix, &start) in starts[..starts.len() - 1].iter().enumerate() {
            let end = start + needle.len();
            if folded.get(start..end) != Some(needle.as_slice()) {
                continue;
            }
            // The match must end where a character's folded form ends.
            if let Ok(last) = starts[ix + 1..].binary_search(&end) {
                found.push(bytes[ix]..bytes[ix + 1 + last]);
            }
        }
    }
    found.sort_by_key(|r| (r.start, r.end));
    let mut merged: Vec<Range<usize>> = Vec::with_capacity(found.len());
    for range in found {
        match merged.last_mut() {
            Some(last) if range.start <= last.end => last.end = last.end.max(range.end),
            _ => merged.push(range),
        }
    }
    merged
}

fn fold(s: &str) -> impl Iterator<Item = char> + '_ {
    s.chars().flat_map(char::to_lowercase)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kanso_core::query::contains_ci;
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 512,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    /// Every occurrence of `term`, found the slow way: each pair of
    /// character boundaries whose text folds to the term.
    fn occurrences(text: &str, term: &str) -> Vec<Range<usize>> {
        let want: String = fold(term).collect();
        if want.is_empty() {
            return Vec::new();
        }
        let bounds: Vec<usize> = text
            .char_indices()
            .map(|(i, _)| i)
            .chain([text.len()])
            .collect();
        let mut out = Vec::new();
        for (i, &a) in bounds.iter().enumerate() {
            for &b in &bounds[i + 1..] {
                if fold(&text[a..b]).collect::<String>() == want {
                    out.push(a..b);
                }
            }
        }
        out
    }

    /// Letters whose case forms differ in length, and a sharp s.
    fn gen_text() -> impl Strategy<Value = String> {
        "[aAbB İıßẞé-]{0,14}"
    }

    fn gen_terms() -> impl Strategy<Value = Vec<String>> {
        prop::collection::vec("[aAbBİiß ]{0,3}", 0..4)
    }

    proptest! {
        #![proptest_config(config())]

        /// Ranges are non-empty, inside the text, on character boundaries,
        /// sorted and apart.
        #[test]
        fn prop_ranges_are_sorted_disjoint_boundaries(text in gen_text(), terms in gen_terms()) {
            let ranges = match_ranges(&text, &terms);
            for r in &ranges {
                prop_assert!(r.start < r.end && r.end <= text.len());
                prop_assert!(text.is_char_boundary(r.start) && text.is_char_boundary(r.end));
            }
            for pair in ranges.windows(2) {
                prop_assert!(pair[0].end < pair[1].start);
            }
        }

        /// Exactly the occurrences: every one is covered, and every
        /// highlighted byte belongs to one.
        #[test]
        fn prop_ranges_cover_exactly_the_occurrences(text in gen_text(), terms in gen_terms()) {
            let ranges = match_ranges(&text, &terms);
            let all: Vec<Range<usize>> = terms.iter().flat_map(|t| occurrences(&text, t)).collect();
            for occ in &all {
                prop_assert!(ranges.iter().any(|r| r.start <= occ.start && occ.end <= r.end), "{:?} not in {:?}", occ, ranges);
            }
            for r in &ranges {
                for byte in r.clone() {
                    prop_assert!(all.iter().any(|o| o.contains(&byte)), "{} of {:?}", byte, r);
                }
            }
        }

        /// Case doesn't matter, in the text or the terms.
        #[test]
        fn prop_case_insensitive(text in "[a-zA-Z .-]{0,20}", terms in prop::collection::vec("[a-zA-Z]{0,3}", 0..4)) {
            let ranges = match_ranges(&text, &terms);
            prop_assert_eq!(&match_ranges(&text.to_ascii_uppercase(), &terms), &ranges);
            let upper: Vec<String> = terms.iter().map(|t| t.to_uppercase()).collect();
            prop_assert_eq!(&match_ranges(&text, &upper), &ranges);
        }

        /// A highlight shows exactly when the search would match the text.
        #[test]
        fn prop_agrees_with_the_search(text in "[a-cA-C ]{0,20}", terms in prop::collection::vec("[a-cA-C]{1,3}", 0..4)) {
            let any = terms.iter().any(|t| contains_ci(&text, t));
            prop_assert_eq!(!match_ranges(&text, &terms).is_empty(), any);
        }
    }

    #[test]
    fn highlights() {
        let spans = |text: &str, terms: &[&str]| -> Vec<(usize, usize)> {
            match_ranges(text, terms)
                .into_iter()
                .map(|r| (r.start, r.end))
                .collect()
        };
        assert_eq!(spans("Sync design", &["sync"]), [(0, 4)]);
        assert_eq!(
            spans("Weekly sync notes", &["SYNC", "notes"]),
            [(7, 11), (12, 17)]
        );
        // Overlapping and touching matches join.
        assert_eq!(spans("aaa", &["aa"]), [(0, 3)]);
        assert_eq!(spans("syncdesign", &["sync", "design"]), [(0, 10)]);
        assert_eq!(spans("Résumé", &["RÉSUMÉ"]), [(0, 8)]);
        // "İ" folds to two characters; half of it is not a match.
        assert_eq!(spans("İstanbul", &["i̇s"]), [(0, 3)]);
        assert!(spans("İstanbul", &["i"]).is_empty());
        assert!(spans("anything", &[""]).is_empty());
        assert!(spans("", &["a"]).is_empty());
    }

    #[test]
    fn terms_come_from_free_text_and_the_field() {
        let query = Query::parse(r#"sync title:"two words" body:deep -skip book:Work"#);
        assert_eq!(highlight_terms(&query, Field::Title), ["sync", "two words"]);
        assert_eq!(highlight_terms(&query, Field::Body), ["sync", "deep"]);
    }
}
