//! Reminders: what to show now, what to have the system show later, what an
//! answer does, and that a reminder says only what the desktop's does.

mod common;

use std::sync::Arc;

use common::*;
use kanso_core::tasks::Due;
use kanso_engine::tasks::display_text;
use kanso_engine::tasks::reminders::SHOWN_META;
use kanso_mobile::{
    KansoError, Library, ReminderAnswer, ReminderNote, ReminderPlan, SCHEDULE_LIMIT, TaskPlace,
};
use kanso_store::Store;
use kanso_sync::MemoryServer;
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use serde_json::json;

/// The phone's zone: two hours ahead of UTC.
const OFFSET: i64 = 2 * 3_600_000;

fn config(cases: u32) -> ProptestConfig {
    ProptestConfig {
        cases,
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
        ..ProptestConfig::default()
    }
}

/// The UTC time of a local `YYYY-MM-DDTHH:MM`.
fn utc(due: &str) -> i64 {
    Due::parse(due).unwrap().local_ms() - OFFSET
}

fn keys(notes: &[ReminderNote]) -> Vec<&str> {
    notes.iter().map(|n| n.title.as_str()).collect()
}

fn plan(library: &Library, now: i64) -> ReminderPlan {
    library.reminders_plan(now, OFFSET, true).unwrap()
}

fn shown(phone: &Phone) -> Option<String> {
    Store::open(phone.path()).unwrap().meta(SHOWN_META).unwrap()
}

// no-prop-test: a fixed note and the plan at fixed times.
#[test]
fn due_now_and_still_to_come_with_words_for_their_day() {
    let phone = lone_phone();
    let library = &phone.library;
    note(
        library,
        "Plans",
        "- [ ] Call **Sam** @2026-10-22T09:00\n- [ ] Later @2026-10-22T18:00\n- [ ] Next day @2026-10-23T08:00\n- [ ] No time @2026-10-22\n- [x] Done @2026-10-22T09:30\n",
    );
    let got = plan(library, utc("2026-10-22T09:00") + 30_000);
    assert_eq!(keys(&got.show_now), ["Call Sam"]);
    assert_eq!(got.show_now[0].body, "Today 09:00 · Plans");
    assert_eq!(got.show_now[0].line, 0);
    assert_eq!(got.show_now[0].fire_at.hour, 9);
    // The ones to come, soonest first, each worded for the day it appears.
    assert_eq!(keys(&got.schedule), ["Later", "Next day"]);
    assert_eq!(got.schedule[0].body, "Today 18:00 · Plans");
    assert_eq!(got.schedule[1].body, "Today 08:00 · Plans");
    assert_eq!(
        (got.schedule[1].fire_at.month, got.schedule[1].fire_at.day),
        (10, 23)
    );
    // What was shown isn't shown again, and the schedule is asked for again.
    let again = plan(library, utc("2026-10-22T09:00") + 60_000);
    assert!(again.show_now.is_empty());
    assert_eq!(again.schedule, got.schedule);
}

// no-prop-test: a fixed note and the plan with reminders off.
#[test]
fn with_reminders_off_nothing_is_planned_or_recorded() {
    let phone = lone_phone();
    note(&phone.library, "Plans", "- [ ] Call @2026-10-22T09:00\n");
    let off = phone
        .library
        .reminders_plan(utc("2026-10-22T09:00"), OFFSET, false)
        .unwrap();
    assert_eq!(off.show_now, []);
    assert_eq!(off.schedule, []);
    assert_eq!(shown(&phone), None);
    // Turned on, it is all still there.
    let on = plan(&phone.library, utc("2026-10-22T09:00"));
    assert_eq!(keys(&on.show_now), ["Call"]);
}

// no-prop-test: a fixed note and the plans either side of the time it fires.
#[test]
fn a_scheduled_reminder_is_not_shown_by_a_later_plan() {
    let phone = lone_phone();
    note(&phone.library, "Plans", "- [ ] Call @2026-10-22T09:00\n");
    let before = plan(&phone.library, utc("2026-10-22T08:00"));
    assert_eq!(keys(&before.schedule), ["Call"]);
    assert!(before.show_now.is_empty());
    // The system showed it at 09:00; the app opens at 09:05.
    let after = plan(&phone.library, utc("2026-10-22T09:05"));
    assert!(after.show_now.is_empty(), "{after:?}");
    assert!(after.schedule.is_empty());
    // And after a re-plan before it fired, it is still to come.
    let again = plan(&phone.library, utc("2026-10-22T08:30"));
    assert_eq!(keys(&again.schedule), ["Call"]);
}

