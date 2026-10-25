//! Signing in and syncing.
//!
//! Small in step 4 and real: a person signs in with a password, the library
//! syncs on request, and the status says where it stands. Signing in with a
//! provider, approving a device and the automatic schedule come later.
//!
//! The session (token and data key) is kept by this crate in the platform's
//! secret store and read back when a sync starts; the shell never holds it.
//! One sync runs at a time per library. Signing in or out stops a running
//! sync first.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, MutexGuard, TryLockError};

use kanso_core::now_ms;
use kanso_engine::account::words::{self, ErrorContext};
use kanso_engine::account::{self, Failure, Flow, Next};
use kanso_engine::sync::{SyncState, forget_ended_session, run_once};
use kanso_sync::keychain;
use kanso_sync::{
    AccountError, Machine, Password, SecretError, Session, SignupState, SyncError, SyncProblem,
    SyncReport, meta_keys,
};

use crate::boundary::{guard, held};
use crate::errors::KansoError;
use crate::library::Inner;
use crate::model::{ServerInfo, SyncPhase, SyncProblemKind, SyncReportRow, SyncStatus};

/// Sign-in and sync of one library.
#[derive(Clone)]
pub struct SyncHandle {
    pub(crate) inner: Arc<Inner>,
}

/// How a sync attempt ended, for the callers that map it themselves (the
/// public call and the background entry).
pub(crate) enum Ran {
    NotSignedIn,
    SecretsUnavailable(SecretError),
    /// Another sync of this library is running.
    Queued,
    Done(Box<SyncReport>),
    Failed(SyncError),
}

impl SyncHandle {
    /// Where sync stands: signed out, idle, syncing, synced or failed.
    pub fn status(&self) -> Result<SyncStatus, KansoError> {
        guard(|| self.current_status())
    }

    /// Calls `sink` with the status now and again each time it changes, until
    /// the sink returns `false`.
    pub fn status_stream(&self, mut sink: impl FnMut(SyncStatus) -> bool + Send + 'static) {
        if let Ok(status) = self.current_status()
            && !sink(status)
        {
            return;
        }
        self.inner.status_listeners.add(sink);
    }

    /// Asks the server at `url` whether it is a Kanso server, and whether it
    /// takes new accounts.
    pub fn check_server(&self, url: String) -> Result<ServerInfo, KansoError> {
        guard(
            || match kanso_sync::server_info(self.inner.backend.as_ref(), url.trim()) {
                Ok(state) => Ok(ServerInfo {
                    signups_open: state == SignupState::Open,
                }),
                Err(err) => Err(account_error(&err, ErrorContext::Server)),
            },
        )
    }

    /// Signs in to `username`'s account on `server` with its password, naming
    /// this phone `device_name` to the account's other devices, and binds
    /// this library to the account.
    ///
    /// A library that already holds notes of its own, and isn't this
    /// account's, is not bound: Kanso can't ask yet whether to add them or
    /// replace them, so it refuses with [`KansoError::Invalid`], and the
    /// server's new session is ended.
    pub fn sign_in_password(
        &self,
        server: String,
        username: String,
        password: String,
        device_name: String,
    ) -> Result<SyncStatus, KansoError> {
        guard(|| {
            let _stopped = self.stop_sync();
            let password = Password::new(&password, &username)
                .map_err(|err| KansoError::invalid(format!("For the password, {err}.")))?;
            let flow = self.flow(&device_name);
            let device = flow.device().map_err(failure)?;
            let signed = kanso_sync::sign_in(
                self.inner.backend.as_ref(),
                server.trim(),
                &username,
                &password,
                &device,
                &|_| {},
            )
            .map_err(|err| account_error(&err, ErrorContext::SignIn))?;
            let auth = flow
                .authenticated(device.library_id, signed.session, signed.evicted, None)
                .map_err(failure)?;
            let next = account::after_auth(
                kanso_sync::Flow::SignIn,
                &auth.library,
                &auth.session.username,
                false,
            );
            match next {
                Next::Bind(choice) => {
                    flow.bind_and_save(auth.session, auth.library_id, choice, None)
                        .map_err(failure)?;
                }
                Next::Abandon | Next::Ask(_) => {
                    flow.abandon(&auth.session);
                    return Err(KansoError::invalid(
                        "This phone's library already has notes that aren't this account's. \
                         Sign in from an empty library.",
                    ));
                }
            }
            *held(&self.inner.sync_state) = SyncState::Off;
            self.notify();
            self.current_status()
        })
    }

