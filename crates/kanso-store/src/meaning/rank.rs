//! The rules the meaning features follow, apart from the store: which
//! notes a search by meaning adds to the text matches, which tags the
//! nearest notes suggest, when two notes count as the same, and how two
//! notes are merged without losing a word.

use std::collections::{HashMap, HashSet};

use crate::RelatedNote;

/// The least similarity for a note to be listed as about a search. Below
/// it the Related tab's words say "Loosely related"; a search list of
/// those is noise.
pub const MEANING_SIMILARITY: f32 = 0.35;

/// The least similarity for two notes to count as saying the same thing.
///
/// The model averages one fixed vector per word piece, so two notes reach
/// this only when nearly all their words are the same ones, in about the
/// same amounts: a copy, a copy with a line or two changed, or the same
/// list written twice. Notes that are merely about the same subject share
/// many words but not their proportions, and land lower (the Related tab
/// calls 0.75 and up "Very close"). 0.95 keeps the warning for notes that
/// are worth merging, and a false warning costs a glance.
pub const DUPLICATE_SIMILARITY: f32 = 0.95;

/// The least similarity for a near note's tags to be suggested: "Close",
/// as the Related tab says it, or nearly.
pub const TAG_NEIGHBOUR_SIMILARITY: f32 = 0.5;

/// The notes found by meaning that a search lists after its text matches:
/// those at least `min` similar, not already among `text_ids`, best first,
/// at most `limit`. Pure.
pub fn blend_meaning(
    text_ids: &[&str],
    found: Vec<RelatedNote>,
    min: f32,
    limit: usize,
) -> Vec<RelatedNote> {
    let listed: HashSet<&str> = text_ids.iter().copied().collect();
    let mut kept: Vec<RelatedNote> = found
        .into_iter()
        .filter(|note| note.score >= min && !listed.contains(note.id.as_str()))
        .collect();
    kept.sort_by(|a, b| b.score.total_cmp(&a.score).then_with(|| a.id.cmp(&b.id)));
    let mut seen = HashSet::new();
    kept.retain(|note| seen.insert(note.id.clone()));
    kept.truncate(limit);
    kept
}

/// Tags suggested for a note from its near notes (`neighbours`: each one's
/// similarity and tag IDs): every tag of a near note at least
/// [`TAG_NEIGHBOUR_SIMILARITY`] similar, weighed by the similarities of the
/// near notes that have it, added up. Tags in `own` are never suggested.
/// Heaviest first, ties by ID, at most `limit`. Pure.
pub fn suggest_tags(
    neighbours: &[(f32, Vec<String>)],
    own: &[String],
    limit: usize,
) -> Vec<(String, f32)> {
    let own: HashSet<&str> = own.iter().map(String::as_str).collect();
    let mut weights: HashMap<&str, f32> = HashMap::new();
    for (score, tags) in neighbours {
        // NaN, or not close enough.
        if score
            .partial_cmp(&TAG_NEIGHBOUR_SIMILARITY)
            .is_none_or(|o| o.is_lt())
        {
            continue;
        }
        let unique: HashSet<&str> = tags.iter().map(String::as_str).collect();
        for tag in unique {
            if !own.contains(tag) {
                *weights.entry(tag).or_default() += score;
            }
        }
    }
    let mut suggested: Vec<(String, f32)> = weights
        .into_iter()
        .map(|(tag, weight)| (tag.to_owned(), weight))
        .collect();
    suggested.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    suggested.truncate(limit);
    suggested
}

/// The body of a note merged with another: its own words, then the
/// other's under the other's title as a heading (unless the other's body
/// already starts with that title, or it has none). Nothing of either is
/// left out. Pure.
pub fn merged_body(body: &str, other_title: &str, other_body: &str) -> String {
    let mine = body.trim_end();
    let theirs = other_body.trim_matches(['\n', '\r']).trim_end();
    let title = other_title.trim();
    let first_line = theirs.lines().next().unwrap_or("");
    let titled = first_line.trim_start_matches('#').trim() == title && first_line.starts_with('#');
    let mut out = String::with_capacity(mine.len() + title.len() + theirs.len() + 8);
    out.push_str(mine);
    let mut part = |text: &str| {
        if text.is_empty() {
            return;
        }
        if !out.is_empty() {
            out.push_str("\n\n");
        }
        out.push_str(text);
    };
    if !title.is_empty() && !titled {
        part(&format!("## {title}"));
    }
    part(theirs);
    if !out.is_empty() {
        out.push('\n');
    }
    out
}

