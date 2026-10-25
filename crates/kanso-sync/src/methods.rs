//! The account's ways in beyond a password (docs/ACCOUNTS.md › Sign-in
//! methods, › Public API the app calls, › Account flows): passkey sign-up and
//! sign-in, fresh proofs for sensitive changes, adding and removing passkeys,
//! the first password, and the recovery-key and delete routes with a proof.
//!
//! Every ceremony is split in two around the platform: a `start_*` call gets
//! the server's challenge and returns what the app hands its passkey sheet
//! (or the Linux security-key form), and the second call sends the result.
//! This crate never calls a platform API.

use std::fmt;

use crate::account::{
    AccountError, PasswordChanged, Rotation, RotationError, Session, SignedIn, SignedUp,
    SignupState, Step, check_not_username, checked_server, current_auth, default_params,
    new_account_id, new_device, opened_name, retrying, retrying_noted, server_info,
    session_library,
};
use crate::api::{
    AccountPasskeysFinishRequest, AccountPasskeysStartRequest, Backend, DeleteRequest,
    DeviceNameRequest, LoginDevice, NewDevice, PasskeyFinishRequest, PasskeyProofAssertion,
    PasskeyProofRequest, PasskeyProofStartRequest, PasskeySignupFinishRequest,
    PasskeySignupStartRequest, PasskeyStartRequest, RecoveryKeyRequest, SignInPasswordRequest,
};
use crate::approvals::LockedSession;
use crate::binding::DeviceIdentity;
use crate::crypto::{
    AuthKey, ContentKeys, CryptoError, DataKey, Password, RecoveryKey, WrapPurpose, b64, key_id,
    password_keys, random, recovery_keys, unb64, wrap,
};
use crate::keychain::{self, SecretStore};
use crate::passkey::{
    PasskeyAssertion, PasskeyAssertionChallenge, PasskeyRegistration, PasskeyRegistrationChallenge,
    open_passkey_session_key, passkey_wrapped_key, user_handle,
};
use crate::provider::{PasskeyInfo, ProofWire};
use crate::username::{check_new_username, normalize_username};
use crate::wire::{
    ApiError, PROTOCOL_VERSION, SignOutReason, SignupClosedReason, is_account_id, is_hex16,
    is_token,
};

// ---------------------------------------------------------------- proofs

/// A fresh proof for a sensitive change (docs/ACCOUNTS.md › Changing a way
/// in needs a fresh proof): the current password's auth key
/// ([`password_proof`]), a passkey assertion over a
/// [`start_passkey_proof`] challenge ([`Proof::passkey`]), or a completed
/// provider proof flow ([`crate::start_identity`] with `intent: proof`).
///
/// A provider proof can only unlink its own provider; setting a password,
/// replacing the recovery key and deleting with it also need the recovery
/// key. The server enforces both; the client passes them through.
pub enum Proof<'a> {
    Password(&'a AuthKey),
    Passkey {
        flow: String,
        assertion: PasskeyAssertion,
    },
    Provider {
        flow: String,
    },
}

impl Proof<'_> {
    /// A passkey proof from the challenge's flow and the platform's
    /// assertion over it.
    pub fn passkey(start: &PasskeyProofStart, assertion: PasskeyAssertion) -> Proof<'static> {
        Proof::Passkey {
            flow: start.flow_id.clone(),
            assertion,
        }
    }

    /// The wire's `proof` object. The assertion's PRF output, if any, is
    /// never sent.
    pub fn to_wire(&self) -> ProofWire {
        match self {
            Proof::Password(auth_key) => ProofWire::password(auth_key.to_base64url()),
            Proof::Passkey { flow, assertion } => ProofWire::passkey(
                flow.clone(),
                PasskeyProofAssertion {
                    credential_id: b64(&assertion.credential_id),
                    authenticator_data: b64(&assertion.authenticator_data),
                    client_data_json: b64(&assertion.client_data_json),
                    signature: b64(&assertion.signature),
                },
            ),
            Proof::Provider { flow } => ProofWire::provider(flow.clone()),
        }
    }

    /// Whether this is a provider proof, which needs the recovery key for a
    /// password, a new recovery key or a deletion.
    pub fn is_provider(&self) -> bool {
        matches!(self, Proof::Provider { .. })
    }
}

impl fmt::Debug for Proof<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Proof::Password(_) => f.write_str("Proof::Password(<redacted>)"),
            Proof::Passkey { flow, assertion } => f
                .debug_struct("Proof::Passkey")
                .field("flow", flow)
                .field("assertion", assertion)
                .finish(),
            Proof::Provider { flow } => f
                .debug_struct("Proof::Provider")
                .field("flow", flow)
                .finish(),
        }
    }
}

