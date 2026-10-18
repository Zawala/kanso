//! The account flows end to end, in real windows, against `MemoryServer` and
//! `MemorySecrets` (docs/ACCOUNTS.md › Account flows, › Library binding, ›
//! Services): sign-up and its recovery key, a second Mac signing in and the
//! question about its notes, another account's notes, signing out, a
//! password change, recovery, a new recovery key, deleting the account, and
//! a relaunch. Each test context is one Mac with its own library and
//! Keychain; the server is shared. Nothing here reaches a network or the
//! real Keychain. Signing in starts the real sync (`kanso_sync::sync` on its
//! own thread, as in the app), so the tests also check what reaches the
//! server and the other Macs, and that account tasks stop a running sync
//! first.

use std::sync::Arc;
use std::time::{Duration, Instant};

use async_channel::Receiver;
use gpui_kit::component::WindowExt as _;
use gpui_kit::{AnyWindowHandle, AppContext as _, Modifiers, TestAppContext, VisualTestContext};
use kanso_core::DocType;
use kanso_store::{Origin, Put, Store};
use kanso_sync::{
    AccountError, Api, Backend, DeviceIdentity, KdfParams, Machine, MemorySecrets, MemoryServer,
    Password, RecoveryKey, Route, SecretStore, Session, SignOutReason, meta_keys,
};
use serde_json::json;

use crate::account_flow;
use crate::account_words as words;
use crate::config::AppConfig;
use crate::dialogs::Progress;
use crate::services::{self, AccountState, Services, SyncState};
use crate::settings::SettingsPage;
use crate::workspace::{AppState, Workspace};

/// The forms' server; `MemoryServer` answers for any.
const SERVER: &str = kanso_sync::DEFAULT_SERVER;
const PASSWORD: &str = "correct horse battery staple";
const NEW_PASSWORD: &str = "a different long passphrase";

/// The shared server, resending a request whose answer was lost at once.
struct Quick(Arc<MemoryServer>);

impl Backend for Quick {
    fn api(&self, server: &str) -> Box<dyn Api + '_> {
        self.0.api(server)
    }
    fn kdf_floor(&self) -> KdfParams {
        self.0.kdf_floor()
    }
    fn kdf_default(&self) -> KdfParams {
        self.0.kdf_default()
    }
    fn retry_pause(&self, _: usize) -> Duration {
        Duration::ZERO
    }
}

/// One Mac: its library, its Keychain and its window.
struct Mac {
    dir: tempfile::TempDir,
    store: Arc<Store>,
    secrets: Arc<MemorySecrets>,
    window: AnyWindowHandle,
}

