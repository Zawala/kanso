//! What has been said so far, while someone is still speaking.
//!
//! The transcriber finishes a piece of the recording each time the
//! microphone thread cuts one off ([`crate::chunk::plan_cut`]), and while it
//! waits for the next it transcribes the audio recorded since the last cut:
//! the tail. The text so far is the finished pieces and the tail's latest
//! transcript; the tail's words may still change as more is heard.

use crate::audio::MODEL_RATE;
use crate::text::join_pieces;

/// How much more must be recorded before the tail is transcribed again:
/// half a second.
pub const PREVIEW_STEP: usize = MODEL_RATE as usize / 2;

/// The text of a dictation so far, and a number that changes with it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LiveText {
    pieces: Vec<String>,
    tail: String,
    version: u64,
}

impl LiveText {
    /// A piece is finished: its text replaces the tail's, which it heard.
    pub fn commit(&mut self, text: String) {
        self.pieces.push(text);
        self.tail.clear();
        self.version += 1;
    }

    /// The tail as heard now.
    pub fn preview(&mut self, text: String) {
        if text != self.tail {
            self.tail = text;
            self.version += 1;
        }
    }

    /// Everything heard so far, the tail's words last.
    pub fn text(&self) -> String {
        join_pieces(
            self.pieces
                .iter()
                .map(String::as_str)
                .chain(std::iter::once(self.tail.as_str())),
        )
    }

    /// The finished pieces' text, without the tail.
    pub fn finished(&self) -> String {
        join_pieces(self.pieces.iter().map(String::as_str))
    }

    /// Changes whenever [`text`](Self::text) may have.
    pub fn version(&self) -> u64 {
        self.version
    }
}

/// Whether a tail of `len` samples, when one of `last` was handed on last,
/// is worth transcribing again: [`PREVIEW_STEP`] more was recorded, or a
/// piece was cut off it since.
pub fn preview_due(len: usize, last: usize) -> bool {
    len < last || len >= last + PREVIEW_STEP
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    #[derive(Clone, Debug)]
    enum Op {
        Commit(String),
        Preview(String),
    }

    fn op() -> impl Strategy<Value = Op> {
        let words = prop::collection::vec("[a-z]{1,6}", 0..4).prop_map(|w| w.join(" "));
        prop_oneof![
            words.clone().prop_map(Op::Commit),
            words.prop_map(Op::Preview)
        ]
    }

    #[test]
    fn the_tail_follows_the_finished_pieces() {
        let mut live = LiveText::default();
        live.preview("Kanso".into());
        assert_eq!(live.text(), "Kanso");
        assert_eq!(live.finished(), "");
        live.commit("Kanso turns".into());
        live.preview("what you".into());
        assert_eq!(live.text(), "Kanso turns what you");
        assert_eq!(live.finished(), "Kanso turns");
        let seen = live.version();
        live.preview("what you".into());
        assert_eq!(live.version(), seen);
        assert!(preview_due(PREVIEW_STEP, 0));
        assert!(!preview_due(PREVIEW_STEP - 1, 0));
        assert!(preview_due(10, 20));
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        })]

        /// Against a model: the text is the committed pieces then the last
        /// preview since the last commit, the finished text the pieces
        /// alone, and the version moves exactly when the text may change.
        #[test]
        fn prop_live_text_matches_its_model(ops in prop::collection::vec(op(), 0..24)) {
            let mut live = LiveText::default();
            let (mut pieces, mut tail): (Vec<String>, String) = (Vec::new(), String::new());
            for op in ops {
                let before = (live.version(), live.text());
                match op {
                    Op::Commit(text) => {
                        live.commit(text.clone());
                        pieces.push(text);
                        tail.clear();
                        prop_assert!(live.version() > before.0);
                    }
                    Op::Preview(text) => {
                        let changed = text != tail;
                        live.preview(text.clone());
                        tail = text;
                        prop_assert_eq!(live.version() > before.0, changed);
                    }
                }
                let mut all = pieces.clone();
                all.push(tail.clone());
                prop_assert_eq!(live.text(), join_pieces(all.iter().map(String::as_str)));
                prop_assert_eq!(live.finished(), join_pieces(pieces.iter().map(String::as_str)));
                prop_assert!(live.text().starts_with(&live.finished()));
            }
        }

        /// A tail is handed on again only once it grew by a step, or
        /// shrank because a piece was cut off it.
        #[test]
        fn prop_preview_due_needs_growth_or_a_cut(last in 0usize..1_000_000, grew in 0usize..20_000) {
            prop_assert_eq!(preview_due(last + grew, last), grew >= PREVIEW_STEP);
            if last > 0 {
                prop_assert!(preview_due(last - 1, last));
            }
        }
    }
}
