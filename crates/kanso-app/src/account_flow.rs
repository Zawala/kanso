//! The account tasks (docs/ACCOUNTS.md › Account flows, › Library binding,
//! › Services). Each runs on a thread named `kanso-account`, reports its
//! steps to [`Services`] (the Account page's button) and to the form that
//! started it, and finishes on the main thread. A task that re-binds the
//! library or touches its Keychain item stops and joins the running sync
//! first, on its own thread, with "Stopping sync…" as its step.
//!
//! Without a backend and a Keychain in [`Services`] (tests and snapshots)
//! no task starts: it ends at once with "Can't reach the server".

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use async_channel::{Receiver, Sender};
use base64::Engine as _;
use gpui_kit::component::button::Button;
use gpui_kit::component::notification::Notification;
use gpui_kit::*;
use kanso_store::Store;
use kanso_sync::api::LoginDevice;
use kanso_sync::keychain;
use kanso_sync::provider::{Provider, ProviderSession, ProviderSignIn, ProviderStart};
use kanso_sync::{
    AccountError, Approval, ApprovalKeys, Backend, BindError, Choice, DeviceIdentity, DeviceInfo,
    Flow, LocalLibrary, LockedSession, Machine, Password, Plan, RecoveryKey, RequestState,
    RotationError, SecretError, SecretStore, Session, SignOutReason, SignedUp, Step,
    cancel_saved_approval, complete_saved_request, finish_unlock, meta_keys, name_device_resumable,
    new_approval_request_id, poll_saved_approval, request_approval_resumable, resume_approval,
    unlock_with_local_key,
};
use sha2::Digest as _;

use crate::account_words::{self as words, Answer, ErrorContext, Question};
use crate::clock::Clock;
use crate::config::{AccountRef, is_default_server};
use crate::dialogs::{self, Described, Progress};
use crate::passkey_accounts::{
    self, PasskeyContext, PasskeyFlowError, PasskeySignedUp, RpRule, SyncPasskeyAccounts,
};
use crate::passkey_ceremony::{PlatformCeremony, platform_ceremony};
use crate::security_key_flow::{OwnedSecurityKey, PinProblem};
use crate::services::{
    self, AccountDetails, AccountState, PendingKey, RunningTask, Services, TaskKind, notify_windows,
};
use crate::settings::{OpenAccountSettings, SettingsPage};
use crate::workspace::AppState;

/// When the library itself couldn't be changed.
const LIBRARY_TROUBLE: &str = "Kanso couldn't update this device's library. Check there is free space on this device, then try again.";
/// When a task's thread stopped without an answer.
const SOMETHING_WRONG: &str = "Something went wrong. Try again.";
/// When another account task is still running.
const BUSY: &str = "Kanso is still busy with your account. Try again in a moment.";

mod methods;
pub use methods::{ChangeRequest, MethodChange, ProofWith, change_method};

// MARK: Running a task

/// Why a task failed: the words to show, the reason when the server ended
/// this Mac's session on the way, and a new recovery key the server took
/// or may have taken, which shows all the same.
#[derive(Debug)]
struct Failure {
    words: String,
    signed_out: Option<SignOutReason>,
    recovery: Option<Box<PendingKey>>,
    /// The backup a Replace made before it failed: said all the same.
    backup: Option<PathBuf>,
    /// A locked candidate was successfully discarded and its pending item
    /// cleared after a bind failure.
    locked_cleanup: bool,
    /// Keychain cleanup or rollback failed; show the established warning.
    keychain_warning: bool,
}

impl Failure {
    fn account(err: AccountError, context: ErrorContext) -> Failure {
        // The details go to the log; the words never repeat them.
        eprintln!("kanso: account: {err}");
        let signed_out = match &err {
            AccountError::Unauthorized { reason } => Some(*reason),
            _ => None,
        };
        let keychain_warning = matches!(&err, AccountError::Keychain(_));
        Failure {
            words: words::error_words(&err, context),
            signed_out,
            recovery: None,
            backup: None,
            locked_cleanup: false,
            keychain_warning,
        }
    }

    /// A call that sets a new recovery key failed. The key it returns
    /// shows, with `note` when Kanso couldn't tell whether the server took
    /// it.
    fn rotation(err: RotationError, context: ErrorContext, note: &'static str) -> Failure {
        Failure {
            recovery: err
                .recovery_key
                .map(|item| Box::new(PendingKey::kept(*item, note))),
            ..Failure::account(err.error, context)
        }
    }

    /// The server took `key`, then this Mac failed: `words` say what
    /// happened instead, and the key shows with them.
    fn after_key(self, words: &'static str, session: &Session, key: RecoveryKey) -> Failure {
        Failure {
            words: words.to_owned(),
            recovery: Some(Box::new(PendingKey {
                key,
                username: session.username.clone(),
                server: session.server.clone(),
                note: Some(words),
            })),
            ..self
        }
    }

    fn keychain(err: SecretError) -> Failure {
        let mut failure = Failure::account(AccountError::Keychain(err), ErrorContext::Account);
        failure.keychain_warning = true;
        failure
    }

    fn library(err: impl std::fmt::Display) -> Failure {
        eprintln!("kanso: account: the library: {err}");
        Failure {
            words: LIBRARY_TROUBLE.to_owned(),
            signed_out: None,
            recovery: None,
            backup: None,
            locked_cleanup: false,
            keychain_warning: false,
        }
    }

    /// A failure that is only its words.
    fn words(words: &str) -> Failure {
        Failure {
            words: words.to_owned(),
            signed_out: None,
            recovery: None,
            backup: None,
            locked_cleanup: false,
            keychain_warning: false,
        }
    }

    fn bind(err: BindError) -> Failure {
        match err {
            BindError::Store(err) => Failure::library(err),
            BindError::Keychain(err) => Failure::keychain(err),
        }
    }
}

/// What a task's thread sends back.
enum Message<T> {
    Step(Step),
    Done(T),
}

/// What the tasks reach the world through, taken from [`Services`] and
/// [`AppState`] on the main thread.
#[derive(Clone)]
struct Seams {
    backend: Arc<dyn Backend>,
    secrets: Arc<dyn SecretStore>,
    store: Arc<Store>,
    machine: Option<Machine>,
    backups: PathBuf,
    now: i64,
}

impl Seams {
    fn get(cx: &App) -> Option<Seams> {
        let services = cx.try_global::<Services>()?;
        let state = cx.try_global::<AppState>()?;
        Some(Seams {
            backend: services.backend.clone()?,
            secrets: services.secrets.clone()?,
            store: state.store.clone(),
            machine: services.machine.clone(),
            backups: crate::paths::backups_dir(&state.data_dir),
            now: Clock::now_ms(cx),
        })
    }

    /// This library's identity on this Mac (on the task's thread: reading
    /// the Mac runs `ioreg` and `scutil`).
    fn device(&self) -> Result<DeviceIdentity, Failure> {
        let machine = self.machine.clone().unwrap_or_else(Machine::this_mac);
        kanso_sync::device_identity(&self.store, &machine).map_err(Failure::library)
    }
}

/// Starts `work` on its own thread as the task `kind`, at step `first`.
/// With `stop_sync`, the running sync is cancelled and joined there first,
/// and started again once the task, and any task its `finish` started, has
/// left the same session signed in (a failure, or a cancelled question).
/// Steps reach [`Services`] and
/// `progress`; `finish` gets the result on the main thread (`None` if the
/// thread stopped without one).
fn run<T: Send + 'static>(
    kind: TaskKind,
    first: Step,
    stop_sync: bool,
    progress: Option<Sender<Progress>>,
    cx: &mut App,
    work: impl FnOnce(&dyn Fn(Step)) -> T + Send + 'static,
    finish: impl FnOnce(Option<T>, &mut App) + 'static,
) {
    let services = cx.global_mut::<Services>();
    let sync = if stop_sync {
        services.take_sync_handle()
    } else {
        None
    };
    // A chain of tasks remembers the session of the first stop.
    if sync.is_some() && services.stopped_for.is_none() {
        services.stopped_for = services.session();
    }
    let first = if sync.is_some() {
        Step::StoppingSync
    } else {
        first
    };
    services.task = Some(RunningTask { kind, step: first });
    if let Some(progress) = &progress {
        progress.try_send(Progress::Step(first)).ok();
    }
    let (tx, rx) = async_channel::unbounded::<Message<T>>();
    let spawned = std::thread::Builder::new()
        .name("kanso-account".into())
        .spawn(move || {
            let step = |step: Step| {
                tx.try_send(Message::Step(step)).ok();
            };
            if let Some(sync) = sync {
                sync.cancel_and_join();
            }
            let result = work(&step);
            tx.try_send(Message::Done(result)).ok();
        });
    if let Err(err) = spawned {
        // The sync handle went with the closure: that sync runs on,
        // detached, to its end. Nothing was changed.
        eprintln!("kanso: couldn't start an account task: {err}");
        cx.global_mut::<Services>().task = None;
        finish(None, cx);
        resume_stopped_sync(cx);
        cx.refresh_windows();
        return;
    }
    cx.refresh_windows();
    cx.spawn(async move |cx| {
        let mut result = None;
        while let Ok(message) = rx.recv().await {
            match message {
                Message::Step(step) => {
                    if let Some(progress) = &progress {
                        progress.try_send(Progress::Step(step)).ok();
                    }
                    cx.update(|cx| {
                        if let Some(task) = &mut cx.global_mut::<Services>().task {
                            task.step = step;
                        }
                        cx.refresh_windows();
                    });
                }
                Message::Done(value) => {
                    result = Some(value);
                    break;
                }
            }
        }
        cx.update(|cx| {
            cx.global_mut::<Services>().task = None;
            finish(result, cx);
            resume_stopped_sync(cx);
            check_unconfirmed_key(cx);
            cx.refresh_windows();
        });
    })
    .detach();
}

/// Once no task runs (a task's `finish` may have started the next of its
/// chain), the sync a task stopped starts again if its session is still
/// signed in (A8).
fn resume_stopped_sync(cx: &mut App) {
    let services = cx.global_mut::<Services>();
    if services.task.is_some() {
        return;
    }
    let Some(before) = services.stopped_for.take() else {
        return;
    };
    if services
        .session()
        .is_some_and(|now| Arc::ptr_eq(&now, &before))
    {
        services::resume_sync(cx);
    }
}

fn done(tx: &Sender<Progress>, result: Result<(), String>) {
    tx.try_send(Progress::Done(result)).ok();
}

/// Ends a task with its failure; a session the server ended is ended here
/// too, and a new recovery key that came with the failure shows.
fn failed(tx: &Sender<Progress>, failure: Option<Failure>, cx: &mut App) {
    let Some(failure) = failure else {
        done(tx, Err(SOMETHING_WRONG.to_owned()));
        return;
    };
    if let Some(reason) = failure.signed_out
        && cx.global::<Services>().signed_in()
    {
        services::end_session(reason, cx);
    }
    if failure.locked_cleanup {
        cx.global_mut::<Services>()
            .clear_session(AccountState::SignedOut);
    }
    if failure.keychain_warning {
        notify_windows(|| Notification::warning(words::SIGNED_OUT_KEYCHAIN), cx);
    }
    if let Some(key) = failure.recovery {
        show_pending(*key, cx);
    }
    if let Some(path) = failure.backup {
        notify_backup(path, cx);
    }
    done(tx, Err(failure.words));
}

/// Saves `session` as the library's Keychain item, then binds the library
/// with `choice` (A1, sync 7): a library is never bound to an account whose
/// item is missing. When binding fails the item is put back as it was (gone,
/// or another account's), and a backup a Replace made first comes back with
/// the failure. `None` leaves the binding as it is.
fn save_and_bind(
    s: &Seams,
    library_id: &str,
    session: &Session,
    choice: Option<Choice>,
) -> Result<Option<PathBuf>, Failure> {
    let item = keychain::account_item(library_id);
    let before = s.secrets.get(&item).map_err(Failure::keychain)?;
    keychain::save_session(s.secrets.as_ref(), library_id, session).map_err(Failure::keychain)?;
    let Some(choice) = choice else {
        return Ok(None);
    };
    let backups_before = backups_in(&s.backups);
    match kanso_sync::bind(&s.store, session, choice, &s.backups, s.now) {
        Ok(backup) => Ok(backup),
        Err(err) => {
            let put_back = match &before {
                Some(raw) => s.secrets.set(&item, raw),
                None => s.secrets.delete(&item),
            };
            if let Err(err) = put_back {
                eprintln!("kanso: putting the library's Keychain item back: {err}");
            }
            let mut failure = Failure::bind(err);
            if choice == Choice::Replace {
                failure.backup = backups_in(&s.backups)
                    .into_iter()
                    .find(|path| !backups_before.contains(path));
            }
            Err(failure)
        }
    }
}

/// The files in the backups folder, to find the one a failed Replace made.
fn backups_in(dir: &std::path::Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut paths: Vec<PathBuf> = entries.flatten().map(|entry| entry.path()).collect();
    paths.sort();
    paths
}

/// The seams, or the task's end when there are none or a task runs.
fn begin(tx: &Sender<Progress>, cx: &App) -> Option<Seams> {
    if cx
        .try_global::<Services>()
        .is_some_and(|services| services.task.is_some() || services.restoring)
    {
        done(tx, Err(BUSY.to_owned()));
        return None;
    }
    let seams = Seams::get(cx);
    if seams.is_none() {
        done(tx, Err(words::CANT_REACH.to_owned()));
    }
    seams
}

/// The seams and the session, for a task while signed in.
fn begin_signed_in(tx: &Sender<Progress>, cx: &App) -> Option<(Seams, Arc<Session>, String)> {
    let seams = begin(tx, cx)?;
    let services = cx.global::<Services>();
    match (services.session(), services.library_id.clone()) {
        (Some(session), Some(library_id)) => Some((seams, session, library_id)),
        _ => {
            done(tx, Err(SOMETHING_WRONG.to_owned()));
            None
        }
    }
}

/// Asks `question` in `window`, then calls `answer` once with the choice
/// (Cancel when the dialog is dismissed or the window is gone).
fn ask(
    window: AnyWindowHandle,
    question: Question,
    cx: &mut App,
    answer: impl FnOnce(Choice, &mut App) + 'static,
) {
    type Reply = Rc<RefCell<Option<Box<dyn FnOnce(Choice, &mut App)>>>>;
    let reply: Reply = Rc::new(RefCell::new(Some(Box::new(answer))));
    let answered = reply.clone();
    let choices: Vec<Choice> = question.answers.iter().map(|a| a.choice).collect();
    let described: Vec<Described> = question
        .answers
        .iter()
        .map(|a| Described {
            label: a.label.clone().into(),
            description: a.description.clone().into(),
        })
        .collect();
    let Question { title, body, .. } = question;
    let shown = window.update(cx, move |_, window, cx| {
        dialogs::choose(title, body, described, window, cx, move |picked, _, cx| {
            let choice = picked
                .and_then(|ix| choices.get(ix).copied())
                .unwrap_or(Choice::Cancel);
            let answer = answered.borrow_mut().take();
            if let Some(answer) = answer {
                answer(choice, cx);
            }
        });
    });
    if shown.is_err() {
        let answer = reply.borrow_mut().take();
        if let Some(answer) = answer {
            answer(Choice::Cancel, cx);
        }
    }
}

// MARK: Signing in

/// The new recovery key, shown on the Account page until "I've saved it".
fn show_key(session: &Session, key: RecoveryKey, cx: &mut App) {
    show_pending(
        PendingKey {
            key,
            username: session.username.clone(),
            server: session.server.clone(),
            note: None,
        },
        cx,
    );
}

/// Shows `key` on the Account page until "I've saved it". When that page
/// isn't showing, a notification leads there.
fn show_pending(key: PendingKey, cx: &mut App) {
    let services = cx.global_mut::<Services>();
    services.key_check_due = key.note.and_then(words::key_checked).is_some();
    services.pending_key = Some(key);
    // Once the task that showed it is done, or the launch check.
    cx.defer(check_unconfirmed_key);
    // Windows can't be read from inside their own updates.
    cx.defer(|cx| {
        if !account_page_showing(cx) {
            notify_windows(
                || {
                    Notification::warning(words::KEY_READY)
                        .id::<KeyReady>()
                        .action(|_, _, _| {
                            Button::new("show-recovery-key")
                                .label("Show key")
                                .debug_selector(|| "show-recovery-key".into())
                                .on_click(|_, window, cx| {
                                    window.dispatch_action(Box::new(OpenAccountSettings), cx)
                                })
                        })
                },
                cx,
            );
        }
    });
}

/// The "Show key" notification's ID: a second key replaces it.
struct KeyReady;