/// A Mac called `name` with `notes` notes, using `server`.
fn mac(cx: &mut TestAppContext, server: &Arc<MemoryServer>, name: &str, notes: usize) -> Mac {
    // The account tasks run on threads of their own and wake the app from
    // there, as real I/O would.
    cx.executor().allow_parking();
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    if notes > 0 {
        let book = store
            .put(
                DocType::Book,
                json!({"name": "Work"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        for n in 0..notes {
            store
                .put(
                    DocType::Note,
                    json!({"title": format!("{name} note {n}"), "body": "text", "bookId": book}),
                    Put::Upsert,
                    Origin::Ui,
                )
                .unwrap();
        }
    }
    let secrets = Arc::new(MemorySecrets::new());
    let backend: Arc<dyn Backend> = Arc::new(Quick(server.clone()));
    let (app_store, data_dir, keychain) =
        (store.clone(), dir.path().to_path_buf(), secrets.clone());
    let machine = Machine {
        hardware_uuid: Some(format!("UUID-{name}")),
        computer_name: name.to_owned(),
    };
    let (window, _) = cx.update(|cx| {
        gpui_kit::init(cx);
        cx.set_global(AppState {
            store: app_store,
            data_dir,
            config: AppConfig::default(),
        });
        cx.set_global(services(backend, keychain, machine));
        crate::workspace::bind_keys(cx);
        gpui_kit::open_window(Default::default(), cx, |window, cx| {
            cx.new(|cx| Workspace::new(window, cx))
        })
        .unwrap()
    });
    cx.run_until_parked();
    Mac {
        dir,
        store,
        secrets,
        window,
    }
}

/// Services as `main.rs` builds them, with the test seams.
fn services(
    backend: Arc<dyn Backend>,
    secrets: Arc<dyn SecretStore>,
    machine: Machine,
) -> Services {
    let mut services = Services::default();
    services.backend = Some(backend);
    services.secrets = Some(secrets);
    services.machine = Some(machine);
    services
}

fn password(text: &str, username: &str) -> Password {
    Password::new(text, username).unwrap()
}

/// Runs the app until `rx` says the task is done.
fn finish(cx: &mut TestAppContext, rx: Receiver<Progress>) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(120);
    loop {
        cx.run_until_parked();
        while let Ok(progress) = rx.try_recv() {
            if let Progress::Done(result) = progress {
                cx.run_until_parked();
                return result;
            }
        }
        assert!(Instant::now() < deadline, "the account task finished");
        std::thread::sleep(Duration::from_millis(2));
    }
}

/// Runs the app until `done` holds; the tasks run on their own threads.
fn wait_until(cx: &mut TestAppContext, what: &str, done: impl Fn(&mut TestAppContext) -> bool) {
    let deadline = Instant::now() + Duration::from_secs(120);
    loop {
        cx.run_until_parked();
        if done(cx) {
            return;
        }
        assert!(Instant::now() < deadline, "waited for {what}");
        std::thread::sleep(Duration::from_millis(2));
    }
}

/// Waits for the sync that is running (or about to start) to finish, and
/// returns its state.
fn synced(cx: &mut TestAppContext) -> SyncState {
    wait_until(cx, "the sync to finish", |cx| {
        cx.read(|cx| {
            let services = cx.global::<Services>();
            matches!(
                services.sync,
                SyncState::Synced { .. } | SyncState::Failed(_)
            ) && services.task.is_none()
        })
    });
    cx.read(|cx| cx.global::<Services>().sync.clone())
}

/// Syncs now and waits for it.
fn sync_now(cx: &mut TestAppContext) -> SyncState {
    cx.update(|cx| {
        // The last sync's state goes, so the wait sees this one's.
        cx.global_mut::<Services>().sync = SyncState::Syncing;
        services::sync_now(cx);
    });
    synced(cx)
}

fn ok(state: &SyncState) -> bool {
    matches!(state, SyncState::Synced { problem: None, .. })
}

fn account(cx: &mut TestAppContext) -> AccountState {
    cx.read(|cx| cx.global::<Services>().account.clone())
}

fn session(cx: &mut TestAppContext) -> Arc<Session> {
    cx.read(|cx| cx.global::<Services>().session())
        .expect("signed in")
}

/// The key screen's words over the key, if any.
fn key_note(cx: &mut TestAppContext) -> Option<&'static str> {
    cx.read(|cx| {
        cx.global::<Services>()
            .pending_key
            .as_ref()
            .and_then(|pending| pending.note)
    })
}

/// Whether `key` is the account's recovery key on `server` now.
fn holds(server: &MemoryServer, username: &str, key: &RecoveryKey) -> bool {
    let origin = kanso_sync::origin(SERVER).unwrap();
    let auth = kanso_sync::crypto::recovery_keys(key, &origin, username).auth_key;
    server
        .api(SERVER)
        .recover_start(&kanso_sync::api::RecoverStartRequest {
            username: username.into(),
            recovery_auth_key: auth.to_base64url(),
        })
        .is_ok()
}

/// "I've saved my recovery key", and its Keychain item's removal.
fn key_saved(cx: &mut TestAppContext) {
    cx.update(account_flow::key_saved);
    cx.run_until_parked();
}

fn pending_key(cx: &mut TestAppContext) -> Option<RecoveryKey> {
    cx.read(|cx| {
        cx.global::<Services>()
            .pending_key
            .as_ref()
            .map(|pending| pending.key.clone())
    })
}

fn library_id(cx: &mut TestAppContext) -> String {
    cx.read(|cx| cx.global::<Services>().library_id.clone())
        .expect("the library's ID is known")
}

fn signed_in_as(username: &str) -> AccountState {
    AccountState::SignedIn {
        username: username.to_owned(),
    }
}

impl Mac {
    fn notes(&self) -> usize {
        self.store
            .counts()
            .unwrap()
            .get(&DocType::Note)
            .copied()
            .unwrap_or(0)
    }

    fn bound_to(&self) -> Option<String> {
        self.store.meta(meta_keys::ACCOUNT_ID).unwrap()
    }

    fn items(&self) -> Vec<String> {
        self.secrets.items()
    }

    fn draw(&self, cx: &mut TestAppContext) {
        self.window
            .update(cx, |_, window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
            })
            .unwrap();
        cx.run_until_parked();
    }

    /// The library's unsaved recovery key, as the Keychain holds it.
    fn kept_key(&self) -> Option<kanso_sync::keychain::PendingRecoveryKey> {
        let library = self
            .items()
            .iter()
            .find_map(|item| item.strip_prefix("sync-recovery-key:").map(str::to_owned))?;
        kanso_sync::keychain::load_any_recovery_key(self.secrets.as_ref(), &library).unwrap()
    }

    fn notifications(&self, cx: &mut TestAppContext) -> usize {
        self.window
            .update(cx, |_, window, cx| window.notifications(cx).len())
            .unwrap()
    }

    /// Whether the window shows Settings › Account.
    fn showing_account(&self, cx: &mut TestAppContext) -> bool {
        cx.update(|cx| {
            let mut showing = false;
            crate::workspace::for_each_workspace(cx, |workspace, cx| {
                showing |= workspace.settings_open()
                    && workspace.settings.read(cx).page() == SettingsPage::Account;
            });
            showing
        })
    }

    /// Clicks what `selector` names in the window.
    fn click(&self, cx: &mut TestAppContext, selector: &'static str) {
        self.draw(cx);
        let mut vcx = VisualTestContext::from_window(self.window, cx);
        let bounds = vcx
            .debug_bounds(selector)
            .unwrap_or_else(|| panic!("{selector} is drawn"));
        vcx.simulate_click(bounds.center(), Modifiers::none());
        cx.run_until_parked();
    }

    fn asking(&self, cx: &mut TestAppContext) -> bool {
        self.window
            .update(cx, |_, window, cx| window.has_active_dialog(cx))
            .unwrap()
    }

    /// Waits for the binding question, then clicks `selector` in it.
    fn answer(&self, cx: &mut TestAppContext, selector: &'static str) {
        wait_until(cx, "the binding question", |cx| self.asking(cx));
        self.draw(cx);
        let mut vcx = VisualTestContext::from_window(self.window, cx);
        let bounds = vcx
            .debug_bounds(selector)
            .unwrap_or_else(|| panic!("{selector} is drawn"));
        vcx.simulate_click(bounds.center(), Modifiers::none());
        cx.run_until_parked();
    }

    fn sign_up(&self, cx: &mut TestAppContext, username: &str) -> Receiver<Progress> {
        let (window, username) = (self.window, username.to_owned());
        cx.update(|cx| {
            let password = password(PASSWORD, &username);
            account_flow::sign_up(SERVER.into(), username, password, window, cx)
        })
    }

    fn sign_in(&self, cx: &mut TestAppContext, username: &str, text: &str) -> Receiver<Progress> {
        let (window, username) = (self.window, username.to_owned());
        let password = password(text, &username);
        cx.update(|cx| account_flow::sign_in(SERVER.into(), username, password, window, cx))
    }

    fn recover(
        &self,
        cx: &mut TestAppContext,
        username: &str,
        key: RecoveryKey,
        text: &str,
    ) -> Receiver<Progress> {
        let (window, username) = (self.window, username.to_owned());
        let password = password(text, &username);
        cx.update(|cx| {
            account_flow::recover(SERVER.into(), username, key, password, Some(window), cx)
        })
    }
}

