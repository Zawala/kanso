//! The account functions the app calls (docs/ACCOUNTS.md › Public API the app
//! calls, › Account flows). Frozen by step 1; the bodies are step 2a.
//!
//! These functions talk to the server only. Saving the Keychain items,
//! binding the library and writing the config are the app's flow, in the
//! order docs/ACCOUNTS.md › Library binding › Order and cancellation gives.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::api::Backend;
use crate::crypto::{DataKey, Password, PasswordError, RecoveryKey, RecoveryKeyError};
use crate::keychain::{SecretError, SecretStore};
use crate::username::UsernameError;
use crate::wire::{ApiError, QuotaKind, SessionToken, SignOutReason, SignupClosedReason};
use crate::{DeviceIdentity, EndpointError};

/// The owner's Worker, compiled in as the default server. Accounts are bound
/// to their server's origin, so it stays this URL (security re-check 14).
pub const DEFAULT_SERVER: &str = "https://kanso-sync.imngonii.workers.dev";

/// What a running account task is doing, for the button's spinner text.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Step {
    /// "Stopping sync…": the app cancels and joins the running sync before
    /// any re-bind or sign-out. The account functions never report it.
    StoppingSync,
    /// "Securing your keys…": Argon2id and wrapping.
    DerivingKeys,
    /// "Contacting the server…".
    Contacting,
    /// "Saving…": Keychain, binding and config (the app's part of a flow).
    Saving,
}

/// A signed-in library. `Send + Sync`, shared as `Arc<Session>`. The token
/// and data key are private and zeroized on drop; `Debug` shows neither.
/// `server` and `origin` come from the Keychain item, never from the config.
pub struct Session {
    /// The URL the client connects to.
    pub server: String,
    /// `url::Url::parse(server)?.origin().ascii_serialization()`; with
    /// `account_id`, what "the same account" means (security re-check 3).
    pub origin: String,
    pub account_id: String,
    /// Normalized.
    pub username: String,
    /// `d_` + 32 hex, this Mac's ID in this account.
    pub device_id: String,
    token: SessionToken,
    #[allow(dead_code, reason = "read by the step 2a bodies")]
    data_key: DataKey,
}

impl Session {
    /// A session from its parts (the account functions, and the Keychain
    /// item on load).
    #[allow(dead_code, reason = "called by the step 2a bodies")]
    pub(crate) fn new(
        server: String,
        origin: String,
        account_id: String,
        username: String,
        device_id: String,
        token: SessionToken,
        data_key: DataKey,
    ) -> Session {
        Session {
            server,
            origin,
            account_id,
            username,
            device_id,
            token,
            data_key,
        }
    }

    pub(crate) fn token(&self) -> &SessionToken {
        &self.token
    }

    #[allow(dead_code, reason = "read by the step 2a bodies")]
    pub(crate) fn data_key(&self) -> &DataKey {
        &self.data_key
    }
}

impl fmt::Debug for Session {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Session")
            .field("server", &self.server)
            .field("origin", &self.origin)
            .field("account_id", &self.account_id)
            .field("username", &self.username)
            .field("device_id", &self.device_id)
            .finish_non_exhaustive()
    }
}

/// An active device of the account, with its name opened ("A Mac" in the UI
/// while `name` is `None`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceInfo {
    pub id: String,
    pub name: Option<String>,
    pub created_at: i64,
    pub last_seen_at: i64,
    /// This Mac.
    pub current: bool,
}

/// The account's effective quotas.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Quota {
    pub docs: u64,
    pub doc_bytes: u64,
    pub files: u64,
    pub file_bytes: u64,
    pub writes_per_day: u64,
}

/// Storage use, from `GET /v2/account`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Usage {
    pub docs: u64,
    pub doc_bytes: u64,
    pub files: u64,
    pub file_bytes: u64,
    pub writes_today: u64,
    pub quota: Quota,
}

/// Whether a server takes new accounts (`GET /v2/health`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignupState {
    Open,
    /// Switched off, or reserved for one name.
    Closed,
    /// `max_accounts` reached.
    Full,
}

/// A new account. Save `recovery_key` in the Keychain at once, then bind.
#[derive(Debug)]
pub struct SignedUp {
    pub session: Session,
    pub recovery_key: RecoveryKey,
}

/// A signed-in device; `evicted` is the device signed out to make room.
#[derive(Debug)]
pub struct SignedIn {
    pub session: Session,
    pub evicted: Option<DeviceInfo>,
}

/// A recovered account and its new recovery key (the used one stops working).
#[derive(Debug)]
pub struct Recovered {
    pub session: Session,
    pub recovery_key: RecoveryKey,
}

