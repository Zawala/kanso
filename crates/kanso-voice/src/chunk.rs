//! Where a long recording is cut into pieces the encoder takes at once.
//!
//! The encoder's attention grows with the square of its input, so it never
//! sees more than [`MAX_SECONDS`]. Once [`CUT_AFTER_SECONDS`] are waiting, the
//! recording is cut at the quietest tenth of a second between
//! [`MIN_SECONDS`] and [`MAX_SECONDS`], most likely a pause between words.

use std::ops::Range;

/// No piece is cut shorter than this, but the last.
pub const MIN_SECONDS: u32 = 15;
/// Once this much is waiting, a piece is cut off.
pub const CUT_AFTER_SECONDS: u32 = 20;
/// No piece is longer than this.
pub const MAX_SECONDS: u32 = 25;

/// The window whose loudness decides a cut: 100 ms.
const WINDOW_MS: u32 = 100;
/// How far the window moves between candidates: 10 ms.
const HOP_MS: u32 = 10;

/// Where to cut `samples` (at `rate` Hz), as the length of the first piece,
/// or `None` while fewer than [`CUT_AFTER_SECONDS`] are waiting. The cut
/// falls in the middle of the quietest 100 ms window that lies between
/// [`MIN_SECONDS`] and [`MAX_SECONDS`]; the earliest one wins a tie.
pub fn plan_cut(samples: &[f32], rate: u32) -> Option<usize> {
    let rate = rate.max(1) as usize;
    if samples.len() < CUT_AFTER_SECONDS as usize * rate {
        return None;
    }
    let lo = MIN_SECONDS as usize * rate;
    let hi = samples.len().min(MAX_SECONDS as usize * rate);
    let window = (WINDOW_MS as usize * rate / 1000).max(1);
    let hop = (HOP_MS as usize * rate / 1000).max(1);
    if hi < lo + window {
        return Some(hi);
    }

    // Energy over [lo, hi) as running sums, so each window is one subtraction.
    let mut sums = Vec::with_capacity(hi - lo + 1);
    let mut total = 0.0f64;
    sums.push(0.0);
    for &s in &samples[lo..hi] {
        total += f64::from(s) * f64::from(s);
        sums.push(total);
    }

    let mut best = (f64::INFINITY, lo);
    let mut start = 0;
    while start + window <= hi - lo {
        let energy = sums[start + window] - sums[start];
        if energy < best.0 {
            best = (energy, start);
        }
        start += hop;
    }
    Some(lo + best.1 + window / 2)
}

/// Cuts a whole recording into pieces with [`plan_cut`], in order; the last
/// piece is whatever is left.
pub fn split(samples: &[f32], rate: u32) -> Vec<Range<usize>> {
    let mut pieces = Vec::new();
    let mut start = 0;
    while let Some(len) = plan_cut(&samples[start..], rate) {
        pieces.push(start..start + len);
        start += len;
    }
    if start < samples.len() {
        pieces.push(start..samples.len());
    }
    pieces
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    const RATE: u32 = 16_000;

    /// Steady noise from a seed, at `amplitude`.
    fn noise(len: usize, seed: u64, amplitude: f32) -> Vec<f32> {
        let mut state = seed | 1;
        (0..len)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                amplitude * ((state >> 40) as f32 / (1u64 << 24) as f32 * 2.0 - 1.0)
            })
            .collect()
    }

    #[test]
    fn nothing_is_cut_before_twenty_seconds() {
        assert_eq!(plan_cut(&vec![0.1; 20 * 16_000 - 1], RATE), None);
        assert!(plan_cut(&vec![0.1; 20 * 16_000], RATE).is_some());
    }

    #[test]
    fn the_cut_lands_in_a_pause() {
        let mut audio = noise(22 * 16_000, 7, 0.3);
        // 300 ms of silence starting at 17.2 s.
        let pause = 17_200 * 16..17_500 * 16;
        audio[pause.clone()].fill(0.0);
        let cut = plan_cut(&audio, RATE).unwrap();
        assert!(pause.contains(&cut), "cut at {cut}");
    }

    proptest! {
        // Each case makes up to a minute of audio; fewer cases keep it quick.
        #![proptest_config(ProptestConfig::with_cases(48))]

        #[test]
        fn pieces_tile_the_recording_within_the_limits(
            seconds in 0.0f64..65.0,
            seed in any::<u64>(),
            rate in prop::sample::select(vec![8_000u32, 16_000, 44_100]),
        ) {
            let len = (seconds * rate as f64) as usize;
            let audio = noise(len, seed, 0.2);
            let pieces = split(&audio, rate);
            let mut at = 0;
            for (i, piece) in pieces.iter().enumerate() {
                prop_assert_eq!(piece.start, at);
                prop_assert!(piece.end > piece.start);
                prop_assert!(piece.len() <= MAX_SECONDS as usize * rate as usize);
                if i + 1 < pieces.len() {
                    prop_assert!(piece.len() >= MIN_SECONDS as usize * rate as usize);
                } else {
                    prop_assert!(piece.len() < CUT_AFTER_SECONDS as usize * rate as usize);
                }
                at = piece.end;
            }
            prop_assert_eq!(at, len);
        }

        #[test]
        fn a_silent_gap_wins(
            seed in any::<u64>(),
            extra in 0usize..5 * 16_000,
            // Where the 200 ms gap starts, in 10 ms steps from 15 s.
            gap_step in 0usize..980,
        ) {
            let len = 20 * 16_000 + extra;
            let gap_start = 15 * 16_000 + gap_step * 160;
            let gap = gap_start..gap_start + 3_200;
            prop_assume!(gap.end <= len.min(25 * 16_000));
            let mut audio = noise(len, seed, 0.3);
            audio[gap.clone()].fill(0.0);
            let cut = plan_cut(&audio, RATE).unwrap();
            prop_assert!(gap.contains(&cut), "cut {} outside {:?}", cut, gap);
        }
    }
}
