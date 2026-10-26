//! Laying the list out under date headers: when it is sorted by date, notes
//! sit under headers for Pinned and each day.
//!
//! Days are numbered in local time, days since 1970-01-01. The caller turns
//! an instant into its local day (the offset from UTC can differ between two
//! instants), so everything here is plain arithmetic. What a header says
//! ("Today", "May 25") is the caller's: each app words it in its own
//! language.

use super::DAY_MS;

/// What a header groups.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bucket {
    Pinned,
    /// A local day.
    Day(i64),
}

/// One row of the list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Row {
    Header(Bucket),
    /// The index of a note in the library's list.
    Note(usize),
}

/// What grouping needs to know about a note.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RowKey {
    pub pinned: bool,
    /// The time the list is sorted by, in milliseconds since the epoch.
    pub at_ms: i64,
}

/// The rows of a list, and where each note landed.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Grouped {
    pub rows: Vec<Row>,
    note_rows: Vec<usize>,
}

impl Grouped {
    /// The row that shows note `note`.
    pub fn note_row(&self, note: usize) -> Option<usize> {
        self.note_rows.get(note).copied()
    }

    pub fn header_count(&self) -> usize {
        self.rows.len() - self.note_rows.len()
    }
}

/// Lays out the notes in list order, with a header wherever the bucket
/// changes when `by_date`. Pinned notes go under "Pinned"; `local_day` gives
/// an instant's local day. One pass, one call of `local_day` per unpinned
/// note.
pub fn group_rows(
    keys: impl IntoIterator<Item = RowKey>,
    mut local_day: impl FnMut(i64) -> i64,
    by_date: bool,
) -> Grouped {
    let keys = keys.into_iter();
    let (lower, _) = keys.size_hint();
    let mut grouped = Grouped {
        rows: Vec::with_capacity(lower + lower / 8),
        note_rows: Vec::with_capacity(lower),
    };
    let mut current = None;
    for (ix, key) in keys.enumerate() {
        if by_date {
            let bucket = if key.pinned {
                Bucket::Pinned
            } else {
                Bucket::Day(local_day(key.at_ms))
            };
            if current != Some(bucket) {
                grouped.rows.push(Row::Header(bucket));
                current = Some(bucket);
            }
        }
        grouped.note_rows.push(grouped.rows.len());
        grouped.rows.push(Row::Note(ix));
    }
    grouped
}

/// The local day of a local time in milliseconds.
pub fn day_of(local_ms: i64) -> i64 {
    local_ms.div_euclid(DAY_MS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 512,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    /// Local day at a fixed offset, as the app computes it.
    fn at_offset(offset_secs: i32) -> impl Fn(i64) -> i64 {
        move |at| day_of(at + i64::from(offset_secs) * 1000)
    }

    fn bucket_of(key: RowKey, offset_secs: i32) -> Bucket {
        if key.pinned {
            Bucket::Pinned
        } else {
            Bucket::Day(at_offset(offset_secs)(key.at_ms))
        }
    }

    fn gen_keys() -> impl Strategy<Value = Vec<RowKey>> {
        // A few weeks either side of 2026, so days repeat and runs form.
        prop::collection::vec(
            (any::<bool>(), 1_770_000_000_000i64..1_790_000_000_000),
            0..60,
        )
        .prop_map(|keys| {
            keys.into_iter()
                .map(|(pinned, at_ms)| RowKey { pinned, at_ms })
                .collect()
        })
    }

    /// The order the library lists notes in: pinned first, newest first.
    fn sorted(mut keys: Vec<RowKey>) -> Vec<RowKey> {
        keys.sort_by_key(|k| (!k.pinned, std::cmp::Reverse(k.at_ms)));
        keys
    }

    fn gen_offset() -> impl Strategy<Value = i32> {
        -12 * 3600..=14 * 3600
    }

    proptest! {
        #![proptest_config(config())]

        /// Projection: dropping the headers leaves exactly the notes, in
        /// order, and `note_row` points back at each of them.
        #[test]
        fn prop_notes_project_back_in_order(keys in gen_keys(), offset in gen_offset(), by_date: bool) {
            let grouped = group_rows(keys.iter().copied(), at_offset(offset), by_date);
            let notes: Vec<usize> = grouped.rows.iter().filter_map(|row| match row {
                Row::Note(ix) => Some(*ix),
                Row::Header(_) => None,
            }).collect();
            prop_assert_eq!(notes, (0..keys.len()).collect::<Vec<_>>());
            for ix in 0..keys.len() {
                let row = grouped.note_row(ix).unwrap();
                prop_assert_eq!(grouped.rows[row], Row::Note(ix));
            }
            prop_assert_eq!(grouped.note_row(keys.len()), None);
        }

        /// Every header has a note under it: none at the end, none twice in
        /// a row, and one per run of notes in the same bucket.
        #[test]
        fn prop_one_header_per_bucket_run(keys in gen_keys(), offset in gen_offset()) {
            let grouped = group_rows(keys.iter().copied(), at_offset(offset), true);
            prop_assert!(!matches!(grouped.rows.last(), Some(Row::Header(_))));
            for pair in grouped.rows.windows(2) {
                prop_assert!(!matches!(pair, [Row::Header(_), Row::Header(_)]));
            }
            let buckets: Vec<Bucket> = keys.iter().map(|k| bucket_of(*k, offset)).collect();
            let runs = buckets.chunk_by(|a, b| a == b).count();
            let headers = grouped.rows.iter().filter(|r| matches!(r, Row::Header(_))).count();
            prop_assert_eq!(headers, runs);
            prop_assert_eq!(grouped.header_count(), runs);
            prop_assert_eq!(grouped.rows.len(), keys.len() + runs);
        }

        /// Each note sits under the header of its own bucket, so pinned notes
        /// are under "Pinned".
        #[test]
        fn prop_notes_sit_under_their_bucket(keys in gen_keys(), offset in gen_offset()) {
            let grouped = group_rows(keys.iter().copied(), at_offset(offset), true);
            let mut header = None;
            for row in &grouped.rows {
                match row {
                    Row::Header(bucket) => header = Some(*bucket),
                    Row::Note(ix) => prop_assert_eq!(header, Some(bucket_of(keys[*ix], offset))),
                }
            }
        }

        /// Sorted by title, there are no headers at all.
        #[test]
        fn prop_no_headers_unless_by_date(keys in gen_keys(), offset in gen_offset()) {
            let grouped = group_rows(keys.iter().copied(), at_offset(offset), false);
            prop_assert_eq!(grouped.header_count(), 0);
            prop_assert!(grouped.rows.iter().all(|r| matches!(r, Row::Note(_))));
        }

        /// In the library's order, Pinned comes first and the days count
        /// down, each header a later day than the next.
        #[test]
        fn prop_buckets_are_monotone(keys in gen_keys(), offset in gen_offset()) {
            let keys = sorted(keys);
            let grouped = group_rows(keys.iter().copied(), at_offset(offset), true);
            let headers: Vec<Bucket> = grouped.rows.iter().filter_map(|r| match r {
                Row::Header(b) => Some(*b),
                Row::Note(_) => None,
            }).collect();
            if let Some(pinned) = headers.iter().position(|b| *b == Bucket::Pinned) {
                prop_assert_eq!(pinned, 0);
            }
            let days: Vec<i64> = headers.iter().filter_map(|b| match b {
                Bucket::Day(d) => Some(*d),
                Bucket::Pinned => None,
            }).collect();
            for pair in days.windows(2) {
                prop_assert!(pair[0] > pair[1]);
            }
        }
    }
}
