//! A server that answers with what it shouldn't: the client must notice
//! (docs/ACCOUNTS.md › Account flows › Sign in with a passkey). Ported from
//! the PR #65 security review.

use std::sync::Mutex;
use std::time::Duration;

use kanso_sync::api::*;
use kanso_sync::wire::{PushRequest, PushResponse, PutFileResponse, WirePage};
use kanso_sync::{
    AccountError, ApiError, Backend, DeviceIdentity, KdfParams, MemorySecrets, MemoryServer,
    PasskeyCeremony, PasskeySignIn, SessionToken, Step, TestAuthenticator,
};

const SERVER: &str = kanso_sync::MEMORY_PASSKEY_SERVER;

fn mac(n: u8) -> DeviceIdentity {
    DeviceIdentity::from_parts(&format!("l_{:032x}", n), &format!("Mac {n}"), [n; 32])
}

fn quiet(_: Step) {}

/// Once armed, answers every passkey finish with a captured session of
/// another account.
struct Swapping {
    server: MemoryServer,
    captured: Mutex<Option<PasskeySessionResponse>>,
    armed: Mutex<bool>,
}

struct SwappingApi<'a> {
    inner: Box<dyn Api + 'a>,
    evil: &'a Swapping,
}

impl Backend for Swapping {
    fn api(&self, server: &str) -> Box<dyn Api + '_> {
        Box::new(SwappingApi {
            inner: self.server.api(server),
            evil: self,
        })
    }
    fn kdf_floor(&self) -> KdfParams {
        KdfParams::TEST
    }
    fn kdf_default(&self) -> KdfParams {
        KdfParams::TEST
    }
    fn retry_pause(&self, _: usize) -> Duration {
        Duration::ZERO
    }
}

impl Api for SwappingApi<'_> {
    fn passkey_start(&self, r: &PasskeyStartRequest) -> Result<PasskeyStartResponse, ApiError> {
        self.inner.passkey_start(r)
    }
    fn passkey_signup_start(
        &self,
        r: &PasskeySignupStartRequest,
    ) -> Result<PasskeySignupStartResponse, ApiError> {
        self.inner.passkey_signup_start(r)
    }
    fn passkey_signup_finish(
        &self,
        r: &PasskeySignupFinishRequest,
    ) -> Result<PasskeySessionResponse, ApiError> {
        self.inner.passkey_signup_finish(r)
    }
    fn passkey_finish(&self, r: &PasskeyFinishRequest) -> Result<PasskeySessionResponse, ApiError> {
        let real = self.inner.passkey_finish(r)?;
        if *self.evil.armed.lock().unwrap() {
            return Ok(self.evil.captured.lock().unwrap().clone().unwrap());
        }
        *self.evil.captured.lock().unwrap() = Some(real.clone());
        Ok(real)
    }
    fn health(&self) -> Result<Health, ApiError> {
        self.inner.health()
    }
    fn prelogin(&self, r: &PreloginRequest) -> Result<PreloginResponse, ApiError> {
        self.inner.prelogin(r)
    }
    fn signup(&self, r: &SignupRequest) -> Result<SessionResponse, ApiError> {
        self.inner.signup(r)
    }
    fn login(&self, r: &LoginRequest) -> Result<LoginResponse, ApiError> {
        self.inner.login(r)
    }
    fn recover_start(&self, r: &RecoverStartRequest) -> Result<RecoverStartResponse, ApiError> {
        self.inner.recover_start(r)
    }
    fn recover_finish(&self, r: &RecoverFinishRequest) -> Result<SessionResponse, ApiError> {
        self.inner.recover_finish(r)
    }
    fn logout(&self, t: &SessionToken) -> Result<(), ApiError> {
        self.inner.logout(t)
    }
    fn account(&self, t: &SessionToken) -> Result<AccountResponse, ApiError> {
        self.inner.account(t)
    }
    fn change_password(
        &self,
        t: &SessionToken,
        r: &PasswordRequest,
    ) -> Result<PasswordResponse, ApiError> {
        self.inner.change_password(t, r)
    }
    fn replace_recovery_key(
        &self,
        t: &SessionToken,
        r: &RecoveryKeyRequest,
    ) -> Result<RecoveryKeyResponse, ApiError> {
        self.inner.replace_recovery_key(t, r)
    }
    fn delete_account(
        &self,
        t: &SessionToken,
        r: &DeleteRequest,
    ) -> Result<DeleteResponse, ApiError> {
        self.inner.delete_account(t, r)
    }
    fn set_device_name(&self, t: &SessionToken, r: &DeviceNameRequest) -> Result<(), ApiError> {
        self.inner.set_device_name(t, r)
    }
    fn devices(&self, t: &SessionToken) -> Result<DevicesResponse, ApiError> {
        self.inner.devices(t)
    }
    fn revoke_device(&self, t: &SessionToken, d: &str) -> Result<(), ApiError> {
        self.inner.revoke_device(t, d)
    }
    fn push(&self, t: &SessionToken, r: &PushRequest) -> Result<PushResponse, ApiError> {
        self.inner.push(t, r)
    }
    fn changes(
        &self,
        t: &SessionToken,
        s: Option<i64>,
        l: Option<usize>,
    ) -> Result<WirePage, ApiError> {
        self.inner.changes(t, s, l)
    }
    fn put_file(&self, t: &SessionToken, w: &str, s: &[u8]) -> Result<PutFileResponse, ApiError> {
        self.inner.put_file(t, w, s)
    }
    fn get_file(&self, t: &SessionToken, w: &str) -> Result<Option<Vec<u8>>, ApiError> {
        self.inner.get_file(t, w)
    }
}