    /// Signs this phone out of its account. The server is told, or told
    /// later if it can't be reached; the library's notes stay.
    pub fn sign_out(&self) -> Result<(), KansoError> {
        guard(|| {
            let _stopped = self.stop_sync();
            if let Some(session) = self.load_session()? {
                self.flow("")
                    .sign_out(&session)
                    .map_err(|err| account_error(&err, ErrorContext::Account))?;
            }
            *held(&self.inner.sync_state) = SyncState::Off;
            self.notify();
            Ok(())
        })
    }

    /// Syncs now: pulls what other devices wrote and pushes what was written
    /// here. If a sync is already running, says so (`queued`) and returns.
    pub fn sync_now(&self) -> Result<SyncReportRow, KansoError> {
        guard(|| match self.run(Arc::new(AtomicBool::new(false))) {
            Ran::NotSignedIn => Err(KansoError::SignedOut),
            Ran::SecretsUnavailable(err) => Err(err.into()),
            Ran::Queued => Ok(SyncReportRow {
                queued: true,
                ..SyncReportRow::default()
            }),
            Ran::Done(report) => Ok(SyncReportRow {
                pulled: count(report.pulled),
                pushed: count(report.pushed),
                conflicts: count(report.conflicts()),
                pending: count(report.pending),
                queued: false,
            }),
            Ran::Failed(err) => Err(sync_error(&err)),
        })
    }

    /// One sync, on this thread, stoppable by setting `cancel`. The status
    /// follows it. Nothing runs when another sync of this library does.
    pub(crate) fn run(&self, cancel: Arc<AtomicBool>) -> Ran {
        let _running = match self.inner.sync_gate.try_lock() {
            Ok(gate) => gate,
            Err(TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
            Err(TryLockError::WouldBlock) => return Ran::Queued,
        };
        let session = match self.load_session() {
            Ok(Some(session)) => session,
            Ok(None) => return Ran::NotSignedIn,
            Err(KansoError::Auth { message }) => {
                return Ran::SecretsUnavailable(SecretError(message));
            }
            Err(_) => {
                return Ran::SecretsUnavailable(SecretError("the secret store failed".into()));
            }
        };
        *held(&self.inner.sync_cancel) = Some(cancel.clone());
        *held(&self.inner.sync_state) = SyncState::Syncing;
        self.notify();
        let library_id = self.inner.store.meta(meta_keys::LIBRARY_ID).ok().flatten();
        let result = run_once(
            &self.inner.store,
            self.inner.backend.as_ref(),
            &session,
            library_id
                .as_deref()
                .map(|id| (self.inner.secrets.as_ref(), id)),
            cancel,
        );
        *held(&self.inner.sync_cancel) = None;
        let now = now_ms();
        let state = match &result {
            Ok(report) => SyncState::Synced {
                at_ms: now,
                conflicts: report.conflicts(),
                conflicted: report.conflicted.clone(),
                problem: kanso_sync::report_problem(report),
            },
            Err(err) => match kanso_sync::problem(err) {
                // The server ended this device's session: sign out.
                Some(SyncProblem::SignedOut(reason)) => {
                    if let Some(library_id) = &library_id {
                        forget_ended_session(
                            self.inner.secrets.as_ref(),
                            library_id,
                            reason,
                            Some(&session),
                            false,
                        );
                    }
                    SyncState::Off
                }
                Some(problem) => SyncState::Failed(problem),
                // Cancelled: nothing to show.
                None => SyncState::Off,
            },
        };
        *held(&self.inner.sync_state) = state;
        self.notify();
        match result {
            Ok(report) => Ran::Done(Box::new(report)),
            Err(err) => Ran::Failed(err),
        }
    }

    /// The saved session of the account this library is bound to, if there is
    /// one. An error when the secret store refuses.
    pub(crate) fn load_session(&self) -> Result<Option<Session>, KansoError> {
        let store = &self.inner.store;
        let meta = |key| store.meta(key).map_err(KansoError::from);
        let (Some(library_id), Some(origin), Some(account_id)) = (
            meta(meta_keys::LIBRARY_ID)?,
            meta(meta_keys::ORIGIN)?,
            meta(meta_keys::ACCOUNT_ID)?,
        ) else {
            return Ok(None);
        };
        keychain::load_session(
            self.inner.secrets.as_ref(),
            &library_id,
            &origin,
            &account_id,
        )
        .map_err(KansoError::from)
    }

    fn current_status(&self) -> Result<SyncStatus, KansoError> {
        let session = self.load_session()?;
        let state = held(&self.inner.sync_state).clone();
        Ok(status_of(session.as_ref(), &state))
    }

    /// Tells the status listeners where sync stands.
    fn notify(&self) {
        if self.inner.status_listeners.is_empty() {
            return;
        }
        if let Ok(status) = self.current_status() {
            self.inner.status_listeners.send(status);
        }
    }

    /// Stops the sync that is running and keeps a new one from starting until
    /// the guard goes.
    fn stop_sync(&self) -> MutexGuard<'_, ()> {
        if let Some(cancel) = held(&self.inner.sync_cancel).as_ref() {
            cancel.store(true, Ordering::SeqCst);
        }
        held(&self.inner.sync_gate)
    }

