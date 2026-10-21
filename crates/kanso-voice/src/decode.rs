//! Greedy decoding for a token-and-duration transducer (TDT), as onnx-asr
//! does it for NeMo's Parakeet TDT models.
//!
//! At each encoder frame the joint network scores every token and every
//! duration. The best token is kept unless it's the blank; the best duration
//! says how many frames to move on. A duration of 0 stays on the frame to
//! emit another token, at most [`MAX_SYMBOLS_PER_FRAME`] times.

use std::sync::atomic::{AtomicBool, Ordering};

use crate::VoiceError;

/// The most tokens one encoder frame may emit before decoding moves on.
pub const MAX_SYMBOLS_PER_FRAME: usize = 10;

/// The durations Parakeet TDT predicts, in frames, in the order of its
/// duration logits.
pub const DURATIONS: [usize; 5] = [0, 1, 2, 3, 4];

/// The prediction network and joint of a transducer, one step at a time.
pub trait Joint {
    /// The prediction network's state between steps.
    type State;

    /// The state before any token.
    fn initial_state(&self) -> Self::State;

    /// Scores encoder frame `frame` after `previous` (the last token
    /// emitted, or the blank at the start) in `state`: fills `logits` with
    /// the token logits followed by the duration logits, and returns the
    /// state after `previous`.
    fn step(
        &mut self,
        frame: usize,
        previous: u32,
        state: &Self::State,
        logits: &mut Vec<f32>,
    ) -> Result<Self::State, VoiceError>;
}

/// A token and the encoder frame it was emitted at.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Emitted {
    pub token: u32,
    pub frame: usize,
}

/// The index of the largest value; the first one on a tie, 0 for none.
/// NaN never wins.
pub fn argmax(values: &[f32]) -> usize {
    let mut best = 0;
    let mut best_value = f32::NEG_INFINITY;
    for (i, &v) in values.iter().enumerate() {
        if v > best_value {
            best = i;
            best_value = v;
        }
    }
    best
}

