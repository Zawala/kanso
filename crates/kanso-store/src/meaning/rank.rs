//! The rules the meaning features follow, apart from the store: which
//! notes a search by meaning adds to the text matches.

use std::collections::HashSet;

use crate::RelatedNote;

/// The least similarity for a note to be listed as about a search. Below
/// it the Related tab's words say "Loosely related"; a search list of
/// those is noise.
pub const MEANING_SIMILARITY: f32 = 0.35;

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

    fn config(cases: u32) -> ProptestConfig {
        ProptestConfig {
            cases,
            failure_persistence: Some(Box::new(
                proptest::test_runner::FileFailurePersistence::WithSource("regressions"),
            )),
            ..ProptestConfig::default()
        }
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
    }
}
