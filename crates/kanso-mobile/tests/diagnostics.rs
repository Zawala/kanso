//! A panic is an error and the library goes on working.

mod common;

use std::sync::{Arc, Mutex};

use common::*;
use kanso_mobile::diagnostics::{panic_probe, tls_ready, version};
use kanso_mobile::{KansoError, ListQuery, LockEvent};

// no-prop-test: the probe's message is any text; boundary.rs covers that by
// property. This is the facade end to end.
#[test]
fn the_probe_is_an_error_and_every_call_still_works_after_it() {
    let phone = lone_phone();
    let id = note(&phone.library, "Before", "text");
    let failed = panic_probe("on purpose".into());
    assert_eq!(
        failed,
        Err(KansoError::Internal {
            message: "on purpose".into()
        })
    );
    // Again, and from threads at once.
    let threads: Vec<_> = (0..4)
        .map(|n| std::thread::spawn(move || panic_probe(format!("thread {n}"))))
        .collect();
    for thread in threads {
        assert!(matches!(
            thread.join().unwrap(),
            Err(KansoError::Internal { .. })
        ));
    }
    assert_eq!(phone.library.note(&id).unwrap().title, "Before");
    assert_eq!(
        phone.library.list(ListQuery::default()).unwrap().rows.len(),
        1
    );
    assert!(
        phone
            .library
            .note(&note(&phone.library, "After", "text"))
            .is_ok()
    );
    assert!(!version().is_empty());
    assert_eq!(tls_ready(), !cfg!(target_os = "android"));
}

// no-prop-test: a sink that panics while the library holds its own locks.
#[test]
fn a_listener_that_panics_is_dropped_and_the_library_goes_on() {
    let phone = lone_phone();
    let locks = phone.library.locks();
    let id = note(&phone.library, "Plan", "text");
    locks.lock_note(&id, LOCK_PASSWORD.into()).unwrap();
    let heard = Arc::new(Mutex::new(Vec::new()));
    let log = heard.clone();
    locks.lock_events(move |event: LockEvent| {
        log.lock().unwrap().push(event);
        true
    });
    locks.lock_events(|_| panic!("a bad listener"));
    // Closing the notes tells both; the bad one panics and is dropped, and
    // the call itself is fine.
    locks.lock().unwrap();
    assert!(!locks.is_unlocked().unwrap());
    assert_eq!(locks.unlock(LOCK_PASSWORD.into()), Ok(true));
    assert_eq!(
        *heard.lock().unwrap(),
        vec![
            LockEvent::Locked {
                reason: kanso_mobile::LockReason::Manual
            },
            LockEvent::Unlocked
        ]
    );
}