/// The current password's auth key for a [`Proof::Password`]: prelogin for
/// the salt, then Argon2id. Counts nothing on the server until it is used.
pub fn password_proof(
    b: &dyn Backend,
    session: &Session,
    password: &Password,
    progress: &dyn Fn(Step),
) -> Result<AuthKey, AccountError> {
    let api = b.api(&session.server);
    Ok(current_auth(b, &*api, session, password, progress)?.auth_key)
}

/// The recovery key's auth key for `session`'s account, as
/// `current_recovery_auth_key` and `recovery_auth_key` carry it.
fn recovery_auth(session: &Session, recovery: Option<&RecoveryKey>) -> Option<String> {
    recovery.map(|key| {
        recovery_keys(key, &session.origin, &session.username)
            .auth_key
            .to_base64url()
    })
}

// ------------------------------------------------------------ the relying party

/// The relying party a server advertises and the client-data origin this
/// client uses with it: the first advertised origin that
/// [`crate::passkey_origin_allowed`] accepts, or `https://<rp_id>` when none
/// is advertised. [`AccountError::UnsupportedMethod`] when the server offers
/// no passkeys, or offers a relying party [`crate::passkey_rp_allowed`]
/// refuses for it (the official one from any server but the owner's, or one
/// its host isn't under), so no flow runs a ceremony another server could
/// relay.
fn relying_party(b: &dyn Backend, server: &str) -> Result<(String, String), AccountError> {
    let health = match b.api(server).health() {
        Ok(health) if health.ok && health.protocol == PROTOCOL_VERSION => health,
        Ok(_) | Err(ApiError::NotFound | ApiError::Protocol(_) | ApiError::Unrecognized { .. }) => {
            return Err(AccountError::ServerOutdated);
        }
        Err(err) => return Err(err.into()),
    };
    let passkeys = health
        .auth
        .and_then(|auth| auth.passkeys)
        .ok_or(AccountError::UnsupportedMethod)?;
    if !crate::passkey_rp_allowed(&passkeys.rp_id, server) {
        return Err(AccountError::UnsupportedMethod);
    }
    let origin = if passkeys.origins.is_empty() {
        Some(format!("https://{}", passkeys.rp_id))
    } else {
        passkeys
            .origins
            .iter()
            .find(|origin| crate::passkey_origin_allowed(origin, &passkeys.rp_id))
            .cloned()
    };
    let origin = origin
        .filter(|origin| crate::passkey_origin_allowed(origin, &passkeys.rp_id))
        .ok_or(AccountError::UnsupportedMethod)?;
    Ok((passkeys.rp_id, origin))
}

fn challenge_bytes(text: &str) -> Result<Vec<u8>, AccountError> {
    unb64(text)
        .filter(|bytes| (16..=64).contains(&bytes.len()))
        .ok_or_else(|| AccountError::Protocol("a malformed passkey challenge".into()))
}

fn credential_ids(
    rows: &[crate::api::PasskeyAllowedCredential],
) -> Result<Vec<Vec<u8>>, AccountError> {
    rows.iter()
        .map(|row| {
            unb64(&row.id).ok_or_else(|| AccountError::Protocol("a malformed credential ID".into()))
        })
        .collect()
}

// ---------------------------------------------------------- passkey proofs

/// A passkey proof's challenge: its flow (single use, five minutes) and what
/// the assertion ceremony needs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PasskeyProofStart {
    pub flow_id: String,
    pub challenge: PasskeyAssertionChallenge,
}

impl fmt::Debug for PasskeyAssertionChallenge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PasskeyAssertionChallenge")
            .field("rp_id", &self.rp_id)
            .field("origin", &self.origin)
            .field("allow_credentials", &self.allow_credentials.len())
            .field("request_prf", &self.request_prf)
            .finish_non_exhaustive()
    }
}

impl fmt::Debug for PasskeyRegistrationChallenge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PasskeyRegistrationChallenge")
            .field("rp_id", &self.rp_id)
            .field("origin", &self.origin)
            .field("username", &self.username)
            .field("exclude_credentials", &self.exclude_credentials.len())
            .field("request_prf", &self.request_prf)
            .finish_non_exhaustive()
    }
}

/// `POST /v2/account/sign-in/proof/start`: a challenge for a passkey proof,
/// for one credential when `credential_id` names it. The proof is single use
/// and is sent with the action it authorizes. Counts `proof-account`.
pub fn start_passkey_proof(
    b: &dyn Backend,
    session: &Session,
    credential_id: Option<&[u8]>,
) -> Result<PasskeyProofStart, AccountError> {
    let (rp_id, origin) = relying_party(b, &session.server)?;
    let api = b.api(&session.server);
    let answer = api.passkey_proof_start(
        session.token(),
        &PasskeyProofStartRequest {
            credential_id: credential_id.map(b64),
        },
    )?;
    Ok(PasskeyProofStart {
        challenge: PasskeyAssertionChallenge {
            rp_id,
            origin,
            challenge: challenge_bytes(&answer.challenge)?,
            allow_credentials: credential_ids(&answer.allow_credentials)?,
            // A proof proves presence; it never unlocks anything.
            request_prf: false,
        },
        flow_id: answer.flow_id,
    })
}