/// Asks the server once whether it took the shown key, when Kanso couldn't
/// tell (review-verify 6). A key the server holds is the account's now, and
/// the screen says so; one it doesn't hold goes, screen and Keychain item,
/// since nothing changed, and a notification says that. It waits for a
/// running task and the launch check, and is asked once per key shown (and
/// so once per launch for a key kept from before): `recover/start` counts
/// against the server's per-address limit. A failed ask leaves the key as
/// it is, its words saying to keep the old key too.
fn check_unconfirmed_key(cx: &mut App) {
    let Some(services) = cx.try_global::<Services>() else {
        return;
    };
    if !services.key_check_due || services.task.is_some() || services.restoring {
        return;
    }
    let Some(shown) = services.pending_key.clone() else {
        return;
    };
    let Some(checked) = shown.note.and_then(words::key_checked) else {
        return;
    };
    let Some(seams) = Seams::get(cx) else {
        return;
    };
    let library_id = services.library_id.clone();
    cx.global_mut::<Services>().key_check_due = false;
    let asked = shown.key.clone();
    run(
        TaskKind::CheckKey,
        Step::Contacting,
        false,
        None,
        cx,
        move |_| {
            let held = kanso_sync::recovery_key_held(
                seams.backend.as_ref(),
                &shown.server,
                &shown.username,
                &shown.key,
            )?;
            settle_kept_key(&seams, library_id, &shown.key, held);
            Ok::<bool, AccountError>(held)
        },
        move |result, cx| {
            let services = cx.global_mut::<Services>();
            let same = services
                .pending_key
                .as_ref()
                .is_some_and(|now| now.key == asked);
            match result {
                Some(Ok(true)) => {
                    if let Some(now) = services.pending_key.as_mut().filter(|_| same) {
                        now.note = Some(checked.taken);
                    }
                }
                Some(Ok(false)) => {
                    if same {
                        services.pending_key = None;
                    }
                    // In place of its "Show key", if that still shows.
                    notify_windows(
                        move || Notification::warning(checked.not_taken).id::<KeyReady>(),
                        cx,
                    );
                }
                Some(Err(err)) => {
                    eprintln!("kanso: asking whether the server took the recovery key: {err}");
                }
                None => {}
            }
        },
    );
}

/// The Keychain's unsaved key once the server answered about `key`:
/// confirmed when the server holds it, gone when it doesn't. An item that
/// holds another key by then, or is confirmed already, stays as it is.
fn settle_kept_key(s: &Seams, library_id: Option<String>, key: &RecoveryKey, held: bool) {
    let library_id = match library_id {
        Some(library_id) => library_id,
        None => match s.device() {
            Ok(device) => device.library_id,
            Err(_) => return,
        },
    };
    let secrets = s.secrets.as_ref();
    let settled =
        keychain::load_any_recovery_key(secrets, &library_id).and_then(|kept| match kept {
            Some(mut kept) if kept.unconfirmed && kept.recovery_key == *key => {
                if held {
                    kept.unconfirmed = false;
                    keychain::save_recovery_key(secrets, &library_id, &kept)
                } else {
                    keychain::delete_recovery_key(secrets, &library_id)
                }
            }
            _ => Ok(()),
        });
    if let Err(err) = settled {
        eprintln!("kanso: settling the unconfirmed recovery key: {err}");
    }
}

/// Whether a window shows Settings › Account.
fn account_page_showing(cx: &mut App) -> bool {
    let mut showing = false;
    crate::workspace::for_each_workspace(cx, |workspace, cx| {
        showing |= workspace.settings_open()
            && workspace.settings.read(cx).page() == SettingsPage::Account;
    });
    showing
}

/// After the binding and the Keychain item: the config, the window, and
/// sync (docs/ACCOUNTS.md › Library binding › Order and cancellation).
/// `turn_on` sets Sync automatically, as signing in does.
fn signed_in(session: Session, library_id: String, turn_on: bool, cx: &mut App) {
    let account = AccountRef {
        origin: session.origin.clone(),
        id: session.account_id.clone(),
        username: session.username.clone(),
    };
    AppState::update_config(cx, |config| {
        config.sync.account = Some(account);
        if turn_on {
            config.sync.enabled = true;
        }
    });
    let services = cx.global_mut::<Services>();
    services.library_id = Some(library_id);
    services.set_signed_in(Arc::new(session));
    services::apply_sync_settings(cx);
    refresh_details(cx);
    crate::device_approval::start_events(cx);
    crate::device_approval::refresh(cx);
}

/// Signed out here: the session, the details and the config's account go;
/// the notes stay.
fn forget_account(cx: &mut App) {
    crate::device_approval::stop_events(cx);
    let services = cx.global_mut::<Services>();
    services.clear_session(AccountState::SignedOut);
    services.pending_key = None;
    crate::collab::signed_out(cx);
    AppState::update_config(cx, |config| config.sync.account = None);
    cx.refresh_windows();
}

/// Leaves a cancelled locked sign-in without changing the prior account's
/// config, recovery-key screen, Keychain item, or library binding.
fn clear_locked_account(cx: &mut App) {
    crate::device_approval::stop_events(cx);
    cx.global_mut::<Services>()
        .clear_session(AccountState::SignedOut);
    crate::collab::signed_out(cx);
    cx.refresh_windows();
}

/// Creates an account on `server` and signs this Mac in with it (Settings
/// › Account › Create account). Notes that came from another account are
/// asked about in `window` first.
pub fn sign_up(
    server: String,
    username: String,
    password: Password,
    window: AnyWindowHandle,
    cx: &mut App,
) -> Receiver<Progress> {
    let (tx, rx) = async_channel::unbounded();
    let Some(seams) = begin(&tx, cx) else {
        return rx;
    };
    let origin = kanso_sync::origin(&server).unwrap_or_default();
    let thread = seams.clone();
    run(
        TaskKind::SignUp,
        Step::Contacting,
        false,
        Some(tx.clone()),
        cx,
        // No account ID yet: a new account is never the library's own.
        move |_| kanso_sync::local_library(&thread.store, &origin, "").map_err(Failure::library),
        move |result, cx| match result {
            Some(Ok(library)) => match kanso_sync::plan(Flow::SignUp, &library) {
                Plan::AskBeforeSignUp { from, .. } => {
                    let (title, body, upload) = words::upload_question(&from);
                    let question = Question {
                        title,
                        body,
                        answers: vec![Answer {
                            choice: Choice::Add,
                            label: upload.to_owned(),
                            description: "They're encrypted and uploaded to the new account. Nothing changes in the other one.".to_owned(),
                        }],
                    };
                    ask(window, question, cx, move |choice, cx| match choice {
                        Choice::Cancel => done(&tx, Ok(())),
                        _ => create(seams, server, username, password, tx, cx),
                    });
                }
                _ => create(seams, server, username, password, tx, cx),
            },
            Some(Err(failure)) => failed(&tx, Some(failure), cx),
            None => failed(&tx, None, cx),
        },
    );
    rx
}

fn create(
    seams: Seams,
    server: String,
    username: String,
    password: Password,
    tx: Sender<Progress>,
    cx: &mut App,
) {
    let thread = seams.clone();
    run(
        TaskKind::SignUp,
        Step::DerivingKeys,
        true,
        Some(tx.clone()),
        cx,
        move |step| {
            let s = thread;
            let device = s.device()?;
            // The new key is in the Keychain before the request goes.
            let signed = kanso_sync::sign_up(
                s.backend.as_ref(),
                s.secrets.as_ref(),
                &server,
                &username,
                &password,
                &device,
                step,
            )
            .map_err(|err| {
                Failure::rotation(err, ErrorContext::SignUp, words::KEY_UNCONFIRMED_SIGN_UP)
            })?;
            step(Step::Saving);
            let bound = save_and_bind(&s, &device.library_id, &signed.session, Some(Choice::Add));
            if let Err(failure) = bound {
                // Nothing here uses the new session: the server forgets it.
                // The account stays, and so does its recovery key, in the
                // Keychain and on screen.
                kanso_sync::discard_session(
                    s.backend.as_ref(),
                    s.secrets.as_ref(),
                    &signed.session,
                )
                .ok();
                let SignedUp {
                    session,
                    recovery_key,
                } = signed;
                return Err(failure.after_key(
                    words::CREATED_NOT_SIGNED_IN,
                    &session,
                    recovery_key,
                ));
            }
            Ok((signed, device.library_id))
        },
        move |result, cx| match result {
            Some(Ok((signed, library_id))) => {
                let SignedUp {
                    session,
                    recovery_key,
                } = signed;
                show_key(&session, recovery_key, cx);
                signed_in(session, library_id, true, cx);
                done(&tx, Ok(()));
            }
            Some(Err(failure)) => failed(&tx, Some(failure), cx),
            None => failed(&tx, None, cx),
        },
    );
}

/// An account this Mac has proved it may use, before anything is saved.
struct Authenticated {
    session: Session,
    evicted: Option<DeviceInfo>,
    library: LocalLibrary,
    library_id: String,
    /// A recovery's new key, in the Keychain already.
    recovery_key: Option<RecoveryKey>,
}

/// An unlocked candidate returned to the UI thread before any library bind,
/// config write, Keychain promotion, or sync start is allowed.
pub(crate) struct UnlockedCandidate {
    session: Session,
    locked_session: LockedSession,
    evicted: Option<DeviceInfo>,
    library_id: String,
    /// Local-key and approval unlocks reuse the provider token in the
    /// pending item. Password unlock and recovery create a new token that
    /// must be revoked if promotion cannot persist it.
    same_token_as_locked: bool,
}

type Authenticate = Box<
    dyn FnOnce(
            &Seams,
            &DeviceIdentity,
            &dyn Fn(Step),
        ) -> Result<(Session, Option<DeviceInfo>, Option<RecoveryKey>), Failure>
        + Send,
>;

/// Signs this Mac in to `username`'s account (Settings › Account › Sign
/// in), asking in `window` what to do with notes already here.
pub fn sign_in(
    server: String,
    username: String,
    password: Password,
    window: AnyWindowHandle,
    cx: &mut App,
) -> Receiver<Progress> {
    let (tx, rx) = async_channel::unbounded();
    let Some(seams) = begin(&tx, cx) else {
        return rx;
    };
    authenticate(
        Flow::SignIn,
        Some(window),
        seams,
        tx,
        cx,
        Box::new(move |s, device, step| {
            let signed = kanso_sync::sign_in(
                s.backend.as_ref(),
                &server,
                &username,
                &password,
                device,
                step,
            )
            .map_err(|err| Failure::account(err, ErrorContext::SignIn))?;
            Ok((signed.session, signed.evicted, None))
        }),
    );
    rx
}

/// Starts a provider flow on the account worker. Browser callbacks and
/// polling run off the UI thread; only the final, typed result returns to GPUI.
pub fn start_provider(
    provider: Provider,
    creating: bool,
    window: AnyWindowHandle,
    cx: &mut App,
) -> Receiver<Progress> {
    let (tx, rx) = async_channel::unbounded();
    let Some(seams) = begin(&tx, cx) else {
        return rx;
    };
    let login_device = LoginDevice::ephemeral();
    let server = form_server(cx);
    let server_generation = cx.global::<Services>().auth_generation;
    let task_kind = if creating {
        TaskKind::SignUp
    } else {
        TaskKind::SignIn
    };
    let thread = seams.clone();
    let start_provider = provider.clone();
    let start_device = login_device.clone();
    let start_server = server.clone();
    run(
        task_kind,
        Step::Contacting,
        true,
        Some(tx.clone()),
        cx,
        move |step| {
            provider_begin(
                thread.backend.as_ref(),
                &ProviderRoute::Public {
                    server: start_server,
                },
                &start_provider,
                &start_device,
                step,
            )
            .map_err(|err| Failure::account(err, ErrorContext::SignIn))
        },
        move |result, cx| match result {
            Some(Ok(started)) => {
                if !provider_request_is_current(cx, &server, server_generation) {
                    // The server changed while start began. Never present an
                    // Apple sheet or begin polling a flow for the old server.
                    done(&tx, Ok(()));
                    return;
                }
                let apple_callback = match (&started.start, &provider) {
                    (ProviderStart::Apple(apple), Provider::Apple) => {
                        match crate::provider_browser::apple_sign_in(apple) {
                            Ok(callback) => Some(callback),
                            Err(_) => {
                                failed(
                                    &tx,
                                    Some(Failure::account(
                                        AccountError::ProviderUnavailable,
                                        ErrorContext::SignIn,
                                    )),
                                    cx,
                                );
                                return;
                            }
                        }
                    }
                    _ => None,
                };
                let cancel = Arc::new(AtomicBool::new(false));
                let (provider_label, user_code, verification_uri) = match &started.start {
                    ProviderStart::Apple(_) => ("Apple", None, None),
                    ProviderStart::Google(_) => ("Google", None, None),
                    ProviderStart::Github(start) => (
                        "GitHub",
                        Some(start.user_code.clone()),
                        Some(start.verification_uri.clone()),
                    ),
                };
                cx.global_mut::<Services>().provider_wait = Some(crate::services::ProviderWait {
                    provider: provider_label,
                    user_code,
                    verification_uri,
                    cancel: cancel.clone(),
                });
                continue_provider(
                    seams,
                    server,
                    provider,
                    login_device,
                    started,
                    apple_callback,
                    cancel,
                    server_generation,
                    task_kind,
                    window,
                    tx,
                    cx,
                );
            }
            Some(Err(failure)) => failed(&tx, Some(failure), cx),
            None => failed(&tx, None, cx),
        },
    );
    rx
}

enum ProviderTaskResult {
    Cancelled,
    NewAccount(kanso_sync::NewAccount),
    Locked(Box<LockedSession>, DeviceIdentity),
}

struct StartedProviderFlow {
    start: ProviderStart,
    loopback: Option<crate::provider_browser::Loopback>,
    verifier: Option<String>,
    deadline: Instant,
}

/// Preserves the provider flow's exact device proof and raw eviction answer
/// in the durable locked item before the UI exposes the locked state.
pub(crate) fn locked_provider_session(
    server: String,
    login_device: LoginDevice,
    session: ProviderSession,
    library_id: &str,
) -> Result<LockedSession, AccountError> {
    let origin = kanso_sync::origin(&server).map_err(AccountError::Endpoint)?;
    let mut locked = LockedSession::new(
        server,
        origin,
        session.account_id,
        session.username,
        login_device.id.clone(),
        session.token,
        session.key_id,
        session.epoch,
        session.passkey_wrapped_key,
        None,
    )
    .with_has_password(session.has_password)
    .with_login_device(login_device)
    .with_evicted_wire_device(session.evicted_device);
    locked.set_library_id(library_id);
    Ok(locked)
}

/// Makes the provider token durable before it can enter the app's locked
/// state. A failed save revokes or queues that token instead of exposing an
/// unusable pending account.
pub(crate) fn save_provider_pending(
    backend: &dyn Backend,
    secrets: &dyn SecretStore,
    locked: &LockedSession,
) -> Result<(), SecretError> {
    let Some(library_id) = locked.library_id() else {
        return Err(SecretError("the locked session has no library ID".into()));
    };
    match keychain::save_locked_pending(secrets, library_id, locked) {
        Ok(()) => Ok(()),
        Err(error) => {
            if let Err(cleanup) = kanso_sync::discard_locked_session(backend, secrets, locked) {
                eprintln!("kanso: discarding provider session after Keychain failure: {cleanup}");
            }
            Err(error)
        }
    }
}

/// Which routes a provider flow uses: the public sign-in and sign-up ones,
/// or a signed-in session's link or proof (docs/ACCOUNTS.md › Sign-in routes
/// › `identities/<provider>/start`).
#[derive(Clone)]
enum ProviderRoute {
    Public {
        server: String,
    },
    Session {
        session: Arc<Session>,
        intent: kanso_sync::provider::IdentityIntent,
        proof: Option<kanso_sync::provider::ProofWire>,
    },
}

/// One poll or finish of a provider flow.
enum Polled {
    Done(ProviderDone),
    Pending { slow_down: bool },
}

/// A provider flow's answer: a sign-in's, or a link's or proof's.
enum ProviderDone {
    SignIn(ProviderSignIn),
    Link(kanso_sync::provider::LinkOutcome),
}

impl Polled {
    fn sign_in(step: kanso_sync::ProviderStep) -> Polled {
        use kanso_sync::ProviderStep;
        match step {
            ProviderStep::Session(session) => {
                Polled::Done(ProviderDone::SignIn(ProviderSignIn::Session(session)))
            }
            ProviderStep::NewAccount(account) => {
                Polled::Done(ProviderDone::SignIn(ProviderSignIn::NewAccount(account)))
            }
            ProviderStep::Pending { slow_down } => Polled::Pending { slow_down },
        }
    }

    fn link(outcome: kanso_sync::provider::LinkOutcome) -> Polled {
        match outcome {
            kanso_sync::provider::LinkOutcome::Pending { slow_down } => {
                Polled::Pending { slow_down }
            }
            done => Polled::Done(ProviderDone::Link(done)),
        }
    }
}

impl ProviderRoute {
    fn start(
        &self,
        backend: &dyn Backend,
        provider: &Provider,
        device: &LoginDevice,
        loopback: Option<&kanso_sync::Loopback>,
        challenge: Option<&str>,
    ) -> Result<ProviderStart, AccountError> {
        match self {
            ProviderRoute::Public { server } => {
                kanso_sync::start_provider(backend, server, provider, device, loopback, challenge)
            }
            ProviderRoute::Session {
                session,
                intent,
                proof,
            } => kanso_sync::account::start_identity(
                backend,
                session,
                provider,
                *intent,
                proof.clone(),
                loopback,
                challenge,
                device,
            ),
        }
    }

