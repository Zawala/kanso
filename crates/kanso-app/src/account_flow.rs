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

use async_channel::{Receiver, Sender};
use gpui_kit::component::button::Button;
use gpui_kit::component::notification::Notification;
use gpui_kit::*;
use kanso_store::Store;
use kanso_sync::keychain::{self, PendingRecoveryKey};
use kanso_sync::{
    AccountError, Backend, BindError, Choice, DeviceIdentity, DeviceInfo, Flow, LocalLibrary,
    Machine, Password, Plan, RecoveryKey, SecretError, SecretStore, Session, SignOutReason,
    SignedUp, Step, meta_keys,
};

use crate::account_words::{self as words, Answer, ErrorContext, Question};
use crate::clock::Clock;
use crate::config::{AccountRef, is_default_server};
use crate::dialogs::{self, Described, Progress};
use crate::services::{
    self, AccountDetails, AccountState, PendingKey, RunningTask, Services, TaskKind, notify_windows,
};
use crate::workspace::AppState;

/// When the library itself couldn't be changed.
const LIBRARY_TROUBLE: &str = "Kanso couldn't update this Mac's library. Check there is free space on this Mac, then try again.";
/// When a task's thread stopped without an answer.
const SOMETHING_WRONG: &str = "Something went wrong. Try again.";
/// When another account task is still running.
const BUSY: &str = "Kanso is still busy with your account. Try again in a moment.";

// MARK: Running a task

/// Why a task failed: the words to show, and the reason when the server
/// ended this Mac's session on the way.
#[derive(Debug)]
struct Failure {
    words: String,
    signed_out: Option<SignOutReason>,
}

impl Failure {
    fn account(err: AccountError, context: ErrorContext) -> Failure {
        // The details go to the log; the words never repeat them.
        eprintln!("kanso: account: {err}");
        let signed_out = match &err {
            AccountError::Unauthorized { reason } => Some(*reason),
            _ => None,
        };
        Failure {
            words: words::error_words(&err, context),
            signed_out,
        }
    }

    fn keychain(err: SecretError) -> Failure {
        Failure::account(AccountError::Keychain(err), ErrorContext::Account)
    }

    fn library(err: impl std::fmt::Display) -> Failure {
        eprintln!("kanso: account: the library: {err}");
        Failure {
            words: LIBRARY_TROUBLE.to_owned(),
            signed_out: None,
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

    /// Keeps a new recovery key in the Keychain until "I've saved it", so a
    /// crash before then doesn't lose it (fix 3).
    fn save_key(
        &self,
        library_id: &str,
        session: &Session,
        key: &RecoveryKey,
    ) -> Result<(), Failure> {
        let pending = PendingRecoveryKey {
            origin: session.origin.clone(),
            account_id: session.account_id.clone(),
            username: session.username.clone(),
            recovery_key: key.clone(),
        };
        keychain::save_recovery_key(self.secrets.as_ref(), library_id, &pending)
            .map_err(Failure::keychain)
    }
}

/// Starts `work` on its own thread as the task `kind`, at step `first`.
/// With `stop_sync`, the running sync is cancelled and joined there first,
/// and started again afterwards if the task left the same session signed in
/// (a failure, or a cancelled question). Steps reach [`Services`] and
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
    let stopped_for = sync.as_ref().and_then(|_| services.session());
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
            if let Some(before) = stopped_for
                && cx
                    .global::<Services>()
                    .session()
                    .is_some_and(|now| Arc::ptr_eq(&now, &before))
            {
                services::resume_sync(cx);
            }
            cx.refresh_windows();
        });
    })
    .detach();
}

fn done(tx: &Sender<Progress>, result: Result<(), String>) {
    tx.try_send(Progress::Done(result)).ok();
}

/// Ends a task with its failure; a session the server ended is ended here
/// too.
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
    done(tx, Err(failure.words));
}