fn sign_out(cx: &mut TestAppContext) -> Result<(), String> {
    let rx = cx.update(account_flow::sign_out);
    finish(cx, rx)
}

/// Waits for the Account page's storage and devices after signing in.
fn details_loaded(cx: &mut TestAppContext) {
    wait_until(cx, "the account's details", |cx| {
        cx.read(|cx| {
            let details = &cx.global::<Services>().details;
            !details.loading && (details.devices.is_some() || details.failed.is_some())
        })
    });
}

/// Signing up creates the account, binds this library to it with its notes,
/// keeps the session and the new recovery key in the Keychain, and shows the
/// key until "I've saved my recovery key", which removes its item.
#[gpui_kit::test]
fn signing_up_binds_this_library_and_shows_the_key(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 3);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));

    assert_eq!(account(cx), signed_in_as("alice"));
    let session = session(cx);
    assert_eq!(a.bound_to(), Some(session.account_id.clone()));
    assert_eq!(
        a.store.meta(meta_keys::USERNAME).unwrap().as_deref(),
        Some("alice")
    );
    assert_eq!(a.notes(), 3, "the notes stay, to be uploaded");
    let library = library_id(cx);
    assert_eq!(
        a.items(),
        [
            format!("sync-account:{library}"),
            format!("sync-recovery-key:{library}")
        ]
    );
    let config = cx.read(|cx| cx.global::<AppState>().config.sync.clone());
    let saved = config.account.expect("the config names the account");
    assert_eq!(
        (saved.username.as_str(), saved.id.as_str()),
        ("alice", session.account_id.as_str())
    );
    assert!(config.enabled, "signing up turns sync on");
    assert!(pending_key(cx).is_some(), "the key screen shows");

    key_saved(cx);
    cx.run_until_parked();
    assert!(pending_key(cx).is_none());
    assert_eq!(a.items(), [format!("sync-account:{library}")]);

    details_loaded(cx);
    let devices = cx.read(|cx| cx.global::<Services>().details.devices.clone());
    let devices = devices.expect("the devices load");
    assert_eq!(devices.len(), 1);
    assert!(devices[0].current);

    // The first sync uploads the notes and their book, encrypted.
    let state = synced(cx);
    assert!(ok(&state), "{state:?}");
    let view = server.plain_view(&session);
    assert_eq!(view.docs().len(), 4);
    let local: Vec<String> = a
        .store
        .changes_since(0, None, false)
        .unwrap()
        .into_iter()
        .map(|doc| doc.id)
        .collect();
    assert!(
        view.wire_docs().iter().all(|doc| !local.contains(&doc.id)),
        "the server never sees an ID"
    );
}

/// A second Mac with notes of its own signs in: it is asked about them, and
/// adding them binds it to the same account with its notes kept. An empty
/// Mac signs in without a question.
#[gpui_kit::test]
fn a_second_mac_is_asked_about_its_notes(
    cx: &mut TestAppContext,
    cx_b: &mut TestAppContext,
    cx_c: &mut TestAppContext,
) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 2);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));
    let account_id = a.bound_to().unwrap();
    assert!(ok(&synced(cx)));

    let b = mac(cx_b, &server, "Laptop", 4);
    let rx = b.sign_in(cx_b, "alice", PASSWORD);
    b.answer(cx_b, "choice-0");
    assert_eq!(finish(cx_b, rx), Ok(()));
    assert_eq!(account(cx_b), signed_in_as("alice"));
    assert_eq!(b.bound_to(), Some(account_id.clone()));
    assert!(pending_key(cx_b).is_none(), "signing in makes no new key");
    let config = cx_b.read(|cx| cx.global::<AppState>().config.sync.clone());
    assert!(config.enabled, "signing in turns sync on");
    details_loaded(cx_b);
    let devices = cx_b.read(|cx| cx.global::<Services>().details.devices.clone());
    assert_eq!(devices.map(|d| d.len()), Some(2), "both Macs are devices");
    // Add keeps this Mac's notes; its first sync brings alice's and sends
    // its own.
    assert!(ok(&synced(cx_b)));
    assert_eq!(b.notes(), 6);
    assert!(ok(&sync_now(cx)));
    assert_eq!(a.notes(), 6, "the first Mac gets the second's notes");

    let c = mac(cx_c, &server, "Mini", 0);
    let rx = c.sign_in(cx_c, "alice", PASSWORD);
    assert_eq!(finish(cx_c, rx), Ok(()));
    assert!(!c.asking(cx_c), "an empty library needs no question");
    assert_eq!(c.bound_to(), Some(account_id));
    assert!(ok(&synced(cx_c)));
    assert_eq!(c.notes(), 6);
}

