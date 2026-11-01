//! Which usage count (docs/METRICS.md › `feature_used`) a user action is.
//! Pure: the decisions only, so they are tested; the calls to
//! `metrics::record` sit where the actions happen. Nothing a person wrote
//! goes in: only a feature's name.

use kanso_core::model::TRASH_BOOK_ID;
use kanso_engine::batch::Change;

use crate::metrics::Feature;

/// What moving a note from notebook `from` to `to` is: to the trash a
/// delete, out of it a restore, else a move; nothing for the same notebook.
pub fn move_feature(from: &str, to: &str) -> Option<Feature> {
    if from == to {
        None
    } else if to == TRASH_BOOK_ID {
        Some(Feature::NoteDeleted)
    } else if from == TRASH_BOOK_ID {
        Some(Feature::NoteRestored)
    } else {
        Some(Feature::NoteMoved)
    }
}

/// What a change to several notes counts as, once per note changed.
/// `from` is each changed note's notebook before; `on` says whether a pin or
/// tag was turned on ([`kanso_engine::batch::turned_on`]).
pub fn batch_features(change: &Change, from: &[String], on: bool) -> Vec<Feature> {
    match change {
        Change::Move(to) => from
            .iter()
            .filter_map(|from| move_feature(from, to))
            .collect(),
        Change::AddTag(_) | Change::ToggleTag(_) if on => {
            vec![Feature::TagAdded; from.len()]
        }
        Change::TogglePin if on => vec![Feature::NotePinned; from.len()],
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn moves_are_told_apart() {
        assert_eq!(move_feature("a", "a"), None);
        assert_eq!(move_feature("a", TRASH_BOOK_ID), Some(Feature::NoteDeleted));
        assert_eq!(
            move_feature(TRASH_BOOK_ID, "a"),
            Some(Feature::NoteRestored)
        );
        assert_eq!(move_feature("a", "b"), Some(Feature::NoteMoved));
    }

    proptest! {
        #![proptest_config(ProptestConfig { rng_seed: proptest::test_runner::RngSeed::Fixed(0x006b_616e_736f), ..ProptestConfig::with_cases(128) })]

        /// A batch counts at most once per note, and only what turned on.
        #[test]
        fn prop_a_batch_counts_each_note_at_most_once(
            from in prop::collection::vec("[a-c]|trash", 0..8),
            to in "[a-c]|trash",
            on in any::<bool>(),
        ) {
            let n = from.len();
            for change in [
                Change::Move(to.clone()),
                Change::AddTag("t".into()),
                Change::RemoveTag("t".into()),
                Change::ToggleTag("t".into()),
                Change::TogglePin,
            ] {
                let counted = batch_features(&change, &from, on);
                prop_assert!(counted.len() <= n);
                if matches!(change, Change::RemoveTag(_)) || (!on && !matches!(change, Change::Move(_))) {
                    prop_assert!(counted.is_empty());
                }
            }
        }
    }
}
