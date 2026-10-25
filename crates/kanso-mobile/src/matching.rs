//! Where search words matched in a title or a snippet, as ranges in the
//! units Flutter counts text in (UTF-16 code units).

use kanso_core::query::find_ci;
use kanso_engine::text::byte_to_utf16;

use crate::model::Range16;

/// Where any of `terms` occurs in `text`, ignoring case: sorted ranges that
/// don't overlap (overlapping matches merge), in UTF-16 units of `text`. Empty terms match
/// nothing.
pub fn ranges(text: &str, terms: &[&str]) -> Vec<Range16> {
    let mut found: Vec<(usize, usize)> = Vec::new();
    for term in terms.iter().filter(|term| !term.is_empty()) {
        let folded: String = term.chars().flat_map(char::to_lowercase).collect();
        let mut from = 0;
        while let Some(at) = text.get(from..).and_then(|rest| find_ci(rest, term)) {
            let start = from + at;
            let end = start + matched_len(&text[start..], &folded);
            found.push((start, end));
            from = end.max(start + 1);
            while !text.is_char_boundary(from) && from < text.len() {
                from += 1;
            }
        }
    }
    found.sort_unstable();
    let mut merged: Vec<(usize, usize)> = Vec::new();
    for (start, end) in found {
        match merged.last_mut() {
            Some(last) if start < last.1 => last.1 = last.1.max(end),
            _ => merged.push((start, end)),
        }
    }
    merged
        .into_iter()
        .map(|(start, end)| Range16 {
            start: byte_to_utf16(text, start) as u32,
            end: byte_to_utf16(text, end) as u32,
        })
        .collect()
}

/// How many bytes of `rest` the lowercased `needle` covers, character by
/// character, as `find_ci` folds them.
fn matched_len(rest: &str, needle: &str) -> usize {
    let mut folded = 0;
    for (at, c) in rest.char_indices() {
        folded += c.to_lowercase().map(char::len_utf8).sum::<usize>();
        if folded >= needle.len() {
            return at + c.len_utf8();
        }
    }
    rest.len()
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    /// The UTF-16 units of `text` in `range`.
    fn slice(text: &str, range: Range16) -> String {
        let units: Vec<u16> = text.encode_utf16().collect();
        String::from_utf16(&units[range.start as usize..range.end as usize]).unwrap()
    }

    proptest! {
        /// Every range lies inside the text, is not empty, and the ranges
        /// are sorted and don't overlap, for any text and any terms.
        #[test]
        fn prop_ranges_are_inside_sorted_and_disjoint(
            text in "\\PC{0,60}",
            terms in proptest::collection::vec("\\PC{0,4}", 0..4),
        ) {
            let terms: Vec<&str> = terms.iter().map(String::as_str).collect();
            let units = text.encode_utf16().count() as u32;
            let found = ranges(&text, &terms);
            let mut previous_end = None;
            for range in &found {
                prop_assert!(range.start < range.end);
                prop_assert!(range.end <= units);
                if let Some(end) = previous_end {
                    prop_assert!(range.start >= end);
                }
                previous_end = Some(range.end);
                // The slice is whole characters: it decodes.
                let _ = slice(&text, *range);
            }
        }

        /// With plain letters, what a range covers is the term, whatever its
        /// case, and every occurrence is found.
        #[test]
        fn prop_ascii_ranges_cover_the_term_and_every_occurrence(
            words in proptest::collection::vec("[a-zA-Z]{1,5}", 1..8),
            term in "[a-z]{1,3}",
        ) {
            let text = words.join(" ");
            let found = ranges(&text, &[&term]);
            let lower = text.to_lowercase();
            // Occurrences that don't overlap another, counted left to right.
            let mut expected = 0;
            let mut from = 0;
            while let Some(at) = lower[from..].find(&term) {
                expected += 1;
                from += at + term.len();
            }
            for range in &found {
                prop_assert_eq!(slice(&text, *range).to_lowercase(), term.clone());
            }
            prop_assert_eq!(found.len(), expected);
        }
    }

    // no-prop-test: a few fixed texts with wide characters and overlap.
    #[test]
    fn examples_in_utf16_units() {
        // "é" is one unit, the emoji two.
        let text = "Café 🙂 café";
        let found = ranges(text, &["CAFÉ"]);
        assert_eq!(
            found,
            vec![Range16 { start: 0, end: 4 }, Range16 { start: 8, end: 12 }]
        );
        // Overlapping matches merge.
        assert_eq!(
            ranges("aaaa", &["aa", "aaa"]),
            vec![Range16 { start: 0, end: 4 }]
        );
        assert_eq!(ranges("abc", &[""]), vec![]);
    }
}