    /// Apple's or GitHub's poll.
    fn poll(
        &self,
        backend: &dyn Backend,
        provider: &Provider,
        flow: &str,
        secret: Option<&str>,
        device: &LoginDevice,
    ) -> Result<Polled, AccountError> {
        match self {
            ProviderRoute::Public { server } => {
                kanso_sync::poll_provider(backend, server, provider, flow, secret, device)
                    .map(Polled::sign_in)
            }
            ProviderRoute::Session { session, .. } => kanso_sync::account::finish_identity(
                backend, session, provider, flow, secret, device,
            )
            .map(Polled::link),
        }
    }

    /// Google's finish.
    fn google(
        &self,
        backend: &dyn Backend,
        flow: &str,
        code: &str,
        verifier: &str,
        device: &LoginDevice,
    ) -> Result<Polled, AccountError> {
        match self {
            ProviderRoute::Public { server } => {
                kanso_sync::finish_google(backend, server, flow, code, verifier, device)
                    .map(Polled::sign_in)
            }
            ProviderRoute::Session { session, .. } => kanso_sync::account::finish_identity_google(
                backend,
                session,
                &Provider::Google,
                flow,
                code,
                verifier,
                device,
            )
            .map(Polled::link),
        }
    }
}

/// Starts the server side of a provider flow. This runs on the worker; any
/// native authentication sheet is opened later by the main-thread callback.
fn provider_begin(
    backend: &dyn Backend,
    route: &ProviderRoute,
    provider: &Provider,
    device: &LoginDevice,
    progress: &dyn Fn(Step),
) -> Result<StartedProviderFlow, AccountError> {
    let (loopback, verifier, challenge) = if matches!(provider, Provider::Google) {
        let loopback = crate::provider_browser::Loopback::bind()
            .map_err(|_| AccountError::ProviderUnavailable)?;
        let verifier: String = (0..64).map(|_| fastrand::alphanumeric()).collect();
        let digest = sha2::Sha256::digest(verifier.as_bytes());
        let challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(digest);
        (Some(loopback), Some(verifier), Some(challenge))
    } else {
        (None, None, None)
    };
    let wire_loopback = loopback.as_ref().map(|loopback| {
        kanso_sync::Loopback::parse(&loopback.uri()).expect("bound loopback has canonical URI")
    });
    let start = route.start(
        backend,
        provider,
        device,
        wire_loopback.as_ref(),
        challenge.as_deref(),
    )?;
    let deadline = provider_deadline(&start);
    progress(Step::Contacting);
    progress(Step::Contacting);
    Ok(StartedProviderFlow {
        start,
        loopback,
        verifier,
        deadline,
    })
}

/// Finishes a started flow off the UI thread. The optional Apple callback
/// receiver was obtained by presenting ASWebAuthenticationSession on the
/// main thread.
#[allow(clippy::too_many_arguments)] // The arguments are the explicit ceremony context.
fn provider_finish(
    backend: &dyn Backend,
    route: &ProviderRoute,
    provider: &Provider,
    device: &LoginDevice,
    started: StartedProviderFlow,
    apple_callback: Option<
        async_channel::Receiver<Result<String, crate::provider_browser::BrowserError>>,
    >,
    cancel: Arc<AtomicBool>,
    _progress: &dyn Fn(Step),
) -> Result<Option<ProviderDone>, AccountError> {
    let StartedProviderFlow {
        start,
        loopback,
        verifier,
        deadline,
    } = started;
    let outcome = match (&start, provider) {
        (ProviderStart::Apple(start), Provider::Apple) => {
            let callback = apple_callback.ok_or(AccountError::ProviderUnavailable)?;
            let callback = loop {
                if wait_cancelled_or_expired(&cancel, deadline)? {
                    return Ok(None);
                }
                match callback.try_recv() {
                    Ok(result) => {
                        break result.map_err(|err| match err {
                            crate::provider_browser::BrowserError::Cancelled
                            | crate::provider_browser::BrowserError::ProviderDenied => {
                                AccountError::ProviderDenied
                            }
                            _ => AccountError::ProviderUnavailable,
                        })?;
                    }
                    Err(async_channel::TryRecvError::Closed) => {
                        return Err(AccountError::ProviderUnavailable);
                    }
                    Err(async_channel::TryRecvError::Empty) => {
                        if wait_provider(&cancel, deadline, Duration::from_millis(50))? {
                            return Ok(None);
                        }
                    }
                }
            };
            // Only the callback for the flow this app started counts; its secret
            // proves to the Worker that this process finished it at Apple.
            let callback = crate::provider_browser::apple_flow(&callback)
                .filter(|callback| callback.flow == start.flow_id)
                .ok_or(AccountError::ProviderDenied)?;
            loop {
                if wait_cancelled_or_expired(&cancel, deadline)? {
                    return Ok(None);
                }
                let answer = route.poll(
                    backend,
                    provider,
                    &callback.flow,
                    Some(&callback.secret),
                    device,
                )?;
                if wait_cancelled_or_expired(&cancel, deadline)? {
                    return Ok(None);
                }
                match answer {
                    Polled::Done(done) => break done,
                    Polled::Pending { .. } => {
                        if wait_provider(&cancel, deadline, Duration::from_secs(4))? {
                            return Ok(None);
                        }
                    }
                }
            }
        }
        (ProviderStart::Google(start), Provider::Google) => {
            crate::provider_browser::google_device_page(start)
                .map_err(|_| AccountError::ProviderUnavailable)?;
            let code = match loopback
                .as_ref()
                .ok_or(AccountError::ProviderUnavailable)?
                .wait_until(&start.state, deadline, &cancel)
            {
                Ok(code) => code,
                Err(crate::provider_browser::BrowserError::Cancelled)
                    if cancel.load(Ordering::Acquire) =>
                {
                    return Ok(None);
                }
                Err(crate::provider_browser::BrowserError::Expired) => {
                    return Err(AccountError::FlowExpired);
                }
                Err(_) => return Err(AccountError::ProviderUnavailable),
            };
            if wait_cancelled_or_expired(&cancel, deadline)? {
                return Ok(None);
            }
            let answer = route.google(
                backend,
                &start.flow_id,
                &code,
                verifier.as_deref().ok_or(AccountError::BadRequest)?,
                device,
            )?;
            if wait_cancelled_or_expired(&cancel, deadline)? {
                return Ok(None);
            }
            match answer {
                Polled::Done(done) => done,
                Polled::Pending { .. } => return Err(AccountError::ProviderUnavailable),
            }
        }
        (ProviderStart::Github(start), Provider::Github) => {
            crate::provider_browser::github_device_page(start)
                .map_err(|_| AccountError::ProviderUnavailable)?;
            let mut interval = github_poll_interval(start.interval);
            // The Worker rejects a poll before next_poll_at.
            if wait_provider(&cancel, deadline, Duration::from_secs(interval))? {
                return Ok(None);
            }
            loop {
                if wait_cancelled_or_expired(&cancel, deadline)? {
                    return Ok(None);
                }
                let answer = route.poll(backend, provider, &start.flow_id, None, device)?;
                if wait_cancelled_or_expired(&cancel, deadline)? {
                    return Ok(None);
                }
                match answer {
                    Polled::Done(done) => break done,
                    Polled::Pending { slow_down } => {
                        if slow_down {
                            interval = github_next_poll_interval(interval, slow_down);
                        }
                        if wait_provider(&cancel, deadline, Duration::from_secs(interval))? {
                            return Ok(None);
                        }
                    }
                }
            }
        }
        _ => return Err(AccountError::ProviderUnavailable),
    };
    Ok(Some(outcome))
}

fn provider_deadline(start: &ProviderStart) -> Instant {
    let remaining = match start {
        ProviderStart::Github(start) => Duration::from_secs(start.expires_in),
        ProviderStart::Apple(start) => remaining_until_epoch_ms(start.expires_at),
        ProviderStart::Google(start) => remaining_until_epoch_ms(start.expires_at),
    };
    Instant::now() + remaining
}

fn remaining_until_epoch_ms(expires_at: i64) -> Duration {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;
    Duration::from_millis(expires_at.saturating_sub(now_ms).max(0) as u64)
}

/// Returns true only for local cancellation; expiry remains a user-visible flow error.
fn wait_cancelled_or_expired(cancel: &AtomicBool, deadline: Instant) -> Result<bool, AccountError> {
    if cancel.load(Ordering::Acquire) {
        return Ok(true);
    }
    if Instant::now() >= deadline {
        return Err(AccountError::FlowExpired);
    }
    Ok(false)
}

/// Waits in short slices so cancellation and expiry are observed promptly.
fn wait_provider(
    cancel: &AtomicBool,
    deadline: Instant,
    duration: Duration,
) -> Result<bool, AccountError> {
    let end = Instant::now() + duration;
    loop {
        if wait_cancelled_or_expired(cancel, deadline)? {
            return Ok(true);
        }
        let remaining = end.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Ok(false);
        }
        std::thread::sleep(remaining.min(Duration::from_millis(50)));
    }
}

fn github_poll_interval(start_interval: u64) -> u64 {
    start_interval.max(1)
}

fn github_next_poll_interval(interval: u64, slow_down: bool) -> u64 {
    if slow_down {
        interval.saturating_add(5)
    } else {
        interval
    }
}

#[allow(clippy::too_many_arguments)] // Keeps the whole durable continuation in one place.
fn continue_provider(
    seams: Seams,
    server: String,
    provider: Provider,
    login_device: LoginDevice,
    started: StartedProviderFlow,
    apple_callback: Option<
        async_channel::Receiver<Result<String, crate::provider_browser::BrowserError>>,
    >,
    cancel: Arc<AtomicBool>,
    server_generation: u64,
    task_kind: TaskKind,
    window: AnyWindowHandle,
    tx: Sender<Progress>,
    cx: &mut App,
) {
    let thread = seams.clone();
    let wait_cancel = cancel.clone();
    let work_server = server.clone();
    let work_provider = provider.clone();
    let work_login_device = login_device.clone();
    let result_server = server.clone();
    run(
        task_kind,
        Step::Contacting,
        false,
        Some(tx.clone()),
        cx,
        move |step| {
            let answer = provider_finish(
                thread.backend.as_ref(),
                &ProviderRoute::Public {
                    server: work_server.clone(),
                },
                &work_provider,
                &work_login_device,
                started,
                apple_callback,
                cancel,
                step,
            )
            .map_err(|err| Failure::account(err, ErrorContext::SignIn))?;
            match answer {
                None => Ok(ProviderTaskResult::Cancelled),
                Some(ProviderDone::Link(_)) => Err(Failure::account(
                    AccountError::Protocol("a link answer to a sign-in".into()),
                    ErrorContext::SignIn,
                )),
                Some(ProviderDone::SignIn(ProviderSignIn::NewAccount(account))) => {
                    Ok(ProviderTaskResult::NewAccount(account))
                }
                Some(ProviderDone::SignIn(ProviderSignIn::Session(session))) => {
                    let device = thread.device()?;
                    let locked = locked_provider_session(
                        work_server,
                        work_login_device.clone(),
                        session,
                        &device.library_id,
                    )
                    .map_err(|error| Failure::account(error, ErrorContext::SignIn))?;
                    if let Err(error) = save_provider_pending(
                        thread.backend.as_ref(),
                        thread.secrets.as_ref(),
                        &locked,
                    ) {
                        return Err(Failure::keychain(error));
                    }
                    Ok(ProviderTaskResult::Locked(Box::new(locked), device))
                }
            }
        },
        move |result, cx| {
            // A later flow may have replaced this one while it was finishing.
            // Clear only the wait this task owns.
            clear_provider_wait_if_owned(
                &mut cx.global_mut::<Services>().provider_wait,
                &wait_cancel,
            );
            match result {
                Some(Ok(ProviderTaskResult::Cancelled)) => {
                    done(&tx, Ok(()));
                }
                Some(Ok(ProviderTaskResult::NewAccount(account))) => {
                    if wait_cancel.load(Ordering::Acquire)
                        || !provider_request_is_current(cx, &result_server, server_generation)
                    {
                        done(&tx, Ok(()));
                        return;
                    }
                    cx.global_mut::<Services>().provider_signup =
                        Some(crate::services::ProviderSignup {
                            server: result_server,
                            provider,
                            ticket: account.ticket,
                            suggested_username: account.suggested_username,
                            login_device,
                        });
                    done(&tx, Ok(()));
                }
                Some(Ok(ProviderTaskResult::Locked(locked, device))) => {
                    if wait_cancel.load(Ordering::Acquire)
                        || !provider_request_is_current(cx, &result_server, server_generation)
                    {
                        // The pending item was already made durable on the worker
                        // so a crash could not lose the token. Remove it and sign
                        // the token out now that this result belongs to a server
                        // the UI no longer shows.
                        discard_stale_locked_provider_session(seams, locked, tx, cx);
                        return;
                    }
                    enter_locked(*locked, device, Some(window), cx);
                    done(&tx, Ok(()));
                }
                Some(Err(failure)) => failed(&tx, Some(failure), cx),
                None => failed(&tx, None, cx),
            }
        },
    );
}

/// A provider answer belongs to the exact server form that started it, and
/// to the same server generation. Generation makes A -> B -> A safe: the
/// server string can return to A while the old flow's answer is still stale.
fn provider_request_matches(
    current_server: &str,
    current_generation: u64,
    asked_server: &str,
    asked_generation: u64,
) -> bool {
    current_server == asked_server && current_generation == asked_generation
}

fn provider_request_is_current(cx: &App, server: &str, generation: u64) -> bool {
    provider_request_matches(
        &form_server(cx),
        cx.global::<Services>().auth_generation,
        server,
        generation,
    )
}

/// Provider callbacks finish after the visible wait is gone. A delayed old
/// callback must not clear a newer flow's wait, so ownership is the same
/// cancellation token the wait was created with.
fn clear_provider_wait_if_owned(
    wait: &mut Option<crate::services::ProviderWait>,
    cancel: &Arc<AtomicBool>,
) {
    if wait
        .as_ref()
        .is_some_and(|wait| Arc::ptr_eq(&wait.cancel, cancel))
    {
        *wait = None;
    }
}

/// Cleans up a locked result that became stale during its final callback.
/// The pending Keychain item and server token are removed off the UI thread;
/// the current server's form and any newer provider flow are left alone.
fn discard_stale_locked_provider_session(
    seams: Seams,
    locked: Box<LockedSession>,
    tx: Sender<Progress>,
    cx: &mut App,
) {
    run(
        TaskKind::SignOut,
        Step::Contacting,
        false,
        None,
        cx,
        move |_| {
            if let Err(error) = kanso_sync::discard_locked_session(
                seams.backend.as_ref(),
                seams.secrets.as_ref(),
                locked.as_ref(),
            ) {
                eprintln!("kanso: discarding a stale provider session: {error}");
            }
        },
        move |_, _cx| done(&tx, Ok(())),
    );
}

/// Creates a provider-backed passwordless account after the user confirms an
/// editable username, then follows the same Keychain, binding and recovery
/// key path as username/password signup.
pub fn provider_sign_up(
    pending: crate::services::ProviderSignup,
    username: String,
    window: AnyWindowHandle,
    cx: &mut App,
) -> Receiver<Progress> {
    let (tx, rx) = async_channel::unbounded();
    let Some(seams) = begin(&tx, cx) else {
        return rx;
    };
    let thread = seams.clone();
    let server = pending.server.clone();
    let asked_server = server.clone();
    run(
        TaskKind::SignUp,
        Step::Contacting,
        false,
        Some(tx.clone()),
        cx,
        move |_| {
            let origin = kanso_sync::origin(&asked_server).map_err(|err| {
                Failure::account(AccountError::Endpoint(err), ErrorContext::SignUp)
            })?;
            kanso_sync::local_library(&thread.store, &origin, "").map_err(Failure::library)
        },
        move |result, cx| match result {
            Some(Ok(library)) => match kanso_sync::plan(Flow::SignUp, &library) {
                Plan::AskBeforeSignUp { from, .. } => {
                    let (title, body, upload) = words::upload_question(&from);
                    ask(
                        window,
                        Question {
                            title,
                            body,
                            answers: vec![Answer {
                                choice: Choice::Add,
                                label: upload.to_owned(),
                                description: "They're encrypted and uploaded to the new account. Nothing changes in the other one.".to_owned(),
                            }],
                        },
                        cx,
                        move |choice, cx| match choice {
                            Choice::Cancel => {
                                cx.global_mut::<Services>().provider_signup = None;
                                done(&tx, Ok(()));
                            }
                            Choice::Add | Choice::Replace => create_provider_signup(
                                seams, server, pending, username, library, tx, cx,
                            ),
                        },
                    );
                }
                _ => create_provider_signup(seams, server, pending, username, library, tx, cx),
            },
            Some(Err(failure)) => failed(&tx, Some(failure), cx),
            None => failed(&tx, None, cx),
        },
    );
    rx
}