/// Decodes `frames` encoder frames greedily over `joint`, whose first
/// `vocab_size` logits are tokens (`blank` among them) and whose remaining
/// ones are scores for `durations`. Checks `cancel` before every step.
pub fn tdt_greedy<J: Joint>(
    joint: &mut J,
    frames: usize,
    vocab_size: usize,
    blank: u32,
    durations: &[usize],
    cancel: &AtomicBool,
) -> Result<Vec<Emitted>, VoiceError> {
    let mut emitted = Vec::new();
    let mut state = joint.initial_state();
    let mut logits = Vec::with_capacity(vocab_size + durations.len());
    let mut t = 0;
    let mut on_this_frame = 0;
    while t < frames {
        if cancel.load(Ordering::Relaxed) {
            return Err(VoiceError::Cancelled);
        }
        let previous = emitted.last().map_or(blank, |e: &Emitted| e.token);
        logits.clear();
        let next_state = joint.step(t, previous, &state, &mut logits)?;
        if logits.len() != vocab_size + durations.len() {
            return Err(VoiceError::Model(format!(
                "the joint gave {} scores, not {} tokens and {} durations",
                logits.len(),
                vocab_size,
                durations.len()
            )));
        }
        let token = argmax(&logits[..vocab_size]) as u32;
        let step = if durations.is_empty() {
            0
        } else {
            durations[argmax(&logits[vocab_size..])]
        };

        if token != blank {
            state = next_state;
            emitted.push(Emitted { token, frame: t });
            on_this_frame += 1;
        }
        if step > 0 {
            t += step;
            on_this_frame = 0;
        } else if token == blank || on_this_frame == MAX_SYMBOLS_PER_FRAME {
            t += 1;
            on_this_frame = 0;
        }
    }
    Ok(emitted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    /// A joint whose scores come from a table: `table[step]` for each call,
    /// cycling. It records the previous token and frame of each call.
    struct Scripted {
        table: Vec<Vec<f32>>,
        calls: usize,
        seen: Vec<(usize, u32)>,
    }

    impl Joint for Scripted {
        type State = usize;
        fn initial_state(&self) -> usize {
            0
        }
        fn step(
            &mut self,
            frame: usize,
            previous: u32,
            state: &usize,
            logits: &mut Vec<f32>,
        ) -> Result<usize, VoiceError> {
            logits.extend_from_slice(&self.table[self.calls % self.table.len()]);
            self.calls += 1;
            self.seen.push((frame, previous));
            Ok(state + 1)
        }
    }

    const VOCAB: usize = 6;
    const BLANK: u32 = 5;

    fn scores(token: usize, duration: usize) -> Vec<f32> {
        let mut v = vec![0.0; VOCAB + DURATIONS.len()];
        v[token] = 1.0;
        v[VOCAB + duration] = 1.0;
        v
    }

    fn decode(table: Vec<Vec<f32>>, frames: usize) -> (Vec<Emitted>, Scripted) {
        let mut joint = Scripted {
            table,
            calls: 0,
            seen: Vec::new(),
        };
        let out = tdt_greedy(
            &mut joint,
            frames,
            VOCAB,
            BLANK,
            &DURATIONS,
            &AtomicBool::new(false),
        )
        .unwrap();
        (out, joint)
    }

    #[test]
    fn argmax_takes_the_first_of_a_tie_and_skips_nan() {
        assert_eq!(argmax(&[1.0, 3.0, 3.0]), 1);
        assert_eq!(argmax(&[f32::NAN, 0.5]), 1);
        assert_eq!(argmax(&[]), 0);
    }

    #[test]
    fn durations_skip_frames() {
        // Token 1 then move 2; token 2 then move 3; blank then move 1...
        let (out, _) = decode(
            vec![scores(1, 2), scores(2, 3), scores(BLANK as usize, 1)],
            7,
        );
        assert_eq!(
            out,
            vec![
                Emitted { token: 1, frame: 0 },
                Emitted { token: 2, frame: 2 },
                Emitted { token: 1, frame: 6 }
            ]
        );
    }

    #[test]
    fn a_frame_emits_at_most_ten_tokens() {
        let (out, _) = decode(vec![scores(3, 0)], 2);
        assert_eq!(out.len(), 2 * MAX_SYMBOLS_PER_FRAME);
        assert!(out[..10].iter().all(|e| e.frame == 0));
    }

    #[test]
    fn the_previous_token_feeds_the_next_step() {
        let (out, joint) = decode(
            vec![scores(2, 0), scores(BLANK as usize, 0), scores(4, 1)],
            2,
        );
        assert_eq!(joint.seen, vec![(0, BLANK), (0, 2), (1, 2)]);
        assert_eq!(
            out,
            vec![
                Emitted { token: 2, frame: 0 },
                Emitted { token: 4, frame: 1 }
            ]
        );
    }

    #[test]
    fn cancelling_stops_decoding() {
        let mut joint = Scripted {
            table: vec![scores(1, 1)],
            calls: 0,
            seen: Vec::new(),
        };
        let result = tdt_greedy(
            &mut joint,
            10,
            VOCAB,
            BLANK,
            &DURATIONS,
            &AtomicBool::new(true),
        );
        assert!(matches!(result, Err(VoiceError::Cancelled)));
        assert_eq!(joint.calls, 0);
    }

    #[test]
    fn a_short_joint_output_is_an_error() {
        let mut joint = Scripted {
            table: vec![vec![0.0; 3]],
            calls: 0,
            seen: Vec::new(),
        };
        let result = tdt_greedy(
            &mut joint,
            4,
            VOCAB,
            BLANK,
            &DURATIONS,
            &AtomicBool::new(false),
        );
        assert!(matches!(result, Err(VoiceError::Model(_))));
    }

    proptest! {
        #[test]
        fn decoding_ends_moves_forward_and_caps_each_frame(
            table in prop::collection::vec(prop::collection::vec(-5.0f32..5.0, VOCAB + DURATIONS.len()), 1..40),
            frames in 0usize..200,
        ) {
            let (out, joint) = decode(table, frames);
            // Every frame takes at most ten tokens and one blank or move.
            prop_assert!(joint.calls <= frames * (MAX_SYMBOLS_PER_FRAME + 1));
            prop_assert!(out.iter().all(|e| e.token != BLANK && (e.token as usize) < VOCAB && e.frame < frames));
            prop_assert!(out.windows(2).all(|w| w[0].frame <= w[1].frame));
            prop_assert!(joint.seen.windows(2).all(|w| w[0].0 <= w[1].0));
            let mut per_frame = std::collections::HashMap::new();
            for e in &out {
                *per_frame.entry(e.frame).or_insert(0usize) += 1;
            }
            prop_assert!(per_frame.values().all(|&n| n <= MAX_SYMBOLS_PER_FRAME));
        }

        #[test]
        fn all_blank_says_nothing(
            durations in prop::collection::vec(0usize..DURATIONS.len(), 1..10),
            frames in 0usize..100,
        ) {
            let table = durations.iter().map(|&d| scores(BLANK as usize, d)).collect();
            let (out, joint) = decode(table, frames);
            prop_assert!(out.is_empty());
            // Without a token, the joint always sees the blank.
            prop_assert!(joint.seen.iter().all(|&(_, previous)| previous == BLANK));
        }
    }
}
