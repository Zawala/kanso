//! Signing in, syncing and the status, against an in-memory server.

mod common;

use std::sync::{Arc, Mutex};

use common::*;
use kanso_mobile::{KansoError, Library, ListQuery, SyncPhase, SyncProblemKind};
use kanso_sync::{DEFAULT_SERVER, HttpBackend, MemorySecrets, MemoryServer, SignOutReason, Signup};
use proptest::prelude::*;

fn server_with_account() -> Arc<MemoryServer> {
    let server = Arc::new(MemoryServer::new());
    make_account(&server);
    server
}

// no-prop-test: a fixed walk through the sign-in calls and what each says.
#[test]
fn signing_in_binds_the_library_and_signing_out_leaves_the_notes() {
    let server = server_with_account();
    let phone = phone(&server);
    let sync = phone.library.sync();
    assert_eq!(sync.status().unwrap().phase, SyncPhase::SignedOut);
    assert_eq!(sync.sync_now(), Err(KansoError::SignedOut));

    // A password that can't be one, a wrong one, an unknown account.
    let try_with = |user: &str, password: &str| {
        sync.sign_in_password(
            DEFAULT_SERVER.into(),
            user.into(),
            password.into(),
            "Phone".into(),
        )
    };
    assert!(matches!(
        try_with(ACCOUNT, "short"),
        Err(KansoError::Invalid { .. })
    ));
    assert!(matches!(
        try_with(ACCOUNT, "not the right password"),
        Err(KansoError::Auth { .. })
    ));
    assert!(matches!(
        try_with("nobody", ACCOUNT_PASSWORD),
        Err(KansoError::Auth { .. })
    ));
    assert_eq!(sync.status().unwrap().phase, SyncPhase::SignedOut);
    assert!(
        phone.secrets.items().is_empty(),
        "a failed sign-in keeps nothing"
    );

    let status = try_with(ACCOUNT, ACCOUNT_PASSWORD).unwrap();
    assert_eq!(status.phase, SyncPhase::Idle);
    assert_eq!(status.username.as_deref(), Some(ACCOUNT));
    assert!(status.server.is_some());
    assert!(
        phone
            .secrets
            .items()
            .iter()
            .any(|item| item.starts_with("sync-account:"))
    );
    assert_eq!(sync.status().unwrap(), status);

    let id = note(&phone.library, "Mine", "stays");
    sync.sign_out().unwrap();
    assert_eq!(sync.status().unwrap().phase, SyncPhase::SignedOut);
    assert!(
        !phone
            .secrets
            .items()
            .iter()
            .any(|item| item.starts_with("sync-account:"))
    );
    assert_eq!(phone.library.note(&id).unwrap().title, "Mine");
    // Signing out twice is fine.
    sync.sign_out().unwrap();
}

// no-prop-test: one refusal.
#[test]
fn a_library_with_notes_of_its_own_is_not_bound_to_an_account() {
    let server = server_with_account();
    let phone = phone(&server);
    note(&phone.library, "Already here", "text");
    let err = phone
        .library
        .sync()
        .sign_in_password(
            DEFAULT_SERVER.into(),
            ACCOUNT.into(),
            ACCOUNT_PASSWORD.into(),
            "Phone".into(),
        )
        .unwrap_err();
    assert!(matches!(err, KansoError::Invalid { .. }), "{err:?}");
    assert_eq!(
        phone.library.sync().status().unwrap().phase,
        SyncPhase::SignedOut
    );
    // The server's new session was ended: this phone holds none.
    assert!(
        !phone
            .secrets
            .items()
            .iter()
            .any(|item| item.starts_with("sync-account:"))
    );
}