fn create_provider_signup(
    seams: Seams,
    server: String,
    pending: crate::services::ProviderSignup,
    username: String,
    library: kanso_sync::LocalLibrary,
    tx: Sender<Progress>,
    cx: &mut App,
) {
    let thread = seams.clone();
    run(
        TaskKind::SignUp,
        Step::DerivingKeys,
        false,
        Some(tx.clone()),
        cx,
        move |step| {
            let device = thread.device()?;
            let signed = kanso_sync::sign_up_with_provider(
                thread.backend.as_ref(),
                thread.secrets.as_ref(),
                &server,
                &pending.provider,
                &pending.ticket,
                &username,
                &device,
                &pending.login_device,
                step,
            )
            .map_err(|err| {
                Failure::rotation(err, ErrorContext::SignUp, words::KEY_UNCONFIRMED_SIGN_UP)
            })?;
            Ok(Authenticated {
                session: signed.session,
                evicted: None,
                library,
                library_id: device.library_id,
                recovery_key: Some(signed.recovery_key),
            })
        },
        move |result, cx| match result {
            Some(Ok(auth)) => {
                cx.global_mut::<Services>().provider_signup = None;
                if let Some(key) = &auth.recovery_key {
                    show_key(&auth.session, key.clone(), cx);
                }
                bind_and_save(
                    TaskKind::SignUp,
                    seams,
                    auth,
                    Some(Choice::Add),
                    false,
                    tx,
                    cx,
                );
            }
            Some(Err(failure)) => failed(&tx, Some(failure), cx),
            None => failed(&tx, None, cx),
        },
    );
}

// MARK: Passkeys and security keys

/// Which authenticator a passkey task uses.
pub enum PasskeyMethod {
    /// The platform's sheet: AuthenticationServices on macOS, Windows Hello
    /// and security keys through webauthn.h on Windows, owned by `window`
    /// (`passkey_ceremony::native_window`; 0 when there is none).
    Platform { window: isize },
    /// A Linux security key, with its PIN when it needs one.
    SecurityKey {
        device: kanso_fido::SecurityKeyDevice,
        pin: Option<zeroize::Zeroizing<String>>,
    },
}

impl PasskeyMethod {
    fn is_security_key(&self) -> bool {
        matches!(self, PasskeyMethod::SecurityKey { .. })
    }
}

/// The ceremony a passkey task's thread runs, made on the main thread.
enum Runner {
    /// The platform's sheet (`passkey_ceremony::platform_ceremony`).
    Platform(PlatformCeremony),
    SecurityKey(OwnedSecurityKey),
    #[cfg(test)]
    Test(Arc<dyn kanso_sync::PasskeyCeremony + Send + Sync>),
}

impl Runner {
    /// The runner, and the main-thread task serving a platform ceremony
    /// (kept until the task finishes).
    fn new(method: PasskeyMethod, cx: &mut App) -> (Runner, Option<Task<()>>) {
        #[cfg(test)]
        if let Some(ceremony) = cx.global::<Services>().test_ceremony.clone() {
            return (Runner::Test(ceremony), None);
        }
        match method {
            PasskeyMethod::Platform { window } => {
                let (ceremony, serving) = platform_ceremony(window, cx);
                (Runner::Platform(ceremony), serving)
            }
            PasskeyMethod::SecurityKey { device, pin } => {
                (Runner::SecurityKey(OwnedSecurityKey { device, pin }), None)
            }
        }
    }

    fn rule(&self) -> RpRule {
        match self {
            Runner::Platform(_) => RpRule::platform(),
            Runner::SecurityKey(_) => RpRule::ServerHost,
            #[cfg(test)]
            Runner::Test(_) => RpRule::ServerHost,
        }
    }

    /// Runs `work` with the ceremony; afterwards, what a security key said
    /// about its PIN, if anything.
    fn run<T>(
        self,
        work: impl FnOnce(&dyn kanso_sync::PasskeyCeremony) -> T,
    ) -> (T, Option<PinProblem>) {
        match self {
            Runner::Platform(ceremony) => (work(ceremony.as_ref()), None),
            Runner::SecurityKey(key) => key.with(work),
            #[cfg(test)]
            Runner::Test(ceremony) => (work(ceremony.as_ref()), None),
        }
    }
}

/// How a passkey task ended without an answer.
enum PasskeyEnd {
    /// The sheet or the key was dismissed: the page stays as it was.
    Cancelled,
    /// The platform's ceremony can't run here: the buttons go.
    Unavailable,
    Failed(Failure),
}

impl From<Failure> for PasskeyEnd {
    fn from(failure: Failure) -> Self {
        PasskeyEnd::Failed(failure)
    }
}

fn passkey_end(
    error: PasskeyFlowError,
    pin: Option<PinProblem>,
    security_key: bool,
    context: ErrorContext,
) -> PasskeyEnd {
    use kanso_sync::PasskeyCeremonyError as Ceremony;
    let failed = if security_key {
        words::SECURITY_KEY_FAILED
    } else {
        words::PASSKEY_FAILED
    };
    match pin {
        Some(PinProblem::Required) => return Failure::words(words::SECURITY_KEY_PIN).into(),
        Some(PinProblem::Invalid { retries }) => {
            return Failure::words(&words::security_key_pin_wrong(retries)).into();
        }
        Some(PinProblem::AuthBlocked) => {
            return Failure::words(words::SECURITY_KEY_PIN_AUTH_BLOCKED).into();
        }
        Some(PinProblem::Blocked) => return Failure::words(words::SECURITY_KEY_PIN_BLOCKED).into(),
        None => {}
    }
    match error {
        PasskeyFlowError::Account(err) => Failure::account(err, context).into(),
        PasskeyFlowError::Rotation(err) => Failure::rotation(
            err,
            ErrorContext::SignUp,
            words::KEY_UNCONFIRMED_PASSKEY_SIGN_UP,
        )
        .into(),
        PasskeyFlowError::Ceremony(Ceremony::Cancelled) => PasskeyEnd::Cancelled,
        // A security key that went away is the page's to show again.
        PasskeyFlowError::Ceremony(Ceremony::Unavailable) if security_key => {
            Failure::words(failed).into()
        }
        PasskeyFlowError::Ceremony(Ceremony::Unavailable) => PasskeyEnd::Unavailable,
        PasskeyFlowError::Ceremony(Ceremony::Failed) => Failure::words(failed).into(),
        PasskeyFlowError::WrongRelyingParty => Failure::words(words::PASSKEY_WRONG_SERVER).into(),
    }
}

fn passkey_ended(end: PasskeyEnd, tx: &Sender<Progress>, cx: &mut App) {
    match end {
        PasskeyEnd::Cancelled => done(tx, Ok(())),
        PasskeyEnd::Unavailable => {
            cx.global_mut::<Services>().platform_passkeys_failed = true;
            done(tx, Err(words::PASSKEY_UNAVAILABLE.to_owned()));
        }
        PasskeyEnd::Failed(failure) => failed(tx, Some(failure), cx),
    }
}

/// What a passkey sign-in's thread found.
enum PasskeyIn {
    /// The key came with it: binding as after a password sign-in.
    Unlocked(Box<Authenticated>),
    /// Signed in without the key, its pending item saved: approval, the
    /// password or the recovery key unlocks it. The flag says the
    /// credential answered PRF.
    Locked(Box<LockedSession>, DeviceIdentity, bool),
}

/// Signs in with a passkey or a security key (Settings › Account › Sign in
/// › Continue with a passkey / a security key). `username` is the security
/// key's username-first name; the platform picker takes none.
pub fn passkey_sign_in(
    server: String,
    method: PasskeyMethod,
    username: Option<String>,
    window: AnyWindowHandle,
    cx: &mut App,
) -> Receiver<Progress> {
    let (tx, rx) = async_channel::unbounded();
    let Some(seams) = begin(&tx, cx) else {
        return rx;
    };
    let security_key = method.is_security_key();
    let (runner, serving) = Runner::new(method, cx);
    let rule = runner.rule();
    let was_signed_in = cx.global::<Services>().signed_in();
    let thread = seams.clone();
    run(
        TaskKind::SignIn,
        Step::Contacting,
        was_signed_in,
        Some(tx.clone()),
        cx,
        move |step| -> Result<PasskeyIn, PasskeyEnd> {
            let s = thread;
            let device = s.device()?;
            let (signed, pin) = runner.run(|ceremony| {
                let context = PasskeyContext {
                    accounts: &SyncPasskeyAccounts,
                    backend: s.backend.as_ref(),
                    secrets: s.secrets.as_ref(),
                    server: &server,
                    device: &device,
                    rule,
                    progress: step,
                };
                passkey_accounts::sign_in(&context, username.as_deref(), ceremony)
            });
            let signed =
                signed.map_err(|err| passkey_end(err, pin, security_key, ErrorContext::Passkey))?;
            match signed.signed {
                kanso_sync::PasskeySignIn::Unlocked(kanso_sync::SignedIn { session, evicted }) => {
                    let auth = authenticated(&s, device.library_id, session, evicted, None)?;
                    Ok(PasskeyIn::Unlocked(Box::new(auth)))
                }
                kanso_sync::PasskeySignIn::Locked(locked) => {
                    save_provider_pending(s.backend.as_ref(), s.secrets.as_ref(), &locked)
                        .map_err(Failure::keychain)?;
                    Ok(PasskeyIn::Locked(Box::new(locked), device, signed.prf))
                }
            }
        },
        move |result, cx| {
            drop(serving);
            match result {
                Some(Ok(PasskeyIn::Unlocked(auth))) => settle_authenticated(
                    Flow::SignIn,
                    Some(window),
                    seams,
                    *auth,
                    was_signed_in,
                    tx,
                    cx,
                ),
                Some(Ok(PasskeyIn::Locked(locked, device, prf))) => {
                    enter_locked(*locked, device, Some(window), cx);
                    if !prf {
                        let notice = words::passkey_signs_in_only(security_key);
                        notify_windows(move || Notification::info(notice), cx);
                    }
                    done(&tx, Ok(()));
                }
                Some(Err(end)) => passkey_ended(end, &tx, cx),
                None => failed(&tx, None, cx),
            }
        },
    );
    rx
}

/// Creates an account whose first way in is a new passkey or security key
/// (Settings › Account › Create account › Create with a passkey / a
/// security key), and signs this device in with it. Notes that came from
/// another account are asked about in `window` first, as for a password
/// sign-up.
pub fn passkey_sign_up(
    server: String,
    method: PasskeyMethod,
    username: String,
    window: AnyWindowHandle,
    cx: &mut App,
) -> Receiver<Progress> {
    let (tx, rx) = async_channel::unbounded();
    let Some(seams) = begin(&tx, cx) else {
        return rx;
    };
    let origin = kanso_sync::origin(&server).unwrap_or_default();
    let thread = seams.clone();
    run(
        TaskKind::SignUp,
        Step::Contacting,
        false,
        Some(tx.clone()),
        cx,
        move |_| kanso_sync::local_library(&thread.store, &origin, "").map_err(Failure::library),
        move |result, cx| match result {
            Some(Ok(library)) => match kanso_sync::plan(Flow::SignUp, &library) {
                Plan::AskBeforeSignUp { from, .. } => {
                    let (title, body, upload) = words::upload_question(&from);
                    let question = Question {
                        title,
                        body,
                        answers: vec![Answer {
                            choice: Choice::Add,
                            label: upload.to_owned(),
                            description: "They're encrypted and uploaded to the new account. Nothing changes in the other one.".to_owned(),
                        }],
                    };
                    ask(window, question, cx, move |choice, cx| match choice {
                        Choice::Cancel => done(&tx, Ok(())),
                        _ => create_with_passkey(seams, server, username, method, tx, cx),
                    });
                }
                _ => create_with_passkey(seams, server, username, method, tx, cx),
            },
            Some(Err(failure)) => failed(&tx, Some(failure), cx),
            None => failed(&tx, None, cx),
        },
    );
    rx
}

fn create_with_passkey(
    seams: Seams,
    server: String,
    username: String,
    method: PasskeyMethod,
    tx: Sender<Progress>,
    cx: &mut App,
) {
    let security_key = method.is_security_key();
    let (runner, serving) = Runner::new(method, cx);
    let rule = runner.rule();
    let thread = seams.clone();
    run(
        TaskKind::SignUp,
        Step::Contacting,
        true,
        Some(tx.clone()),
        cx,
        move |step| -> Result<(PasskeySignedUp, String), PasskeyEnd> {
            let s = thread;
            let device = s.device()?;
            let (made, pin) = runner.run(|ceremony| {
                let context = PasskeyContext {
                    accounts: &SyncPasskeyAccounts,
                    backend: s.backend.as_ref(),
                    secrets: s.secrets.as_ref(),
                    server: &server,
                    device: &device,
                    rule,
                    progress: step,
                };
                passkey_accounts::sign_up(&context, &username, ceremony)
            });
            let made =
                made.map_err(|err| passkey_end(err, pin, security_key, ErrorContext::SignUp))?;
            step(Step::Saving);
            let bound = save_and_bind(
                &s,
                &device.library_id,
                &made.signed.session,
                Some(Choice::Add),
            );
            if let Err(failure) = bound {
                // As after a password sign-up: the account and its key stay.
                kanso_sync::discard_session(
                    s.backend.as_ref(),
                    s.secrets.as_ref(),
                    &made.signed.session,
                )
                .ok();
                let SignedUp {
                    session,
                    recovery_key,
                } = made.signed;
                return Err(failure
                    .after_key(words::CREATED_NOT_SIGNED_IN_PASSKEY, &session, recovery_key)
                    .into());
            }
            Ok((made, device.library_id))
        },
        move |result, cx| {
            drop(serving);
            match result {
                Some(Ok((made, library_id))) => {
                    let PasskeySignedUp { signed, can_unlock } = made;
                    let SignedUp {
                        session,
                        recovery_key,
                    } = signed;
                    show_key(&session, recovery_key, cx);
                    signed_in(session, library_id, true, cx);
                    let notice = words::passkey_created(security_key, can_unlock);
                    if can_unlock {
                        notify_windows(move || Notification::success(notice), cx);
                    } else {
                        notify_windows(move || Notification::warning(notice), cx);
                    }
                    done(&tx, Ok(()));
                }
                Some(Err(end)) => passkey_ended(end, &tx, cx),
                None => failed(&tx, None, cx),
            }
        },
    );
}

/// Sets a new password with the recovery key, and signs this Mac in
/// ("Forgot your password?"). Signed out, it asks in `window` what to do
/// with notes already here; signed in, it replaces this Mac's session.
pub fn recover(
    server: String,
    username: String,
    key: RecoveryKey,
    password: Password,
    window: Option<AnyWindowHandle>,
    cx: &mut App,
) -> Receiver<Progress> {
    let (tx, rx) = async_channel::unbounded();
    let Some(seams) = begin(&tx, cx) else {
        return rx;
    };
    authenticate(
        Flow::Recover,
        window,
        seams,
        tx,
        cx,
        Box::new(move |s, device, step| {
            // The new key is in the Keychain before the request goes.
            let recovered = kanso_sync::recover(
                s.backend.as_ref(),
                s.secrets.as_ref(),
                &server,
                &username,
                &key,
                &password,
                device,
                step,
            )
            .map_err(|err| {
                Failure::rotation(err, ErrorContext::Recovery, words::KEY_UNCONFIRMED_PASSWORD)
            })?;
            Ok((recovered.session, None, Some(recovered.recovery_key)))
        }),
    );
    rx
}

/// "Forgot your password?" while signed in: the account and server are this
/// Mac's own.
pub fn recover_signed_in(key: RecoveryKey, password: Password, cx: &mut App) -> Receiver<Progress> {
    let account = cx
        .global::<Services>()
        .session()
        .map(|session| (session.server.clone(), session.username.clone()));
    match account {
        Some((server, username)) => recover(server, username, key, password, None, cx),
        None => {
            let (tx, rx) = async_channel::unbounded();
            done(&tx, Err(SOMETHING_WRONG.to_owned()));
            rx
        }
    }
}

fn authenticate(
    flow: Flow,
    window: Option<AnyWindowHandle>,
    seams: Seams,
    tx: Sender<Progress>,
    cx: &mut App,
    auth: Authenticate,
) {
    let kind = match flow {
        Flow::SignUp => TaskKind::SignUp,
        Flow::SignIn => TaskKind::SignIn,
        Flow::Recover => TaskKind::Recover,
    };
    // Signed in, the running sync stops first: its session is about to be
    // replaced, and nothing may sync in between.
    let was_signed_in = cx.global::<Services>().signed_in();
    let thread = seams.clone();
    run(
        kind,
        Step::Contacting,
        was_signed_in,
        Some(tx.clone()),
        cx,
        move |step| {
            let s = thread;
            let device = s.device()?;
            let (session, evicted, recovery_key) = auth(&s, &device, step)?;
            authenticated(&s, device.library_id, session, evicted, recovery_key)
        },
        move |result, cx| match result {
            Some(Ok(auth)) => {
                settle_authenticated(flow, window, seams, auth, was_signed_in, tx, cx)
            }
            Some(Err(failure)) => failed(&tx, Some(failure), cx),
            None => failed(&tx, None, cx),
        },
    );
}