    fn flow(&self, device_name: &str) -> Flow {
        let backups = self
            .inner
            .path
            .parent()
            .map_or_else(|| "backups".into(), |dir| dir.join("backups"));
        Flow {
            backend: self.inner.backend.clone(),
            secrets: self.inner.secrets.clone(),
            store: self.inner.store.clone(),
            machine: Some(Machine {
                hardware_uuid: None,
                computer_name: device_name.to_owned(),
            }),
            backups,
            now_ms: now_ms(),
        }
    }
}

/// A count as the shell's `u32`.
fn count(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

/// The status for a library with `session` (if signed in) in `state`.
pub(crate) fn status_of(session: Option<&Session>, state: &SyncState) -> SyncStatus {
    let Some(session) = session else {
        return SyncStatus::default();
    };
    let signed_in = SyncStatus {
        phase: SyncPhase::Idle,
        username: Some(session.username.clone()),
        server: Some(session.server.clone()),
        ..SyncStatus::default()
    };
    match state {
        SyncState::Off => signed_in,
        SyncState::Syncing => SyncStatus {
            phase: SyncPhase::Syncing,
            ..signed_in
        },
        SyncState::Synced {
            at_ms,
            conflicts,
            problem,
            ..
        } => SyncStatus {
            phase: SyncPhase::Synced,
            last_synced_ms: Some(*at_ms),
            conflicts: count(*conflicts),
            problem: problem.as_ref().map(problem_kind),
            ..signed_in
        },
        SyncState::Failed(problem) => SyncStatus {
            phase: SyncPhase::Failed,
            problem: Some(problem_kind(problem)),
            ..signed_in
        },
    }
}

pub(crate) fn problem_kind(problem: &SyncProblem) -> SyncProblemKind {
    match problem {
        SyncProblem::SignedOut(_) => SyncProblemKind::SignedOut,
        SyncProblem::AccountDisabled => SyncProblemKind::AccountDisabled,
        SyncProblem::Paused { .. } => SyncProblemKind::Paused,
        SyncProblem::ServerBusy { .. } => SyncProblemKind::ServerBusy,
        SyncProblem::BackOff { .. } => SyncProblemKind::BackOff,
        SyncProblem::QuotaExceeded(_) => SyncProblemKind::AccountFull,
        SyncProblem::ServerFull => SyncProblemKind::ServerFull,
        SyncProblem::Skipped { .. } => SyncProblemKind::SomeItemsSkipped,
        SyncProblem::UpdateKanso => SyncProblemKind::UpdateKanso,
        SyncProblem::WrongKey => SyncProblemKind::KeysDontMatch,
        SyncProblem::ServerOutdated => SyncProblemKind::ServerNeedsUpdate,
        SyncProblem::Offline => SyncProblemKind::Offline,
        SyncProblem::Other => SyncProblemKind::Other,
    }
}

/// What a failed sync is to the shell.
pub(crate) fn sync_error(err: &SyncError) -> KansoError {
    match kanso_sync::problem(err) {
        Some(SyncProblem::SignedOut(_)) => KansoError::SignedOut,
        Some(SyncProblem::AccountDisabled | SyncProblem::WrongKey) => {
            KansoError::auth(err.to_string())
        }
        Some(SyncProblem::UpdateKanso | SyncProblem::ServerOutdated) => {
            KansoError::invalid(err.to_string())
        }
        Some(_) => KansoError::network(err.to_string()),
        None => KansoError::network("The sync was stopped before it finished."),
    }
}

/// What a failed account call is to the shell, in the words the desktop uses.
pub(crate) fn account_error(err: &AccountError, context: ErrorContext) -> KansoError {
    let message = words::error_words(err, context);
    match err {
        AccountError::Transport(_)
        | AccountError::RateLimited { .. }
        | AccountError::Internal
        | AccountError::Misconfigured => KansoError::Network { message },
        AccountError::Unauthorized { .. } => KansoError::SignedOut,
        AccountError::InvalidCredentials
        | AccountError::AccountDisabled
        | AccountError::UsernameMismatch
        | AccountError::KeyMismatch
        | AccountError::Keychain(_) => KansoError::Auth { message },
        _ => KansoError::Invalid { message },
    }
}

/// What a failed account task is to the shell.
fn failure(failure: Failure) -> KansoError {
    if failure.signed_out.is_some() {
        KansoError::SignedOut
    } else if failure.keychain_warning {
        KansoError::Auth {
            message: failure.words,
        }
    } else {
        KansoError::Storage {
            message: failure.words,
        }
    }
}

#[cfg(test)]
mod tests {
    use kanso_engine::strategies;
    use kanso_sync::{Session, SignOutReason};
    use proptest::prelude::*;

    use std::sync::OnceLock;

    use super::*;

    static SESSION: OnceLock<Session> = OnceLock::new();

    /// A real session of the account "ada", on an in-memory server.
    fn session() -> Session {
        let server = kanso_sync::MemoryServer::new();
        let device = kanso_sync::DeviceIdentity::from_parts(
            "l_00000000000000000000000000000001",
            "Phone",
            [1; 32],
        );
        let password = Password::new("correct horse battery", "ada").unwrap();
        kanso_sync::sign_up(
            &server,
            &kanso_sync::MemorySecrets::new(),
            kanso_sync::DEFAULT_SERVER,
            "ada",
            &password,
            &device,
            &|_| {},
        )
        .unwrap()
        .session
    }

    proptest! {
        /// A signed-out library says so whatever sync last said; a signed-in
        /// one never does, and names its account.
        #[test]
        fn prop_status_follows_the_session_and_the_last_sync(
            problem in strategies::problem(),
            at_ms in any::<i64>(),
            conflicts in 0usize..5,
        ) {
            let states = [
                SyncState::Off,
                SyncState::Syncing,
                SyncState::Synced { at_ms, conflicts, conflicted: Vec::new(), problem: Some(problem.clone()) },
                SyncState::Synced { at_ms, conflicts, conflicted: Vec::new(), problem: None },
                SyncState::Failed(problem.clone()),
            ];
            let signed_in = SESSION.get_or_init(session);
            for state in &states {
                prop_assert_eq!(status_of(None, state), SyncStatus::default());
                let status = status_of(Some(signed_in), state);
                prop_assert_ne!(status.phase, SyncPhase::SignedOut);
                prop_assert_eq!(status.username.as_deref(), Some("ada"));
                match state {
                    SyncState::Failed(p) => {
                        prop_assert_eq!(status.phase, SyncPhase::Failed);
                        prop_assert_eq!(status.problem, Some(problem_kind(p)));
                    }
                    SyncState::Synced { .. } => {
                        prop_assert_eq!(status.phase, SyncPhase::Synced);
                        prop_assert_eq!(status.last_synced_ms, Some(at_ms));
                    }
                    SyncState::Syncing => prop_assert_eq!(status.phase, SyncPhase::Syncing),
                    SyncState::Off => prop_assert_eq!(status.phase, SyncPhase::Idle),
                }
            }
        }
    }

    // no-prop-test: a table of the errors the shell tells apart.
    #[test]
    fn sync_errors_map_to_the_kinds_the_shell_tells_apart() {
        let ended = SyncError::Api(kanso_sync::ApiError::Unauthorized {
            reason: SignOutReason::Revoked,
        });
        assert_eq!(sync_error(&ended), KansoError::SignedOut);
        assert!(matches!(
            sync_error(&SyncError::Cancelled),
            KansoError::Network { .. }
        ));
        assert!(matches!(
            sync_error(&SyncError::WrongKey),
            KansoError::Auth { .. }
        ));
    }

    // no-prop-test: a table of the errors the shell tells apart.
    #[test]
    fn account_errors_map_to_the_kinds_the_shell_tells_apart() {
        let sign_in = ErrorContext::SignIn;
        assert!(matches!(
            account_error(&AccountError::Transport("no route".into()), sign_in),
            KansoError::Network { .. }
        ));
        assert!(matches!(
            account_error(&AccountError::InvalidCredentials, sign_in),
            KansoError::Auth { .. }
        ));
        assert!(matches!(
            account_error(&AccountError::InvalidUsername, sign_in),
            KansoError::Invalid { .. }
        ));
    }

    // no-prop-test: one lock held by hand.
    #[test]
    fn a_sync_asked_for_while_one_runs_is_queued_and_does_not_wait() {
        let dir = tempfile::tempdir().unwrap();
        let library = crate::Library::open_for_test(
            &dir.path().join("library.db"),
            Arc::new(kanso_sync::MemorySecrets::new()),
            Arc::new(kanso_sync::MemoryServer::new()),
        )
        .unwrap();
        let running = library.inner.sync_gate.lock().unwrap();
        assert_eq!(
            library.sync().sync_now(),
            Ok(SyncReportRow {
                queued: true,
                ..SyncReportRow::default()
            })
        );
        drop(running);
        assert_eq!(library.sync().sync_now(), Err(KansoError::SignedOut));
    }
}
