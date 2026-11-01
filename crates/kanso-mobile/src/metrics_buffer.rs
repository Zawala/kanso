//! Feature counts waiting to be written to the ledger (docs/METRICS.md ›
//! Sending). Hot actions add to this in memory; `MobileMetrics` writes it to
//! the ledger at most every 30 seconds and on every send and every end of a
//! session, so counting is not a SQLite write per action. A process killed
//! between writes loses at most 30 seconds of counts; the ledger itself is
//! still only cleared by an accepted answer.

use std::time::{Duration, Instant};

use kanso_metrics::{Day, Feature};

/// The longest counts wait in memory.
pub(crate) const FLUSH_EVERY: Duration = Duration::from_secs(30);

/// The counts not yet in the ledger, and when they were last written.
#[derive(Debug)]
pub(crate) struct Buffer {
    pending: Vec<(Day, Feature, u32)>,
    flushed: Instant,
}

impl Default for Buffer {
    fn default() -> Buffer {
        Buffer {
            pending: Vec::new(),
            flushed: Instant::now(),
        }
    }
}

impl Buffer {
    /// Adds `n` uses of `feature` on `day`, merged with those waiting.
    pub(crate) fn add(&mut self, day: Day, feature: Feature, n: u32) {
        match self
            .pending
            .iter_mut()
            .find(|(d, f, _)| *d == day && *f == feature)
        {
            Some((_, _, total)) => *total = total.saturating_add(n),
            None => self.pending.push((day, feature, n)),
        }
    }

    /// Whether the counts have waited long enough to be written.
    pub(crate) fn flush_due(&self) -> bool {
        self.flushed.elapsed() >= FLUSH_EVERY
    }

    /// Takes everything waiting, and marks the buffer as written now.
    pub(crate) fn take(&mut self) -> Vec<(Day, Feature, u32)> {
        self.flushed = Instant::now();
        std::mem::take(&mut self.pending)
    }

    /// Puts back what a failed write didn't keep.
    pub(crate) fn put_back(&mut self, pending: Vec<(Day, Feature, u32)>) {
        for (day, feature, n) in pending {
            self.add(day, feature, n);
        }
    }

    #[cfg(test)]
    pub(crate) fn total(&self) -> u64 {
        self.pending.iter().map(|(_, _, n)| u64::from(*n)).sum()
    }

    #[cfg(test)]
    pub(crate) fn backdate(&mut self, by: Duration) {
        self.flushed = Instant::now().checked_sub(by).unwrap_or(self.flushed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig {
            rng_seed: proptest::test_runner::RngSeed::Fixed(0x006b_616e_736f),
            ..ProptestConfig::with_cases(256)
        })]

        /// Whatever is added comes out exactly once, however it is merged,
        /// and a put-back loses nothing.
        #[test]
        fn prop_buffer_keeps_every_count(
            adds in prop::collection::vec((0usize..4, 1u32..50), 0..40),
        ) {
            let day = Day::from_unix_ms(1_793_404_800_000);
            let features = [Feature::Lock, Feature::Search, Feature::NoteMoved, Feature::TagAdded];
            let mut buffer = Buffer::default();
            let mut expected = 0u64;
            for (which, n) in &adds {
                buffer.add(day, features[*which], *n);
                expected += u64::from(*n);
            }
            prop_assert_eq!(buffer.total(), expected);
            let taken = buffer.take();
            prop_assert_eq!(buffer.total(), 0);
            prop_assert!(taken.len() <= features.len());
            buffer.put_back(taken);
            prop_assert_eq!(buffer.total(), expected);
        }
    }

    #[test]
    fn writes_are_due_only_after_the_interval() {
        let mut buffer = Buffer::default();
        assert!(!buffer.flush_due());
        buffer.backdate(FLUSH_EVERY);
        assert!(buffer.flush_due());
        buffer.take();
        assert!(!buffer.flush_due());
    }
}