fn sign_up(b: &Swapping, auth: &TestAuthenticator, name: &str, m: u8) -> String {
    let start = kanso_sync::start_passkey_sign_up(b, SERVER, name, &mac(m)).unwrap();
    let registration = auth.register(&start.challenge).unwrap();
    kanso_sync::sign_up_with_passkey(
        b,
        &MemorySecrets::new(),
        &start,
        &mac(m),
        registration,
        &quiet,
    )
    .map_err(|e| e.error)
    .unwrap()
    .session
    .account_id
    .clone()
}

/// On a new Mac a passkey without PRF comes back locked, and the account it
/// names is the one the password will unlock. A server that answers with
/// another account than the user handle the authenticator returned is
/// refused, and the session it handed over is signed out.
#[test]
fn a_session_for_another_account_than_the_user_handle_is_refused() {
    let evil = Swapping {
        server: MemoryServer::new(),
        captured: Mutex::new(None),
        armed: Mutex::new(false),
    };
    let alice = TestAuthenticator::new(1);
    let mallory = TestAuthenticator::new(2).without_prf();
    let alice_id = sign_up(&evil, &alice, "alice", 1);
    let mallory_id = sign_up(&evil, &mallory, "mallory", 2);
    assert_ne!(alice_id, mallory_id);

    // The handle the authenticator returns is the one it registered with.
    let s =
        kanso_sync::start_passkey_sign_in(&evil, SERVER, None, &LoginDevice::ephemeral()).unwrap();
    let a = mallory.assert(&s.challenge).unwrap();
    assert_eq!(
        a.user_handle,
        Some(kanso_sync::passkey::user_handle(&mallory_id))
    );
    // Capture a real session of mallory's account.
    let PasskeySignIn::Locked(_) =
        kanso_sync::sign_in_with_passkey(&evil, &MemorySecrets::new(), &s, &mac(9), a, &quiet)
            .unwrap()
    else {
        panic!("mallory's passkey has no PRF");
    };
    let captured = evil.captured.lock().unwrap().clone().unwrap();
    *evil.armed.lock().unwrap() = true;

    // Alice signs in on a new Mac with her own passkey.
    let s =
        kanso_sync::start_passkey_sign_in(&evil, SERVER, None, &LoginDevice::ephemeral()).unwrap();
    let a = alice.assert(&s.challenge).unwrap();
    assert_eq!(
        a.user_handle,
        Some(kanso_sync::passkey::user_handle(&alice_id))
    );
    let out =
        kanso_sync::sign_in_with_passkey(&evil, &MemorySecrets::new(), &s, &mac(3), a, &quiet);
    assert!(
        matches!(out, Err(AccountError::Protocol(_))),
        "{:?}",
        out.as_ref().map(|o| match o {
            PasskeySignIn::Locked(l) => format!("Locked({})", l.username),
            PasskeySignIn::Unlocked(_) => "Unlocked".into(),
        })
    );
    // The handed-over session is signed out.
    assert!(matches!(
        evil.server.api(SERVER).account(&captured.token),
        Err(ApiError::Unauthorized { .. })
    ));
}