// no-prop-test: a fixed note and each answer.
#[test]
fn answers_tick_acknowledge_and_say_where_to_open() {
    let phone = lone_phone();
    let library = &phone.library;
    let id = note(
        library,
        "Plans",
        "- [ ] Call Sam @2026-10-22T09:00\n- [ ] Pay @2026-10-22T09:00\n- [ ] Read @2026-10-22T09:00\n",
    );
    let now = utc("2026-10-22T09:01");
    let got = plan(library, now);
    let key = |text: &str| {
        got.show_now
            .iter()
            .find(|n| n.title == text)
            .unwrap()
            .key
            .clone()
    };

    let open = library.reminder_answer(&key("Call Sam"), ReminderAnswer::Open, now);
    assert_eq!(
        open,
        Ok(Some(TaskPlace {
            note_id: id.clone(),
            line: 0
        }))
    );
    assert_eq!(
        library.reminder_answer(&key("Pay"), ReminderAnswer::MarkDone, now),
        Ok(None)
    );
    assert_eq!(
        library.reminder_answer(&key("Read"), ReminderAnswer::Dismiss, now),
        Ok(None)
    );
    let body = library.note(&id).unwrap().body.unwrap();
    assert!(body.contains("- [ ] Call Sam"), "{body}");
    assert!(body.contains("- [x] Pay"), "{body}");
    assert!(body.contains("- [ ] Read"), "{body}");
    // A reminder that is gone (or never was) is nothing to do.
    assert_eq!(
        library.reminder_answer(&key("Pay"), ReminderAnswer::Open, now),
        Ok(None)
    );
    assert_eq!(
        library.reminder_answer("note:x#0@2026-10-22T09:00", ReminderAnswer::Open, now),
        Ok(None)
    );
    // Answered on this device: not planned again, even with the record gone.
    Store::open(phone.path())
        .unwrap()
        .set_meta(SHOWN_META, "{}")
        .unwrap();
    let later = plan(library, now + 60_000);
    assert!(later.show_now.is_empty(), "{later:?}");
}

// no-prop-test: a fixed count past the limit.
#[test]
fn the_schedule_holds_the_next_sixty_in_thirty_days() {
    let phone = lone_phone();
    let body: String = (0..70)
        .map(|i| {
            format!(
                "- [ ] T{i:02} @2026-10-{:02}T{:02}:00\n",
                23 + i / 24 % 8,
                i % 24
            )
        })
        .chain(["- [ ] Far @2026-12-30T09:00\n".to_owned()])
        .collect();
    note(&phone.library, "Many", &body);
    let got = plan(&phone.library, utc("2026-10-22T09:00"));
    assert_eq!(got.schedule.len(), SCHEDULE_LIMIT);
    let times: Vec<_> = got
        .schedule
        .iter()
        .map(|n| n.fire_at)
        .map(|t| (t.month, t.day, t.hour))
        .collect();
    let mut sorted = times.clone();
    sorted.sort();
    assert_eq!(times, sorted);
    assert!(got.schedule.iter().all(|n| n.title != "Far"));
}

// no-prop-test: invalid times are refused by the same check as the Journal's.
#[test]
fn a_plan_survives_any_clock() {
    let phone = lone_phone();
    note(&phone.library, "Plans", "- [ ] Call @2026-10-22T09:00\n");
    for (now, offset) in [(i64::MAX, i64::MAX), (i64::MIN, 0), (0, i64::MIN)] {
        assert!(!matches!(
            phone.library.reminders_plan(now, offset, true),
            Err(KansoError::Internal { .. })
        ));
    }
}

/// What a note in a random state does to its reminders.
#[derive(Clone, Copy, Debug)]
enum State {
    Plain,
    Locked,
    LockedOpen,
    Trashed,
    InLockedNotebook,
}

fn state() -> impl Strategy<Value = State> {
    prop_oneof![
        Just(State::Plain),
        Just(State::Locked),
        Just(State::LockedOpen),
        Just(State::Trashed),
        Just(State::InLockedNotebook),
    ]
}

