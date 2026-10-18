//! The account functions the app calls (docs/ACCOUNTS.md › Public API the app
//! calls, › Account flows). Frozen by step 1; the bodies are step 2a.
//!
//! These functions talk to the server only. Saving the Keychain items,
//! binding the library and writing the config are the app's flow, in the
//! order docs/ACCOUNTS.md › Library binding › Order and cancellation gives.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::api::{
    Api, Backend, DeleteRequest, DeviceNameRequest, LoginDevice, LoginRequest, NewDevice,
    PasswordRequest, PreloginRequest, RecoverFinishRequest, RecoverStartRequest,
    RecoveryKeyRequest, SignupRequest,
};
use crate::crypto::{
    ContentKeys, CryptoError, DataKey, KdfParams, Opened, Password, PasswordError, RecoveryKey,
    RecoveryKeyError, WrapPurpose, b64, hex, key_id, password_keys, random, recovery_keys, unb64,
    unwrap, wrap,
};
use crate::keychain::{self, PendingLogout, SecretError, SecretStore};
use crate::username::{UsernameError, check_new_username, normalize_username};
use crate::wire::{
    ApiError, PROTOCOL_VERSION, QuotaKind, SessionToken, SignOutReason, SignupClosedReason,
    is_account_id, is_token,
};
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
    data_key: DataKey,
    /// The library whose Keychain items hold this session (sign-out deletes
    /// them and queues its logout there).
    library_id: Option<String>,
}