/// The seams, or the task's end when there are none or a task runs.
fn begin(tx: &Sender<Progress>, cx: &App) -> Option<Seams> {
    if cx
        .try_global::<Services>()
        .is_some_and(|services| services.task.is_some())
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
    cx.global_mut::<Services>().pending_key = Some(PendingKey {
        key,
        username: session.username.clone(),
        server: session.server.clone(),
    });
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
}

/// Signed out here: the session, the details and the config's account go;
/// the notes stay.
fn forget_account(cx: &mut App) {
    let services = cx.global_mut::<Services>();
    services.clear_session(AccountState::SignedOut);
    services.pending_key = None;
    AppState::update_config(cx, |config| config.sync.account = None);
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
            let signed = kanso_sync::sign_up(
                s.backend.as_ref(),
                &server,
                &username,
                &password,
                &device,
                step,
            )
            .map_err(|err| Failure::account(err, ErrorContext::SignUp))?;
            s.save_key(&device.library_id, &signed.session, &signed.recovery_key)?;
            step(Step::Saving);
            let bound = kanso_sync::bind(&s.store, &signed.session, Choice::Add, &s.backups, s.now)
                .map_err(Failure::bind)
                .and_then(|_| {
                    keychain::save_session(s.secrets.as_ref(), &device.library_id, &signed.session)
                        .map_err(Failure::keychain)
                });
            if let Err(failure) = bound {
                // Nothing here uses the new session: the server forgets it,
                // and so does the Keychain its recovery key, which nobody saw.
                kanso_sync::discard_session(
                    s.backend.as_ref(),
                    s.secrets.as_ref(),
                    &signed.session,
                )
                .ok();
                keychain::delete_recovery_key(s.secrets.as_ref(), &device.library_id).ok();
                return Err(failure);
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
            let recovered = kanso_sync::recover(
                s.backend.as_ref(),
                &server,
                &username,
                &key,
                &password,
                device,
                step,
            )
            .map_err(|err| Failure::account(err, ErrorContext::Recovery))?;
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
            let (session, evicted, key) = auth(&s, &device, step)?;
            if let Some(key) = &key {
                s.save_key(&device.library_id, &session, key)?;
            }
            let library = kanso_sync::local_library(&s.store, &session.origin, &session.account_id)
                .map_err(Failure::library)?;
            Ok((
                Authenticated {
                    session,
                    evicted,
                    library,
                    library_id: device.library_id,
                },
                key,
            ))
        },
        move |result, cx| {
            let (auth, key) = match result {
                Some(Ok(authenticated)) => authenticated,
                Some(Err(failure)) => return failed(&tx, Some(failure), cx),
                None => return failed(&tx, None, cx),
            };
            // A used recovery key is gone: the new one shows at once, even if
            // the binding question is cancelled.
            if let Some(key) = key {
                show_key(&auth.session, key, cx);
            }
            let plan = kanso_sync::plan(flow, &auth.library);
            let question = words::binding_question(&plan, &auth.session.username);
            match (plan, question, window) {
                (Plan::Ask { .. }, Some(question), Some(window)) => {
                    ask(window, question, cx, move |choice, cx| match choice {
                        Choice::Cancel => abandon(seams, auth.session, tx, cx),
                        choice => {
                            bind_and_save(kind, seams, auth, Some(choice), was_signed_in, tx, cx)
                        }
                    });
                }
                (Plan::Ask { .. }, _, _) => abandon(seams, auth.session, tx, cx),
                (Plan::Resume, _, _) => {
                    bind_and_save(kind, seams, auth, None, was_signed_in, tx, cx)
                }
                (Plan::Fresh | Plan::AskBeforeSignUp { .. }, _, _) => {
                    bind_and_save(kind, seams, auth, Some(Choice::Add), was_signed_in, tx, cx)
                }
            }
        },
    );
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
        ..
    } = auth;
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
            let saved = match choice {
                Some(choice) => kanso_sync::bind(&s.store, &session, choice, &s.backups, s.now)
                    .map_err(Failure::bind),
                None => Ok(None),
            }
            .and_then(|backup| {
                keychain::save_session(s.secrets.as_ref(), &library_id, &session)
                    .map_err(Failure::keychain)
                    .map(|()| backup)
            });
            match saved {
                Ok(backup) => Ok((session, library_id, backup)),
                Err(failure) => {
                    if choice.is_some() {
                        // Nothing here uses the new session. The library's
                        // Keychain items may still be another account's.
                        kanso_sync::discard_session(
                            s.backend.as_ref(),
                            s.secrets.as_ref(),
                            &session,
                        )
                        .ok();
                    }
                    Err(failure)
                }
            }
        },
        move |result, cx| match result {
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
        },
    );
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