/// A note's tags merged with another's: its own, then the other's it
/// hasn't, in order. Pure.
pub fn merged_tags(tags: &[String], other: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::with_capacity(tags.len() + other.len());
    for tag in tags.iter().chain(other) {
        if !out.contains(tag) {
            out.push(tag.clone());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn hit(id: &str, score: f32) -> RelatedNote {
        RelatedNote {
            id: id.into(),
            title: id.into(),
            score,
        }
    }

    #[test]
    fn meaning_hits_follow_the_text_matches() {
        let found = vec![hit("note:a", 0.9), hit("note:b", 0.6), hit("note:c", 0.2)];
        let kept = blend_meaning(&["note:a"], found, MEANING_SIMILARITY, 5);
        assert_eq!(kept, vec![hit("note:b", 0.6)]);
    }

    #[test]
    fn tags_come_from_close_notes_by_weight() {
        let s = |v: &[&str]| v.iter().map(|t| (*t).to_owned()).collect::<Vec<_>>();
        let neighbours = vec![
            (0.9, s(&["tag:work", "tag:plans"])),
            (0.7, s(&["tag:work", "tag:home"])),
            (0.3, s(&["tag:far"])),
        ];
        let suggested = suggest_tags(&neighbours, &s(&["tag:plans"]), 5);
        let names: Vec<&str> = suggested.iter().map(|(t, _)| t.as_str()).collect();
        assert_eq!(names, ["tag:work", "tag:home"]);
        assert!((suggested[0].1 - 1.6).abs() < 1e-6);
    }

    #[test]
    fn merging_keeps_both_texts() {
        assert_eq!(
            merged_body("Mine\n", "Theirs", "Their words\n"),
            "Mine\n\n## Theirs\n\nTheir words\n"
        );
        // The other note already starts with its title.
        assert_eq!(
            merged_body("Mine", "Theirs", "# Theirs\nWords"),
            "Mine\n\n# Theirs\nWords\n"
        );
        assert_eq!(merged_body("", "", "Words"), "Words\n");
        assert_eq!(merged_body("Mine", "Theirs", ""), "Mine\n\n## Theirs\n");
        let s = |v: &[&str]| v.iter().map(|t| (*t).to_owned()).collect::<Vec<_>>();
        assert_eq!(
            merged_tags(&s(&["a", "b"]), &s(&["b", "c"])),
            s(&["a", "b", "c"])
        );
    }

    fn config(cases: u32) -> ProptestConfig {
        ProptestConfig {
            cases,
            failure_persistence: Some(Box::new(
                proptest::test_runner::FileFailurePersistence::WithSource("regressions"),
            )),
            ..ProptestConfig::default()
        }
    }

    fn neighbours() -> impl Strategy<Value = Vec<(f32, Vec<String>)>> {
        prop::collection::vec((-1f32..1., prop::collection::vec("tag:[a-f]", 0..4)), 0..10)
    }

    proptest! {
        #![proptest_config(config(256))]

        /// Meaning hits never repeat a text match or each other, are all
        /// similar enough, best first, at most `limit`, and none left out
        /// beats one kept.
        #[test]
        fn prop_blend_adds_only_new_close_notes_best_first(
            text in prop::collection::vec(0usize..12, 0..6),
            found in prop::collection::vec((0usize..12, -1f32..1.), 0..16),
            limit in 0usize..8,
        ) {
            let text: Vec<String> = text.iter().map(|i| format!("note:{i}")).collect();
            let text_ids: Vec<&str> = text.iter().map(String::as_str).collect();
            let found: Vec<RelatedNote> = found.iter().map(|(i, s)| hit(&format!("note:{i}"), *s)).collect();
            let kept = blend_meaning(&text_ids, found.clone(), MEANING_SIMILARITY, limit);
            prop_assert!(kept.len() <= limit);
            let mut ids = HashSet::new();
            for note in &kept {
                prop_assert!(!text_ids.contains(&note.id.as_str()));
                prop_assert!(note.score >= MEANING_SIMILARITY);
                prop_assert!(ids.insert(note.id.clone()));
            }
            for pair in kept.windows(2) {
                prop_assert!(pair[0].score >= pair[1].score);
            }
            if kept.len() == limit {
                if let Some(worst) = kept.last() {
                    for note in &found {
                        if !ids.contains(&note.id) && !text_ids.contains(&note.id.as_str()) {
                            prop_assert!(note.score <= worst.score);
                        }
                    }
                }
            } else {
                // Room to spare: every eligible note is there.
                for note in &found {
                    if note.score >= MEANING_SIMILARITY && !text_ids.contains(&note.id.as_str()) {
                        prop_assert!(ids.contains(&note.id));
                    }
                }
            }
        }

        /// A suggestion is never a tag the note has, never repeats, comes
        /// from a close enough near note, and the list is heaviest first.
        #[test]
        fn prop_suggestions_are_new_tags_heaviest_first(
            neighbours in neighbours(),
            own in prop::collection::vec("tag:[a-f]", 0..3),
            limit in 0usize..8,
        ) {
            let suggested = suggest_tags(&neighbours, &own, limit);
            prop_assert!(suggested.len() <= limit);
            let mut seen = HashSet::new();
            for (tag, weight) in &suggested {
                prop_assert!(!own.contains(tag));
                prop_assert!(seen.insert(tag.clone()));
                prop_assert!(*weight >= TAG_NEIGHBOUR_SIMILARITY);
                prop_assert!(neighbours.iter().any(|(s, tags)| *s >= TAG_NEIGHBOUR_SIMILARITY && tags.contains(tag)));
            }
            for pair in suggested.windows(2) {
                prop_assert!(pair[0].1 > pair[1].1 || (pair[0].1 == pair[1].1 && pair[0].0 < pair[1].0));
            }
        }

        /// Ordering follows similarity: a near note moving closer never
        /// makes any of its tags lighter, and a tag every near note with
        /// another tag also has weighs at least as much as that one.
        #[test]
        fn prop_suggestions_are_monotone_in_similarity(
            neighbours in neighbours(),
            own in prop::collection::vec("tag:[a-f]", 0..3),
            which in 0usize..10,
            closer in 0f32..1.,
        ) {
            let weights = |n: &[(f32, Vec<String>)]| -> HashMap<String, f32> {
                suggest_tags(n, &own, usize::MAX).into_iter().collect()
            };
            let before = weights(&neighbours);
            if !neighbours.is_empty() {
                let mut moved = neighbours.clone();
                let i = which % moved.len();
                moved[i].0 = (moved[i].0 + closer).min(1.);
                let after = weights(&moved);
                for (tag, weight) in &before {
                    prop_assert!(after.get(tag).copied().unwrap_or(0.) >= *weight - 1e-5);
                }
            }
            for (a, wa) in &before {
                for (b, wb) in &before {
                    let covers = neighbours.iter().all(|(_, tags)| !tags.contains(b) || tags.contains(a));
                    if covers {
                        prop_assert!(wa >= &(wb - 1e-5), "{a} {wa} {b} {wb}");
                    }
                }
            }
        }

        /// Merging loses no text: the merged body starts with the note's
        /// own words and holds the other's words and title, and the tags
        /// are both notes' tags, each once.
        #[test]
        fn prop_merging_loses_no_text(
            body in "[a-z #\n-]{0,40}",
            title in "[A-Za-z ]{0,12}",
            other in "[a-z #\n-]{0,40}",
            tags in prop::collection::vec("[a-c]", 0..4),
            other_tags in prop::collection::vec("[a-e]", 0..4),
        ) {
            let merged = merged_body(&body, &title, &other);
            prop_assert!(merged.starts_with(body.trim_end()));
            let theirs = other.trim();
            prop_assert!(merged.contains(theirs));
            prop_assert!(merged.contains(title.trim()));
            for word in body.split_whitespace().chain(other.split_whitespace()) {
                prop_assert!(merged.contains(word));
            }
            let all = merged_tags(&tags, &other_tags);
            for tag in tags.iter().chain(&other_tags) {
                prop_assert!(all.contains(tag));
            }
            let unique: HashSet<&String> = all.iter().collect();
            prop_assert_eq!(unique.len(), all.len());
            prop_assert!(all.starts_with(&merged_tags(&tags, &[])));
        }
    }
}
