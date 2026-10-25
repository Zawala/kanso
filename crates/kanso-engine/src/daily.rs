//! Daily notes and templates, without a window: today's note in the Journal
//! (made from the daily template when there is one), the previous and next
//! day that has a note, and a note made from a template in the Templates
//! notebook. The store does the making (`kanso_store::roles`); this is the
//! app's side of it, shared by the desktop and mobile apps.
//!
//! Nothing here reads a clock: the caller passes the local date and time.

use kanso_core::dates::{Date, DateTime};
use kanso_core::roles::{Role, daily_date};
use kanso_store::{Origin, Store};

/// A note found or made, for the app to open.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Made {
    pub id: String,
    /// Where a template put `{{cursor}}`, as a byte offset into the body.
    pub cursor: Option<usize>,
    /// Whether the keyboard goes to the title first: the note has none yet.
    pub title_first: bool,
}

/// The day before or after `current` that has a daily note, or today
/// (which opening today makes if needed), whichever is nearer. `None` when
/// there is no such day. Pure.
pub fn step_day(dates: &[Date], current: Date, today: Date, forward: bool) -> Option<Date> {
    let candidates = dates.iter().copied().chain([today]);
    if forward {
        candidates.filter(|&d| d > current).min()
    } else {
        candidates.filter(|&d| d < current).max()
    }
}

/// The day to open when stepping from the open note (a daily note, or any
/// other, which steps from today): see [`step_day`] over the days the store
/// has daily notes for.
pub fn step(store: &Store, open_note: Option<&str>, today: Date, forward: bool) -> Option<Date> {
    let current = open_note.and_then(daily_date).unwrap_or(today);
    let dates = store.daily_dates().unwrap_or_default();
    step_day(&dates, current, today, forward)
}

/// Where a note made from a template goes: the notebook the list shows,
/// unless that is the Templates notebook, where the note would itself be a
/// template; then the Inbox (`None`).
pub fn template_target(store: &Store, book: Option<String>) -> Option<String> {
    let templates = store.find_role_book(Role::Templates).ok().flatten();
    book.filter(|book| Some(book) != templates.as_ref())
}

/// Today's note at local time `now`, made from the daily template when it
/// is missing.
pub fn today(store: &Store, now: DateTime) -> kanso_store::Result<Made> {
    let daily = store.open_daily(now, Origin::Ui)?;
    Ok(Made {
        id: daily.id,
        cursor: daily.cursor,
        title_first: false,
    })
}

/// A note from template `template` in `book` (the notebook the list shows;
/// see [`template_target`]), its variables filled in for `now`. The title
/// is the keyboard's first stop when the template gave none.
pub fn from_template(
    store: &Store,
    template: &str,
    book: Option<String>,
    now: DateTime,
) -> kanso_store::Result<Made> {
    let book = template_target(store, book);
    let made = store.create_from_template(template, None, book.as_deref(), now, Origin::Ui)?;
    Ok(Made {
        id: made.id,
        cursor: made.cursor,
        title_first: made.title.trim().is_empty(),
    })
}

/// An empty note in the Templates notebook (made if needed), to write a
/// template in. Returns its ID.
pub fn new_template(store: &Store) -> kanso_store::Result<String> {
    let book = store.role_book(Role::Templates, Origin::Ui)?;
    new_note_in(store, &book)
}