/// Replace saves a backup of the library first, then empties it for the
/// account's notes.
#[gpui_kit::test]
fn replacing_the_notes_keeps_a_backup_first(cx: &mut TestAppContext, cx_b: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 1);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));
    assert!(ok(&synced(cx)));

    let b = mac(cx_b, &server, "Laptop", 3);
    let rx = b.sign_in(cx_b, "alice", PASSWORD);
    b.answer(cx_b, "choice-1");
    assert_eq!(finish(cx_b, rx), Ok(()));
    assert_eq!(account(cx_b), signed_in_as("alice"));
    assert_eq!(b.bound_to(), a.bound_to());
    let backups: Vec<_> = std::fs::read_dir(crate::paths::backups_dir(b.dir.path()))
        .expect("a backups folder")
        .collect();
    assert_eq!(backups.len(), 1, "one backup, made first");
    assert!(ok(&synced(cx_b)));
    assert_eq!(b.notes(), 1, "the account's notes, and only those");
    assert!(ok(&sync_now(cx)));
    assert_eq!(a.notes(), 1, "the replaced notes never reach the account");
}

/// Signing in to another account on a Mac bound to the first asks about
/// that account's notes; Cancel signs the new session out and changes
/// nothing here.
#[gpui_kit::test]
fn another_accounts_notes_are_asked_about(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let elsewhere = DeviceIdentity::from_parts(&format!("l_{:032x}", 9), "Elsewhere", [9; 32]);
    let bob = kanso_sync::sign_up(
        server.as_ref(),
        &MemorySecrets::new(),
        SERVER,
        "bob",
        &password(PASSWORD, "bob"),
        &elsewhere,
        &|_| {},
    )
    .unwrap();

    let a = mac(cx, &server, "Studio", 2);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));
    key_saved(cx);
    let alice = a.bound_to();
    assert_eq!(sign_out(cx), Ok(()));

    let rx = a.sign_in(cx, "bob", PASSWORD);
    a.answer(cx, "choice-cancel");
    assert_eq!(finish(cx, rx), Ok(()));
    assert_eq!(account(cx), AccountState::SignedOut);
    assert_eq!(a.bound_to(), alice, "still alice's library");
    assert_eq!(a.notes(), 2);
    assert!(a.items().is_empty(), "no session is kept");
    let devices = kanso_sync::devices(server.as_ref(), &bob.session).unwrap();
    assert_eq!(devices.len(), 1, "the cancelled session is gone");
}

/// Cancelling the question about another account while signed in leaves this
/// one as it was: its Keychain item, its session on the server, and its
/// syncing, which starts again.
#[gpui_kit::test]
fn cancelling_another_account_keeps_this_one(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let elsewhere = DeviceIdentity::from_parts(&format!("l_{:032x}", 9), "Elsewhere", [9; 32]);
    let bob = kanso_sync::sign_up(
        server.as_ref(),
        &MemorySecrets::new(),
        SERVER,
        "bob",
        &password(PASSWORD, "bob"),
        &elsewhere,
        &|_| {},
    )
    .unwrap();
    let a = mac(cx, &server, "Studio", 2);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));
    assert!(ok(&synced(cx)));
    let alice = session(cx);
    let library = library_id(cx);
    let items = a.items();

    // Signing in stops the running sync first.
    let window = a.window;
    let rx = cx.update(|cx| {
        services::sync_now(cx);
        account_flow::sign_in(
            SERVER.into(),
            "bob".into(),
            password(PASSWORD, "bob"),
            window,
            cx,
        )
    });
    a.answer(cx, "choice-cancel");
    assert_eq!(finish(cx, rx), Ok(()));
    assert_eq!(account(cx), signed_in_as("alice"));
    assert!(Arc::ptr_eq(&session(cx), &alice));
    assert_eq!(a.items(), items, "alice's items stay");
    let kept = kanso_sync::keychain::load_session(
        a.secrets.as_ref(),
        &library,
        &alice.origin,
        &alice.account_id,
    )
    .unwrap();
    assert!(kept.is_some(), "a relaunch still finds alice");
    assert!(kanso_sync::usage(server.as_ref(), &alice).is_ok());
    let devices = kanso_sync::devices(server.as_ref(), &bob.session).unwrap();
    assert_eq!(devices.len(), 1, "bob's cancelled session is gone");
    assert!(ok(&synced(cx)), "alice syncs again");
}

