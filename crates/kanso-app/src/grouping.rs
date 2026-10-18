//! The note list's rows: when it is sorted by date, notes sit under headers
//! for Pinned, Today, Yesterday and each older day ("May 25", or
//! "May 25, 2025" in another year). Rows show a time on the same scale.
//!
//! Days are numbered in local time, days since 1970-01-01. The caller turns
//! an instant into its local day (the offset from UTC can differ between
//! two instants), so everything here is plain arithmetic.

use chrono::{Datelike as _, NaiveDate};

const DAY_MS: i64 = 86_400_000;
/// `NaiveDate::from_num_days_from_ce` of 1970-01-01.
const UNIX_EPOCH_CE_DAYS: i64 = 719_163;
const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

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

    #[cfg(test)]
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

/// A header's text: "Pinned", "Today", "Yesterday", "May 25" in the current
/// year and "May 25, 2025" in any other.
pub fn bucket_label(bucket: Bucket, today: i64) -> String {
    match bucket {
        Bucket::Pinned => "Pinned".into(),
        Bucket::Day(day) => day_label(day, today),
    }
}

/// "Today", "Yesterday" or the date, for local day `day`.
pub fn day_label(day: i64, today: i64) -> String {
    if day == today {
        return "Today".into();
    }
    if day == today - 1 {
        return "Yesterday".into();
    }
    let (Some(date), Some(now)) = (civil(day), civil(today)) else {
        return String::new();
    };
    let month = MONTHS[date.month0() as usize];
    if date.year() == now.year() {
        format!("{month} {}", date.day())
    } else {
        format!("{month} {}, {}", date.day(), date.year())
    }
}

/// The time a note row shows: "09:41" today, "Yesterday", then the date as
/// headers write it. Both times are local, in milliseconds.
pub fn row_time_label(at_local_ms: i64, now_local_ms: i64) -> String {
    let (day, today) = (day_of(at_local_ms), day_of(now_local_ms));
    if day == today {
        let minutes = at_local_ms.rem_euclid(DAY_MS) / 60_000;
        format!("{:02}:{:02}", minutes / 60, minutes % 60)
    } else {
        day_label(day, today)
    }
}

fn civil(day: i64) -> Option<NaiveDate> {
    NaiveDate::from_num_days_from_ce_opt(i32::try_from(day + UNIX_EPOCH_CE_DAYS).ok()?)
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

        /// Only the day before today reads "Yesterday", only today reads as a
        /// time, and a time reads back as the local time of day.
        #[test]
        fn prop_row_time_label_matches_the_day(
            at in -2_000_000_000_000i64..4_000_000_000_000,
            // Mostly the days around today, where the labels change.
            back in prop_oneof![3 => -3i64..5, 1 => -800i64..800],
            ms in 0i64..DAY_MS,
        ) {
            let now = day_of(at) * DAY_MS + back * DAY_MS + ms;
            let label = row_time_label(at, now);
            let (day, today) = (day_of(at), day_of(now));
            prop_assert_eq!(label == "Yesterday", day == today - 1, "{}", label);
            let as_time = label.len() == 5
                && label.as_bytes()[2] == b':'
                && label[..2].parse::<i64>().is_ok()
                && label[3..].parse::<i64>().is_ok();
            prop_assert_eq!(as_time, day == today, "{}", label);
            if as_time {
                let minutes = label[..2].parse::<i64>().unwrap() * 60 + label[3..].parse::<i64>().unwrap();
                prop_assert_eq!(minutes, at.rem_euclid(DAY_MS) / 60_000);
            }
        }

        /// Different days get different headers, and a date names its year
        /// exactly when that is not the current one.
        #[test]
        fn prop_day_labels_are_distinct(today in -20_000i64..40_000, a in -1_500i64..1_500, b in -1_500i64..1_500) {
            let (la, lb) = (day_label(today + a, today), day_label(today + b, today));
            prop_assert!(!la.is_empty());
            prop_assert_eq!(la == lb, a == b, "{} / {}", la, lb);
            let year = |d: i64| civil(d).unwrap().year();
            if a != 0 && a != -1 {
                prop_assert_eq!(la.contains(", "), year(today + a) != year(today), "{}", la);
            }
        }
    }

    #[test]
    fn groups_like_the_mockup() {
        // Thursday 28 May 2026, 09:45 at UTC+2.
        let now = 1_779_954_300_000i64;
        let offset = 2 * 3600;
        let hour = 3_600_000i64;
        let keys = [
            RowKey {
                pinned: true,
                at_ms: now - 30 * DAY_MS,
            },
            RowKey {
                pinned: false,
                at_ms: now - hour,
            },
            RowKey {
                pinned: false,
                at_ms: now - 2 * hour,
            },
            RowKey {
                pinned: false,
                at_ms: now - DAY_MS,
            },
            RowKey {
                pinned: false,
                at_ms: now - 3 * DAY_MS,
            },
            RowKey {
                pinned: false,
                at_ms: now - 400 * DAY_MS,
            },
        ];
        let grouped = group_rows(keys, at_offset(offset), true);
        let today = at_offset(offset)(now);
        let labels: Vec<String> = grouped
            .rows
            .iter()
            .map(|row| match row {
                Row::Header(bucket) => bucket_label(*bucket, today),
                Row::Note(ix) => format!("note {ix}"),
            })
            .collect();
        assert_eq!(
            labels,
            [
                "Pinned",
                "note 0",
                "Today",
                "note 1",
                "note 2",
                "Yesterday",
                "note 3",
                "May 25",
                "note 4",
                "Apr 23, 2025",
                "note 5",
            ]
        );
        assert_eq!(grouped.note_row(3), Some(6));
    }

    #[test]
    fn row_times() {
        let local = |y, m, d, h, min| {
            NaiveDate::from_ymd_opt(y, m, d)
                .unwrap()
                .and_hms_opt(h, min, 0)
                .unwrap()
                .and_utc()
                .timestamp_millis()
        };
        let now = local(2026, 5, 28, 9, 45);
        assert_eq!(row_time_label(local(2026, 5, 28, 9, 41), now), "09:41");
        assert_eq!(row_time_label(local(2026, 5, 28, 0, 0), now), "00:00");
        assert_eq!(row_time_label(local(2026, 5, 27, 23, 59), now), "Yesterday");
        assert_eq!(row_time_label(local(2026, 5, 26, 12, 0), now), "May 26");
        assert_eq!(row_time_label(local(2026, 1, 1, 12, 0), now), "Jan 1");
        assert_eq!(
            row_time_label(local(2025, 12, 31, 12, 0), now),
            "Dec 31, 2025"
        );
        // A clock that runs ahead shows the date, never a time.
        assert_eq!(row_time_label(local(2026, 5, 29, 8, 0), now), "May 29");
    }
}