/// "Saved a backup of this Mac's library first", with Show in Finder.
fn notify_backup(path: PathBuf, cx: &mut App) {
    let shown = crate::settings::display_path(&path, &crate::paths::home());
    notify_windows(
        move || {
            let path = path.clone();
            Notification::success(format!(
                "Kanso saved a backup of this Mac's library first, in {shown}."
            ))
            .action(move |_, _, _| {
                let path = path.clone();
                Button::new("show-backup")
                    .label("Show in Finder")
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
            // account, a leftover Keychain item is never loaded again.
            if let Some(Err(err)) = &result {
                eprintln!("kanso: signing out: {err}");
            }
            forget_account(cx);
            done(&tx, Ok(()));
        },
    );
    rx
}

/// Changes the account's password; the other Macs ask for it.
pub fn change_password(current: Password, new: Password, cx: &mut App) -> Receiver<Progress> {
    let (tx, rx) = async_channel::unbounded();
    let Some((seams, session, _)) = begin_signed_in(&tx, cx) else {
        return rx;
    };
    run(
        TaskKind::ChangePassword,
        Step::DerivingKeys,
        false,
        Some(tx.clone()),
        cx,
        move |step| {
            kanso_sync::change_password(seams.backend.as_ref(), &session, &current, &new, step)
                .map_err(|err| Failure::account(err, ErrorContext::CurrentPassword))
        },
        move |result, cx| match result {
            Some(Ok(_)) => {
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
    let Some((seams, session, library_id)) = begin_signed_in(&tx, cx) else {
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
            let key =
                kanso_sync::new_recovery_key(seams.backend.as_ref(), &session, &current, step)
                    .map_err(|err| Failure::account(err, ErrorContext::CurrentPassword))?;
            seams.save_key(&library_id, &session, &key)?;
            Ok(key)
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
                            "Deleted {username}'s account. The notes on this Mac stay here."
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

/// Signs another device of the account out.
pub fn revoke_device(device_id: String, cx: &mut App) -> Receiver<Progress> {
    let (tx, rx) = async_channel::unbounded();
    let Some((seams, session, _)) = begin_signed_in(&tx, cx) else {
        return rx;
    };
    run(
        TaskKind::RevokeDevice,
        Step::Contacting,
        false,
        Some(tx.clone()),
        cx,
        move |_| {
            kanso_sync::revoke_device(seams.backend.as_ref(), &session, &device_id)
                .map_err(|err| Failure::account(err, ErrorContext::Account))
        },
        move |result, cx| {
            match result {
                Some(Ok(())) => done(&tx, Ok(())),
                Some(Err(failure)) => failed(&tx, Some(failure), cx),
                None => failed(&tx, None, cx),
            }
            refresh_details(cx);
        },
    );
    rx
}

/// "I've saved my recovery key": the key screen and its Keychain item go.
pub fn key_saved(cx: &mut App) {
    let services = cx.global_mut::<Services>();
    services.pending_key = None;
    if let (Some(secrets), Some(library_id)) =
        (services.secrets.clone(), services.library_id.clone())
    {
        cx.background_spawn(async move {
            if let Err(err) = keychain::delete_recovery_key(secrets.as_ref(), &library_id) {
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
    check_signup(cx);
    cx.refresh_windows();
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
                cx.global_mut::<Services>().signup = Some((endpoint.clone(), signup));
                AppState::update_config(cx, |config| {
                    config.sync.server = (!is_default_server(&endpoint)).then_some(endpoint)
                });
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
/// account form, unless it has answered already.
pub fn check_signup(cx: &mut App) {
    let server = form_server(cx);
    let services = cx.global_mut::<Services>();
    let Some(backend) = services.backend.clone() else {
        return;
    };
    if services
        .signup
        .as_ref()
        .is_some_and(|(asked, _)| *asked == server)
    {
        return;
    }
    let (tx, rx) = async_channel::bounded(1);
    let asked = server.clone();
    let spawned = std::thread::Builder::new()
        .name("kanso-account".into())
        .spawn(move || {
            tx.try_send(kanso_sync::server_info(backend.as_ref(), &asked))
                .ok();
        });
    if spawned.is_err() {
        return;
    }
    cx.spawn(async move |cx| {
        if let Ok(Ok(signup)) = rx.recv().await {
            cx.update(|cx| {
                cx.global_mut::<Services>().signup = Some((server, signup));
                cx.refresh_windows();
            });
        }
    })
    .detach();
}

// MARK: Storage and devices

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
            let Some((usage, devices)) = answer else {
                services.details.loading = false;
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
                failed,
            };
            if let Some(devices) = &mut details.devices {
                words::sort_devices(devices);
            }
            services.details = details;
            cx.refresh_windows();
        });
    })
    .detach();
}

// MARK: Launch

/// At launch: signed in only when the config's account, the library's
/// binding and the Keychain item agree (security re-check 3). Retries
/// pending logouts too.
pub fn restore(cx: &mut App) {
    let Some(seams) = Seams::get(cx) else {
        return;
    };
    let account = cx.global::<AppState>().config.sync.account.clone();
    let (tx, rx) = async_channel::bounded(1);
    let spawned = std::thread::Builder::new()
        .name("kanso-account".into())
        .spawn(move || {
            let device = match seams.device() {
                Ok(device) => device,
                Err(failure) => {
                    eprintln!("kanso: this Mac's identity: {}", failure.words);
                    return;
                }
            };
            kanso_sync::flush_pending_logouts(
                seams.backend.as_ref(),
                seams.secrets.as_ref(),
                &device.library_id,
            );
            let Some(account) = account else {
                tx.try_send((device.library_id, Ok(None))).ok();
                return;
            };
            let bound = |key| seams.store.meta(key).ok().flatten();
            let agree = bound(meta_keys::ORIGIN).as_deref() == Some(account.origin.as_str())
                && bound(meta_keys::ACCOUNT_ID).as_deref() == Some(account.id.as_str());
            let loaded = if agree {
                keychain::load_session(
                    seams.secrets.as_ref(),
                    &device.library_id,
                    &account.origin,
                    &account.id,
                )
                .map(|session| {
                    session.map(|session| {
                        let key = keychain::load_recovery_key(
                            seams.secrets.as_ref(),
                            &device.library_id,
                            &account.origin,
                            &account.id,
                        )
                        .ok()
                        .flatten();
                        (session, key)
                    })
                })
            } else {
                Ok(None)
            };
            tx.try_send((device.library_id, loaded)).ok();
        });
    if spawned.is_err() {
        return;
    }
    cx.spawn(async move |cx| {
        let Ok((library_id, loaded)) = rx.recv().await else {
            return;
        };
        cx.update(|cx| {
            cx.global_mut::<Services>().library_id = Some(library_id);
            match loaded {
                Ok(Some((session, key))) => {
                    if let Some(key) = key {
                        show_key(&session, key.recovery_key, cx);
                    }
                    let services = cx.global_mut::<Services>();
                    services.set_signed_in(Arc::new(session));
                    services::apply_sync_settings(cx);
                    refresh_details(cx);
                }
                // The three don't agree: signed out, and the config says so.
                Ok(None) => {
                    if cx.global::<AppState>().config.sync.account.is_some() {
                        AppState::update_config(cx, |config| config.sync.account = None);
                    }
                }
                // The Keychain wouldn't say (locked): signed out for now,
                // with the account kept for the next launch.
                Err(err) => eprintln!("kanso: reading the account from the Keychain: {err}"),
            }
            cx.refresh_windows();
        });
    })
    .detach();
}