/// A recovery whose binding question is cancelled keeps the new recovery
/// key on screen and in the Keychain: the used one no longer works.
#[gpui_kit::test]
fn a_cancelled_recovery_keeps_the_new_key(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let elsewhere = DeviceIdentity::from_parts(&format!("l_{:032x}", 9), "Elsewhere", [9; 32]);
    let bob = kanso_sync::sign_up(
        server.as_ref(),
        &MemorySecrets::new(),
        SERVER,
        "bob",
        &password(PASSWORD, "bob"),
        &elsewhere,
        &|_| {},
    )
    .unwrap();
    let a = mac(cx, &server, "Studio", 2);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));
    key_saved(cx);
    assert_eq!(sign_out(cx), Ok(()));
    let library = library_id(cx);

    let rx = a.recover(cx, "bob", bob.recovery_key.clone(), NEW_PASSWORD);
    a.answer(cx, "choice-cancel");
    assert_eq!(finish(cx, rx), Ok(()));
    assert_eq!(account(cx), AccountState::SignedOut);
    let shown = pending_key(cx).expect("the new key shows");
    assert!(shown != bob.recovery_key);
    assert!(holds(&server, "bob", &shown));
    assert_eq!(a.items(), [format!("sync-recovery-key:{library}")]);
    let devices = kanso_sync::devices(server.as_ref(), &bob.session);
    assert!(devices.is_err(), "recovery signed bob's other session out");

    // A relaunch, signed out, shows it again.
    let backend: Arc<dyn Backend> = Arc::new(Quick(server.clone()));
    let secrets: Arc<dyn SecretStore> = a.secrets.clone();
    cx.update(|cx| {
        let machine = cx.global::<Services>().machine.clone().unwrap();
        cx.set_global(services(backend, secrets, machine));
        account_flow::restore(cx);
    });
    wait_until(cx, "the launch check", |cx| {
        cx.read(|cx| cx.global::<Services>().library_id.is_some())
    });
    assert_eq!(account(cx), AccountState::SignedOut);
    assert_eq!(pending_key(cx), Some(shown));
    assert_eq!(key_note(cx), None, "the server took it");
}

/// Signing out while a sync runs stops it first ("Stopping sync…"), and its
/// result changes nothing afterwards.
#[gpui_kit::test]
fn signing_out_during_a_sync_stops_it_first(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 40);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));
    assert!(ok(&synced(cx)));
    let old = session(cx);

    let rx = cx.update(|cx| {
        services::sync_now(cx);
        account_flow::sign_out(cx)
    });
    assert_eq!(
        rx.try_recv().ok(),
        Some(Progress::Step(kanso_sync::Step::StoppingSync))
    );
    assert_eq!(finish(cx, rx), Ok(()));
    assert_eq!(account(cx), AccountState::SignedOut);
    for _ in 0..20 {
        cx.run_until_parked();
        std::thread::sleep(Duration::from_millis(2));
    }
    let state = cx.read(|cx| cx.global::<Services>().sync.clone());
    assert_eq!(state, SyncState::Off, "the stopped sync's news is dropped");
    assert!(a.items().is_empty());
    assert!(matches!(
        kanso_sync::usage(server.as_ref(), &old),
        Err(AccountError::Unauthorized { .. })
    ));
}

/// Signing out ends the session on the server, forgets it here, and keeps
/// the notes and the binding: signing in again resumes without a question.
#[gpui_kit::test]
fn signing_out_keeps_the_notes(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 2);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));
    let old = session(cx);
    let bound = a.bound_to();

    assert_eq!(sign_out(cx), Ok(()));
    assert_eq!(account(cx), AccountState::SignedOut);
    assert!(pending_key(cx).is_none());
    assert!(a.items().is_empty(), "the session and key items go");
    let config = cx.read(|cx| cx.global::<AppState>().config.sync.account.clone());
    assert!(config.is_none());
    assert_eq!(a.notes(), 2);
    assert_eq!(a.bound_to(), bound);
    assert!(matches!(
        kanso_sync::usage(server.as_ref(), &old),
        Err(AccountError::Unauthorized { .. })
    ));

    let rx = a.sign_in(cx, "alice", PASSWORD);
    assert_eq!(finish(cx, rx), Ok(()));
    assert!(!a.asking(cx), "the same account resumes");
    assert_eq!(account(cx), signed_in_as("alice"));
    assert_eq!(a.bound_to(), bound);
}

/// Changing the password keeps this Mac signed in and signs the others out
/// with the reason; they sign in again with the new password only.
#[gpui_kit::test]
fn changing_the_password_signs_the_other_macs_out(
    cx: &mut TestAppContext,
    cx_b: &mut TestAppContext,
) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 1);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));
    let b = mac(cx_b, &server, "Laptop", 0);
    let rx = b.sign_in(cx_b, "alice", PASSWORD);
    assert_eq!(finish(cx_b, rx), Ok(()));
    details_loaded(cx_b);

    let rx = cx.update(|cx| {
        account_flow::change_password(
            password("not the password at all", "alice"),
            password(NEW_PASSWORD, "alice"),
            cx,
        )
    });
    assert_eq!(
        finish(cx, rx),
        Err("That isn't your current password.".to_owned())
    );
    let first = pending_key(cx).unwrap();
    key_saved(cx);
    let rx = cx.update(|cx| {
        account_flow::change_password(
            password(PASSWORD, "alice"),
            password(NEW_PASSWORD, "alice"),
            cx,
        )
    });
    assert_eq!(finish(cx, rx), Ok(()));
    assert_eq!(account(cx), signed_in_as("alice"));
    // The recovery key changed with the password.
    let second = pending_key(cx).expect("the new key shows");
    assert_eq!(key_note(cx), Some(words::KEY_AFTER_PASSWORD));
    assert!(holds(&server, "alice", &second));
    assert!(
        !holds(&server, "alice", &first),
        "the saved key stops working"
    );

    cx_b.update(account_flow::refresh_details);
    wait_until(cx_b, "the other Mac to hear", |cx| {
        matches!(account(cx), AccountState::SessionEnded { .. })
    });
    assert_eq!(
        account(cx_b),
        AccountState::SessionEnded {
            username: "alice".into(),
            reason: SignOutReason::PasswordChanged,
        }
    );
    assert!(
        b.items().is_empty(),
        "the ended session leaves the Keychain"
    );

    let rx = b.sign_in(cx_b, "alice", PASSWORD);
    assert_eq!(
        finish(cx_b, rx),
        Err("That username and password don't match.".to_owned())
    );
    let rx = b.sign_in(cx_b, "alice", NEW_PASSWORD);
    assert_eq!(finish(cx_b, rx), Ok(()));
    assert_eq!(account(cx_b), signed_in_as("alice"));
}

