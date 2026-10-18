//! The account flows end to end, in real windows, against `MemoryServer` and
//! `MemorySecrets` (docs/ACCOUNTS.md › Account flows, › Library binding, ›
//! Services): sign-up and its recovery key, a second Mac signing in and the
//! question about its notes, another account's notes, signing out, a
//! password change, recovery, a new recovery key, deleting the account, and
//! a relaunch. Each test context is one Mac with its own library and
//! Keychain; the server is shared. Nothing here reaches a network or the
//! real Keychain. Sync itself is step 2b: these tests keep it from starting
//! (`Services::no_sync`) and check what the flows leave behind.

use std::sync::Arc;
use std::time::{Duration, Instant};

use async_channel::Receiver;
use gpui_kit::component::WindowExt as _;
use gpui_kit::{AnyWindowHandle, AppContext as _, Modifiers, TestAppContext, VisualTestContext};
use kanso_core::DocType;
use kanso_store::{Origin, Put, Store};
use kanso_sync::{
    AccountError, Backend, DeviceIdentity, Machine, MemorySecrets, MemoryServer, Password,
    RecoveryKey, SecretStore, Session, SignOutReason, meta_keys,
};
use serde_json::json;

use crate::account_flow;
use crate::config::AppConfig;
use crate::dialogs::Progress;
use crate::services::{AccountState, Services};
use crate::workspace::{AppState, Workspace};

/// The forms' server; `MemoryServer` answers for any.
const SERVER: &str = kanso_sync::DEFAULT_SERVER;
const PASSWORD: &str = "correct horse battery staple";
const NEW_PASSWORD: &str = "a different long passphrase";

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
    let backend: Arc<dyn Backend> = server.clone();
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
    services.no_sync = true;
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

fn account(cx: &mut TestAppContext) -> AccountState {
    cx.read(|cx| cx.global::<Services>().account.clone())
}

fn session(cx: &mut TestAppContext) -> Arc<Session> {
    cx.read(|cx| cx.global::<Services>().session())
        .expect("signed in")
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

    cx.update(account_flow::key_saved);
    cx.run_until_parked();
    assert!(pending_key(cx).is_none());
    assert_eq!(a.items(), [format!("sync-account:{library}")]);

    details_loaded(cx);
    let devices = cx.read(|cx| cx.global::<Services>().details.devices.clone());
    let devices = devices.expect("the devices load");
    assert_eq!(devices.len(), 1);
    assert!(devices[0].current);
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

    let b = mac(cx_b, &server, "Laptop", 4);
    let rx = b.sign_in(cx_b, "alice", PASSWORD);
    b.answer(cx_b, "choice-0");
    assert_eq!(finish(cx_b, rx), Ok(()));
    assert_eq!(account(cx_b), signed_in_as("alice"));
    assert_eq!(b.bound_to(), Some(account_id.clone()));
    assert_eq!(b.notes(), 4, "Add keeps this Mac's notes");
    assert!(pending_key(cx_b).is_none(), "signing in makes no new key");
    let config = cx_b.read(|cx| cx.global::<AppState>().config.sync.clone());
    assert!(config.enabled, "signing in turns sync on");
    details_loaded(cx_b);
    let devices = cx_b.read(|cx| cx.global::<Services>().details.devices.clone());
    assert_eq!(devices.map(|d| d.len()), Some(2), "both Macs are devices");

    let c = mac(cx_c, &server, "Mini", 0);
    let rx = c.sign_in(cx_c, "alice", PASSWORD);
    assert_eq!(finish(cx_c, rx), Ok(()));
    assert!(!c.asking(cx_c), "an empty library needs no question");
    assert_eq!(c.bound_to(), Some(account_id));
}

/// Replace saves a backup of the library first, then empties it for the
/// account's notes.
#[gpui_kit::test]
fn replacing_the_notes_keeps_a_backup_first(cx: &mut TestAppContext, cx_b: &mut TestAppContext) {
    let server = Arc::new(MemoryServer::new());
    let a = mac(cx, &server, "Studio", 1);
    let rx = a.sign_up(cx, "alice");
    assert_eq!(finish(cx, rx), Ok(()));

    let b = mac(cx_b, &server, "Laptop", 3);
    let rx = b.sign_in(cx_b, "alice", PASSWORD);
    b.answer(cx_b, "choice-1");
    assert_eq!(finish(cx_b, rx), Ok(()));
    assert_eq!(account(cx_b), signed_in_as("alice"));
    assert_eq!(b.bound_to(), a.bound_to());
    assert_eq!(b.notes(), 0, "the account's notes come with sync");
    let backups: Vec<_> = std::fs::read_dir(crate::paths::backups_dir(b.dir.path()))
        .expect("a backups folder")
        .collect();
    assert_eq!(backups.len(), 1, "one backup, made first");
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
    cx.update(account_flow::key_saved);
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
    let rx = cx.update(|cx| {
        account_flow::change_password(
            password(PASSWORD, "alice"),
            password(NEW_PASSWORD, "alice"),
            cx,
        )
    });
    assert_eq!(finish(cx, rx), Ok(()));
    assert_eq!(account(cx), signed_in_as("alice"));

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
    cx.update(account_flow::key_saved);
    assert_eq!(sign_out(cx), Ok(()));

    let rx = a.recover(cx, "alice", first.clone(), NEW_PASSWORD);
    assert_eq!(finish(cx, rx), Ok(()));
    assert_eq!(account(cx), signed_in_as("alice"));
    let second = pending_key(cx).expect("a new key shows");
    assert!(second != first);
    let library = library_id(cx);
    assert!(a.items().contains(&format!("sync-recovery-key:{library}")));
    cx.update(account_flow::key_saved);

    let rx = cx.update(|cx| account_flow::new_recovery_key(password(NEW_PASSWORD, "alice"), cx));
    assert_eq!(finish(cx, rx), Ok(()));
    let third = pending_key(cx).expect("the new key shows");
    assert!(third != second);
    cx.update(account_flow::key_saved);

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
        let backend: Arc<dyn Backend> = server.clone();
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
