//! Uses counted in memory and written to the [`Ledger`] in one go, so a hot
//! action (a keystroke's edit, a batch of 500 notes) never costs a write of
//! the settings. The app flushes it on a debounce ([`FLUSH_EVERY_MS`]), before
//! every send and on quit or background, so nothing is lost but a crash
//! inside the debounce.

use crate::ledger::Ledger;
use crate::values::{Day, Feature};

/// The longest uses wait in memory before they are written.
pub const FLUSH_EVERY_MS: i64 = 30_000;

/// Uses counted and not yet in the ledger.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CountBuffer {
    counts: Vec<(Feature, u32)>,
    /// When the buffer was last written (ms), for the debounce.
    last_flush_ms: Option<i64>,
}

impl CountBuffer {
    /// Counts `n` uses of `feature`.
    pub fn add(&mut self, feature: Feature, n: u32) {
        if n == 0 {
            return;
        }
        match self.counts.iter_mut().find(|(f, _)| *f == feature) {
            Some((_, have)) => *have = have.saturating_add(n),
            None => self.counts.push((feature, n)),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.counts.is_empty()
    }

    /// The uses waiting, as `(feature, count)`.
    pub fn pending(&self) -> &[(Feature, u32)] {
        &self.counts
    }

    /// Whether a write is due at `now_ms`: something waits and the debounce
    /// has passed since the last one (or there was none).
    pub fn flush_due(&self, now_ms: i64) -> bool {
        !self.counts.is_empty()
            && self
                .last_flush_ms
                .is_none_or(|last| now_ms.saturating_sub(last) >= FLUSH_EVERY_MS)
    }

    /// Moves everything waiting into `ledger` as of `today` and empties the
    /// buffer. Says whether the ledger changed.
    pub fn flush(&mut self, ledger: &mut Ledger, today: Day, now_ms: i64) -> bool {
        self.last_flush_ms = Some(now_ms);
        let mut changed = false;
        for (feature, n) in self.counts.drain(..) {
            changed |= ledger.record_n(today, feature, n);
        }
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn feature() -> impl Strategy<Value = Feature> {
        (0usize..Feature::all().count()).prop_map(|ix| Feature::all().nth(ix).unwrap())
    }

    #[derive(Clone, Debug)]
    enum Op {
        Add(Feature, u32),
        Flush,
        Tick(i64),
    }

    fn op() -> impl Strategy<Value = Op> {
        prop_oneof![
            4 => (feature(), 0u32..50).prop_map(|(f, n)| Op::Add(f, n)),
            1 => Just(Op::Flush),
            2 => (0i64..90_000).prop_map(Op::Tick),
        ]
    }

    proptest! {
        #![proptest_config(ProptestConfig { rng_seed: proptest::test_runner::RngSeed::Fixed(0x006b_616e_736f), ..ProptestConfig::with_cases(256) })]

        /// However uses are added, flushed on the debounce or by force, the
        /// ledger ends with exactly what was added: nothing lost, nothing
        /// counted twice; and a debounced flush is never due inside the
        /// window.
        #[test]
        fn prop_flushing_neither_loses_nor_doubles(ops in prop::collection::vec(op(), 0..80)) {
            let today = Day::parse("2026-10-30").unwrap();
            let mut ledger = Ledger::default();
            let mut buffer = CountBuffer::default();
            let mut added = 0u64;
            let mut now = 0i64;
            let mut last_write: Option<i64> = None;
            for op in ops {
                match op {
                    Op::Add(f, n) => { buffer.add(f, n); added += u64::from(n); }
                    Op::Flush => { buffer.flush(&mut ledger, today, now); last_write = Some(now); }
                    Op::Tick(ms) => {
                        now += ms;
                        if buffer.flush_due(now) {
                            if let Some(last) = last_write {
                                prop_assert!(now - last >= FLUSH_EVERY_MS);
                            }
                            buffer.flush(&mut ledger, today, now);
                            last_write = Some(now);
                        }
                    }
                }
            }
            buffer.flush(&mut ledger, today, now);
            let saved = serde_json::to_string(&ledger).unwrap();
            let ledger: Ledger = serde_json::from_str(&saved).unwrap();
            let id = crate::values::install_id();
            let device = crate::values::Device::new(
                crate::values::Platform::Macos,
                crate::values::Form::Desktop,
                "1.0.0",
            );
            let active = crate::values::Active::new("1", crate::values::SyncMode::Off, false, 0);
            let mut total = 0u64;
            let mut ledger = ledger;
            // The batch is capped at 32 events; settle until nothing is due.
            while let Some(batch) = ledger.due(today, &id, &device, &active) {
                for event in &batch.events {
                    if let crate::batch::EventKind::FeatureUsed(_, n) = event.kind {
                        total += u64::from(n);
                    }
                }
                ledger.settle(&batch, crate::send::Outcome::Accepted);
            }
            // Counts cap at 1,000,000 per feature and day, so only compare
            // below the cap.
            if added < 1_000_000 {
                prop_assert_eq!(total, added);
            }
        }
    }
}