/// The recovery key sets a new password and signs this Mac in; Kanso shows
/// a new key, and the used one stops working. A new recovery key replaces
/// the current one the same way.
#[gpui_kit::test]
fn recovery_replaces_the_key(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 1);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));
    let first = pending_key(cx).unwrap();
    key_saved(cx);
    assert_eq!(sign_out(cx), Ok(()));

    let rx = a.recover(cx, "alice", first.clone(), NEW_PASSWORD);
    assert_eq!(finish(cx, rx), Ok(()));
    assert_eq!(account(cx), signed_in_as("alice"));
    let second = pending_key(cx).expect("a new key shows");
    assert!(second != first);
    let library = library_id(cx);
    assert!(a.items().contains(&format!("sync-recovery-key:{library}")));
    key_saved(cx);

    let rx = cx.update(|cx| account_flow::new_recovery_key(password(NEW_PASSWORD, "alice"), cx));
    assert_eq!(finish(cx, rx), Ok(()));
    let third = pending_key(cx).expect("the new key shows");
    assert!(third != second);
    key_saved(cx);

    assert_eq!(sign_out(cx), Ok(()));
    for used in [first, second] {
        let rx = a.recover(cx, "alice", used, "yet another long passphrase");
        assert_eq!(
            finish(cx, rx),
            Err("That recovery key doesn't match this account.".to_owned())
        );
    }
    let rx = a.sign_in(cx, "alice", PASSWORD);
    assert_eq!(
        finish(cx, rx),
        Err("That username and password don't match.".to_owned())
    );
    let rx = a.recover(cx, "alice", third, "yet another long passphrase");
    assert_eq!(finish(cx, rx), Ok(()));
    assert_eq!(account(cx), signed_in_as("alice"));
}

/// Deleting the account needs its name and password, unbinds the library
/// and keeps the notes; the account can't be signed in to afterwards.
#[gpui_kit::test]
fn deleting_the_account_keeps_the_notes(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 2);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));

    let rx = cx
        .update(|cx| account_flow::delete_account(password(PASSWORD, "alice"), "alice".into(), cx));
    assert_eq!(finish(cx, rx), Ok(()));
    assert_eq!(account(cx), AccountState::SignedOut);
    assert_eq!(a.bound_to(), None);
    assert_eq!(a.notes(), 2);
    assert!(a.items().is_empty());
    let config = cx.read(|cx| cx.global::<AppState>().config.sync.account.clone());
    assert!(config.is_none());

    let rx = a.sign_in(cx, "alice", PASSWORD);
    assert_eq!(
        finish(cx, rx),
        Err("That username and password don't match.".to_owned())
    );
}

/// At launch the session comes back when the config, the binding and the
/// Keychain agree, with the key screen if the key wasn't saved yet; without
/// the Keychain item, the Mac starts signed out and the config forgets the
/// account.
#[gpui_kit::test]
fn relaunching_restores_the_session(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 1);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));
    let first = session(cx);
    let library = library_id(cx);

    let relaunch = |cx: &mut TestAppContext| {
        let backend: Arc<dyn Backend> = Arc::new(Quick(server.clone()));
        let secrets: Arc<dyn SecretStore> = a.secrets.clone();
        cx.update(|cx| {
            let machine = cx.global::<Services>().machine.clone().unwrap();
            cx.set_global(services(backend, secrets, machine));
            account_flow::restore(cx);
        });
        wait_until(cx, "the launch check", |cx| {
            cx.read(|cx| cx.global::<Services>().library_id.is_some())
        });
    };

    relaunch(cx);
    assert_eq!(account(cx), signed_in_as("alice"));
    assert_eq!(session(cx).account_id, first.account_id);
    assert!(pending_key(cx).is_some(), "the unsaved key shows again");

    a.secrets
        .delete(&format!("sync-account:{library}"))
        .unwrap();
    relaunch(cx);
    assert_eq!(account(cx), AccountState::SignedOut);
    let config = cx.read(|cx| cx.global::<AppState>().config.sync.account.clone());
    assert!(config.is_none());
    assert_eq!(a.notes(), 1);
}

// MARK: New recovery keys are never lost