// ------------------------------------------------------- passkey sign-in

/// A started passkey sign-in: the flow, the device it is bound to, and what
/// the assertion ceremony needs. The device is the one every later step of
/// the sign-in uses, including a password unlock of the locked session.
#[derive(Clone)]
pub struct PasskeySignInStart {
    pub server: String,
    pub origin: String,
    pub flow_id: String,
    /// The username typed first (Linux security keys, or a key without
    /// discoverable storage), whose credentials `challenge` lists.
    pub username: Option<String>,
    pub challenge: PasskeyAssertionChallenge,
    login_device: LoginDevice,
}

impl PasskeySignInStart {
    /// The device ID and proof the flow is bound to.
    pub fn login_device(&self) -> &LoginDevice {
        &self.login_device
    }
}

impl fmt::Debug for PasskeySignInStart {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PasskeySignInStart")
            .field("server", &self.server)
            .field("flow_id", &self.flow_id)
            .field("username", &self.username)
            .field("challenge", &self.challenge)
            .field("login_device", &self.login_device)
            .finish()
    }
}

/// A passkey sign-in's outcome: the device holds the data key (this
/// library's own, or the credential's PRF wrap opened it), or the session is
/// locked and needs a password, an approval or the recovery key.
#[derive(Debug)]
#[allow(
    clippy::large_enum_variant,
    reason = "returned once per sign-in; the spec's shape"
)]
pub enum PasskeySignIn {
    Unlocked(SignedIn),
    Locked(LockedSession),
}

/// Where a passkey sign-in's data key came from (docs/ACCOUNTS.md › Account
/// flows › Sign in with a passkey, step 2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PasskeyUnlock {
    /// This library already holds the account's data key, with the
    /// server's key ID.
    LocalKey,
    /// The credential's PRF output opened its wrap, with the server's key
    /// ID.
    Prf,
    /// Neither: the session is locked.
    Locked,
}

/// The passkey sign-in's decision, pure. The library's own key wins when
/// its ID is the server's; otherwise a PRF output and a wrap must open to the
/// server's key ID ([`CryptoError::KeyMismatch`] when they don't: the server
/// answered with a wrap this credential can't open); otherwise the session
/// is locked. The key is returned exactly when the answer isn't `Locked`.
pub fn passkey_unlock(
    local: Option<&DataKey>,
    prf: Option<&[u8; 32]>,
    wrapped: Option<&str>,
    origin: &str,
    account_id: &str,
    credential_id: &[u8],
    expected_key_id: &str,
) -> Result<(PasskeyUnlock, Option<DataKey>), CryptoError> {
    if let Some(local) = local
        && key_id(local) == expected_key_id
    {
        return Ok((
            PasskeyUnlock::LocalKey,
            Some(DataKey::from_bytes(*local.bytes())),
        ));
    }
    match (prf, wrapped) {
        (Some(prf), Some(wrapped)) => {
            let key = open_passkey_session_key(
                prf,
                origin,
                account_id,
                credential_id,
                Some(wrapped),
                expected_key_id,
            )?;
            Ok((PasskeyUnlock::Prf, Some(key)))
        }
        _ => Ok((PasskeyUnlock::Locked, None)),
    }
}

/// `POST /v2/auth/passkey/start`: a username-less challenge by default, so
/// the platform shows its picker; with `username`, that account's
/// credentials are listed (Linux security keys). `login_device` is the
/// device the flow is bound to: [`LoginDevice::for_identity`] when this
/// library knows the account, else [`LoginDevice::ephemeral`]. Counts
/// `passkey-challenge`.
pub fn start_passkey_sign_in(
    b: &dyn Backend,
    server: &str,
    username: Option<&str>,
    login_device: &LoginDevice,
) -> Result<PasskeySignInStart, AccountError> {
    let username = username.map(normalize_username).transpose()?;
    let (server, origin) = checked_server(server)?;
    let (rp_id, client_origin) = relying_party(b, &server)?;
    let api = b.api(&server);
    let request = PasskeyStartRequest {
        username: username.clone(),
        device: login_device.clone(),
    };
    let answer = retrying(b, || api.passkey_start(&request))?;
    if answer.rp_id != rp_id {
        return Err(AccountError::Protocol(
            "the passkey challenge names another relying party".into(),
        ));
    }
    Ok(PasskeySignInStart {
        challenge: PasskeyAssertionChallenge {
            rp_id,
            origin: client_origin,
            challenge: challenge_bytes(&answer.challenge)?,
            allow_credentials: credential_ids(&answer.allow_credentials)?,
            request_prf: true,
        },
        server,
        origin,
        flow_id: answer.flow_id,
        username,
        login_device: login_device.clone(),
    })
}