// no-prop-test: the server answers three ways, and one isn't there.
#[test]
fn a_server_says_whether_it_takes_new_accounts_and_an_absent_one_is_a_network_error() {
    let server = Arc::new(MemoryServer::new());
    let phone = phone(&server);
    let sync = phone.library.sync();
    assert!(
        sync.check_server(DEFAULT_SERVER.into())
            .unwrap()
            .signups_open
    );
    server.set_signup(Signup::Closed);
    assert!(
        !sync
            .check_server(DEFAULT_SERVER.into())
            .unwrap()
            .signups_open
    );

    // The real client, to a port nothing listens on.
    let dir = tempfile::tempdir().unwrap();
    let library = Library::open_for_test(
        &dir.path().join("library.db"),
        Arc::new(MemorySecrets::new()),
        Arc::new(HttpBackend::new()),
    )
    .unwrap();
    let err = library
        .sync()
        .check_server("http://127.0.0.1:1".into())
        .unwrap_err();
    assert!(matches!(err, KansoError::Network { .. }), "{err:?}");
    let err = library
        .sync()
        .sign_in_password(
            "http://127.0.0.1:1".into(),
            ACCOUNT.into(),
            ACCOUNT_PASSWORD.into(),
            "Phone".into(),
        )
        .unwrap_err();
    assert!(matches!(err, KansoError::Network { .. }), "{err:?}");
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    /// Notes written on one phone and synced reach another phone signed in
    /// to the account, with their titles and texts; the first sync reports
    /// what moved.
    #[test]
    fn prop_notes_reach_the_other_phone(
        notes in proptest::collection::vec(("[A-Za-z0-9 ]{1,20}", "[A-Za-z0-9 \n]{0,60}"), 1..5),
    ) {
        let server = server_with_account();
        let (a, b) = (phone(&server), phone(&server));
        sign_in(&a, "Phone A");
        sign_in(&b, "Phone B");
        let ids: Vec<_> = notes.iter().map(|(title, body)| note(&a.library, title, body)).collect();
        let sent = a.library.sync().sync_now().unwrap();
        prop_assert!(sent.pushed as usize >= notes.len());
        prop_assert_eq!(sent.pending, 0);
        let got = b.library.sync().sync_now().unwrap();
        prop_assert!(got.pulled as usize >= notes.len());
        for (id, (title, body)) in ids.iter().zip(&notes) {
            let view = b.library.note(id).unwrap();
            prop_assert_eq!(&view.title, title);
            prop_assert_eq!(view.body.as_deref(), Some(body.as_str()));
        }
        let status = b.library.sync().status().unwrap();
        prop_assert_eq!(status.phase, SyncPhase::Synced);
        prop_assert!(status.last_synced_ms.is_some());
        prop_assert_eq!(b.library.list(ListQuery::default()).unwrap().rows.len(), notes.len());
    }
}

// no-prop-test: a fixed run, read as the sidebar sees it.
#[test]
fn the_status_stream_follows_a_sign_in_a_sync_and_a_sign_out() {
    let server = server_with_account();
    let phone = phone(&server);
    let sync = phone.library.sync();
    let phases = Arc::new(Mutex::new(Vec::new()));
    let log = phases.clone();
    sync.status_stream(move |status| {
        log.lock().unwrap().push(status.phase);
        true
    });
    sign_in(&phone, "Phone");
    note(&phone.library, "One", "text");
    sync.sync_now().unwrap();
    sync.sign_out().unwrap();
    assert_eq!(
        *phases.lock().unwrap(),
        vec![
            SyncPhase::SignedOut,
            SyncPhase::Idle,
            SyncPhase::Syncing,
            SyncPhase::Synced,
            SyncPhase::SignedOut,
        ]
    );
}

// no-prop-test: a fixed run: the server ends the session, and the account is
// turned off.
#[test]
fn a_session_the_server_ended_signs_the_phone_out_and_a_turned_off_account_fails() {
    let server = server_with_account();
    let phone = phone(&server);
    sign_in(&phone, "Phone");
    server.disable(ACCOUNT);
    let err = phone.library.sync().sync_now().unwrap_err();
    assert!(matches!(err, KansoError::Auth { .. }), "{err:?}");
    let status = phone.library.sync().status().unwrap();
    assert_eq!(status.phase, SyncPhase::Failed);
    assert_eq!(status.problem, Some(SyncProblemKind::AccountDisabled));

    server.enable(ACCOUNT);
    phone.library.sync().sync_now().unwrap();
    server.revoke_all(ACCOUNT, SignOutReason::Revoked);
    assert_eq!(phone.library.sync().sync_now(), Err(KansoError::SignedOut));
    assert_eq!(
        phone.library.sync().status().unwrap().phase,
        SyncPhase::SignedOut
    );
    assert!(
        !phone
            .secrets
            .items()
            .iter()
            .any(|item| item.starts_with("sync-account:"))
    );
}

// no-prop-test: the secret store refusing, once.
#[test]
fn a_secret_store_that_refuses_is_an_auth_error_not_a_signed_out_phone() {
    let server = server_with_account();
    let phone = phone(&server);
    sign_in(&phone, "Phone");
    phone.secrets.fail_reads(true);
    let err = phone.library.sync().sync_now().unwrap_err();
    assert!(matches!(err, KansoError::Auth { .. }), "{err:?}");
    assert!(matches!(
        phone.library.sync().status(),
        Err(KansoError::Auth { .. })
    ));
    phone.secrets.heal();
    assert_eq!(
        phone.library.sync().status().unwrap().phase,
        SyncPhase::Idle
    );
}

// no-prop-test: one constant, the one the desktop app signs in to.
#[test]
fn the_default_server_is_the_owners() {
    assert_eq!(
        kanso_mobile::SyncHandle::default_server(),
        kanso_sync::DEFAULT_SERVER
    );
    assert!(kanso_mobile::SyncHandle::default_server().starts_with("https://"));
}