/// Sign-up's answer is lost three times: the account may exist, so its key
/// shows anyway (saying so), stays in the Keychain, and is the key the
/// server holds. The Account page isn't showing, so a notification's "Show
/// key" leads there.
#[gpui_kit::test]
fn a_lost_sign_up_answer_still_shows_the_key(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    // No notes: signing in later asks nothing.
    let a = mac(cx, &server, "Studio", 0);
    let before = a.notifications(cx);
    server.drop_next_responses(Route::Signup, 3);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Err(words::CANT_REACH.to_owned()));
    assert_eq!(account(cx), AccountState::SignedOut);
    let shown = pending_key(cx).expect("the key shows");
    assert_eq!(key_note(cx), Some(words::KEY_UNCONFIRMED_SIGN_UP));
    let kept = a.kept_key().expect("the Keychain keeps it");
    assert!(kept.unconfirmed);
    assert_eq!(kept.recovery_key, shown);
    assert!(holds(&server, "alice", &shown), "the account was created");

    assert_eq!(a.notifications(cx), before + 1, "a notification says so");
    assert!(!a.showing_account(cx));
    a.click(cx, "show-recovery-key");
    assert!(a.showing_account(cx), "Show key opens the Account page");

    // On the Account page, the next key needs no notification.
    key_saved(cx);
    let rx = a.sign_in(cx, "alice", PASSWORD);
    assert_eq!(finish(cx, rx), Ok(()));
    a.window
        .update(cx, |_, window, cx| window.clear_notifications(cx))
        .unwrap();
    cx.executor().advance_clock(Duration::from_secs(2));
    cx.run_until_parked();
    assert_eq!(a.notifications(cx), 0);
    let rx = cx.update(|cx| account_flow::new_recovery_key(password(PASSWORD, "alice"), cx));
    assert_eq!(finish(cx, rx), Ok(()));
    assert!(pending_key(cx).is_some());
    assert!(a.showing_account(cx));
    assert_eq!(a.notifications(cx), 0);
}

/// The account is created but this Mac can't keep its session: the key
/// still shows and stays in the Keychain, and the words say the account
/// exists.
#[gpui_kit::test]
fn a_sign_up_that_cant_finish_still_shows_the_key(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 1);
    a.secrets.fail_writes_to(Some("sync-account:"));
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Err(words::CREATED_NOT_SIGNED_IN.to_owned()));
    a.secrets.heal();
    assert_eq!(account(cx), AccountState::SignedOut);
    let shown = pending_key(cx).expect("the key shows");
    assert_eq!(key_note(cx), Some(words::CREATED_NOT_SIGNED_IN));
    assert_eq!(
        a.kept_key().map(|kept| kept.recovery_key),
        Some(shown.clone())
    );
    assert!(holds(&server, "alice", &shown));
    assert_eq!(a.bound_to(), None, "never bound without its Keychain item");
    let rx = a.sign_in(cx, "alice", PASSWORD);
    a.answer(cx, "choice-0");
    assert_eq!(finish(cx, rx), Ok(()), "the account is there to sign in to");
}

/// A recovery that changed the password but couldn't finish signing in says
/// so, and its new key stays on screen and in the Keychain.
#[gpui_kit::test]
fn a_recovery_that_cant_finish_says_the_password_changed(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 1);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));
    let first = pending_key(cx).unwrap();
    key_saved(cx);
    assert_eq!(sign_out(cx), Ok(()));

    a.secrets.fail_writes_to(Some("sync-account:"));
    let rx = a.recover(cx, "alice", first.clone(), NEW_PASSWORD);
    assert_eq!(finish(cx, rx), Err(words::CHANGED_NOT_SIGNED_IN.to_owned()));
    a.secrets.heal();
    assert_eq!(account(cx), AccountState::SignedOut);
    let second = pending_key(cx).expect("the new key shows");
    assert!(second != first);
    assert_eq!(key_note(cx), Some(words::CHANGED_NOT_SIGNED_IN));
    assert_eq!(
        a.kept_key().map(|kept| kept.recovery_key),
        Some(second.clone())
    );
    assert!(holds(&server, "alice", &second));
    let rx = a.sign_in(cx, "alice", NEW_PASSWORD);
    assert_eq!(finish(cx, rx), Ok(()), "the new password works");
}

/// "New recovery key" whose answer is lost shows the key, saying Kanso
/// couldn't tell; a Keychain that refuses the new key sends nothing.
#[gpui_kit::test]
fn a_new_key_is_kept_before_it_is_sent(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 1);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));
    let first = pending_key(cx).unwrap();
    key_saved(cx);

    a.secrets.fail_writes_to(Some("sync-recovery-key:"));
    let rx = cx.update(|cx| account_flow::new_recovery_key(password(PASSWORD, "alice"), cx));
    let refused = finish(cx, rx);
    a.secrets.heal();
    assert_eq!(
        refused,
        Err(words::error_words(
            &AccountError::Keychain(kanso_sync::SecretError(String::new())),
            words::ErrorContext::CurrentPassword
        ))
    );
    assert_eq!(
        server.stats().count(Route::RecoveryKey),
        0,
        "nothing was sent"
    );
    assert!(pending_key(cx).is_none());
    assert!(holds(&server, "alice", &first));

    server.drop_next_responses(Route::RecoveryKey, 3);
    let rx = cx.update(|cx| account_flow::new_recovery_key(password(PASSWORD, "alice"), cx));
    assert_eq!(finish(cx, rx), Err(words::CANT_REACH.to_owned()));
    let shown = pending_key(cx).expect("the key shows");
    assert_eq!(key_note(cx), Some(words::KEY_UNCONFIRMED));
    assert!(holds(&server, "alice", &shown), "the server took it");
    assert_eq!(account(cx), signed_in_as("alice"));
}