/// Why an account function failed: one variant per server error code, plus
/// the client's own failures (docs/ACCOUNTS.md › Public API the app calls).
#[derive(Debug, thiserror::Error)]
pub enum AccountError {
    #[error("the server refused the request as malformed")]
    BadRequest,
    #[error("invalid username")]
    InvalidUsername,
    #[error("reserved username")]
    ReservedUsername,
    #[error("the server doesn't support these key parameters")]
    UnsupportedKdf,
    #[error("the server refused these key parameters")]
    WeakKdf,
    #[error("the typed username isn't this account's")]
    UsernameMismatch,
    #[error("a file didn't match its checksum")]
    ChecksumMismatch,
    /// 401 `unauthorized`: this Mac's session ended, for `reason`.
    #[error("signed out ({})", reason.as_str())]
    Unauthorized { reason: SignOutReason },
    /// 401: wrong password or recovery key, or an unknown username.
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("the server refused a browser request")]
    ForbiddenOrigin,
    #[error("the account is turned off")]
    AccountDisabled,
    #[error("sign-ups are closed ({})", reason.as_str())]
    SignupClosed { reason: SignupClosedReason },
    #[error("not found")]
    NotFound,
    #[error("method not allowed")]
    MethodNotAllowed,
    #[error("username taken")]
    UsernameTaken,
    /// The client generates a new account ID and retries; it never surfaces.
    #[error("account ID taken")]
    AccountIdTaken,
    #[error("too large")]
    TooLarge,
    #[error("quota exceeded ({})", quota.as_str())]
    QuotaExceeded {
        quota: QuotaKind,
        used: u64,
        limit: u64,
    },
    #[error("unsupported media type")]
    UnsupportedMediaType,
    /// "Too many attempts. Try again in *N* minutes."
    #[error("rate limited for {retry_after} s")]
    RateLimited { retry_after: u64 },
    #[error("the server failed")]
    Internal,
    #[error("the server is misconfigured")]
    Misconfigured,
    /// No HTTP answer: "Can't reach the server."
    #[error("can't reach the server: {0}")]
    Transport(String),
    /// An answer outside the protocol, or an unknown error code.
    #[error("unexpected answer from the server: {0}")]
    Protocol(String),
    /// 404 from `/v2/health`: not a v2 Kanso server.
    #[error("that server doesn't run this version of Kanso sync")]
    ServerOutdated,
    /// Prelogin parameters outside the backend's floor or the ceilings; no
    /// key was derived or sent.
    #[error("the server asked for unsafe key parameters")]
    UnsafeKdf,
    /// A wrap didn't open, or the key ID differs. Nothing was saved, and a
    /// new session was logged out.
    #[error("the server's keys don't match")]
    KeyMismatch,
    /// The server URL failed [`crate::check_endpoint`].
    #[error("server URL: {0}")]
    Endpoint(#[from] EndpointError),
    #[error("password: {0}")]
    Password(#[from] PasswordError),
    #[error("username: {0}")]
    Username(#[from] UsernameError),
    #[error("recovery key: {0}")]
    RecoveryKey(#[from] RecoveryKeyError),
    /// The Keychain refused (sign-out and pending logouts only).
    #[error("{0}")]
    Keychain(#[from] SecretError),
}

impl From<ApiError> for AccountError {
    fn from(err: ApiError) -> AccountError {
        match err {
            ApiError::BadRequest => AccountError::BadRequest,
            ApiError::InvalidUsername => AccountError::InvalidUsername,
            ApiError::ReservedUsername => AccountError::ReservedUsername,
            ApiError::UnsupportedKdf => AccountError::UnsupportedKdf,
            ApiError::WeakKdf => AccountError::WeakKdf,
            ApiError::UsernameMismatch => AccountError::UsernameMismatch,
            ApiError::ChecksumMismatch => AccountError::ChecksumMismatch,
            ApiError::Unauthorized { reason } => AccountError::Unauthorized { reason },
            ApiError::InvalidCredentials => AccountError::InvalidCredentials,
            ApiError::ForbiddenOrigin => AccountError::ForbiddenOrigin,
            ApiError::AccountDisabled => AccountError::AccountDisabled,
            ApiError::SignupClosed { reason } => AccountError::SignupClosed { reason },
            ApiError::NotFound => AccountError::NotFound,
            ApiError::MethodNotAllowed => AccountError::MethodNotAllowed,
            ApiError::UsernameTaken => AccountError::UsernameTaken,
            ApiError::AccountIdTaken => AccountError::AccountIdTaken,
            ApiError::TooLarge => AccountError::TooLarge,
            ApiError::QuotaExceeded { quota, used, limit } => {
                AccountError::QuotaExceeded { quota, used, limit }
            }
            ApiError::UnsupportedMediaType => AccountError::UnsupportedMediaType,
            ApiError::RateLimited { retry_after, .. } => AccountError::RateLimited { retry_after },
            ApiError::Internal => AccountError::Internal,
            ApiError::Misconfigured => AccountError::Misconfigured,
            ApiError::Transport(message) => AccountError::Transport(message),
            ApiError::Protocol(message) => AccountError::Protocol(message),
            ApiError::Unrecognized { status, code } => {
                AccountError::Protocol(format!("{status} {code}"))
            }
        }
    }
}

/// `GET /v2/health` on `server`: whether it takes new accounts. A 404 is
/// [`AccountError::ServerOutdated`].
pub fn server_info(b: &dyn Backend, server: &str) -> Result<SignupState, AccountError> {
    crate::stub!("2a", b, server)
}

/// Creates an account and this device's session (docs/ACCOUNTS.md › Account
/// flows › Sign up). Refuses invalid and reserved names and bad passwords
/// offline. A lost 201 is retried with the identical body, which the server
/// answers idempotently (security re-check 7).
pub fn sign_up(
    b: &dyn Backend,
    server: &str,
    username: &str,
    password: &Password,
    device: &DeviceIdentity,
    progress: &dyn Fn(Step),
) -> Result<SignedUp, AccountError> {
    crate::stub!("2a", b, server, username, password, device, progress)
}

/// Signs this device in: prelogin (refusing unsafe parameters), derive, log
/// in with the device proof, unwrap and compare the key ID, then `PUT
/// /v2/account/device` with the sealed name.
pub fn sign_in(
    b: &dyn Backend,
    server: &str,
    username: &str,
    password: &Password,
    device: &DeviceIdentity,
    progress: &dyn Fn(Step),
) -> Result<SignedIn, AccountError> {
    crate::stub!("2a", b, server, username, password, device, progress)
}

/// Recovers the account with its recovery key and sets `new_password`. The
/// used key stops working; the result carries the new one.
pub fn recover(
    b: &dyn Backend,
    server: &str,
    username: &str,
    key: &RecoveryKey,
    new_password: &Password,
    device: &DeviceIdentity,
    progress: &dyn Fn(Step),
) -> Result<Recovered, AccountError> {
    crate::stub!(
        "2a",
        b,
        server,
        username,
        key,
        new_password,
        device,
        progress
    )
}

/// Deletes `sync-account:<library>` and any `sync-recovery-key:<library>`,
/// then tries `logout` (5 s). On failure it queues `{server, token}` in
/// `sync-logout:<library>`. The caller has already cancelled and joined the
/// running sync.
pub fn sign_out(
    b: &dyn Backend,
    secrets: &dyn SecretStore,
    session: &Session,
) -> Result<(), AccountError> {
    crate::stub!("2a", b, secrets, session)
}

/// Retries every pending logout of the library until the server answers 204
/// or 401. Runs at launch and before each sync; failures stay queued.
pub fn flush_pending_logouts(b: &dyn Backend, secrets: &dyn SecretStore, library_id: &str) {
    crate::stub!("2a", b, secrets, library_id)
}

/// Changes the password; returns how many other devices were signed out.
pub fn change_password(
    b: &dyn Backend,
    session: &Session,
    current: &Password,
    new: &Password,
    progress: &dyn Fn(Step),
) -> Result<usize, AccountError> {
    crate::stub!("2a", b, session, current, new, progress)
}

/// Replaces the recovery key while signed in; returns the new one.
pub fn new_recovery_key(
    b: &dyn Backend,
    session: &Session,
    current: &Password,
    progress: &dyn Fn(Step),
) -> Result<RecoveryKey, AccountError> {
    crate::stub!("2a", b, session, current, progress)
}

/// Deletes the account and its server data. A retry whose response was lost
/// gets 401; only a 401 with reason `unknown` counts as success (security
/// re-check 12). The app then unbinds.
pub fn delete_account(
    b: &dyn Backend,
    session: &Session,
    password: &Password,
    typed_username: &str,
    progress: &dyn Fn(Step),
) -> Result<(), AccountError> {
    crate::stub!("2a", b, session, password, typed_username, progress)
}

/// The account's active devices, names opened, most recently seen first.
pub fn devices(b: &dyn Backend, session: &Session) -> Result<Vec<DeviceInfo>, AccountError> {
    crate::stub!("2a", b, session)
}

/// Signs out another device of this account (revoking this one is a logout).
pub fn revoke_device(
    b: &dyn Backend,
    session: &Session,
    device_id: &str,
) -> Result<(), AccountError> {
    crate::stub!("2a", b, session, device_id)
}

/// Storage use and quotas.
pub fn usage(b: &dyn Backend, session: &Session) -> Result<Usage, AccountError> {
    crate::stub!("2a", b, session)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every coded `ApiError` keeps its meaning as an `AccountError`.
    #[test]
    fn api_errors_map_to_account_errors() {
        let reason = SignOutReason::PasswordChanged;
        assert!(matches!(
            AccountError::from(ApiError::Unauthorized { reason }),
            AccountError::Unauthorized {
                reason: SignOutReason::PasswordChanged
            }
        ));
        assert!(matches!(
            AccountError::from(ApiError::RateLimited {
                retry_after: 120,
                limit: "login-user".into()
            }),
            AccountError::RateLimited { retry_after: 120 }
        ));
        assert!(matches!(
            AccountError::from(ApiError::Unrecognized {
                status: 418,
                code: "teapot".into()
            }),
            AccountError::Protocol(_)
        ));
        assert!(matches!(
            AccountError::from(ApiError::Transport("reset".into())),
            AccountError::Transport(_)
        ));
    }
}