/// On the task's thread, after a sign-in: the library the account would
/// bind, or the failure (the new session is logged out).
fn authenticated(
    s: &Seams,
    library_id: String,
    session: Session,
    evicted: Option<DeviceInfo>,
    recovery_key: Option<RecoveryKey>,
) -> Result<Authenticated, Failure> {
    let library = match kanso_sync::local_library(&s.store, &session.origin, &session.account_id) {
        Ok(library) => library,
        Err(err) => {
            let failure = Failure::library(err);
            kanso_sync::discard_session(s.backend.as_ref(), s.secrets.as_ref(), &session).ok();
            return Err(match recovery_key {
                Some(key) => failure.after_key(words::CHANGED_NOT_SIGNED_IN, &session, key),
                None => failure,
            });
        }
    };
    Ok(Authenticated {
        session,
        evicted,
        library,
        library_id,
        recovery_key,
    })
}

/// On the main thread, after [`authenticated`]: the binding question when
/// there is one, then the binding and the Keychain item.
fn settle_authenticated(
    flow: Flow,
    window: Option<AnyWindowHandle>,
    seams: Seams,
    auth: Authenticated,
    was_signed_in: bool,
    tx: Sender<Progress>,
    cx: &mut App,
) {
    let kind = match flow {
        Flow::SignUp => TaskKind::SignUp,
        Flow::SignIn => TaskKind::SignIn,
        Flow::Recover => TaskKind::Recover,
    };
    // A used recovery key is gone: the new one shows at once, even if
    // the binding question is cancelled.
    if let Some(key) = &auth.recovery_key {
        show_key(&auth.session, key.clone(), cx);
    }
    let plan = kanso_sync::plan(flow, &auth.library);
    let question = words::binding_question(&plan, &auth.session.username);
    match (plan, question, window) {
        (Plan::Ask { .. }, Some(question), Some(window)) => {
            ask(window, question, cx, move |choice, cx| match choice {
                Choice::Cancel => abandon(seams, auth.session, tx, cx),
                choice => bind_and_save(kind, seams, auth, Some(choice), was_signed_in, tx, cx),
            });
        }
        (Plan::Ask { .. }, _, _) => abandon(seams, auth.session, tx, cx),
        (Plan::Resume, _, _) => bind_and_save(kind, seams, auth, None, was_signed_in, tx, cx),
        (Plan::Fresh | Plan::AskBeforeSignUp { .. }, _, _) => {
            bind_and_save(kind, seams, auth, Some(Choice::Add), was_signed_in, tx, cx)
        }
    }
}

/// Binds the library (unless `choice` is `None`: it already is), saves the
/// Keychain item, then signs the window in.
fn bind_and_save(
    kind: TaskKind,
    seams: Seams,
    auth: Authenticated,
    choice: Option<Choice>,
    was_signed_in: bool,
    tx: Sender<Progress>,
    cx: &mut App,
) {
    let Authenticated {
        session,
        evicted,
        library_id,
        recovery_key,
        ..
    } = auth;
    // A Replace backs the library up, then empties it: a write in between
    // would be in neither. Edits so far are saved into the backup first;
    // the editors and the local API wait until it is done (A10). The
    // editors are reached once the window that chose Replace has finished
    // its update.
    let replacing = choice == Some(Choice::Replace);
    let begin = move |cx: &mut App| {
        let (api, restart_api) = if replacing {
            services::hold_writes(cx)
        } else {
            (None, false)
        };
        let thread = seams.clone();
        run(
            kind,
            Step::Saving,
            true,
            Some(tx.clone()),
            cx,
            move |step| {
                let s = thread;
                step(Step::Saving);
                if let Some(api) = api
                    && let Err(err) = api.stop()
                {
                    eprintln!("kanso: the API server stopped with an error: {err:#}");
                }
                match save_and_bind(&s, &library_id, &session, choice) {
                    Ok(backup) => Ok((session, library_id, backup)),
                    Err(failure) => {
                        // Nothing here uses the new session. The library's
                        // Keychain items are as they were: maybe another
                        // account's.
                        kanso_sync::discard_session(
                            s.backend.as_ref(),
                            s.secrets.as_ref(),
                            &session,
                        )
                        .ok();
                        // The password changed all the same.
                        Err(match recovery_key {
                            Some(key) => {
                                failure.after_key(words::CHANGED_NOT_SIGNED_IN, &session, key)
                            }
                            None => failure,
                        })
                    }
                }
            },
            move |result, cx| {
                if replacing {
                    services::release_writes(restart_api, cx);
                }
                bound(result, evicted, was_signed_in, tx, cx)
            },
        );
    };
    if replacing {
        cx.defer(begin);
    } else {
        begin(cx);
    }
}

/// What binding gives: the session, the library's ID, and a Replace's
/// backup.
type Bound = (Session, String, Option<PathBuf>);

/// The end of [`bind_and_save`]: signed in, or the failure.
fn bound(
    result: Option<Result<Bound, Failure>>,
    evicted: Option<DeviceInfo>,
    was_signed_in: bool,
    tx: Sender<Progress>,
    cx: &mut App,
) {
    match result {
        Some(Ok((session, library_id, backup))) => {
            signed_in(session, library_id, !was_signed_in, cx);
            if let Some(device) = evicted {
                let notice = words::evicted_notice(&device);
                notify_windows(move || Notification::info(notice.clone()), cx);
            }
            if let Some(path) = backup {
                notify_backup(path, cx);
            }
            done(&tx, Ok(()));
        }
        Some(Err(failure)) => failed(&tx, Some(failure), cx),
        None => failed(&tx, None, cx),
    }
}

/// The binding question was cancelled: the new session is logged out, and
/// nothing here changes. The library's Keychain items stay: they hold the
/// account this Mac may still be signed in to, and a recovery key that may
/// still be on screen.
fn abandon(seams: Seams, session: Session, tx: Sender<Progress>, cx: &mut App) {
    run(
        TaskKind::SignOut,
        Step::Contacting,
        false,
        Some(tx.clone()),
        cx,
        move |_| {
            if let Err(err) = kanso_sync::discard_session(
                seams.backend.as_ref(),
                seams.secrets.as_ref(),
                &session,
            ) {
                eprintln!("kanso: signing out the cancelled session: {err}");
            }
        },
        move |_, _| done(&tx, Ok(())),
    );
}

/// Leaves the provider-authenticated UI locked and tries only a saved local
/// full session for an offline key reuse. Approval never starts until the
/// locked item has been written.
pub(crate) fn enter_locked(
    mut locked: LockedSession,
    device: DeviceIdentity,
    window: Option<AnyWindowHandle>,
    cx: &mut App,
) {
    locked.set_library_id(&device.library_id);
    let Some(seams) = Seams::get(cx) else {
        return;
    };
    crate::device_approval::stop_events(cx);
    cx.global_mut::<Services>()
        .clear_session(AccountState::SignedOut);
    {
        let services = cx.global_mut::<Services>();
        services.library_id = Some(device.library_id.clone());
        services.locked_has_password = Some(locked.has_password);
        services.locked_session = Some(Arc::new(locked.clone()));
        services.account = AccountState::SignedInLocked {
            username: locked.username.clone(),
        };
    }

    // A matching full item is an offline key source. Its origin, account and
    // key ID are checked by the loader and unlock function before consent.
    match keychain::load_session(
        seams.secrets.as_ref(),
        &device.library_id,
        &locked.origin,
        &locked.account_id,
    ) {
        Ok(Some(local)) => match unlock_with_local_key(&locked, &local) {
            Ok(unlocked) => {
                finish_unlocked_candidate(
                    UnlockedCandidate {
                        session: unlocked.session,
                        locked_session: locked.clone(),
                        evicted: unlocked.evicted,
                        library_id: device.library_id,
                        same_token_as_locked: true,
                    },
                    window,
                    None,
                    cx,
                );
                return;
            }
            Err(error) => {
                eprintln!("kanso: local key did not unlock provider session: {error}")
            }
        },
        Ok(None) => {}
        Err(error) => eprintln!("kanso: reading the saved local session: {error}"),
    }
    if locked.approval_resume().is_some() {
        wait_for_device_approval(locked, device, window, cx);
    }
    cx.refresh_windows();
}

/// Runs on the UI thread after every unlock path. The local-library plan is
/// computed here so no worker can bind, choose a disposition, or start sync.
pub(crate) fn finish_unlocked_candidate(
    candidate: UnlockedCandidate,
    window: Option<AnyWindowHandle>,
    completion: Option<Sender<Progress>>,
    cx: &mut App,
) {
    {
        let locked = candidate.locked_session.clone();
        let services = cx.global_mut::<Services>();
        services.library_id = locked.library_id().map(str::to_owned);
        services.locked_has_password = Some(locked.has_password);
        services.locked_session = Some(Arc::new(locked.clone()));
        services.account = AccountState::SignedInLocked {
            username: locked.username.clone(),
        };
    }
    let window = window.or_else(|| cx.active_window());
    let Some(seams) = Seams::get(cx) else {
        return;
    };
    let library = match kanso_sync::local_library(
        &seams.store,
        &candidate.session.origin,
        &candidate.session.account_id,
    ) {
        Ok(library) => library,
        Err(error) => {
            eprintln!("kanso: reading the library before unlock promotion: {error}");
            cleanup_candidate(
                seams,
                candidate,
                completion,
                Err(LIBRARY_TROUBLE.to_owned()),
                cx,
            );
            cx.refresh_windows();
            return;
        }
    };
    let plan = kanso_sync::plan(Flow::SignIn, &library);
    let question = words::binding_question(&plan, &candidate.session.username);
    match (plan, question, window) {
        (Plan::Ask { .. }, Some(question), Some(window)) => {
            ask(window, question, cx, move |choice, cx| {
                if choice == Choice::Cancel {
                    let Some(seams) = Seams::get(cx) else {
                        clear_locked_account(cx);
                        if let Some(tx) = completion {
                            done(&tx, Err("Sign-in was cancelled.".to_owned()));
                        }
                        return;
                    };
                    cleanup_candidate(
                        seams,
                        candidate,
                        completion,
                        Err("Sign-in was cancelled.".to_owned()),
                        cx,
                    );
                    return;
                }
                promote_unlocked_candidate(candidate, Some(choice), seams, completion, cx);
            });
        }
        (Plan::Ask { .. }, _, _) => {
            // The launch restore has no window handle. Keep the durable
            // pending session and its candidate available for the Account
            // page rather than silently choosing or cancelling.
            cx.global_mut::<Services>().pending_unlocked_candidate = Some(candidate);
            if let Some(tx) = completion {
                done(&tx, Err(SOMETHING_WRONG.to_owned()));
            }
            cx.refresh_windows();
        }
        (Plan::Resume, _, _) => promote_unlocked_candidate(candidate, None, seams, completion, cx),
        (Plan::Fresh, _, _) => {
            promote_unlocked_candidate(candidate, Some(Choice::Add), seams, completion, cx)
        }
        (Plan::AskBeforeSignUp { .. }, _, _) => {
            cleanup_candidate(
                seams,
                candidate,
                completion,
                Err(SOMETHING_WRONG.to_owned()),
                cx,
            );
        }
    }
}

/// Cancels or rejects a candidate without touching the previous full item,
/// binding, recovery item, or library contents. Logout and Keychain cleanup
/// run on the account worker, never on the UI thread.
fn cleanup_candidate(
    seams: Seams,
    candidate: UnlockedCandidate,
    completion: Option<Sender<Progress>>,
    outcome: Result<(), String>,
    cx: &mut App,
) {
    let locked = cx.global::<Services>().locked_session.clone();
    let thread = seams.clone();
    let library_id = candidate.library_id.clone();
    let (internal_tx, _) = async_channel::unbounded();
    let tx = completion.unwrap_or(internal_tx);
    run(
        TaskKind::SignOut,
        Step::Contacting,
        false,
        Some(tx.clone()),
        cx,
        move |_| {
            let mut keychain_failed = false;
            let locked_cleanup = match locked {
                Some(locked) => kanso_sync::sign_out_locked(
                    thread.backend.as_ref(),
                    thread.secrets.as_ref(),
                    &locked,
                ),
                None => keychain::delete_locked_pending(thread.secrets.as_ref(), &library_id)
                    .map_err(AccountError::Keychain),
            };
            if let Err(error) = locked_cleanup {
                eprintln!("kanso: clearing locked pending session: {error}");
                keychain_failed |= matches!(&error, AccountError::Keychain(_));
            }
            if let Err(error) = kanso_sync::discard_session(
                thread.backend.as_ref(),
                thread.secrets.as_ref(),
                &candidate.session,
            ) {
                eprintln!("kanso: discarding unlocked candidate: {error}");
                keychain_failed |= matches!(&error, AccountError::Keychain(_));
            }
            keychain_failed
        },
        move |keychain_failed, cx| {
            clear_locked_account(cx);
            if keychain_failed == Some(true) {
                notify_windows(|| Notification::warning(words::SIGNED_OUT_KEYCHAIN), cx);
            }
            done(&tx, outcome);
        },
    );
}

/// A password unlock or recovery yields a fresh token; local-key and
/// approval unlocks reuse the locked token and must leave it usable if
/// promotion fails.
fn discard_fresh_candidate(
    backend: &dyn Backend,
    secrets: &dyn SecretStore,
    session: &Session,
    same_token_as_locked: bool,
) -> bool {
    if same_token_as_locked {
        return false;
    }
    match kanso_sync::discard_session(backend, secrets, session) {
        Ok(()) => false,
        Err(error) => {
            eprintln!("kanso: discarding unpromoted session: {error}");
            matches!(&error, AccountError::Keychain(_))
        }
    }
}

fn promote_unlocked_candidate(
    candidate: UnlockedCandidate,
    choice: Option<Choice>,
    seams: Seams,
    completion: Option<Sender<Progress>>,
    cx: &mut App,
) {
    let replacing = choice == Some(Choice::Replace);
    let (api, restart_api) = if replacing {
        cx.global_mut::<Services>()
            .begin_locked_replace_write_consent();
        let held = services::hold_writes(cx);
        cx.global_mut::<Services>()
            .end_locked_replace_write_consent();
        held
    } else {
        (None, false)
    };
    let session = candidate.session;
    let library_id = candidate.library_id;
    let evicted = candidate.evicted;
    let same_token_as_locked = candidate.same_token_as_locked;
    let locked = cx.global::<Services>().locked_session.clone();
    let thread = seams.clone();
    let (internal_tx, _) = async_channel::unbounded();
    let tx = completion.unwrap_or(internal_tx);
    run(
        TaskKind::SignIn,
        Step::Saving,
        false,
        Some(tx.clone()),
        cx,
        move |_| {
            if let Some(api) = api
                && let Err(error) = api.stop()
            {
                eprintln!("kanso: the API server stopped with an error: {error:#}");
            }
            let item = keychain::account_item(&library_id);
            let before = thread.secrets.get(&item).map_err(Failure::keychain)?;
            let rollback = |secrets: &dyn SecretStore| match &before {
                Some(raw) => secrets.set(&item, raw),
                None => secrets.delete(&item),
            };
            if let Err(error) =
                keychain::save_session(thread.secrets.as_ref(), &library_id, &session)
            {
                let mut failure = Failure::keychain(error);
                if let Err(cleanup) = rollback(thread.secrets.as_ref()) {
                    eprintln!(
                        "kanso: restoring the previous account item after save failure: {cleanup}"
                    );
                    failure.keychain_warning = true;
                }
                failure.keychain_warning |= discard_fresh_candidate(
                    thread.backend.as_ref(),
                    thread.secrets.as_ref(),
                    &session,
                    same_token_as_locked,
                );
                return Err(failure);
            }
            if let Err(error) =
                keychain::delete_locked_pending(thread.secrets.as_ref(), &library_id)
            {
                let mut failure = Failure::keychain(error);
                if let Err(cleanup) = rollback(thread.secrets.as_ref()) {
                    eprintln!(
                        "kanso: restoring the previous account item after pending-delete failure: {cleanup}"
                    );
                    failure.keychain_warning = true;
                }
                failure.keychain_warning |= discard_fresh_candidate(
                    thread.backend.as_ref(),
                    thread.secrets.as_ref(),
                    &session,
                    same_token_as_locked,
                );
                return Err(failure);
            }
            if let Some(choice) = choice
                && let Err(error) =
                    kanso_sync::bind(&thread.store, &session, choice, &thread.backups, thread.now)
            {
                let rollback_error = rollback(thread.secrets.as_ref()).err();
                let mut cleanup_failed = false;
                if let Some(locked) = &locked
                    && let Err(cleanup) = kanso_sync::sign_out_locked(
                        thread.backend.as_ref(),
                        thread.secrets.as_ref(),
                        locked,
                    )
                {
                    eprintln!("kanso: discarding failed binding's locked session: {cleanup}");
                    cleanup_failed |= matches!(&cleanup, AccountError::Keychain(_));
                } else if let Err(cleanup) =
                    keychain::delete_locked_pending(thread.secrets.as_ref(), &library_id)
                {
                    eprintln!("kanso: clearing failed binding's locked session: {cleanup}");
                    cleanup_failed = true;
                }
                if let Err(cleanup) = kanso_sync::discard_session(
                    thread.backend.as_ref(),
                    thread.secrets.as_ref(),
                    &session,
                ) {
                    eprintln!("kanso: discarding failed binding candidate: {cleanup}");
                    cleanup_failed |= matches!(&cleanup, AccountError::Keychain(_));
                }
                let mut failure = Failure::bind(error);
                failure.locked_cleanup = true;
                failure.keychain_warning |= cleanup_failed;
                if let Some(error) = rollback_error {
                    eprintln!(
                        "kanso: restoring the previous account item after bind failure: {error}"
                    );
                    let keychain_failure = Failure::keychain(error);
                    failure.words = keychain_failure.words;
                    failure.keychain_warning = true;
                }
                return Err(failure);
            }
            Ok((session, library_id))
        },
        move |result, cx| {
            if replacing {
                services::release_writes(restart_api, cx);
            }
            match result {
                Some(Ok((session, library_id))) => {
                    signed_in(session, library_id, true, cx);
                    if let Some(device) = evicted {
                        let notice = words::evicted_notice(&device);
                        notify_windows(move || Notification::info(notice.clone()), cx);
                    }
                    done(&tx, Ok(()));
                }
                Some(Err(failure)) => {
                    failed(&tx, Some(failure), cx);
                }
                None => {
                    failed(&tx, None, cx);
                }
            }
        },
    );
}