proptest! {
    #![proptest_config(config(16))]

    /// Invariants: a reminder never comes from a locked note (sealed, open or
    /// in a locked notebook, sealed or not) or from a note in the trash; the
    /// plain notes' reminders are all there; and answering a locked note's
    /// reminder by key writes nothing.
    #[test]
    fn prop_no_reminders_from_locked_or_trashed_notes(
        states in prop::collection::vec(state(), 1..5),
    ) {
        let phone = phone_with(&Arc::new(MemoryServer::new()), |_| {});
        let library = &phone.library;
        let store = Store::open(phone.path()).unwrap();
        let vault = book(&store, "Vault");
        let key = kanso_sync::lock::LockKey::create(
            &kanso_sync::lock::LockPassword::new(LOCK_PASSWORD).unwrap(),
            kanso_sync::KdfParams::TEST,
        ).unwrap();
        // The notebook is locked before any note is slipped into it.
        raw_note(&store, json!({"title": "First", "body": "x", "bookId": vault}));
        kanso_engine::lock::seal_notebook(&store, &vault, &key).unwrap();
        let mut plain = Vec::new();
        let mut hidden: Vec<(String, usize)> = Vec::new();
        for (i, state) in states.iter().enumerate() {
            let body = format!("- [ ] Task{i} @2026-10-22T09:{:02}\n", i + 1);
            match state {
                State::Plain => {
                    plain.push(note(library, &format!("Note{i}"), &body));
                }
                State::Locked | State::LockedOpen => {
                    let id = note(library, &format!("Note{i}"), &body);
                    library.locks().lock_note(&id, LOCK_PASSWORD.into()).unwrap();
                    if matches!(state, State::Locked) {
                        library.locks().lock().unwrap();
                    }
                    hidden.push((id, i));
                }
                State::Trashed => {
                    let id = note(library, &format!("Note{i}"), &body);
                    library.trash(&id).unwrap();
                    hidden.push((id, i));
                }
                State::InLockedNotebook => {
                    let id = raw_note(
                        &store,
                        json!({"title": format!("Note{i}"), "body": body, "bookId": vault}),
                    );
                    hidden.push((id, i));
                }
            }
        }
        let now = utc("2026-10-22T09:00") + 30_000;
        let got = plan(library, now);
        let all: Vec<&ReminderNote> = got.show_now.iter().chain(&got.schedule).collect();
        for note in &all {
            prop_assert!(hidden.iter().all(|(id, _)| *id != note.note_id), "{note:?}");
            prop_assert!(plain.contains(&note.note_id), "{note:?}");
        }
        prop_assert_eq!(all.len(), plain.len());
        for (id, i) in &hidden {
            // Nothing to answer: the key such a task would have.
            let due = format!("2026-10-22T09:{:02}", i + 1);
            let key = kanso_core::reminders::task_key(id, &format!("Task{i} @{due}"), &due);
            prop_assert_eq!(library.reminder_answer(&key, ReminderAnswer::MarkDone, now), Ok(None));
        }
    }

    /// Invariants: a reminder's title is the task's words as a list shows
    /// them and its body is the due words and the note's title; nothing of
    /// the note's other lines is in what the plan says, whatever they are.
    #[test]
    fn prop_a_reminder_says_only_the_task_the_time_and_the_title(
        before in prop::collection::vec("[A-Za-z0-9 #*_>|-]{0,20}", 0..4),
        after in prop::collection::vec("[A-Za-z0-9 #*_>|-]{0,20}", 0..4),
        words in "[a-z][a-z ]{0,12}[a-z]",
        title in "[A-Z][a-z]{0,10}",
        hour in 9u32..20,
    ) {
        let phone = lone_phone();
        let library = &phone.library;
        // The other lines carry a marker no reminder may repeat.
        let marked = |lines: &[String], tag: &str| -> Vec<String> {
            lines.iter().enumerate().map(|(i, l)| format!("LEAK{tag}{i} {l}")).collect()
        };
        let before = marked(&before, "b");
        let after = marked(&after, "a");
        let task = format!("{words} @2026-10-22T{hour:02}:00");
        let body = format!("{}\n- [ ] {task}\n{}\n", before.join("\n\n"), after.join("\n\n"));
        let id = note(library, &title, &body);
        let got = plan(library, utc("2026-10-22T08:00"));
        prop_assert_eq!(got.schedule.len(), 1, "{:?}", got);
        let reminder = &got.schedule[0];
        prop_assert_eq!(&reminder.note_id, &id);
        prop_assert_eq!(&reminder.title, &display_text(&task));
        prop_assert_eq!(&reminder.body, &format!("Today {hour:02}:00 · {title}"));
        let said = format!("{} {}", reminder.title, reminder.body);
        prop_assert!(!said.contains("LEAK"), "{said}");
        // The line is where the note has it.
        let line = body.lines().position(|l| l.starts_with("- [ ] ")).unwrap();
        prop_assert_eq!(reminder.line as usize, line);
    }
}