impl Session {
    /// A session from its parts (the account functions, and the Keychain
    /// item on load).
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
            library_id: None,
        }
    }

    pub(crate) fn token(&self) -> &SessionToken {
        &self.token
    }

    pub(crate) fn data_key(&self) -> &DataKey {
        &self.data_key
    }

    pub(crate) fn set_library_id(&mut self, library_id: &str) {
        self.library_id = Some(library_id.to_owned());
    }

    /// The library this session was created for or loaded from: its
    /// Keychain items are `sync-account:<library_id>` and friends.
    pub fn library_id(&self) -> Option<&str> {
        self.library_id.as_deref()
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
            .field("library_id", &self.library_id)
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

impl From<CryptoError> for AccountError {
    fn from(err: CryptoError) -> AccountError {
        match err {
            CryptoError::KeyMismatch => AccountError::KeyMismatch,
            CryptoError::Kdf(message) => {
                AccountError::Protocol(format!("key derivation: {message}"))
            }
        }
    }
}

/// How often a request is sent in all when its answer is lost (a transport
/// error). Only routes whose repeat the server answers idempotently use it
/// (docs/ACCOUNTS.md › Routes, "Retrying").
const ATTEMPTS: usize = 3;

fn retrying<T>(mut send: impl FnMut() -> Result<T, ApiError>) -> Result<T, ApiError> {
    let mut attempt = 1;
    loop {
        match send() {
            Err(ApiError::Transport(_)) if attempt < ATTEMPTS => attempt += 1,
            result => return result,
        }
    }
}

/// The checked server URL and the origin the derivations bind.
fn checked_server(server: &str) -> Result<(String, String), AccountError> {
    let server = crate::check_endpoint(server)?;
    let origin = crate::origin(&server)?;
    Ok((server, origin))
}

/// Refuses a password equal to the (normalized) username, ignoring case.
fn check_not_username(password: &Password, username: &str) -> Result<(), AccountError> {
    let text = String::from_utf8_lossy(password.as_bytes()).to_lowercase();
    if text == username.to_lowercase() {
        return Err(PasswordError::SameAsUsername.into());
    }
    Ok(())
}

/// The parameters new keys use: the backend's default, which must itself
/// lie within the backend's floor and the ceilings.
fn default_params(b: &dyn Backend) -> Result<KdfParams, AccountError> {
    let params = b.kdf_default();
    if params.is_within(&b.kdf_floor()) {
        Ok(params)
    } else {
        Err(AccountError::UnsafeKdf)
    }
}

/// A prelogin answer, checked.
struct Prelogin {
    params: Option<KdfParams>,
    salt: [u8; 16],
    account_id: String,
}

impl Prelogin {
    /// The account's parameters, refused unless within the backend's floor
    /// and the ceilings, before any key is derived or sent.
    fn params(&self, b: &dyn Backend) -> Result<KdfParams, AccountError> {
        self.params
            .filter(|params| params.is_within(&b.kdf_floor()))
            .ok_or(AccountError::UnsafeKdf)
    }
}

fn prelogin(api: &dyn Api, username: &str) -> Result<Prelogin, AccountError> {
    let answer = retrying(|| {
        api.prelogin(&PreloginRequest {
            username: username.to_owned(),
        })
    })?;
    let salt = unb64(&answer.salt)
        .and_then(|salt| <[u8; 16]>::try_from(salt.as_slice()).ok())
        .ok_or_else(|| AccountError::Protocol("prelogin: the salt isn't 16 bytes".into()))?;
    if !is_account_id(&answer.account_id) {
        return Err(AccountError::Protocol(
            "prelogin: a malformed account ID".into(),
        ));
    }
    Ok(Prelogin {
        params: answer.kdf.params(),
        salt,
        account_id: answer.account_id,
    })
}

/// `a_` and 32 random lowercase hex digits.
fn new_account_id() -> String {
    format!("a_{}", hex(&random::<16>()))
}

/// The value of a sealed name, or `None` when it doesn't open.
fn opened_name(
    keys: &ContentKeys,
    account_id: &str,
    device_id: &str,
    sealed: Option<&str>,
) -> Option<String> {
    match keys.open_name(account_id, device_id, sealed?) {
        Opened::Opened(name) => Some(name),
        Opened::Undecryptable | Opened::TooNew => None,
    }
}

/// This device as a sign-up or recovery sends it: its ID, sealed name and
/// proof for the account.
fn new_device(
    device: &DeviceIdentity,
    keys: &ContentKeys,
    origin: &str,
    account_id: &str,
) -> NewDevice {
    let id = device.device_id(account_id);
    NewDevice {
        sealed_name: Some(keys.seal_name(account_id, &id, &device.name)),
        proof: device.device_proof(origin, account_id).to_base64url(),
        id,
    }
}

/// Checks a session answer (sign-up, recovery): its account and username,
/// and the token's shape.
fn check_answer(
    account_id: &str,
    username: &str,
    answer: (&str, &str, &SessionToken),
) -> Result<(), AccountError> {
    let (answer_account, answer_username, token) = answer;
    if answer_account != account_id || answer_username != username || !is_token(token.expose()) {
        return Err(AccountError::Protocol(
            "the server answered for another account".into(),
        ));
    }
    Ok(())
}

/// `GET /v2/health` on `server`: whether it takes new accounts. A 404, or
/// an answer that isn't a protocol-2 health check, is
/// [`AccountError::ServerOutdated`].
pub fn server_info(b: &dyn Backend, server: &str) -> Result<SignupState, AccountError> {
    let (server, _) = checked_server(server)?;
    match b.api(&server).health() {
        Ok(health) if health.ok && health.protocol == PROTOCOL_VERSION => Ok(health.signup),
        Ok(_) | Err(ApiError::NotFound | ApiError::Protocol(_) | ApiError::Unrecognized { .. }) => {
            Err(AccountError::ServerOutdated)
        }
        Err(err) => Err(err.into()),
    }
}

/// Creates an account and this device's session (docs/ACCOUNTS.md › Account
/// flows › Sign up). Refuses invalid and reserved names and bad passwords
/// offline. A lost 201 is retried with the identical body, which the server
/// answers idempotently (security re-check 7); an account ID someone else
/// took is replaced and the keys rewrapped. The caller saves
/// `result.recovery_key` in the Keychain at once (fix 3), then binds.
pub fn sign_up(
    b: &dyn Backend,
    server: &str,
    username: &str,
    password: &Password,
    device: &DeviceIdentity,
    progress: &dyn Fn(Step),
) -> Result<SignedUp, AccountError> {
    let username = check_new_username(username)?;
    check_not_username(password, &username)?;
    let (server, origin) = checked_server(server)?;
    progress(Step::Contacting);
    match server_info(b, &server)? {
        SignupState::Open => {}
        SignupState::Closed => {
            return Err(AccountError::SignupClosed {
                reason: SignupClosedReason::Closed,
            });
        }
        SignupState::Full => {
            return Err(AccountError::SignupClosed {
                reason: SignupClosedReason::Full,
            });
        }
    }
    let params = default_params(b)?;
    let salt = random::<16>();
    let data_key = DataKey::generate();
    let recovery_key = RecoveryKey::generate();
    progress(Step::DerivingKeys);
    let keys = password_keys(password, &origin, &username, &salt, params)?;
    let recovery = recovery_keys(&recovery_key, &origin, &username);
    let content = ContentKeys::derive(&data_key);
    let api = b.api(&server);
    let mut fresh_ids = 0;
    loop {
        let account_id = new_account_id();
        let request = SignupRequest {
            username: username.clone(),
            account_id: account_id.clone(),
            kdf: params.into(),
            salt: b64(&salt),
            auth_key: keys.auth_key.to_base64url(),
            wrapped_key: wrap(&keys.kek, WrapPurpose::Password, &account_id, &data_key),
            recovery_auth_key: recovery.auth_key.to_base64url(),
            recovery_wrapped_key: wrap(
                &recovery.kek,
                WrapPurpose::Recovery,
                &account_id,
                &data_key,
            ),
            key_id: key_id(&data_key),
            device: new_device(device, &content, &origin, &account_id),
        };
        progress(Step::Contacting);
        match retrying(|| api.signup(&request)) {
            Ok(answer) => {
                check_answer(
                    &account_id,
                    &username,
                    (&answer.account_id, &answer.username, &answer.token),
                )?;
                let mut session = Session::new(
                    server,
                    origin,
                    account_id,
                    username,
                    request.device.id,
                    answer.token,
                    data_key,
                );
                session.set_library_id(&device.library_id);
                return Ok(SignedUp {
                    session,
                    recovery_key,
                });
            }
            Err(ApiError::AccountIdTaken) if fresh_ids < 3 => fresh_ids += 1,
            Err(err) => return Err(err.into()),
        }
    }
}

/// Signs this device in: prelogin (refusing unsafe parameters), derive, log
/// in with the device proof, unwrap and compare the key ID, then `PUT
/// /v2/account/device` with the sealed name. A key mismatch logs the new
/// session out and saves nothing.
pub fn sign_in(
    b: &dyn Backend,
    server: &str,
    username: &str,
    password: &Password,
    device: &DeviceIdentity,
    progress: &dyn Fn(Step),
) -> Result<SignedIn, AccountError> {
    let username = normalize_username(username)?;
    let (server, origin) = checked_server(server)?;
    let api = b.api(&server);
    progress(Step::Contacting);
    let pre = prelogin(&*api, &username)?;
    let params = pre.params(b)?;
    progress(Step::DerivingKeys);
    let keys = password_keys(password, &origin, &username, &pre.salt, params)?;
    let account_id = pre.account_id;
    let device_id = device.device_id(&account_id);
    let request = LoginRequest {
        username: username.clone(),
        auth_key: keys.auth_key.to_base64url(),
        device: LoginDevice {
            id: device_id.clone(),
            proof: device.device_proof(&origin, &account_id).to_base64url(),
        },
    };
    progress(Step::Contacting);
    let answer = retrying(|| api.login(&request))?;
    let fail = |err: AccountError| {
        let _ = api.logout(&answer.token);
        Err(err)
    };
    if let Err(err) = check_answer(
        &account_id,
        &username,
        (&answer.account_id, &answer.username, &answer.token),
    ) {
        return fail(err);
    }
    let data_key = match unwrap(
        &keys.kek,
        WrapPurpose::Password,
        &account_id,
        &answer.wrapped_key,
    ) {
        Ok(data_key) if key_id(&data_key) == answer.key_id => data_key,
        _ => return fail(AccountError::KeyMismatch),
    };
    let content = ContentKeys::derive(&data_key);
    // The name is cosmetic ("A Mac" until set); a failure here doesn't undo
    // the sign-in.
    let _ = api.set_device_name(
        &answer.token,
        &DeviceNameRequest {
            sealed_name: content.seal_name(&account_id, &device_id, &device.name),
        },
    );
    let evicted = answer.evicted_device.as_ref().map(|evicted| DeviceInfo {
        id: evicted.id.clone(),
        name: opened_name(
            &content,
            &account_id,
            &evicted.id,
            evicted.sealed_name.as_deref(),
        ),
        created_at: 0,
        last_seen_at: 0,
        current: false,
    });
    let mut session = Session::new(
        server,
        origin,
        account_id,
        username,
        device_id,
        answer.token.clone(),
        data_key,
    );
    session.set_library_id(&device.library_id);
    Ok(SignedIn { session, evicted })
}

/// Recovers the account with its recovery key and sets `new_password`. The
/// used key stops working; the result carries the new one, which the caller
/// saves in the Keychain before binding.
pub fn recover(
    b: &dyn Backend,
    server: &str,
    username: &str,
    key: &RecoveryKey,
    new_password: &Password,
    device: &DeviceIdentity,
    progress: &dyn Fn(Step),
) -> Result<Recovered, AccountError> {
    let username = normalize_username(username)?;
    check_not_username(new_password, &username)?;
    let (server, origin) = checked_server(server)?;
    let api = b.api(&server);
    progress(Step::Contacting);
    let pre = prelogin(&*api, &username)?;
    let used = recovery_keys(key, &origin, &username);
    let start = retrying(|| {
        api.recover_start(&RecoverStartRequest {
            username: username.clone(),
            recovery_auth_key: used.auth_key.to_base64url(),
        })
    })?;
    if start.account_id != pre.account_id {
        return Err(AccountError::Protocol(
            "recovery answered for another account".into(),
        ));
    }
    let account_id = start.account_id;
    let data_key = match unwrap(
        &used.kek,
        WrapPurpose::Recovery,
        &account_id,
        &start.recovery_wrapped_key,
    ) {
        Ok(data_key) if key_id(&data_key) == start.key_id => data_key,
        _ => return Err(AccountError::KeyMismatch),
    };
    let params = default_params(b)?;
    let salt = random::<16>();
    let recovery_key = RecoveryKey::generate();
    progress(Step::DerivingKeys);
    let keys = password_keys(new_password, &origin, &username, &salt, params)?;
    let next = recovery_keys(&recovery_key, &origin, &username);
    let content = ContentKeys::derive(&data_key);
    let request = RecoverFinishRequest {
        username: username.clone(),
        recovery_auth_key: used.auth_key.to_base64url(),
        kdf: params.into(),
        salt: b64(&salt),
        auth_key: keys.auth_key.to_base64url(),
        wrapped_key: wrap(&keys.kek, WrapPurpose::Password, &account_id, &data_key),
        recovery_auth_key_new: next.auth_key.to_base64url(),
        recovery_wrapped_key_new: wrap(&next.kek, WrapPurpose::Recovery, &account_id, &data_key),
        device: new_device(device, &content, &origin, &account_id),
    };
    progress(Step::Contacting);
    let answer = retrying(|| api.recover_finish(&request))?;
    check_answer(
        &account_id,
        &username,
        (&answer.account_id, &answer.username, &answer.token),
    )?;
    let mut session = Session::new(
        server,
        origin,
        account_id,
        username,
        request.device.id,
        answer.token,
        data_key,
    );
    session.set_library_id(&device.library_id);
    Ok(Recovered {
        session,
        recovery_key,
    })
}

/// Deletes `sync-account:<library>` and any `sync-recovery-key:<library>`,
/// then tries `logout` (the HTTP backend gives it 5 s). On failure it queues
/// `{server, token}` in `sync-logout:<library>`; a 401 counts as done. The
/// caller has already cancelled and joined the running sync.
pub fn sign_out(
    b: &dyn Backend,
    secrets: &dyn SecretStore,
    session: &Session,
) -> Result<(), AccountError> {
    if let Some(library_id) = session.library_id() {
        keychain::delete_session(secrets, library_id)?;
        keychain::delete_recovery_key(secrets, library_id)?;
    }
    logout_or_queue(b, secrets, session)
}

/// Logs out a session this Mac never saved or used: one whose binding
/// question was cancelled, or whose saving failed. Like [`sign_out`], a
/// failed logout is queued in `sync-logout:<library>`, but the library's
/// `sync-account` and `sync-recovery-key` items stay: they may hold the
/// account this Mac is still signed in to, or a recovery key still on
/// screen.
pub fn discard_session(
    b: &dyn Backend,
    secrets: &dyn SecretStore,
    session: &Session,
) -> Result<(), AccountError> {
    logout_or_queue(b, secrets, session)
}

/// `logout`; on failure other than a 401, the session's token is queued.
fn logout_or_queue(
    b: &dyn Backend,
    secrets: &dyn SecretStore,
    session: &Session,
) -> Result<(), AccountError> {
    match b.api(&session.server).logout(session.token()) {
        Ok(()) => Ok(()),
        Err(err) if err.status() == Some(401) => Ok(()),
        Err(_) => {
            if let Some(library_id) = session.library_id() {
                let mut pending = keychain::pending_logouts(secrets, library_id)?;
                let logout = PendingLogout {
                    server: session.server.clone(),
                    token: session.token().clone(),
                };
                if !pending.contains(&logout) {
                    pending.push(logout);
                }
                keychain::set_pending_logouts(secrets, library_id, &pending)?;
            }
            Ok(())
        }
    }
}

/// Retries every pending logout of the library until the server answers 204
/// or 401. Runs at launch and before each sync; failures stay queued, and
/// Keychain errors are left for the next run.
pub fn flush_pending_logouts(b: &dyn Backend, secrets: &dyn SecretStore, library_id: &str) {
    let Ok(pending) = keychain::pending_logouts(secrets, library_id) else {
        return;
    };
    if pending.is_empty() {
        return;
    }
    let remaining: Vec<PendingLogout> = pending
        .into_iter()
        .filter(|logout| {
            let Ok(server) = crate::check_endpoint(&logout.server) else {
                return false;
            };
            match b.api(&server).logout(&logout.token) {
                Ok(()) => false,
                Err(err) => err.status() != Some(401),
            }
        })
        .collect();
    let _ = keychain::set_pending_logouts(secrets, library_id, &remaining);
}

/// The current password's auth key: prelogin for the salt and parameters
/// (refused outside the floor or ceilings), then Argon2id.
fn current_auth(
    b: &dyn Backend,
    api: &dyn Api,
    session: &Session,
    current: &Password,
    progress: &dyn Fn(Step),
) -> Result<crate::crypto::DerivedKeys, AccountError> {
    progress(Step::Contacting);
    let pre = prelogin(api, &session.username)?;
    let params = pre.params(b)?;
    progress(Step::DerivingKeys);
    Ok(password_keys(
        current,
        &session.origin,
        &session.username,
        &pre.salt,
        params,
    )?)
}

/// Changes the password; returns how many other devices were signed out.
/// The data key is rewrapped, not replaced (M4).
pub fn change_password(
    b: &dyn Backend,
    session: &Session,
    current: &Password,
    new: &Password,
    progress: &dyn Fn(Step),
) -> Result<usize, AccountError> {
    check_not_username(new, &session.username)?;
    let api = b.api(&session.server);
    let current_keys = current_auth(b, &*api, session, current, progress)?;
    let params = default_params(b)?;
    let salt = random::<16>();
    let keys = password_keys(new, &session.origin, &session.username, &salt, params)?;
    let request = PasswordRequest {
        current_auth_key: current_keys.auth_key.to_base64url(),
        kdf: params.into(),
        salt: b64(&salt),
        auth_key: keys.auth_key.to_base64url(),
        wrapped_key: wrap(
            &keys.kek,
            WrapPurpose::Password,
            &session.account_id,
            session.data_key(),
        ),
    };
    progress(Step::Contacting);
    let answer = retrying(|| api.change_password(session.token(), &request))?;
    Ok(usize::try_from(answer.signed_out_devices).unwrap_or(usize::MAX))
}

/// Replaces the recovery key while signed in; returns the new one, which
/// the caller saves in the Keychain and shows.
pub fn new_recovery_key(
    b: &dyn Backend,
    session: &Session,
    current: &Password,
    progress: &dyn Fn(Step),
) -> Result<RecoveryKey, AccountError> {
    let api = b.api(&session.server);
    let current_keys = current_auth(b, &*api, session, current, progress)?;
    let recovery_key = RecoveryKey::generate();
    let next = recovery_keys(&recovery_key, &session.origin, &session.username);
    let request = RecoveryKeyRequest {
        current_auth_key: current_keys.auth_key.to_base64url(),
        recovery_auth_key: next.auth_key.to_base64url(),
        recovery_wrapped_key: wrap(
            &next.kek,
            WrapPurpose::Recovery,
            &session.account_id,
            session.data_key(),
        ),
    };
    progress(Step::Contacting);
    retrying(|| api.replace_recovery_key(session.token(), &request))?;
    Ok(recovery_key)
}

/// Deletes the account and its server data. A retry whose response was lost
/// gets 401; only a 401 with reason `unknown` counts as success (security
/// re-check 12). A typed username that isn't this account's is refused
/// before anything is sent. The app then unbinds.
pub fn delete_account(
    b: &dyn Backend,
    session: &Session,
    password: &Password,
    typed_username: &str,
    progress: &dyn Fn(Step),
) -> Result<(), AccountError> {
    let typed = normalize_username(typed_username).map_err(|_| AccountError::UsernameMismatch)?;
    if typed != session.username {
        return Err(AccountError::UsernameMismatch);
    }
    let api = b.api(&session.server);
    let keys = current_auth(b, &*api, session, password, progress)?;
    let request = DeleteRequest {
        auth_key: keys.auth_key.to_base64url(),
        username: typed,
    };
    progress(Step::Contacting);
    match retrying(|| api.delete_account(session.token(), &request)) {
        Ok(_)
        | Err(ApiError::Unauthorized {
            reason: SignOutReason::Unknown,
        }) => Ok(()),
        Err(err) => Err(err.into()),
    }
}

/// The account's active devices, names opened, most recently seen first.
pub fn devices(b: &dyn Backend, session: &Session) -> Result<Vec<DeviceInfo>, AccountError> {
    let answer = b.api(&session.server).devices(session.token())?;
    let content = ContentKeys::derive(session.data_key());
    let mut devices: Vec<DeviceInfo> = answer
        .devices
        .into_iter()
        .map(|row| DeviceInfo {
            name: opened_name(
                &content,
                &session.account_id,
                &row.id,
                row.sealed_name.as_deref(),
            ),
            current: row.id == session.device_id,
            id: row.id,
            created_at: row.created_at,
            last_seen_at: row.last_seen_at,
        })
        .collect();
    devices.sort_by_key(|d| std::cmp::Reverse(d.last_seen_at));
    Ok(devices)
}

/// Signs out another device of this account (revoking this one is a logout).
pub fn revoke_device(
    b: &dyn Backend,
    session: &Session,
    device_id: &str,
) -> Result<(), AccountError> {
    Ok(b.api(&session.server)
        .revoke_device(session.token(), device_id)?)
}

/// Storage use and quotas.
pub fn usage(b: &dyn Backend, session: &Session) -> Result<Usage, AccountError> {
    let answer = b.api(&session.server).account(session.token())?;
    Ok(Usage {
        docs: answer.usage.docs,
        doc_bytes: answer.usage.doc_bytes,
        files: answer.usage.files,
        file_bytes: answer.usage.file_bytes,
        writes_today: answer.usage.writes_today,
        quota: Quota {
            docs: answer.quota.docs,
            doc_bytes: answer.quota.doc_bytes,
            files: answer.quota.files,
            file_bytes: answer.quota.file_bytes,
            writes_per_day: answer.quota.writes_per_day,
        },
    })
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