/// "Saved a backup of this Mac's library first", with Show in Finder.
fn notify_backup(path: PathBuf, cx: &mut App) {
    let shown = crate::settings::display_path(&path, &crate::paths::home());
    notify_windows(
        move || {
            let path = path.clone();
            Notification::success(format!(
                "Kanso saved a backup of this device's library first, in {shown}."
            ))
            .action(move |_, _, _| {
                let path = path.clone();
                Button::new("show-backup")
                    .label(crate::paths::SHOW_IN_FILE_MANAGER)
                    .on_click(move |_, _, cx| cx.reveal_path(&path))
            })
        },
        cx,
    );
}

// MARK: Signed in

/// Signs this Mac out of its account. Notes stay; sync stops until it signs
/// in again.
pub fn sign_out(cx: &mut App) -> Receiver<Progress> {
    let (tx, rx) = async_channel::unbounded();
    let Some((seams, session, _)) = begin_signed_in(&tx, cx) else {
        return rx;
    };
    run(
        TaskKind::SignOut,
        Step::Contacting,
        true,
        Some(tx.clone()),
        cx,
        move |_| kanso_sync::sign_out(seams.backend.as_ref(), seams.secrets.as_ref(), &session),
        move |result, cx| {
            // Signed out here whatever the server said: without the config's
            // account, a leftover Keychain item is never loaded again. The
            // logout was tried (or queued) either way.
            if let Some(Err(err)) = &result {
                eprintln!("kanso: signing out: {err}");
                if matches!(err, AccountError::Keychain(_)) {
                    notify_windows(|| Notification::warning(words::SIGNED_OUT_KEYCHAIN), cx);
                }
            }
            forget_account(cx);
            done(&tx, Ok(()));
        },
    );
    rx
}

/// Changes the account's password, and with it the recovery key: the
/// other Macs ask for the new password, and the Account page shows the new
/// key.
pub fn change_password(current: Password, new: Password, cx: &mut App) -> Receiver<Progress> {
    let (tx, rx) = async_channel::unbounded();
    let Some((seams, session, _)) = begin_signed_in(&tx, cx) else {
        return rx;
    };
    let shown = session.clone();
    run(
        TaskKind::ChangePassword,
        Step::DerivingKeys,
        false,
        Some(tx.clone()),
        cx,
        move |step| {
            // The new key is in the Keychain before the request goes.
            kanso_sync::change_password(
                seams.backend.as_ref(),
                seams.secrets.as_ref(),
                &session,
                &current,
                &new,
                step,
            )
            .map_err(|err| {
                Failure::rotation(
                    err,
                    ErrorContext::CurrentPassword,
                    words::KEY_UNCONFIRMED_PASSWORD,
                )
            })
        },
        move |result, cx| match result {
            Some(Ok(changed)) => {
                show_pending(
                    PendingKey {
                        key: changed.recovery_key,
                        username: shown.username.clone(),
                        server: shown.server.clone(),
                        note: Some(words::KEY_AFTER_PASSWORD),
                    },
                    cx,
                );
                notify_windows(|| Notification::success(words::PASSWORD_CHANGED), cx);
                done(&tx, Ok(()));
            }
            Some(Err(failure)) => failed(&tx, Some(failure), cx),
            None => failed(&tx, None, cx),
        },
    );
    rx
}

/// Replaces the recovery key; the Account page then shows the new one.
pub fn new_recovery_key(current: Password, cx: &mut App) -> Receiver<Progress> {
    let (tx, rx) = async_channel::unbounded();
    let Some((seams, session, _)) = begin_signed_in(&tx, cx) else {
        return rx;
    };
    let shown = session.clone();
    run(
        TaskKind::NewRecoveryKey,
        Step::DerivingKeys,
        false,
        Some(tx.clone()),
        cx,
        move |step| {
            // The new key is in the Keychain before the request goes.
            kanso_sync::new_recovery_key(
                seams.backend.as_ref(),
                seams.secrets.as_ref(),
                &session,
                &current,
                step,
            )
            .map_err(|err| {
                Failure::rotation(err, ErrorContext::CurrentPassword, words::KEY_UNCONFIRMED)
            })
        },
        move |result, cx| match result {
            Some(Ok(key)) => {
                show_key(&shown, key, cx);
                done(&tx, Ok(()));
            }
            Some(Err(failure)) => failed(&tx, Some(failure), cx),
            None => failed(&tx, None, cx),
        },
    );
    rx
}

/// Deletes the account and its server data, then unbinds the library.
/// Notes on this Mac stay.
pub fn delete_account(password: Password, typed: String, cx: &mut App) -> Receiver<Progress> {
    let (tx, rx) = async_channel::unbounded();
    let Some((seams, session, library_id)) = begin_signed_in(&tx, cx) else {
        return rx;
    };
    let username = session.username.clone();
    run(
        TaskKind::DeleteAccount,
        Step::DerivingKeys,
        true,
        Some(tx.clone()),
        cx,
        move |step| {
            kanso_sync::delete_account(seams.backend.as_ref(), &session, &password, &typed, step)
                .map_err(|err| Failure::account(err, ErrorContext::CurrentPassword))?;
            step(Step::Saving);
            kanso_sync::unbind(&seams.store, seams.secrets.as_ref()).map_err(Failure::bind)?;
            keychain::delete_recovery_key(seams.secrets.as_ref(), &library_id).ok();
            Ok(())
        },
        move |result, cx| match result {
            Some(Ok(())) => {
                forget_account(cx);
                notify_windows(
                    move || {
                        Notification::info(format!(
                            "Deleted {username}'s account. The notes on this device stay here."
                        ))
                    },
                    cx,
                );
                done(&tx, Ok(()));
            }
            Some(Err(failure)) => failed(&tx, Some(failure), cx),
            None => failed(&tx, None, cx),
        },
    );
    rx
}

/// Signs another device of the account out, and says how it went: nothing
/// else waits on it once its dialog closes (A5). A device the server no
/// longer knows is signed out already.
pub fn revoke_device(device_id: String, name: String, cx: &mut App) -> Receiver<Progress> {
    let (tx, rx) = async_channel::unbounded();
    let Some((seams, session, _)) = begin_signed_in(&tx, cx) else {
        if let Ok(Progress::Done(Err(words))) = rx.try_recv() {
            let told = words.clone();
            notify_windows(move || Notification::error(told.clone()), cx);
            done(&tx, Err(words));
        }
        return rx;
    };
    run(
        TaskKind::RevokeDevice,
        Step::Contacting,
        false,
        Some(tx.clone()),
        cx,
        move |_| match kanso_sync::revoke_device(seams.backend.as_ref(), &session, &device_id) {
            Ok(()) | Err(AccountError::NotFound) => Ok(()),
            Err(err) => Err(Failure::account(err, ErrorContext::Account)),
        },
        move |result, cx| {
            match result {
                Some(Ok(())) => {
                    let notice = words::revoked_notice(&name);
                    notify_windows(move || Notification::success(notice.clone()), cx);
                    done(&tx, Ok(()));
                }
                failure => {
                    let failure = failure.and_then(Result::err);
                    let told = failure
                        .as_ref()
                        .map_or(SOMETHING_WRONG.to_owned(), |failure| failure.words.clone());
                    notify_windows(move || Notification::error(told.clone()), cx);
                    failed(&tx, failure, cx);
                }
            }
            refresh_details(cx);
        },
    );
    rx
}

/// "I've saved my recovery key": the key screen and its Keychain item go,
/// unless the item holds another key by then.
pub fn key_saved(cx: &mut App) {
    let services = cx.global_mut::<Services>();
    let saved = services.pending_key.take();
    let library_id = services.library_id.clone();
    if let (Some(saved), Some(seams)) = (saved, Seams::get(cx)) {
        cx.background_spawn(async move {
            // Before the launch check, as after a failed sign-up in tests.
            let library_id = match library_id {
                Some(library_id) => library_id,
                None => match seams.device() {
                    Ok(device) => device.library_id,
                    Err(_) => return,
                },
            };
            let secrets = seams.secrets.as_ref();
            let removed =
                keychain::load_any_recovery_key(secrets, &library_id).and_then(|kept| match kept {
                    Some(kept) if kept.recovery_key == saved.key => {
                        keychain::delete_recovery_key(secrets, &library_id)
                    }
                    _ => Ok(()),
                });
            if let Err(err) = removed {
                eprintln!("kanso: removing the saved recovery key: {err}");
            }
        })
        .detach();
    }
    cx.refresh_windows();
}

// MARK: Servers

/// The server the forms use: `None` for the default.
pub fn set_server(server: Option<String>, cx: &mut App) {
    let server = server.filter(|server| !is_default_server(server));
    AppState::update_config(cx, |config| config.sync.server = server);
    {
        let services = cx.global_mut::<Services>();
        if let Some(wait) = services.provider_wait.take() {
            wait.cancel.store(true, Ordering::Release);
        }
        services.auth_generation = services.auth_generation.wrapping_add(1);
        services.auth = None;
        services.auth_asking = None;
        services.auth_attempted = None;
        services.signup = None;
        services.provider_signup = None;
    }
    check_signup(cx);
    check_auth(cx);
    cx.refresh_windows();
}

/// Requests cancellation of the active provider flow. Its completion callback
/// owns clearing the visible wait state.
pub fn cancel_provider_wait(cx: &mut App) {
    if let Some(wait) = cx.global::<Services>().provider_wait.as_ref() {
        wait.cancel.store(true, Ordering::Release);
    }
}

/// "Use a different server…": the address must be a Kanso server, which
/// says whether it takes new accounts.
pub fn use_server(input: String, cx: &mut App) -> Receiver<Progress> {
    let (tx, rx) = async_channel::unbounded();
    let endpoint = match kanso_sync::check_endpoint(&input) {
        Ok(endpoint) => endpoint,
        Err(err) => {
            done(&tx, Err(words::endpoint_words(&err).to_owned()));
            return rx;
        }
    };
    let Some(seams) = begin(&tx, cx) else {
        return rx;
    };
    let generation = cx.global::<Services>().auth_generation;
    run(
        TaskKind::CheckServer,
        Step::Contacting,
        false,
        Some(tx.clone()),
        cx,
        move |_| {
            kanso_sync::server_info(seams.backend.as_ref(), &endpoint)
                .map(|signup| (endpoint, signup))
                .map_err(|err| Failure::account(err, ErrorContext::Server))
        },
        move |result, cx| match result {
            Some(Ok((endpoint, signup))) => {
                if cx.global::<Services>().auth_generation != generation {
                    done(
                        &tx,
                        Err("The server changed while it was being checked. Try again.".into()),
                    );
                    return;
                }
                set_server(Some(endpoint.clone()), cx);
                {
                    let services = cx.global_mut::<Services>();
                    services.provider_signup = None;
                    services.signup = Some((endpoint.clone(), signup));
                }
                done(&tx, Ok(()));
            }
            Some(Err(failure)) => failed(&tx, Some(failure), cx),
            None => failed(&tx, None, cx),
        },
    );
    rx
}

/// The server the forms use, as a URL.
pub fn form_server(cx: &App) -> String {
    cx.global::<AppState>()
        .config
        .sync
        .server
        .clone()
        .unwrap_or_else(|| kanso_sync::DEFAULT_SERVER.to_owned())
}

/// Asks the forms' server whether it takes new accounts, for the Create
/// account form: again each time the form opens, since a server's answer
/// changes as it fills (A6). One question per server at a time; an answer
/// is kept only if the form still shows that server.
pub fn check_signup(cx: &mut App) {
    let server = form_server(cx);
    let services = cx.global_mut::<Services>();
    let Some(backend) = services.backend.clone() else {
        return;
    };
    if services.signup_asking.as_deref() == Some(server.as_str()) {
        return;
    }
    services.signup_asking = Some(server.clone());
    let (tx, rx) = async_channel::bounded(1);
    let asked = server.clone();
    let spawned = std::thread::Builder::new()
        .name("kanso-account".into())
        .spawn(move || {
            tx.try_send(kanso_sync::server_info(backend.as_ref(), &asked))
                .ok();
        });
    if spawned.is_err() {
        cx.global_mut::<Services>().signup_asking = None;
        return;
    }
    cx.spawn(async move |cx| {
        let answer = rx.recv().await.ok();
        cx.update(|cx| {
            let shown = form_server(cx);
            let services = cx.global_mut::<Services>();
            if services.signup_asking.as_deref() == Some(server.as_str()) {
                services.signup_asking = None;
            }
            if let Some(Ok(signup)) = answer
                && shown == server
            {
                services.signup = Some((server, signup));
            }
            cx.refresh_windows();
        });
    })
    .detach();
}

/// Fetches the current server's provider advertisement while a signed-out
/// form is open. Answers are tagged with their server so an older request
/// cannot replace the methods for a newer server. An omitted `auth` is the
/// old-server password-only behavior; a health error likewise leaves the
/// password form available.
pub fn check_auth(cx: &mut App) {
    let server = form_server(cx);
    let services = cx.global_mut::<Services>();
    let generation = services.auth_generation;
    let request = (server.clone(), generation);
    if services.auth_asking.as_ref() == Some(&request)
        || services.auth_attempted.as_ref() == Some(&request)
    {
        return;
    }
    let Some(backend) = services.backend.clone() else {
        services.auth = Some((server, kanso_sync::ServerAuth::default()));
        services.auth_attempted = Some(request);
        return;
    };
    services.auth_asking = Some(request.clone());
    services.auth_attempted = Some(request.clone());
    let (tx, rx) = async_channel::bounded(1);
    let asked = server.clone();
    let spawned = std::thread::Builder::new()
        .name("kanso-account".into())
        .spawn(move || {
            tx.try_send(kanso_sync::server_auth(backend.as_ref(), &asked))
                .ok();
        });
    if spawned.is_err() {
        let services = cx.global_mut::<Services>();
        if services.auth_asking.as_ref() == Some(&request) {
            services.auth_asking = None;
        }
        return;
    }
    cx.spawn(async move |cx| {
        let answer = rx.recv().await.ok();
        cx.update(|cx| {
            let shown = form_server(cx);
            let visible = account_page_showing(cx) && !cx.global::<Services>().signed_in();
            let services = cx.global_mut::<Services>();
            if services.auth_asking.as_ref() == Some(&request) {
                services.auth_asking = None;
            }
            if shown == server
                && visible
                && auth_answer_is_current(
                    &server,
                    generation,
                    &shown,
                    services.auth_generation,
                    services.auth_attempted.as_ref(),
                    &request,
                )
            {
                let auth = match answer {
                    Some(Ok(auth)) => auth,
                    _ => kanso_sync::ServerAuth::default(),
                };
                services.auth = Some((server, auth));
            }
            cx.refresh_windows();
        });
    })
    .detach();
}

pub(crate) fn auth_answer_is_current(
    server: &str,
    generation: u64,
    shown: &str,
    current_generation: u64,
    attempted: Option<&(String, u64)>,
    request: &(String, u64),
) -> bool {
    shown == server
        && current_generation == generation
        && attempted == Some(request)
        && request == &(server.to_owned(), generation)
}

/// Starts a fresh health request for a newly opened signed-out form.
pub fn refresh_auth(cx: &mut App) {
    let services = cx.global_mut::<Services>();
    services.auth_generation = services.auth_generation.wrapping_add(1);
    services.auth = None;
    services.auth_asking = None;
    services.auth_attempted = None;
    services.signup = None;
    services.provider_signup = None;
    check_auth(cx);
}

// MARK: Storage and devices

/// Fetches the account's details again after a sync that changed what the
/// server holds. A fetch already running may have been answered before the
/// sync's changes landed, so it is followed by another.
pub fn refresh_details_after_sync(cx: &mut App) {
    let details = &mut cx.global_mut::<Services>().details;
    if details.loading {
        details.stale = true;
        return;
    }
    refresh_details(cx);
}

