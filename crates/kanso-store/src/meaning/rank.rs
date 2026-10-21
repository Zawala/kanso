//! The rules the meaning features follow, apart from the store: which
//! notes a search by meaning adds to the text matches, and which tags the
//! nearest notes suggest.

use std::collections::{HashMap, HashSet};

use crate::RelatedNote;

/// The least similarity for a note to be listed as about a search. Below
/// it the Related tab's words say "Loosely related"; a search list of
/// those is noise.
pub const MEANING_SIMILARITY: f32 = 0.35;

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
    }
}