/// A password change whose answer is lost shows the new key anyway, saying
/// Kanso couldn't tell whether the password changed.
#[gpui_kit::test]
fn a_lost_password_change_still_shows_the_key(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 1);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));
    key_saved(cx);

    server.drop_next_responses(Route::Password, 3);
    let rx = cx.update(|cx| {
        account_flow::change_password(
            password(PASSWORD, "alice"),
            password(NEW_PASSWORD, "alice"),
            cx,
        )
    });
    assert_eq!(finish(cx, rx), Err(words::CANT_REACH.to_owned()));
    let shown = pending_key(cx).expect("the key shows");
    assert_eq!(key_note(cx), Some(words::KEY_UNCONFIRMED_PASSWORD));
    assert!(holds(&server, "alice", &shown), "the change went through");
    assert!(a.kept_key().is_some_and(|kept| kept.unconfirmed));
}

/// A Keychain that won't delete the session's item still signs this Mac out
/// and ends the session on the server, and says so (A14).
#[gpui_kit::test]
fn signing_out_with_a_stuck_keychain_still_logs_out(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 1);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));
    key_saved(cx);
    let old = session(cx);
    let before = a.notifications(cx);

    a.secrets.fail_writes_to(Some("sync-account:"));
    assert_eq!(sign_out(cx), Ok(()));
    a.secrets.heal();
    assert_eq!(account(cx), AccountState::SignedOut);
    assert!(matches!(
        kanso_sync::usage(server.as_ref(), &old),
        Err(AccountError::Unauthorized { .. })
    ));
    assert_eq!(
        a.notifications(cx),
        before + 1,
        "the Keychain trouble is told"
    );
    let config = cx.read(|cx| cx.global::<AppState>().config.sync.account.clone());
    assert!(config.is_none(), "the leftover item is never loaded");
}

// MARK: Saving, then binding

/// Makes the library refuse `statement`'s writes until the trigger is
/// dropped, as a full disk or a damaged file would.
fn refuse(mac: &Mac, trigger: &str) -> rusqlite::Connection {
    let conn = rusqlite::Connection::open(mac.dir.path().join("kanso.db")).unwrap();
    conn.execute_batch(trigger).unwrap();
    conn
}

/// A Replace that fails after its backup leaves this Mac as it was: still
/// signed in to the first account with its Keychain item, its notes and its
/// binding, and syncing again; the backup made first is said all the same
/// (A1, sync 7).
#[gpui_kit::test]
fn a_failed_replace_keeps_this_account_and_says_where_the_backup_is(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let elsewhere = DeviceIdentity::from_parts(&format!("l_{:032x}", 9), "Elsewhere", [9; 32]);
    let bob = kanso_sync::sign_up(
        server.as_ref(),
        &MemorySecrets::new(),
        SERVER,
        "bob",
        &password(PASSWORD, "bob"),
        &elsewhere,
        &|_| {},
    )
    .unwrap();
    let a = mac(cx, &server, "Studio", 2);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));
    key_saved(cx);
    assert!(ok(&synced(cx)));
    let alice = session(cx);
    let library = library_id(cx);
    let item = kanso_sync::keychain::account_item(&library);
    let kept = a.secrets.get(&item).unwrap().expect("alice's item");
    let before = a.notifications(cx);

    let conn = refuse(
        &a,
        "CREATE TRIGGER refuse BEFORE DELETE ON docs BEGIN SELECT RAISE(ABORT, 'refused'); END;",
    );
    let rx = a.sign_in(cx, "bob", PASSWORD);
    a.answer(cx, "choice-1");
    let failed = finish(cx, rx);
    conn.execute_batch("DROP TRIGGER refuse").unwrap();
    assert!(failed.is_err(), "{failed:?}");

    assert_eq!(account(cx), signed_in_as("alice"));
    assert!(Arc::ptr_eq(&session(cx), &alice));
    assert_eq!(
        a.secrets.get(&item).unwrap().map(|now| now.to_string()),
        Some(kept.to_string()),
        "alice's item is back"
    );
    assert_eq!(a.bound_to(), Some(alice.account_id.clone()));
    assert_eq!(a.notes(), 2);
    let backups: Vec<_> = std::fs::read_dir(crate::paths::backups_dir(a.dir.path()))
        .expect("a backups folder")
        .collect();
    assert_eq!(backups.len(), 1, "the backup was made first");
    assert_eq!(a.notifications(cx), before + 1, "and is said");
    let devices = kanso_sync::devices(server.as_ref(), &bob.session).unwrap();
    assert_eq!(devices.len(), 1, "bob's new session is logged out");
    assert!(ok(&sync_now(cx)), "alice syncs");
}

/// A sign-up whose binding fails deletes the session's Keychain item again:
/// a library is never left with an item for an account it isn't bound to.
#[gpui_kit::test]
fn a_sign_up_whose_binding_fails_leaves_no_session_item(cx: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 1);
    // The library has its ID; binding writes the account's.
    cx.update(|cx| {
        let machine = cx.global::<Services>().machine.clone().unwrap();
        kanso_sync::device_identity(&a.store, &machine).unwrap();
    });
    let conn = refuse(
        &a,
        &format!(
            "CREATE TRIGGER refuse BEFORE INSERT ON meta WHEN NEW.key = '{}' \
             BEGIN SELECT RAISE(ABORT, 'refused'); END;",
            meta_keys::ACCOUNT_ID
        ),
    );
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Err(words::CREATED_NOT_SIGNED_IN.to_owned()));
    conn.execute_batch("DROP TRIGGER refuse").unwrap();
    assert_eq!(account(cx), AccountState::SignedOut);
    assert_eq!(a.bound_to(), None);
    assert!(
        a.items()
            .iter()
            .all(|item| !item.starts_with("sync-account:")),
        "{:?}",
        a.items()
    );
    assert!(pending_key(cx).is_some(), "the key still shows");
}