/// Fetches the account's storage and devices for the Account page.
pub fn refresh_details(cx: &mut App) {
    let services = cx.global_mut::<Services>();
    let (Some(session), Some(backend)) = (services.session(), services.backend.clone()) else {
        return;
    };
    if services.details.loading {
        return;
    }
    services.details.loading = true;
    let (tx, rx) = async_channel::bounded(1);
    let asked = session.clone();
    let spawned = std::thread::Builder::new()
        .name("kanso-account".into())
        .spawn(move || {
            let usage = kanso_sync::usage(backend.as_ref(), &asked);
            let devices = kanso_sync::devices(backend.as_ref(), &asked);
            tx.try_send((usage, devices)).ok();
        });
    if spawned.is_err() {
        cx.global_mut::<Services>().details.loading = false;
        return;
    }
    cx.spawn(async move |cx| {
        let answer = rx.recv().await.ok();
        cx.update(|cx| {
            let services = cx.global_mut::<Services>();
            let current = services
                .session()
                .is_some_and(|now| Arc::ptr_eq(&now, &session));
            if !current {
                return;
            }
            let stale = services.details.stale;
            let Some((usage, devices)) = answer else {
                services.details.loading = false;
                services.details.stale = false;
                services.details.failed = Some(SOMETHING_WRONG.to_owned());
                cx.refresh_windows();
                return;
            };
            let ended = [usage.as_ref().err(), devices.as_ref().err()]
                .into_iter()
                .flatten()
                .find_map(|err| match err {
                    AccountError::Unauthorized { reason } => Some(*reason),
                    _ => None,
                });
            if let Some(reason) = ended {
                services::end_session(reason, cx);
                return;
            }
            let failed = [usage.as_ref().err(), devices.as_ref().err()]
                .into_iter()
                .flatten()
                .next()
                .map(|err| words::error_words(err, ErrorContext::Account));
            let mut details = AccountDetails {
                usage: usage.ok(),
                devices: devices.ok(),
                loading: false,
                stale: false,
                failed,
            };
            if let Some(devices) = &mut details.devices {
                words::sort_devices(devices);
            }
            services.details = details;
            cx.refresh_windows();
            if stale {
                refresh_details(cx);
            }
        });
    })
    .detach();
}

/// Fetches the current account's ways in for Settings › Account. Every page
/// opening starts a new generation, so an older answer cannot replace it.
/// Only an unlocked `SignedIn` session is eligible; provider-only locked
/// sessions never reach this endpoint through the app.
pub fn refresh_account_methods(cx: &mut App) {
    let services = cx.global_mut::<Services>();
    if !services.signed_in() || services.has_locked_session() {
        return;
    }
    let (Some(session), Some(backend)) = (services.session(), services.backend.clone()) else {
        return;
    };
    let generation = services.account_methods.begin_refresh();

    let (tx, rx) = async_channel::bounded(1);
    let asked = session.clone();
    let spawned = std::thread::Builder::new()
        .name("kanso-account".into())
        .spawn(move || {
            let methods = kanso_sync::account::methods(backend.as_ref(), &asked);
            let passkeys = kanso_sync::server_auth(backend.as_ref(), &asked.server)
                .ok()
                .and_then(|auth| auth.passkeys);
            tx.try_send((methods, passkeys)).ok();
        });
    if spawned.is_err() {
        cx.global_mut::<Services>()
            .account_methods
            .finish_failure(generation, SOMETHING_WRONG.to_owned());
        cx.refresh_windows();
        return;
    }
    cx.spawn(async move |cx| {
        let answer = rx.recv().await.ok();
        cx.update(|cx| {
            let services = cx.global_mut::<Services>();
            let current_session = services
                .session()
                .is_some_and(|now| Arc::ptr_eq(&now, &session));
            if !current_session {
                return;
            }
            let Some((answer, passkeys)) = answer else {
                services
                    .account_methods
                    .finish_failure(generation, SOMETHING_WRONG.to_owned());
                cx.refresh_windows();
                return;
            };
            match answer {
                Ok(methods) => {
                    if !services.account_methods.finish_success(generation, methods) {
                        return;
                    }
                    services.account_methods.passkeys = passkeys;
                    cx.refresh_windows();
                }
                Err(AccountError::Unauthorized { reason }) => {
                    if services.account_methods.generation != generation {
                        return;
                    }
                    services::end_session(reason, cx);
                }
                Err(err) => {
                    services.account_methods.finish_failure(
                        generation,
                        words::error_words(&err, ErrorContext::Account),
                    );
                    cx.refresh_windows();
                }
            }
        });
    })
    .detach();
}

// MARK: Launch

/// At launch: signed in only when the config's account, the library's
/// binding and the Keychain item agree (security re-check 3). An unsaved
/// recovery key shows whatever the account. Account tasks wait while it
/// reads (`restoring`), and its answer is dropped if a session appeared or
/// the config's account changed meanwhile (A4). Pending logouts are retried
/// after the answer is sent.
pub fn restore(cx: &mut App) {
    let Some(seams) = Seams::get(cx) else {
        return;
    };
    let account = cx.global::<AppState>().config.sync.account.clone();
    let asked = account.clone();
    let (tx, rx) = async_channel::bounded(1);
    let spawned = std::thread::Builder::new()
        .name("kanso-account".into())
        .spawn(move || {
            let device = match seams.device() {
                Ok(device) => device,
                Err(failure) => {
                    eprintln!("kanso: this device's identity: {}", failure.words);
                    tx.try_send(None).ok();
                    return;
                }
            };
            // An unsaved recovery key shows whatever this Mac is signed in
            // to: a cancelled recovery's, or one whose answer was lost.
            let kept = keychain::load_any_recovery_key(seams.secrets.as_ref(), &device.library_id)
                .unwrap_or_else(|err| {
                    eprintln!("kanso: reading the unsaved recovery key: {err}");
                    None
                });
            let loaded = match &account {
                Some(account) => {
                    let bound = |key| seams.store.meta(key).ok().flatten();
                    let agree = bound(meta_keys::ORIGIN).as_deref()
                        == Some(account.origin.as_str())
                        && bound(meta_keys::ACCOUNT_ID).as_deref() == Some(account.id.as_str());
                    if agree {
                        keychain::load_session(
                            seams.secrets.as_ref(),
                            &device.library_id,
                            &account.origin,
                            &account.id,
                        )
                    } else {
                        Ok(None)
                    }
                }
                None => Ok(None),
            };
            // The locked provider flow is keyed only by this library ID;
            // it deliberately does not depend on config's account or the
            // current library binding.
            let mut pending =
                keychain::load_locked_pending(seams.secrets.as_ref(), &device.library_id);
            let local = match &mut pending {
                Ok(Some(locked)) => {
                    let previous = locked.clone();
                    if kanso_sync::refresh_locked_methods(seams.backend.as_ref(), locked).is_ok()
                        && let Err(error) = keychain::update_locked_pending(
                            seams.secrets.as_ref(),
                            &device.library_id,
                            locked,
                        )
                    {
                        eprintln!("kanso: saving refreshed locked sign-in methods: {error}");
                        *locked = previous;
                    }
                    keychain::load_session(
                        seams.secrets.as_ref(),
                        &device.library_id,
                        &locked.origin,
                        &locked.account_id,
                    )
                }
                _ => Ok(None),
            };
            tx.try_send(Some((
                device.library_id.clone(),
                loaded,
                kept,
                pending,
                local,
            )))
            .ok();
            // The session is in; old tokens can take their time.
            kanso_sync::flush_pending_logouts(
                seams.backend.as_ref(),
                seams.secrets.as_ref(),
                &device.library_id,
            );
        });
    if spawned.is_err() {
        return;
    }
    cx.global_mut::<Services>().restoring = true;
    cx.spawn(async move |cx| {
        let answer = rx.recv().await.ok().flatten();
        cx.update(|cx| {
            cx.global_mut::<Services>().restoring = false;
            let Some((library_id, loaded, kept, pending, local)) = answer else {
                cx.refresh_windows();
                return;
            };
            let services = cx.global_mut::<Services>();
            services.library_id = Some(library_id.clone());
            if let Some(item) = kept
                && services.pending_key.is_none()
            {
                show_pending(PendingKey::kept(item, words::KEY_UNCONFIRMED), cx);
            }
            // Something signed in or out meanwhile: its state stands.
            let services = cx.global::<Services>();
            let moved = services.session().is_some()
                || services.task.is_some()
                || cx.global::<AppState>().config.sync.account != asked;
            if moved {
                cx.refresh_windows();
                return;
            }
            match loaded {
                Ok(Some(session)) => {
                    let services = cx.global_mut::<Services>();
                    services.set_signed_in(Arc::new(session));
                    services::apply_sync_settings(cx);
                    refresh_details(cx);
                    crate::device_approval::start_events(cx);
                    crate::device_approval::refresh(cx);
                }
                // The three don't agree: signed out, and the config says so.
                Ok(None) => {
                    if !matches!(&pending, Ok(Some(_)))
                        && cx.global::<AppState>().config.sync.account.is_some()
                    {
                        AppState::update_config(cx, |config| config.sync.account = None);
                    }
                }
                // The Keychain wouldn't say (locked): signed out for now,
                // with the account kept for the next launch.
                Err(err) => eprintln!("kanso: reading the account from the Keychain: {err}"),
            }
            // A valid full session above wins. Otherwise resume the exact
            // provider token/approval persisted for this library, whether
            // or not the account config currently names it.
            if !cx.global::<Services>().signed_in() {
                match pending {
                    Ok(Some(locked)) => {
                        let services = cx.global_mut::<Services>();
                        services.locked_has_password = Some(locked.has_password);
                        services.locked_session = Some(Arc::new(locked.clone()));
                        services.account = AccountState::SignedInLocked {
                            username: locked.username.clone(),
                        };
                        let unlocked = match local {
                            Ok(Some(local)) => match unlock_with_local_key(&locked, &local) {
                                Ok(unlocked) => Some(UnlockedCandidate {
                                    session: unlocked.session,
                                    locked_session: locked.clone(),
                                    evicted: unlocked.evicted,
                                    library_id: library_id.clone(),
                    same_token_as_locked: true,
                                }),
                                Err(error) => {
                                    eprintln!("kanso: saved local key doesn't unlock pending sign-in: {error}");
                                    None
                                }
                            },
                            Ok(None) => None,
                            Err(error) => {
                                eprintln!("kanso: reading the saved local session: {error}");
                                None
                            }
                        };
                        if let Some(candidate) = unlocked {
                            finish_unlocked_candidate(candidate, None, None, cx);
                        } else if locked.approval_resume().is_some()
                            && let Some(seams) = Seams::get(cx)
                            && let Ok(device) = seams.device()
                        {
                            let window = cx.active_window();
                            wait_for_device_approval(locked, device, window, cx);
                        }
                    }
                    Ok(None) => {}
                    Err(err) => {
                        eprintln!("kanso: reading locked pending session: {err}");
                        notify_windows(|| Notification::warning(words::SIGNED_OUT_KEYCHAIN), cx);
                    }
                }
            }
            cx.refresh_windows();
        });
    })
    .detach();
}

// MARK: Device approval

enum WaitUpdate {
    OtherDevices(usize),
    Request(String),
    Code(String),
    Tick,
    Done(Box<Result<UnlockedCandidate, WaitEnd>>),
}

enum WaitEnd {
    Cancelled,
    Expired,
    NoDevices,
    Failed(Failure),
}

impl WaitEnd {
    fn phase(&self) -> crate::device_approval::WaitPhase {
        match self {
            WaitEnd::Cancelled => crate::device_approval::WaitPhase::Cancelled,
            WaitEnd::Expired => crate::device_approval::WaitPhase::Expired,
            WaitEnd::NoDevices => crate::device_approval::WaitPhase::NoDevices,
            WaitEnd::Failed(failure) => {
                crate::device_approval::WaitPhase::Failed(failure.words.clone())
            }
        }
    }
}

fn approval_failure(error: AccountError) -> WaitEnd {
    let must_sign_out = matches!(
        &error,
        AccountError::Unauthorized { .. }
            | AccountError::KeyMismatch
            | AccountError::ApprovalChanged
            | AccountError::ApprovalRequestIdConflict
            | AccountError::Protocol(_)
    );
    let mut failure = Failure::account(error, ErrorContext::Account);
    failure.locked_cleanup = must_sign_out;
    WaitEnd::Failed(failure)
}

fn clear_approval_resume(
    secrets: &dyn SecretStore,
    locked: &LockedSession,
) -> Result<(), AccountError> {
    let library_id = locked.library_id().ok_or_else(|| {
        AccountError::Protocol("the locked session has no library ID for Keychain storage".into())
    })?;
    keychain::clear_pending_approval(secrets, library_id).map_err(AccountError::Keychain)
}

/// Invalid or ended locked credentials cannot remain available to a later
/// action. Clear the app state even if the Keychain refuses deletion; the
/// sync API still tries logout and queues it when possible.
fn end_invalid_locked_session(locked: LockedSession, failure: Failure, cx: &mut App) {
    let Some(seams) = Seams::get(cx) else {
        clear_locked_account(cx);
        return;
    };
    let thread = seams.clone();
    run(
        TaskKind::SignOut,
        Step::Contacting,
        false,
        None,
        cx,
        move |_| {
            kanso_sync::sign_out_locked(thread.backend.as_ref(), thread.secrets.as_ref(), &locked)
                .map_err(|error| Failure::account(error, ErrorContext::Account))
        },
        move |result, cx| {
            let keychain_failed = failure.keychain_warning
                || matches!(
                    &result,
                    Some(Err(Failure {
                        keychain_warning: true,
                        ..
                    }))
                );
            if let Some(Err(error)) = result {
                eprintln!("kanso: ending invalid locked sign-in: {}", error.words);
            }
            clear_locked_account(cx);
            if keychain_failed {
                notify_windows(|| Notification::warning(words::SIGNED_OUT_KEYCHAIN), cx);
            }
        },
    );
}