fn new_note_in(store: &Store, book: &str) -> kanso_store::Result<String> {
    let fields = serde_json::json!({"title": "", "body": "", "bookId": book});
    store
        .put(
            kanso_core::DocType::Note,
            fields,
            kanso_store::Put::Upsert,
            Origin::Ui,
        )
        .map(|made| made.id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kanso_core::roles::daily_id;
    use proptest::prelude::*;
    use serde_json::json;

    fn d(day: u32) -> Date {
        Date::new(2026, 5, day).unwrap()
    }

    fn at(day: u32) -> DateTime {
        DateTime {
            date: d(day),
            hour: 9,
            minute: 30,
            second: 0,
        }
    }

    fn store() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("kanso.db")).unwrap();
        (dir, store)
    }

    #[test]
    fn steps_between_days_with_notes_and_today() {
        let dates = [d(3), d(10), d(12)];
        let today = d(20);
        assert_eq!(step_day(&dates, d(12), today, false), Some(d(10)));
        assert_eq!(step_day(&dates, d(10), today, true), Some(d(12)));
        // Today, though it has no note yet.
        assert_eq!(step_day(&dates, d(12), today, true), Some(today));
        assert_eq!(step_day(&dates, today, today, true), None);
        assert_eq!(step_day(&dates, d(3), today, false), None);
        // From a day without a note (today's, not made yet).
        assert_eq!(step_day(&dates, today, today, false), Some(d(12)));
        assert_eq!(step_day(&[], today, today, false), None);
    }

    #[test]
    fn stepping_reads_the_days_from_the_store() {
        let (_dir, store) = store();
        for day in [3, 10] {
            assert_eq!(today(&store, at(day)).unwrap().id, daily_id(d(day)));
        }
        let today_ = d(20);
        // From the 10th's note, back to the 3rd and forward to today.
        let open = daily_id(d(10));
        assert_eq!(step(&store, Some(&open), today_, false), Some(d(3)));
        assert_eq!(step(&store, Some(&open), today_, true), Some(today_));
        // From a note that isn't a daily note, or none: from today.
        assert_eq!(step(&store, Some("note:other"), today_, false), Some(d(10)));
        assert_eq!(step(&store, None, today_, false), Some(d(10)));
        // A note in the trash is not a day to step to.
        store
            .put(
                kanso_core::DocType::Note,
                json!({"_id": daily_id(d(10)), "bookId": "trash"}),
                kanso_store::Put::Merge,
                Origin::Ui,
            )
            .unwrap();
        assert_eq!(step(&store, None, today_, false), Some(d(3)));
    }

    #[test]
    fn today_is_made_once_with_the_daily_template() {
        let (_dir, store) = store();
        let template = store
            .role_book(Role::Templates, Origin::Ui)
            .and_then(|book| {
                store
                    .put(
                        kanso_core::DocType::Note,
                        json!({"title": "Daily", "bookId": book, "body": "Plan:\n{{cursor}}\n"}),
                        kanso_store::Put::Upsert,
                        Origin::Ui,
                    )
                    .map(|made| made.id)
            })
            .unwrap();
        store
            .set_daily_template(Some(&template), Origin::Ui)
            .unwrap();
        let first = today(&store, at(5)).unwrap();
        assert_eq!(first.id, daily_id(d(5)));
        assert_eq!(first.cursor, Some("Plan:\n".len()));
        assert!(!first.title_first);
        // Opening it again finds it: no cursor, nothing new.
        let again = today(&store, at(5)).unwrap();
        assert_eq!((again.id, again.cursor), (first.id, None));
    }

    /// A note made from a template never lands in the Templates notebook,
    /// and the keyboard goes to the title exactly when the note has none.
    #[test]
    fn a_note_from_a_template_goes_to_the_shown_notebook() {
        let (_dir, store) = store();
        let templates = store.role_book(Role::Templates, Origin::Ui).unwrap();
        let notes = store
            .put(
                kanso_core::DocType::Book,
                json!({"name": "Notes"}),
                kanso_store::Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        let template = |title: &str| {
            store
                .put(
                    kanso_core::DocType::Note,
                    json!({"title": title, "bookId": templates, "body": "Body"}),
                    kanso_store::Put::Upsert,
                    Origin::Ui,
                )
                .unwrap()
                .id
        };
        let titled = template("Meeting");
        let untitled = template("");
        let book_of = |id: &str| store.note(id).unwrap().unwrap().book_id;

        let made = from_template(&store, &titled, Some(notes.clone()), at(5)).unwrap();
        assert!(!made.title_first);
        assert_eq!(book_of(&made.id), notes);
        let made = from_template(&store, &untitled, Some(notes.clone()), at(5)).unwrap();
        assert!(made.title_first);
        // Shown the Templates notebook, or none: the Inbox.
        let inbox = store.role_book(Role::Inbox, Origin::Ui).unwrap();
        for book in [Some(templates.clone()), None] {
            let made = from_template(&store, &titled, book, at(5)).unwrap();
            assert_eq!(book_of(&made.id), inbox);
        }
        assert!(from_template(&store, "note:missing", None, at(5)).is_err());
    }

    #[test]
    fn a_new_template_is_an_empty_note_in_the_templates_notebook() {
        let (_dir, store) = store();
        let id = new_template(&store).unwrap();
        let note = store.note(&id).unwrap().unwrap();
        assert_eq!(
            Some(note.book_id.clone()),
            store.find_role_book(Role::Templates).unwrap()
        );
        assert!(note.title.is_empty() && note.body.is_empty());
        assert!(store.templates().unwrap().iter().any(|(t, _)| *t == id));
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Stepping lands on the nearest candidate in that direction, and
        /// stepping back again returns, unless a candidate lies between.
        #[test]
        fn prop_step_is_nearest(
            days in prop::collection::btree_set(1u32..=28, 0..10),
            today in 1u32..=28,
            current in 1u32..=28,
            forward in any::<bool>(),
        ) {
            let dates: Vec<Date> = days.iter().map(|&n| d(n)).collect();
            let mut all: Vec<Date> = dates.clone();
            all.push(d(today));
            match step_day(&dates, d(current), d(today), forward) {
                Some(next) => {
                    prop_assert!(all.contains(&next));
                    let past = if forward { next > d(current) } else { next < d(current) };
                    prop_assert!(past);
                    let between = all.iter().any(|&x| {
                        if forward { x > d(current) && x < next } else { x < d(current) && x > next }
                    });
                    prop_assert!(!between);
                }
                None => {
                    let none = all.iter().all(|&x| if forward { x <= d(current) } else { x >= d(current) });
                    prop_assert!(none);
                }
            }
        }

        /// The target is never the Templates notebook, and any other
        /// notebook is kept.
        #[test]
        fn prop_the_target_is_never_the_templates_notebook(
            book in prop::option::of(prop_oneof![Just("book:templates".to_owned()), "[a-z]{1,6}"]),
        ) {
            let (_dir, store) = store();
            let templates = store.role_book(Role::Templates, Origin::Ui).unwrap();
            let book = book.map(|b| if b == "book:templates" { templates.clone() } else { b });
            let target = template_target(&store, book.clone());
            prop_assert_ne!(target.as_deref(), Some(templates.as_str()));
            if book.as_deref() != Some(templates.as_str()) {
                prop_assert_eq!(target, book);
            }
        }
    }
}