/// Signs in with a passkey assertion over `start`'s challenge
/// (docs/ACCOUNTS.md › Account flows › Sign in with a passkey). The library's
/// saved key for the account is used when its key ID is the server's;
/// otherwise the PRF output opens the credential's wrap; otherwise the
/// session comes back locked, with the flow's device kept for a password
/// unlock. A wrap that doesn't open to the server's key ID is
/// [`AccountError::KeyMismatch`]: the new session is logged out and nothing
/// is saved. An answer for another account than the assertion's user handle
/// names is [`AccountError::Protocol`], also logged out. An unlocked session gets its sealed name; saving it, the
/// binding question and `bind` are the app's, as for a password sign-in.
pub fn sign_in_with_passkey(
    b: &dyn Backend,
    secrets: &dyn SecretStore,
    start: &PasskeySignInStart,
    device: &DeviceIdentity,
    assertion: PasskeyAssertion,
    progress: &dyn Fn(Step),
) -> Result<PasskeySignIn, AccountError> {
    let (server, origin) = checked_server(&start.server)?;
    if origin != start.origin {
        return Err(AccountError::KeyMismatch);
    }
    let api = b.api(&server);
    let request = PasskeyFinishRequest {
        flow_id: start.flow_id.clone(),
        credential_id: b64(&assertion.credential_id),
        authenticator_data: b64(&assertion.authenticator_data),
        client_data_json: b64(&assertion.client_data_json),
        signature: b64(&assertion.signature),
        user_handle: assertion.user_handle.as_deref().map(b64),
        device: start.login_device.clone(),
    };
    progress(Step::Contacting);
    // A finish consumes the flow, so a resend could only be refused: a lost
    // answer means starting again.
    let answer = api.passkey_finish(&request)?;
    let fail = |err: AccountError| {
        let _ = api.logout(&answer.token);
        Err(err)
    };
    // The authenticator names the account its passkey belongs to. A server
    // that answers with another (one whose password the user would then
    // type on a locked Mac) is refused before anything else is believed.
    if let Some(handle) = &assertion.user_handle
        && *handle != crate::passkey::user_handle(&answer.account_id)
    {
        return fail(AccountError::Protocol(
            "the passkey sign-in answered for another account than the passkey's".into(),
        ));
    }
    let username_ok = normalize_username(&answer.username).is_ok_and(|n| n == answer.username)
        && start
            .username
            .as_ref()
            .is_none_or(|typed| typed == &answer.username);
    if !is_account_id(&answer.account_id)
        || !username_ok
        || !is_token(answer.token.expose())
        || !is_hex16(&answer.key_id)
    {
        return fail(AccountError::Protocol(
            "the passkey sign-in answered with a malformed session".into(),
        ));
    }
    let account_id = answer.account_id.clone();
    let local = keychain::load_session(secrets, &device.library_id, &origin, &account_id)
        .ok()
        .flatten();
    progress(Step::DerivingKeys);
    let decided = passkey_unlock(
        local.as_ref().map(Session::data_key),
        assertion.prf(),
        answer.passkey_wrapped_key.as_deref(),
        &origin,
        &account_id,
        &assertion.credential_id,
        &answer.key_id,
    );
    let data_key = match decided {
        Ok((_, Some(data_key))) => data_key,
        Ok((_, None)) => {
            let mut locked = LockedSession::new(
                server,
                origin,
                account_id,
                answer.username.clone(),
                start.login_device.id.clone(),
                answer.token.clone(),
                answer.key_id.clone(),
                answer.epoch.clone(),
                None,
                None,
            )
            .with_has_password(answer.has_password)
            .with_login_device(start.login_device.clone())
            .with_evicted_wire_device(answer.evicted_device.clone());
            locked.set_library_id(&device.library_id);
            return Ok(PasskeySignIn::Locked(locked));
        }
        Err(err) => return fail(err.into()),
    };
    let device_id = start.login_device.id.clone();
    let content = ContentKeys::derive(&data_key);
    // The name is cosmetic ("A Mac" until set); a failure doesn't undo the
    // sign-in.
    let _ = api.set_device_name(
        &answer.token,
        &DeviceNameRequest {
            sealed_name: content.seal_name(&account_id, &device_id, &device.name),
        },
    );
    let evicted = answer
        .evicted_device
        .as_ref()
        .map(|evicted| crate::DeviceInfo {
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
        answer.username.clone(),
        device_id,
        answer.token.clone(),
        data_key,
    );
    session.set_library_id(&device.library_id);
    Ok(PasskeySignIn::Unlocked(SignedIn { session, evicted }))
}

// ------------------------------------------------------- passkey sign-up

/// A started passkey sign-up: the chosen username and account ID, the flow,
/// and what the registration ceremony needs.
#[derive(Clone, Debug)]
pub struct PasskeySignUpStart {
    pub server: String,
    pub origin: String,
    pub username: String,
    pub account_id: String,
    pub flow_id: String,
    pub challenge: PasskeyRegistrationChallenge,
}

fn signup_open(b: &dyn Backend, server: &str) -> Result<(), AccountError> {
    match server_info(b, server)? {
        SignupState::Open => Ok(()),
        SignupState::Closed => Err(AccountError::SignupClosed {
            reason: SignupClosedReason::Closed,
        }),
        SignupState::Full => Err(AccountError::SignupClosed {
            reason: SignupClosedReason::Full,
        }),
    }
}

/// `POST /v2/auth/passkey/signup/start` (docs/ACCOUNTS.md › Account flows ›
/// Sign up with a passkey, step 1): checks the name offline, chooses the
/// account ID, and returns the registration challenge with PRF requested.
/// An `account_id_taken` start is retried with a new ID. Counts `signup-ip`.
pub fn start_passkey_sign_up(
    b: &dyn Backend,
    server: &str,
    username: &str,
    device: &DeviceIdentity,
) -> Result<PasskeySignUpStart, AccountError> {
    let username = check_new_username(username)?;
    let (server, origin) = checked_server(server)?;
    signup_open(b, &server)?;
    let (rp_id, client_origin) = relying_party(b, &server)?;
    let api = b.api(&server);
    let mut fresh_ids = 0;
    loop {
        let account_id = new_account_id();
        let request = PasskeySignupStartRequest {
            username: username.clone(),
            account_id: account_id.clone(),
            // The name is sealed with the data key, which the finish makes.
            device: NewDevice {
                id: device.device_id(&account_id),
                sealed_name: None,
                proof: device.device_proof(&origin, &account_id).to_base64url(),
            },
        };
        match retrying(b, || api.passkey_signup_start(&request)) {
            Ok(answer) => {
                let handle = user_handle(&account_id);
                if answer.rp_id != rp_id
                    || answer.user.id != b64(&handle)
                    || answer.user.name != username
                {
                    return Err(AccountError::Protocol(
                        "the registration options name another account".into(),
                    ));
                }
                return Ok(PasskeySignUpStart {
                    challenge: PasskeyRegistrationChallenge {
                        rp_id,
                        origin: client_origin,
                        challenge: challenge_bytes(&answer.challenge)?,
                        user_handle: handle,
                        username: username.clone(),
                        display_name: answer.user.display_name,
                        exclude_credentials: credential_ids(&answer.exclude_credentials)?,
                        request_prf: true,
                    },
                    server,
                    origin,
                    username,
                    account_id,
                    flow_id: answer.flow_id,
                });
            }
            Err(ApiError::AccountIdTaken) if fresh_ids < 3 => fresh_ids += 1,
            Err(err) => return Err(err.into()),
        }
    }
}

/// Creates the account a [`start_passkey_sign_up`] began (steps 2–4): a new
/// data key and recovery secret, the recovery key saved in
/// `sync-recovery-key:<library>` unconfirmed before anything is sent, and,
/// when the platform answered PRF, the data key wrapped under the
/// credential's KEK. A lost answer is sent again with the identical body,
/// which the server answers idempotently. The caller saves the session,
/// binds and shows the recovery key; without `registration.prf` that screen
/// says a new device will need approval or the recovery key.
pub fn sign_up_with_passkey(
    b: &dyn Backend,
    secrets: &dyn SecretStore,
    start: &PasskeySignUpStart,
    device: &DeviceIdentity,
    registration: PasskeyRegistration,
    progress: &dyn Fn(Step),
) -> Result<SignedUp, RotationError> {
    let (server, origin) = checked_server(&start.server)?;
    if origin != start.origin {
        return Err(AccountError::KeyMismatch.into());
    }
    let account_id = start.account_id.clone();
    let username = start.username.clone();
    let mut rotation = Rotation::choose(
        secrets,
        &device.library_id,
        &server,
        &origin,
        &account_id,
        &username,
        None,
    )?;
    progress(Step::DerivingKeys);
    let data_key = DataKey::generate();
    let recovery = recovery_keys(rotation.key(), &origin, &username);
    let content = ContentKeys::derive(&data_key);
    let passkey_wrap = registration.prf().map(|prf| {
        passkey_wrapped_key(
            prf,
            &origin,
            &account_id,
            &registration.credential_id,
            &data_key,
        )
    });
    let request = PasskeySignupFinishRequest {
        flow_id: start.flow_id.clone(),
        username: username.clone(),
        account_id: account_id.clone(),
        credential_id: b64(&registration.credential_id),
        attestation_object: b64(&registration.attestation_object),
        client_data_json: b64(&registration.client_data_json),
        transports: registration.transports.clone(),
        passkey_wrapped_key: passkey_wrap,
        recovery_auth_key: recovery.auth_key.to_base64url(),
        recovery_wrapped_key: wrap(&recovery.kek, WrapPurpose::Recovery, &account_id, &data_key),
        key_id: key_id(&data_key),
        device: new_device(device, &content, &origin, &account_id),
    };
    progress(Step::Contacting);
    // The recovery key is saved, unconfirmed, before the wrap leaves.
    if let Err(err) = rotation.keep(&account_id) {
        return Err(rotation.failed(err.into(), false));
    }
    let api = b.api(&server);
    let sent = retrying_noted(b, || api.passkey_signup_finish(&request));
    match sent.result {
        Ok(answer) => {
            if answer.account_id != account_id
                || answer.username != username
                || !is_token(answer.token.expose())
                || answer.key_id != request.key_id
            {
                return Err(rotation.failed(
                    AccountError::Protocol("the server answered for another account".into()),
                    true,
                ));
            }
            let recovery_key = rotation.accepted();
            let mut session = Session::new(
                server,
                origin,
                account_id,
                username,
                request.device.id.clone(),
                answer.token,
                data_key,
            );
            session.set_library_id(&device.library_id);
            Ok(SignedUp {
                session,
                recovery_key,
            })
        }
        Err(err) => Err(rotation.failed_api(err, sent.reached)),
    }
}

// --------------------------------------------- adding and removing passkeys

/// A started passkey registration for a signed-in account.
#[derive(Clone, Debug)]
pub struct PasskeyRegistrationStart {
    pub flow_id: String,
    pub challenge: PasskeyRegistrationChallenge,
}

/// `POST /v2/account/sign-in/passkeys/start` with a password or passkey
/// proof (a provider proof can't add a way in). Counts `passkey-manage`.
pub fn start_add_passkey(
    b: &dyn Backend,
    session: &Session,
    proof: &Proof,
) -> Result<PasskeyRegistrationStart, AccountError> {
    let (rp_id, client_origin) = relying_party(b, &session.server)?;
    let api = b.api(&session.server);
    let answer = api.account_passkeys_start(
        session.token(),
        &AccountPasskeysStartRequest {
            proof: proof.to_wire(),
        },
    )?;
    if answer.rp_id != rp_id {
        return Err(AccountError::Protocol(
            "the registration options name another relying party".into(),
        ));
    }
    let handle = user_handle(&session.account_id);
    if let Some(user) = &answer.user
        && (user.id != b64(&handle) || user.name != session.username)
    {
        return Err(AccountError::Protocol(
            "the registration options name another account".into(),
        ));
    }
    let display_name = answer.user.as_ref().map_or_else(
        || session.username.clone(),
        |user| user.display_name.clone(),
    );
    Ok(PasskeyRegistrationStart {
        challenge: PasskeyRegistrationChallenge {
            rp_id,
            origin: client_origin,
            challenge: challenge_bytes(&answer.challenge)?,
            user_handle: handle,
            username: session.username.clone(),
            display_name,
            exclude_credentials: credential_ids(&answer.exclude_credentials)?,
            request_prf: true,
        },
        flow_id: answer.flow_id,
    })
}

/// Stores the credential a [`start_add_passkey`] registered, with the data
/// key wrapped under its PRF KEK when the platform answered PRF. The answer
/// says whether it can unlock a new device ([`PasskeyInfo::prf`]). A lost
/// answer whose resend finds the credential already stored counts as done.
pub fn add_passkey(
    b: &dyn Backend,
    session: &Session,
    start: &PasskeyRegistrationStart,
    registration: PasskeyRegistration,
) -> Result<PasskeyInfo, AccountError> {
    let wrapped = registration.prf().map(|prf| {
        passkey_wrapped_key(
            prf,
            &session.origin,
            &session.account_id,
            &registration.credential_id,
            session.data_key(),
        )
    });
    let request = AccountPasskeysFinishRequest {
        flow_id: start.flow_id.clone(),
        credential_id: b64(&registration.credential_id),
        attestation_object: b64(&registration.attestation_object),
        client_data_json: b64(&registration.client_data_json),
        transports: registration.transports.clone(),
        passkey_wrapped_key: wrapped,
    };
    let prf = request.passkey_wrapped_key.is_some();
    let api = b.api(&session.server);
    let sent = retrying_noted(b, || api.account_passkeys_finish(session.token(), &request));
    match sent.result {
        Ok(answer) => {
            if answer.credential_id != request.credential_id || answer.prf != prf {
                return Err(AccountError::Protocol(
                    "the server stored another credential".into(),
                ));
            }
            Ok(PasskeyInfo {
                credential_id: registration.credential_id.clone(),
                created_at: answer.created_at,
                last_used_at: None,
                prf,
                transports: registration.transports.clone(),
            })
        }
        Err(ApiError::CredentialExists) if sent.reached => Ok(PasskeyInfo {
            credential_id: registration.credential_id.clone(),
            created_at: 0,
            last_used_at: None,
            prf,
            transports: registration.transports.clone(),
        }),
        Err(err) => Err(err.into()),
    }
}

/// `POST /v2/account/sign-in/passkeys/<credential_id>/remove` with a
/// password or passkey proof, and the recovery key when it is the account's
/// last way in (409 [`AccountError::LastMethod`] without it). Other devices
/// keep their data key. A lost answer whose resend finds it gone counts as
/// done.
pub fn remove_passkey(
    b: &dyn Backend,
    session: &Session,
    credential_id: &[u8],
    proof: &Proof,
    recovery: Option<&RecoveryKey>,
) -> Result<(), AccountError> {
    let request = PasskeyProofRequest {
        proof: proof.to_wire(),
        recovery_auth_key: recovery_auth(session, recovery),
    };
    let id = b64(credential_id);
    let api = b.api(&session.server);
    let sent = retrying_noted(b, || {
        api.account_passkey_remove(session.token(), &id, &request)
    });
    match sent.result {
        Ok(()) => Ok(()),
        Err(ApiError::NotFound) if sent.reached => Ok(()),
        Err(err) => Err(err.into()),
    }
}

// -------------------------------------------------- password and rotations

/// Sets the first password of an account made with a passkey or a provider
/// (docs/ACCOUNTS.md › Account flows › Set a password): derives it, wraps the
/// data key under its KEK and makes a new recovery key, saved unconfirmed
/// before the request leaves. A provider proof also needs `recovery`, the
/// current recovery key. Other devices are signed out (`password_changed`);
/// the result carries how many and the new recovery key. An account that
/// already has a password gets [`AccountError::AlreadyHasPassword`] and uses
/// [`crate::change_password`].
pub fn set_password(
    b: &dyn Backend,
    secrets: &dyn SecretStore,
    session: &Session,
    new: &Password,
    proof: &Proof,
    recovery: Option<&RecoveryKey>,
    progress: &dyn Fn(Step),
) -> Result<PasswordChanged, RotationError> {
    check_not_username(new, &session.username)?;
    let library_id = session_library(session)?;
    let params = default_params(b)?;
    let salt = random::<16>();
    progress(Step::DerivingKeys);
    let keys = password_keys(new, &session.origin, &session.username, &salt, params)
        .map_err(AccountError::from)?;
    let mut rotation = Rotation::choose(
        secrets,
        library_id,
        &session.server,
        &session.origin,
        &session.account_id,
        &session.username,
        recovery,
    )?;
    let next = recovery_keys(rotation.key(), &session.origin, &session.username);
    let request = SignInPasswordRequest {
        proof: proof.to_wire(),
        current_recovery_auth_key: recovery_auth(session, recovery),
        kdf: params.into(),
        salt: b64(&salt),
        auth_key: keys.auth_key.to_base64url(),
        wrapped_key: wrap(
            &keys.kek,
            WrapPurpose::Password,
            &session.account_id,
            session.data_key(),
        ),
        recovery_auth_key: next.auth_key.to_base64url(),
        recovery_wrapped_key: wrap(
            &next.kek,
            WrapPurpose::Recovery,
            &session.account_id,
            session.data_key(),
        ),
    };
    progress(Step::Contacting);
    if let Err(err) = rotation.keep(&session.account_id) {
        return Err(rotation.failed(err.into(), false));
    }
    let api = b.api(&session.server);
    let sent = retrying_noted(b, || api.set_first_password(session.token(), &request));
    match sent.result {
        Ok(answer) => Ok(PasswordChanged {
            signed_out_devices: usize::try_from(answer.signed_out_devices).unwrap_or(usize::MAX),
            recovery_key: rotation.accepted(),
        }),
        Err(err) => Err(rotation.failed_api(err, sent.reached)),
    }
}

/// "New recovery key" with a proof instead of the password, which is how a
/// passwordless account replaces its key. A provider proof also needs the
/// current recovery key. The new key is saved unconfirmed before the request
/// leaves, as for [`crate::new_recovery_key`].
pub fn new_recovery_key_with_proof(
    b: &dyn Backend,
    secrets: &dyn SecretStore,
    session: &Session,
    proof: &Proof,
    recovery: Option<&RecoveryKey>,
    progress: &dyn Fn(Step),
) -> Result<RecoveryKey, RotationError> {
    let library_id = session_library(session)?;
    let mut rotation = Rotation::choose(
        secrets,
        library_id,
        &session.server,
        &session.origin,
        &session.account_id,
        &session.username,
        recovery,
    )?;
    progress(Step::DerivingKeys);
    let next = recovery_keys(rotation.key(), &session.origin, &session.username);
    let request = RecoveryKeyRequest {
        current_auth_key: None,
        proof: Some(proof.to_wire()),
        current_recovery_auth_key: recovery_auth(session, recovery),
        recovery_auth_key: next.auth_key.to_base64url(),
        recovery_wrapped_key: wrap(
            &next.kek,
            WrapPurpose::Recovery,
            &session.account_id,
            session.data_key(),
        ),
    };
    progress(Step::Contacting);
    if let Err(err) = rotation.keep(&session.account_id) {
        return Err(rotation.failed(err.into(), false));
    }
    let api = b.api(&session.server);
    let sent = retrying_noted(b, || api.replace_recovery_key(session.token(), &request));
    match sent.result {
        Ok(_) => Ok(rotation.accepted()),
        Err(err) => Err(rotation.failed_api(err, sent.reached)),
    }
}

/// Deletes the account with a proof instead of the password (a
/// passwordless account can still delete). A provider proof also needs the
/// recovery key. A typed username that isn't this account's is refused
/// before anything is sent; a retry whose answer was lost gets 401
/// `unknown`, which counts as done. The app then unbinds.
pub fn delete_account_with_proof(
    b: &dyn Backend,
    session: &Session,
    proof: &Proof,
    recovery: Option<&RecoveryKey>,
    typed_username: &str,
    progress: &dyn Fn(Step),
) -> Result<(), AccountError> {
    let typed = normalize_username(typed_username).map_err(|_| AccountError::UsernameMismatch)?;
    if typed != session.username {
        return Err(AccountError::UsernameMismatch);
    }
    let request = DeleteRequest {
        auth_key: None,
        proof: Some(proof.to_wire()),
        current_recovery_auth_key: recovery_auth(session, recovery),
        username: typed,
    };
    progress(Step::Contacting);
    let api = b.api(&session.server);
    match retrying(b, || api.delete_account(session.token(), &request)) {
        Ok(_)
        | Err(ApiError::Unauthorized {
            reason: SignOutReason::Unknown,
        }) => Ok(()),
        Err(err) => Err(err.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_proof_never_sends_the_prf_output_or_prints_a_secret() {
        let auth = AuthKey::from_bytes([7; 32]);
        let password = Proof::Password(&auth);
        assert_eq!(password.to_wire(), ProofWire::password(b64(&[7; 32])));
        assert!(!format!("{password:?}").contains(&b64(&[7; 32])));
        let passkey = Proof::Passkey {
            flow: "F".repeat(43),
            assertion: PasskeyAssertion {
                credential_id: vec![1; 16],
                authenticator_data: vec![2; 37],
                client_data_json: b"{}".to_vec(),
                signature: vec![3; 70],
                user_handle: Some(vec![4; 16]),
                prf: Some([9; 32]),
            },
        };
        let wire = serde_json::to_string(&passkey.to_wire()).unwrap();
        assert!(!wire.contains(&b64(&[9; 32])), "{wire}");
        assert!(!wire.contains("user_handle"), "{wire}");
        assert!(!passkey.is_provider());
        assert!(
            Proof::Provider {
                flow: "G".repeat(43)
            }
            .is_provider()
        );
    }

    #[test]
    fn the_local_key_wins_then_the_prf_wrap_then_locked() {
        let key = DataKey::generate();
        let id = key_id(&key);
        let prf = [5u8; 32];
        let wrapped = passkey_wrapped_key(&prf, "https://o", "a_x", b"cred", &key);
        let (how, got) =
            passkey_unlock(Some(&key), None, None, "https://o", "a_x", b"cred", &id).unwrap();
        assert_eq!(how, PasskeyUnlock::LocalKey);
        assert_eq!(key_id(&got.unwrap()), id);
        let (how, got) = passkey_unlock(
            None,
            Some(&prf),
            Some(&wrapped),
            "https://o",
            "a_x",
            b"cred",
            &id,
        )
        .unwrap();
        assert_eq!(how, PasskeyUnlock::Prf);
        assert_eq!(key_id(&got.unwrap()), id);
        let (how, got) =
            passkey_unlock(None, None, Some(&wrapped), "https://o", "a_x", b"cred", &id).unwrap();
        assert_eq!((how, got.is_none()), (PasskeyUnlock::Locked, true));
        assert_eq!(
            passkey_unlock(
                None,
                Some(&[6; 32]),
                Some(&wrapped),
                "https://o",
                "a_x",
                b"cred",
                &id
            )
            .unwrap_err(),
            CryptoError::KeyMismatch
        );
    }
}