/// Requests or resumes approval. The worker only creates, polls, names and
/// unlocks; binding decisions and promotion return to the UI thread.
pub fn wait_for_device_approval(
    locked: LockedSession,
    device: DeviceIdentity,
    window: Option<AnyWindowHandle>,
    cx: &mut App,
) {
    if cx
        .try_global::<Services>()
        .is_some_and(|s| s.approval.waiting.is_some())
    {
        return;
    }
    let started = Clock::now_ms(cx);
    cx.global_mut::<Services>().approval.waiting =
        Some(crate::device_approval::WaitingState::requesting(started));
    let Some(seams) = Seams::get(cx) else {
        cx.global_mut::<Services>().approval.waiting = None;
        return;
    };
    let (tx, rx) = async_channel::unbounded();
    let backend = seams.backend.clone();
    let secrets = seams.secrets.clone();
    let locked_for_cleanup = locked.clone();
    let cancel = cx
        .global::<Services>()
        .approval
        .waiting
        .as_ref()
        .map(|w| w.cancel.clone());
    let thread = std::thread::Builder::new()
        .name("kanso-account".into())
        .spawn(move || {
            let result = (|| {
                let request = if let Some(resume) = locked.approval_resume() {
                    let request = match resume_approval(backend.as_ref(), secrets.as_ref(), &locked)
                    {
                        Ok(request) => request,
                        Err(
                            AccountError::ApprovalExpired
                            | AccountError::ApprovalUsed
                            | AccountError::NotFound,
                        ) => {
                            clear_approval_resume(secrets.as_ref(), &locked)
                                .map_err(approval_failure)?;
                            return Err(WaitEnd::Expired);
                        }
                        Err(error) => return Err(approval_failure(error)),
                    };
                    if request.id != resume.request_id() {
                        return Err(approval_failure(AccountError::ApprovalChanged));
                    }
                    request
                } else {
                    let keys = ApprovalKeys::generate();
                    request_approval_resumable(
                        backend.as_ref(),
                        secrets.as_ref(),
                        &locked,
                        &new_approval_request_id(),
                        &keys,
                    )
                    .map_err(approval_failure)?
                };
                if request.other_devices == 0 {
                    cancel_saved_approval(backend.as_ref(), secrets.as_ref(), &locked)
                        .map_err(approval_failure)?;
                    return Err(WaitEnd::NoDevices);
                }
                tx.try_send(WaitUpdate::OtherDevices(request.other_devices))
                    .ok();
                tx.try_send(WaitUpdate::Request(request.id.clone())).ok();
                let mut approval: Option<Approval> = None;
                loop {
                    if cancel
                        .as_ref()
                        .is_some_and(|flag| flag.load(std::sync::atomic::Ordering::SeqCst))
                    {
                        cancel_saved_approval(backend.as_ref(), secrets.as_ref(), &locked)
                            .map_err(approval_failure)?;
                        return Err(WaitEnd::Cancelled);
                    }
                    match poll_saved_approval(backend.as_ref(), secrets.as_ref(), &locked) {
                        Ok(RequestState::Waiting) => {}
                        Ok(RequestState::Challenged { approver_epk, .. }) => {
                            let completed = complete_saved_request(&locked, &approver_epk)
                                .map_err(|e| {
                                    WaitEnd::Failed(Failure::account(e, ErrorContext::Account))
                                })?;
                            let code = completed.code();
                            if let Err(error) = name_device_resumable(
                                backend.as_ref(),
                                secrets.as_ref(),
                                &locked,
                                &device,
                            ) {
                                match error {
                                    AccountError::ApprovalExpired
                                    | AccountError::ApprovalUsed
                                    | AccountError::NotFound => {
                                        clear_approval_resume(secrets.as_ref(), &locked)
                                            .map_err(approval_failure)?;
                                        return Err(WaitEnd::Expired);
                                    }
                                    other => return Err(approval_failure(other)),
                                }
                            }
                            tx.try_send(WaitUpdate::Code(code)).ok();
                            approval = Some(completed);
                        }
                        Ok(RequestState::Approved(approved)) => {
                            let epk = locked
                                .approval_resume()
                                .and_then(|r| r.approver_epk())
                                .ok_or_else(|| {
                                    WaitEnd::Failed(Failure::account(
                                        AccountError::ApprovalChanged,
                                        ErrorContext::Account,
                                    ))
                                })?;
                            let approval = match approval.take() {
                                Some(approval) => approval,
                                None => complete_saved_request(&locked, &epk)
                                    .map_err(approval_failure)?,
                            };
                            let unlocked = finish_unlock(
                                backend.as_ref(),
                                &locked,
                                &approval,
                                &approved,
                                &device,
                            )
                            .map_err(approval_failure)?;
                            return Ok(UnlockedCandidate {
                                session: unlocked.session,
                                locked_session: locked.clone(),
                                evicted: unlocked.evicted,
                                library_id: device.library_id.clone(),
                                same_token_as_locked: true,
                            });
                        }
                        Ok(RequestState::Cancelled) => {
                            clear_approval_resume(secrets.as_ref(), &locked)
                                .map_err(approval_failure)?;
                            return Err(WaitEnd::Cancelled);
                        }
                        Ok(RequestState::Expired) => {
                            clear_approval_resume(secrets.as_ref(), &locked)
                                .map_err(approval_failure)?;
                            return Err(WaitEnd::Expired);
                        }
                        Err(
                            AccountError::ApprovalExpired
                            | AccountError::ApprovalUsed
                            | AccountError::NotFound,
                        ) => {
                            clear_approval_resume(secrets.as_ref(), &locked)
                                .map_err(approval_failure)?;
                            return Err(WaitEnd::Expired);
                        }
                        Err(
                            error @ (AccountError::Unauthorized { .. }
                            | AccountError::KeyMismatch
                            | AccountError::ApprovalChanged
                            | AccountError::ApprovalRequestIdConflict
                            | AccountError::Protocol(_)),
                        ) => return Err(approval_failure(error)),
                        Err(_) => {}
                    }
                    tx.try_send(WaitUpdate::Tick).ok();
                    std::thread::sleep(crate::device_approval::POLL_EVERY);
                }
            })();
            tx.try_send(WaitUpdate::Done(Box::new(result))).ok();
        });
    if thread.is_err() {
        cx.global_mut::<Services>().approval.waiting = None;
        return;
    }
    cx.spawn(async move |cx| {
        while let Ok(update) = rx.recv().await {
            let mut completed = None;
            cx.update(|cx| {
                let now = Clock::now_ms(cx);
                let services = cx.global_mut::<Services>();
                let Some(waiting) = services.approval.waiting.as_mut() else {
                    return;
                };
                match update {
                    WaitUpdate::OtherDevices(count) => {
                        waiting.know_devices(count, now);
                        waiting.phase = if count == 0 {
                            crate::device_approval::WaitPhase::NoDevices
                        } else {
                            crate::device_approval::WaitPhase::Waiting
                        };
                    }
                    WaitUpdate::Request(id) => waiting.request_id = Some(id),
                    WaitUpdate::Code(code) => {
                        waiting.code = Some(code.clone());
                        waiting.phase = crate::device_approval::WaitPhase::Compare { code };
                    }
                    WaitUpdate::Tick => waiting.offer_recovery(now),
                    WaitUpdate::Done(result) => completed = Some(*result),
                }
                cx.refresh_windows();
            });
            if let Some(result) = completed {
                match result {
                    Ok(candidate) => cx.update(|cx| {
                        cx.global_mut::<Services>().approval.waiting = None;
                        finish_unlocked_candidate(candidate, window, None, cx);
                    }),
                    Err(
                        end @ WaitEnd::Failed(Failure {
                            locked_cleanup: true,
                            ..
                        }),
                    ) => cx.update(|cx| {
                        if let WaitEnd::Failed(failure) = end {
                            end_invalid_locked_session(locked_for_cleanup, failure, cx);
                        }
                    }),
                    Err(end) => cx.update(|cx| {
                        if let Some(waiting) = cx.global_mut::<Services>().approval.waiting.as_mut()
                        {
                            if matches!(
                                &end,
                                WaitEnd::Cancelled | WaitEnd::Expired | WaitEnd::NoDevices
                            ) {
                                waiting.code = None;
                            }
                            waiting.phase = end.phase();
                            waiting.recovery_offered = true;
                            if let WaitEnd::Failed(failure) = end {
                                waiting.phase =
                                    crate::device_approval::WaitPhase::Failed(failure.words);
                            }
                        }
                        cx.refresh_windows();
                    }),
                }
                break;
            }
        }
    })
    .detach();
    cx.refresh_windows();
}

pub fn restart_device_approval(window: Option<AnyWindowHandle>, cx: &mut App) {
    let Some(seams) = Seams::get(cx) else {
        return;
    };
    let Ok(device) = seams.device() else {
        return;
    };
    let locked = match keychain::load_locked_pending(seams.secrets.as_ref(), &device.library_id) {
        Ok(Some(locked)) => locked,
        Ok(None) => {
            notify_windows(|| Notification::warning(words::SIGNED_OUT_KEYCHAIN), cx);
            return;
        }
        Err(error) => {
            notify_windows(|| Notification::warning(words::SIGNED_OUT_KEYCHAIN), cx);
            eprintln!("kanso: reloading locked pending session: {error}");
            return;
        }
    };
    {
        let services = cx.global_mut::<Services>();
        services.locked_has_password = Some(locked.has_password);
        services.locked_session = Some(Arc::new(locked.clone()));
        services.account = AccountState::SignedInLocked {
            username: locked.username.clone(),
        };
    }
    cx.global_mut::<Services>().approval.waiting = None;
    wait_for_device_approval(locked, device, window, cx);
}

/// Cancels a locked sign-in. `discard_locked_session` attempts both pending
/// deletion and logout even when deletion fails; the app state is cleared
/// either way so a Keychain error cannot leave the editor pretending to be
/// an active approval session.
pub fn cancel_locked_session(cx: &mut App) {
    let Some(locked) = cx.global::<Services>().locked_session.clone() else {
        return;
    };
    if let Some(waiting) = cx.global::<Services>().approval.waiting.as_ref() {
        waiting
            .cancel
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }
    let (tx, _rx) = async_channel::unbounded();
    let Some(seams) = begin(&tx, cx) else {
        return;
    };
    let thread = seams.clone();
    run(
        TaskKind::SignOut,
        Step::Contacting,
        false,
        Some(tx.clone()),
        cx,
        move |_| {
            let approval = locked
                .approval_resume()
                .map(|_| {
                    cancel_saved_approval(thread.backend.as_ref(), thread.secrets.as_ref(), &locked)
                })
                .unwrap_or(Ok(()));
            let sign_out = kanso_sync::discard_locked_session(
                thread.backend.as_ref(),
                thread.secrets.as_ref(),
                &locked,
            );
            sign_out.and(approval)
        },
        move |result, cx| {
            let keychain_failed = matches!(&result, Some(Err(AccountError::Keychain(_))));
            if let Some(Err(error)) = result.as_ref() {
                eprintln!("kanso: cancelling locked sign-in: {error}");
            }
            clear_locked_account(cx);
            if keychain_failed {
                notify_windows(|| Notification::warning(words::SIGNED_OUT_KEYCHAIN), cx);
            }
            done(&tx, Ok(()));
        },
    );
}

/// Uses the persisted provider LoginDevice to replace the locked token with a
/// password-authenticated session, then returns it to UI-thread consent.
pub fn unlock_locked_password(
    password: Password,
    window: Option<AnyWindowHandle>,
    cx: &mut App,
) -> Receiver<Progress> {
    let (tx, rx) = async_channel::unbounded();
    let Some(locked) = cx.global::<Services>().locked_session.clone() else {
        done(&tx, Err(SOMETHING_WRONG.into()));
        return rx;
    };
    let Some(seams) = begin(&tx, cx) else {
        return rx;
    };
    let thread = seams.clone();
    run(
        TaskKind::SignIn,
        Step::Contacting,
        false,
        Some(tx.clone()),
        cx,
        move |step| {
            let device = thread.device()?;
            let unlocked = kanso_sync::unlock_with_password(
                thread.backend.as_ref(),
                &locked,
                &password,
                &device,
                step,
            )
            .map_err(|error| Failure::account(error, ErrorContext::SignIn))?;
            let mut locked_session = (*locked).clone();
            if let Err(error) = locked_session
                .adopt_unlocked_session(&unlocked.session)
                .and_then(|()| {
                    keychain::update_locked_pending(
                        thread.secrets.as_ref(),
                        &device.library_id,
                        &locked_session,
                    )
                    .map_err(AccountError::Keychain)
                })
            {
                let mut failure = Failure::account(error, ErrorContext::SignIn);
                failure.keychain_warning |= discard_fresh_candidate(
                    thread.backend.as_ref(),
                    thread.secrets.as_ref(),
                    &unlocked.session,
                    false,
                );
                return Err(failure);
            }
            Ok(UnlockedCandidate {
                session: unlocked.session,
                locked_session,
                evicted: unlocked.evicted,
                library_id: device.library_id,
                same_token_as_locked: true,
            })
        },
        move |result, cx| match result {
            Some(Ok(candidate)) => finish_unlocked_candidate(candidate, window, Some(tx), cx),
            Some(Err(failure)) => failed(&tx, Some(failure), cx),
            None => failed(&tx, None, cx),
        },
    );
    rx
}

/// Recovers a locked provider session using its own server and username.
/// Rotation uses the existing recovery transaction and exposes the new key
/// before binding consent is requested.
pub fn recover_locked(
    key: RecoveryKey,
    password: Password,
    window: Option<AnyWindowHandle>,
    cx: &mut App,
) -> Receiver<Progress> {
    let (tx, rx) = async_channel::unbounded();
    let Some(locked) = cx.global::<Services>().locked_session.clone() else {
        done(&tx, Err(SOMETHING_WRONG.into()));
        return rx;
    };
    let Some(seams) = begin(&tx, cx) else {
        return rx;
    };
    let thread = seams.clone();
    run(
        TaskKind::Recover,
        Step::Contacting,
        false,
        Some(tx.clone()),
        cx,
        move |step| {
            let device = thread.device()?;
            let recovered = kanso_sync::recover(
                thread.backend.as_ref(),
                thread.secrets.as_ref(),
                &locked.server,
                &locked.username,
                &key,
                &password,
                &device,
                step,
            )
            .map_err(|error| {
                Failure::rotation(
                    error,
                    ErrorContext::Recovery,
                    words::KEY_UNCONFIRMED_PASSWORD,
                )
            })?;
            let mut locked_session = (*locked).clone();
            if let Err(error) = locked_session
                .adopt_unlocked_session(&recovered.session)
                .and_then(|()| {
                    keychain::update_locked_pending(
                        thread.secrets.as_ref(),
                        &device.library_id,
                        &locked_session,
                    )
                    .map_err(AccountError::Keychain)
                })
            {
                let mut failure = Failure::account(error, ErrorContext::Recovery);
                failure.keychain_warning |= discard_fresh_candidate(
                    thread.backend.as_ref(),
                    thread.secrets.as_ref(),
                    &recovered.session,
                    false,
                );
                // Recovery rotated the server's keys and ended every session
                // that came before it, including this locked provider token.
                // The updated pending item could not be written, so remove the
                // old one and do not leave the app presenting a dead token.
                if let Err(cleanup) = kanso_sync::sign_out_locked(
                    thread.backend.as_ref(),
                    thread.secrets.as_ref(),
                    &locked,
                ) {
                    eprintln!("kanso: clearing a recovery's stale locked session: {cleanup}");
                    failure.keychain_warning |= matches!(&cleanup, AccountError::Keychain(_));
                }
                failure.locked_cleanup = true;
                return Err(failure.after_key(
                    words::CHANGED_NOT_SIGNED_IN,
                    &recovered.session,
                    recovered.recovery_key,
                ));
            }
            Ok((
                UnlockedCandidate {
                    session: recovered.session,
                    locked_session,
                    evicted: locked.evicted.clone(),
                    library_id: device.library_id,
                    same_token_as_locked: true,
                },
                recovered.recovery_key,
            ))
        },
        move |result, cx| match result {
            Some(Ok((candidate, recovery_key))) => {
                show_key(&candidate.session, recovery_key, cx);
                finish_unlocked_candidate(candidate, window, Some(tx), cx);
            }
            Some(Err(failure)) => failed(&tx, Some(failure), cx),
            None => failed(&tx, None, cx),
        },
    );
    rx
}

#[cfg(test)]
mod passkey_end_tests {
    use super::{ErrorContext, PasskeyEnd, PasskeyFlowError, PinProblem, passkey_end, words};

    fn said(pin: Option<PinProblem>) -> Option<String> {
        let failed = PasskeyFlowError::Ceremony(kanso_sync::PasskeyCeremonyError::Failed);
        match passkey_end(failed, pin, true, ErrorContext::Passkey) {
            PasskeyEnd::Failed(failure) => Some(failure.words),
            _ => None,
        }
    }

    /// Each thing a key says about its PIN is its own line: a wrong PIN
    /// isn't "enter your PIN", and a locked one isn't "try again".
    #[test]
    fn each_pin_problem_says_what_to_do() {
        let lines = [
            said(Some(PinProblem::Required)),
            said(Some(PinProblem::Invalid { retries: Some(3) })),
            said(Some(PinProblem::Invalid { retries: Some(1) })),
            said(Some(PinProblem::Invalid { retries: None })),
            said(Some(PinProblem::AuthBlocked)),
            said(Some(PinProblem::Blocked)),
            said(None),
        ]
        .map(Option::unwrap);
        assert_eq!(lines[0], words::SECURITY_KEY_PIN);
        assert!(
            lines[1].contains("after 3 more wrong tries"),
            "{}",
            lines[1]
        );
        assert!(lines[2].contains("one more wrong try"), "{}", lines[2]);
        assert!(!lines[3].contains("more wrong"), "{}", lines[3]);
        assert_eq!(lines[4], words::SECURITY_KEY_PIN_AUTH_BLOCKED);
        assert_eq!(lines[5], words::SECURITY_KEY_PIN_BLOCKED);
        assert_eq!(lines[6], words::SECURITY_KEY_FAILED);
        let distinct: std::collections::HashSet<_> = lines.iter().collect();
        assert_eq!(distinct.len(), lines.len());
    }
}

#[cfg(test)]
mod provider_wait_tests {
    use super::{
        clear_provider_wait_if_owned, github_next_poll_interval, github_poll_interval,
        provider_request_matches, wait_cancelled_or_expired,
    };
    use kanso_sync::AccountError;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::{Duration, Instant};

    #[test]
    fn provider_wait_observes_cancellation_and_deadline_without_sleeping() {
        let cancel = AtomicBool::new(false);
        let deadline = Instant::now() + Duration::from_secs(60);
        assert!(matches!(
            wait_cancelled_or_expired(&cancel, deadline),
            Ok(false)
        ));

        cancel.store(true, Ordering::Release);
        assert!(matches!(
            wait_cancelled_or_expired(&cancel, deadline),
            Ok(true)
        ));

        cancel.store(false, Ordering::Release);
        assert!(matches!(
            wait_cancelled_or_expired(&cancel, Instant::now() - Duration::from_millis(1)),
            Err(AccountError::FlowExpired)
        ));
    }

    #[test]
    fn github_poll_schedule_starts_at_server_interval_and_slow_down_is_cumulative() {
        let mut interval = github_poll_interval(6);
        assert_eq!(interval, 6);
        interval = github_next_poll_interval(interval, true);
        assert_eq!(interval, 11);
        interval = github_next_poll_interval(interval, true);
        assert_eq!(interval, 16);
        assert_eq!(github_next_poll_interval(interval, false), 16);
        assert_eq!(github_poll_interval(0), 1);
    }

    #[test]
    fn provider_answer_rejects_a_new_server_generation_even_after_a_to_b_to_a() {
        assert!(provider_request_matches("a", 7, "a", 7));
        assert!(!provider_request_matches("b", 7, "a", 7));
        assert!(!provider_request_matches("a", 8, "a", 7));
    }

    #[test]
    fn stale_provider_callback_does_not_clear_a_newer_wait() {
        let old = Arc::new(AtomicBool::new(false));
        let new = Arc::new(AtomicBool::new(false));
        let mut wait = Some(crate::services::ProviderWait {
            provider: "GitHub",
            user_code: Some("CODE".into()),
            verification_uri: Some("https://github.com/login/device".into()),
            cancel: new.clone(),
        });

        clear_provider_wait_if_owned(&mut wait, &old);
        assert!(wait.is_some(), "the new flow's wait stays visible");

        clear_provider_wait_if_owned(&mut wait, &new);
        assert!(wait.is_none(), "the owning callback clears its wait");
    }
}
