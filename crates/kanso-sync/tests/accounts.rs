//! Account flows against `MemoryServer` and `MemorySecrets`
//! (docs/ACCOUNTS.md › Account flows, › Keychain items): sign-up, sign-in,
//! recovery, password change, new recovery key, deletion, devices, the
//! documented retries after lost responses, the KDF floor and ceilings, and
//! the Keychain items. Tests never touch the real Keychain. CI pins the seed
//! with `PROPTEST_RNG_SEED`.

use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use kanso_sync::api::{
    AccountResponse, Api, DeleteRequest, DeleteResponse, DeviceNameRequest, DevicesResponse,
    Health, LoginRequest, LoginResponse, PasswordRequest, PasswordResponse, PreloginRequest,
    PreloginResponse, RecoverFinishRequest, RecoverStartRequest, RecoverStartResponse,
    RecoveryKeyRequest, RecoveryKeyResponse, SessionResponse, SignupRequest, WireKdf,
};
use kanso_sync::keychain::{self, PendingLogout, PendingRecoveryKey};
use kanso_sync::wire::{PushRequest, PushResponse, PutFileResponse, WirePage};
use kanso_sync::{
    AccountError, ApiError, Backend, DeviceIdentity, KdfParams, MemorySecrets, MemoryServer,
    Password, RecoveryKey, Route, SecretStore, Session, SessionToken, SignOutReason, SignupState,
    Step,
};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;

const SERVER: &str = "http://127.0.0.1:8787";
const ORIGIN: &str = "http://127.0.0.1:8787";

fn config(cases: u32) -> ProptestConfig {
    ProptestConfig {
        cases,
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
        ..ProptestConfig::default()
    }
}

/// The rotating account functions with a Keychain of their own each time
/// and the error alone: for the tests that aren't about the new recovery
/// key's item.
mod plain {
    use kanso_sync::{
        AccountError, Backend, DeviceIdentity, MemorySecrets, Password, PasswordChanged, Recovered,
        RecoveryKey, Session, SignedUp, Step,
    };

    pub fn change_password(
        b: &dyn Backend,
        session: &Session,
        current: &Password,
        new: &Password,
        progress: &dyn Fn(Step),
    ) -> Result<PasswordChanged, AccountError> {
        let secrets = MemorySecrets::new();
        kanso_sync::change_password(b, &secrets, session, current, new, progress)
            .map_err(|err| err.error)
    }

    pub fn sign_up(
        b: &dyn Backend,
        server: &str,
        username: &str,
        password: &Password,
        device: &DeviceIdentity,
        progress: &dyn Fn(Step),
    ) -> Result<SignedUp, AccountError> {
        let secrets = MemorySecrets::new();
        kanso_sync::sign_up(b, &secrets, server, username, password, device, progress)
            .map_err(|err| err.error)
    }

    pub fn recover(
        b: &dyn Backend,
        server: &str,
        username: &str,
        key: &RecoveryKey,
        password: &Password,
        device: &DeviceIdentity,
        progress: &dyn Fn(Step),
    ) -> Result<Recovered, AccountError> {
        let secrets = MemorySecrets::new();
        kanso_sync::recover(
            b, &secrets, server, username, key, password, device, progress,
        )
        .map_err(|err| err.error)
    }

    pub fn new_recovery_key(
        b: &dyn Backend,
        session: &Session,
        current: &Password,
        progress: &dyn Fn(Step),
    ) -> Result<RecoveryKey, AccountError> {
        let secrets = MemorySecrets::new();
        kanso_sync::new_recovery_key(b, &secrets, session, current, progress)
            .map_err(|err| err.error)
    }
}

/// `MemoryServer` without the pause before a lost request is sent again.
struct Quick(MemoryServer);

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

fn mac(n: u8) -> DeviceIdentity {
    DeviceIdentity::from_parts(&format!("l_{:032x}", n), &format!("Mac {n}"), [n; 32])
}

fn password(text: &str) -> Password {
    Password::new(text, "alice").unwrap()
}

fn quiet(_: Step) {}

fn sign_up(b: &dyn Backend, username: &str, device: &DeviceIdentity) -> kanso_sync::SignedUp {
    plain::sign_up(
        b,
        SERVER,
        username,
        &password("correct horse battery"),
        device,
        &quiet,
    )
    .unwrap()
}

fn sign_in(
    b: &dyn Backend,
    username: &str,
    text: &str,
    device: &DeviceIdentity,
) -> Result<kanso_sync::SignedIn, AccountError> {
    kanso_sync::sign_in(b, SERVER, username, &password(text), device, &quiet)
}

/// What a session's own token gets from the server now.
fn check(server: &MemoryServer, session: &Session) -> Result<AccountResponse, ApiError> {
    let token = token_of(server, session);
    server.api(SERVER).account(&token)
}

/// The session's token, read back through the Keychain item format.
fn token_of(_server: &MemoryServer, session: &Session) -> SessionToken {
    let secrets = MemorySecrets::new();
    keychain::save_session(&secrets, "l_00000000000000000000000000000000", session).unwrap();
    let json = secrets
        .get(&keychain::account_item(
            "l_00000000000000000000000000000000",
        ))
        .unwrap()
        .unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    SessionToken::new(value["token"].as_str().unwrap())
}

#[test]
fn sign_up_then_sign_in_on_another_mac() {
    let server = MemoryServer::new();
    server.set_now(1_800_000_000_000);
    assert_eq!(
        kanso_sync::server_info(&server, SERVER).unwrap(),
        SignupState::Open
    );
    let steps = Mutex::new(Vec::new());
    let up = plain::sign_up(
        &server,
        SERVER,
        " Alice ",
        &password("correct horse battery"),
        &mac(1),
        &|step| steps.lock().unwrap().push(step),
    )
    .unwrap();
    assert!(steps.lock().unwrap().contains(&Step::DerivingKeys));
    let a = &up.session;
    assert_eq!(a.username, "alice");
    assert_eq!(a.origin, ORIGIN);
    assert_eq!(a.device_id, mac(1).device_id(&a.account_id));
    assert_eq!(a.library_id(), Some(mac(1).library_id.as_str()));
    assert_eq!(check(&server, a).unwrap().account_id, a.account_id);
    // The recovery key's text parses back.
    assert_eq!(
        RecoveryKey::parse(&up.recovery_key.display()).unwrap(),
        up.recovery_key
    );

    server.set_now(1_800_000_001_000);
    let b = sign_in(&server, "ALICE", "correct horse battery", &mac(2)).unwrap();
    assert!(b.evicted.is_none());
    assert_eq!(b.session.account_id, a.account_id);
    assert_ne!(b.session.device_id, a.device_id);
    // Both Macs hold the same data key: one Mac's sealed name opens on the other.
    let devices = kanso_sync::devices(&server, &b.session).unwrap();
    let names: Vec<(Option<String>, bool)> = devices
        .iter()
        .map(|d| (d.name.clone(), d.current))
        .collect();
    assert_eq!(
        names,
        vec![(Some("Mac 2".into()), true), (Some("Mac 1".into()), false)]
    );

    assert!(matches!(
        sign_in(&server, "alice", "wrong horse battery", &mac(3)),
        Err(AccountError::InvalidCredentials)
    ));
    assert!(matches!(
        sign_in(&server, "nobody", "correct horse battery", &mac(3)),
        Err(AccountError::InvalidCredentials)
    ));
    assert!(matches!(
        plain::sign_up(
            &server,
            SERVER,
            "alice",
            &password("another long one"),
            &mac(3),
            &quiet
        ),
        Err(AccountError::UsernameTaken)
    ));
    assert!(matches!(
        plain::sign_up(
            &server,
            SERVER,
            "admin",
            &password("another long one"),
            &mac(3),
            &quiet
        ),
        Err(AccountError::Username(_))
    ));
    assert!(matches!(
        plain::sign_up(
            &server,
            SERVER,
            "alice",
            &password("another long one"),
            &mac(3),
            &quiet
        ),
        Err(AccountError::UsernameTaken)
    ));
    let usage = kanso_sync::usage(&server, a).unwrap();
    assert_eq!(usage.docs, 0);
    assert_eq!(usage.quota.docs, 50_000);
}

#[test]
fn sign_up_is_refused_offline_and_when_closed() {
    let server = MemoryServer::new();
    server.set_signup(kanso_sync::Signup::Closed);
    assert_eq!(
        kanso_sync::server_info(&server, SERVER).unwrap(),
        SignupState::Closed
    );
    assert!(matches!(
        plain::sign_up(
            &server,
            SERVER,
            "alice",
            &password("correct horse battery"),
            &mac(1),
            &quiet
        ),
        Err(AccountError::SignupClosed { .. })
    ));
    // Offline checks send nothing.
    let before = server.stats();
    assert!(matches!(
        plain::sign_up(
            &server,
            SERVER,
            "a",
            &password("correct horse battery"),
            &mac(1),
            &quiet
        ),
        Err(AccountError::Username(_))
    ));
    assert!(matches!(
        plain::sign_up(
            &server,
            "http://example.com",
            "alice",
            &password("correct horse battery"),
            &mac(1),
            &quiet
        ),
        Err(AccountError::Endpoint(_))
    ));
    assert!(matches!(
        plain::sign_up(
            &server,
            SERVER,
            "correct-horse-bat",
            &Password::new("Correct-Horse-Bat", "x").unwrap(),
            &mac(1),
            &quiet
        ),
        Err(AccountError::Password(_))
    ));
    assert_eq!(server.stats(), before);
}

#[test]
fn recovery_rotates_the_key_and_signs_everyone_out() {
    let server = MemoryServer::new();
    let up = sign_up(&server, "alice", &mac(1));
    let other = sign_in(&server, "alice", "correct horse battery", &mac(2))
        .unwrap()
        .session;
    let recovered = plain::recover(
        &server,
        SERVER,
        "alice",
        &up.recovery_key,
        &password("a brand new passphrase"),
        &mac(3),
        &quiet,
    )
    .unwrap();
    assert_ne!(recovered.recovery_key, up.recovery_key);
    assert_eq!(recovered.session.account_id, up.session.account_id);
    for old in [&up.session, &other] {
        assert_eq!(
            check(&server, old).unwrap_err(),
            ApiError::Unauthorized {
                reason: SignOutReason::Recovered
            }
        );
    }
    assert!(check(&server, &recovered.session).is_ok());
    // The used key stops working; the new one works; the new password signs in.
    assert!(matches!(
        plain::recover(
            &server,
            SERVER,
            "alice",
            &up.recovery_key,
            &password("yet another passphrase"),
            &mac(3),
            &quiet
        ),
        Err(AccountError::InvalidCredentials)
    ));
    assert!(sign_in(&server, "alice", "correct horse battery", &mac(4)).is_err());
    let back = sign_in(&server, "alice", "a brand new passphrase", &mac(4)).unwrap();
    // The data key survived recovery: names sealed before it still open.
    let names: Vec<Option<String>> = kanso_sync::devices(&server, &back.session)
        .unwrap()
        .into_iter()
        .map(|d| d.name)
        .collect();
    assert!(names.contains(&Some("Mac 3".into())));
    plain::recover(
        &server,
        SERVER,
        "alice",
        &recovered.recovery_key,
        &password("yet another passphrase"),
        &mac(3),
        &quiet,
    )
    .unwrap();
}

#[test]
fn a_password_change_signs_out_the_others_and_keeps_the_data_key() {
    let server = MemoryServer::new();
    let up = sign_up(&server, "alice", &mac(1));
    let other = sign_in(&server, "alice", "correct horse battery", &mac(2))
        .unwrap()
        .session;
    let third = sign_in(&server, "alice", "correct horse battery", &mac(3))
        .unwrap()
        .session;
    assert!(matches!(
        plain::change_password(
            &server,
            &up.session,
            &password("wrong horse battery"),
            &password("new long passphrase"),
            &quiet
        ),
        Err(AccountError::InvalidCredentials)
    ));
    let changed = plain::change_password(
        &server,
        &up.session,
        &password("correct horse battery"),
        &password("new long passphrase"),
        &quiet,
    )
    .unwrap();
    assert_eq!(changed.signed_out_devices, 2);
    for gone in [&other, &third] {
        assert_eq!(
            check(&server, gone).unwrap_err(),
            ApiError::Unauthorized {
                reason: SignOutReason::PasswordChanged
            }
        );
    }
    assert!(check(&server, &up.session).is_ok());
    assert!(sign_in(&server, "alice", "correct horse battery", &mac(2)).is_err());
    let again = sign_in(&server, "alice", "new long passphrase", &mac(2)).unwrap();
    let names: Vec<Option<String>> = kanso_sync::devices(&server, &again.session)
        .unwrap()
        .into_iter()
        .map(|d| d.name)
        .collect();
    assert!(names.contains(&Some("Mac 1".into())), "{names:?}");
}

#[test]
fn a_new_recovery_key_replaces_the_old_one() {
    let server = MemoryServer::new();
    let up = sign_up(&server, "alice", &mac(1));
    let key = plain::new_recovery_key(
        &server,
        &up.session,
        &password("correct horse battery"),
        &quiet,
    )
    .unwrap();
    assert!(check(&server, &up.session).is_ok());
    assert!(matches!(
        plain::recover(
            &server,
            SERVER,
            "alice",
            &up.recovery_key,
            &password("another long one"),
            &mac(2),
            &quiet
        ),
        Err(AccountError::InvalidCredentials)
    ));
    plain::recover(
        &server,
        SERVER,
        "alice",
        &key,
        &password("another long one"),
        &mac(2),
        &quiet,
    )
    .unwrap();
}

#[test]
fn deleting_the_account() {
    let quick = Quick(MemoryServer::new());
    let server = &quick.0;
    let up = sign_up(&quick, "alice", &mac(1));
    let other = sign_in(&quick, "alice", "correct horse battery", &mac(2))
        .unwrap()
        .session;
    // A typed username that isn't the account's is refused before sending.
    assert!(matches!(
        kanso_sync::delete_account(
            &quick,
            &up.session,
            &password("correct horse battery"),
            "bob",
            &quiet
        ),
        Err(AccountError::UsernameMismatch)
    ));
    assert_eq!(server.stats().count(Route::Delete), 0);
    assert!(matches!(
        kanso_sync::delete_account(
            &quick,
            &up.session,
            &password("wrong horse battery"),
            "alice",
            &quiet
        ),
        Err(AccountError::InvalidCredentials)
    ));
    let before = server.stats().count(Route::Delete);
    // The answer is lost; the retry gets 401 unknown, which is success.
    server.drop_next_responses(Route::Delete, 1);
    kanso_sync::delete_account(
        &quick,
        &up.session,
        &password("correct horse battery"),
        " ALICE ",
        &quiet,
    )
    .unwrap();
    assert_eq!(server.stats().count(Route::Delete), before + 2);
    assert_eq!(
        check(server, &other).unwrap_err(),
        ApiError::Unauthorized {
            reason: SignOutReason::Unknown
        }
    );
    assert!(server.account_id("alice").is_none());
    // Any other 401 means the session ended first: that is reported.
    let again = sign_up(&quick, "alice", &mac(1));
    server.revoke_all("alice", SignOutReason::Revoked);
    assert!(matches!(
        kanso_sync::delete_account(
            &quick,
            &again.session,
            &password("correct horse battery"),
            "alice",
            &quiet
        ),
        Err(AccountError::Unauthorized {
            reason: SignOutReason::Revoked
        })
    ));
}

#[test]
fn devices_eviction_and_revocation() {
    let server = MemoryServer::new();
    server.set_now(1_800_000_000_000);
    let first = sign_up(&server, "alice", &mac(1)).session;
    let mut sessions = vec![first];
    for n in 2..=10u8 {
        server.set_now(1_800_000_000_000 + i64::from(n) * 1000);
        let signed = sign_in(&server, "alice", "correct horse battery", &mac(n)).unwrap();
        assert!(signed.evicted.is_none());
        sessions.push(signed.session);
    }
    server.set_now(1_800_000_100_000);
    let eleventh = sign_in(&server, "alice", "correct horse battery", &mac(11)).unwrap();
    let evicted = eleventh
        .evicted
        .expect("the least recently seen Mac is signed out");
    assert_eq!(evicted.id, sessions[0].device_id);
    assert_eq!(evicted.name.as_deref(), Some("Mac 1"));
    assert_eq!(
        check(&server, &sessions[0]).unwrap_err(),
        ApiError::Unauthorized {
            reason: SignOutReason::Evicted
        }
    );
    let list = kanso_sync::devices(&server, &eleventh.session).unwrap();
    assert_eq!(list.len(), 10);
    assert!(
        list.windows(2)
            .all(|w| w[0].last_seen_at >= w[1].last_seen_at)
    );
    assert!(list[0].current);
    kanso_sync::revoke_device(&server, &eleventh.session, &sessions[1].device_id).unwrap();
    assert_eq!(
        check(&server, &sessions[1]).unwrap_err(),
        ApiError::Unauthorized {
            reason: SignOutReason::Revoked
        }
    );
    // Revoking it again, or a device already signed out another way (the
    // evicted one), succeeds and marks it revoked, so the owner can always
    // say "that Mac is gone" (security review 4).
    kanso_sync::revoke_device(&server, &eleventh.session, &sessions[1].device_id).unwrap();
    kanso_sync::revoke_device(&server, &eleventh.session, &sessions[0].device_id).unwrap();
    assert_eq!(
        check(&server, &sessions[0]).unwrap_err(),
        ApiError::Unauthorized {
            reason: SignOutReason::Revoked
        }
    );
    assert_eq!(
        kanso_sync::devices(&server, &eleventh.session)
            .unwrap()
            .len(),
        9
    );
}

#[test]
fn lost_responses_are_recovered_by_the_documented_retry() {
    let quick = Quick(MemoryServer::new());
    let server = &quick.0;
    server.drop_next_responses(Route::Signup, 1);
    let up = sign_up(&quick, "alice", &mac(1));
    assert_eq!(server.stats().count(Route::Signup), 2);
    assert!(check(server, &up.session).is_ok());

    server.drop_next_responses(Route::Password, 1);
    plain::change_password(
        &quick,
        &up.session,
        &password("correct horse battery"),
        &password("new long passphrase"),
        &quiet,
    )
    .unwrap();
    assert_eq!(server.stats().count(Route::Password), 2);
    assert!(sign_in(&quick, "alice", "new long passphrase", &mac(2)).is_ok());

    server.drop_next_responses(Route::RecoveryKey, 1);
    let key = plain::new_recovery_key(
        &quick,
        &up.session,
        &password("new long passphrase"),
        &quiet,
    )
    .unwrap();
    assert_eq!(server.stats().count(Route::RecoveryKey), 2);

    server.drop_next_responses(Route::RecoverFinish, 1);
    let recovered = plain::recover(
        &quick,
        SERVER,
        "alice",
        &key,
        &password("third long passphrase"),
        &mac(3),
        &quiet,
    )
    .unwrap();
    assert_eq!(server.stats().count(Route::RecoverFinish), 2);
    assert!(check(server, &recovered.session).is_ok());
    plain::recover(
        &quick,
        SERVER,
        "alice",
        &recovered.recovery_key,
        &password("fourth long passphrase"),
        &mac(3),
        &quiet,
    )
    .unwrap();

    // Three lost answers in a row surface as a transport error.
    server.drop_next_responses(Route::Signup, 3);
    assert!(matches!(
        plain::sign_up(
            &quick,
            SERVER,
            "bob",
            &password("correct horse battery"),
            &mac(5),
            &quiet
        ),
        Err(AccountError::Transport(_))
    ));
    assert_eq!(server.stats().count(Route::Signup), 5);
}

// ---------------------------------------------------------------------------
// A server that misbehaves on purpose
// ---------------------------------------------------------------------------

/// Wraps `MemoryServer` and changes chosen answers.
#[derive(Default)]
struct Faults {
    /// Sign-ups answered `account_id_taken` without being applied.
    taken: AtomicUsize,
    /// The prelogin answer's parameters.
    prelogin_kdf: Mutex<Option<WireKdf>>,
    /// The login answer's key ID.
    login_key_id: Mutex<Option<String>>,
    /// Every request fails with a transport error, unapplied.
    unreachable: std::sync::atomic::AtomicBool,
    /// Every request fails before it leaves the Mac (`ApiError::unsent`).
    unsent: std::sync::atomic::AtomicBool,
    /// Health answers 404, like a v1 Worker.
    outdated: std::sync::atomic::AtomicBool,
    logins: AtomicUsize,
    logouts: AtomicUsize,
    /// Every password change sent, as sent.
    passwords: Mutex<Vec<PasswordRequest>>,
    /// The next logout says so on the first channel, then waits for the
    /// second before it reaches the server.
    logout_gate: Mutex<Option<(std::sync::mpsc::Sender<()>, std::sync::mpsc::Receiver<()>)>>,
    /// A server with the backend's floor and default.
    floor: Mutex<Option<KdfParams>>,
    /// The route's first request reaches the server; every later one fails
    /// with the error made here, unapplied (and how many went so far).
    after_first: Mutex<Option<AfterFirst>>,
}

/// See [`Faults::after_first`].
type AfterFirst = (Route, fn() -> ApiError, usize);

struct Wrapped<'a> {
    inner: Box<dyn Api + 'a>,
    faults: &'a Faults,
}

struct Hostile {
    server: MemoryServer,
    faults: Faults,
}

impl Backend for Hostile {
    fn api(&self, server: &str) -> Box<dyn Api + '_> {
        Box::new(Wrapped {
            inner: self.server.api(server),
            faults: &self.faults,
        })
    }

    fn kdf_floor(&self) -> KdfParams {
        self.faults.floor.lock().unwrap().unwrap_or(KdfParams::TEST)
    }

    fn kdf_default(&self) -> KdfParams {
        KdfParams::TEST
    }

    fn retry_pause(&self, _: usize) -> Duration {
        Duration::ZERO
    }
}

impl Wrapped<'_> {
    fn reach(&self) -> Result<(), ApiError> {
        if self.faults.unsent.load(Ordering::SeqCst) {
            Err(ApiError::unsent("connection refused"))
        } else if self.faults.unreachable.load(Ordering::SeqCst) {
            Err(ApiError::Transport("unreachable".into()))
        } else {
            Ok(())
        }
    }

    /// [`Faults::after_first`] for a request on `route`.
    fn reach_route(&self, route: Route) -> Result<(), ApiError> {
        self.reach()?;
        let mut after_first = self.faults.after_first.lock().unwrap();
        match after_first.as_mut() {
            Some((on, make, sent)) if *on == route => {
                *sent += 1;
                if *sent > 1 { Err(make()) } else { Ok(()) }
            }
            _ => Ok(()),
        }
    }
}

impl Api for Wrapped<'_> {
    fn health(&self) -> Result<Health, ApiError> {
        self.reach()?;
        if self.faults.outdated.load(Ordering::SeqCst) {
            return Err(ApiError::NotFound);
        }
        self.inner.health()
    }
    fn prelogin(&self, request: &PreloginRequest) -> Result<PreloginResponse, ApiError> {
        self.reach()?;
        let mut answer = self.inner.prelogin(request)?;
        if let Some(kdf) = self.faults.prelogin_kdf.lock().unwrap().clone() {
            answer.kdf = kdf;
        }
        Ok(answer)
    }
    fn signup(&self, request: &SignupRequest) -> Result<SessionResponse, ApiError> {
        self.reach_route(Route::Signup)?;
        if self.faults.taken.load(Ordering::SeqCst) > 0 {
            self.faults.taken.fetch_sub(1, Ordering::SeqCst);
            return Err(ApiError::AccountIdTaken);
        }
        self.inner.signup(request)
    }
    fn login(&self, request: &LoginRequest) -> Result<LoginResponse, ApiError> {
        self.reach()?;
        self.faults.logins.fetch_add(1, Ordering::SeqCst);
        let mut answer = self.inner.login(request)?;
        if let Some(key_id) = self.faults.login_key_id.lock().unwrap().clone() {
            answer.key_id = key_id;
        }
        Ok(answer)
    }
    fn recover_start(
        &self,
        request: &RecoverStartRequest,
    ) -> Result<RecoverStartResponse, ApiError> {
        self.reach()?;
        self.inner.recover_start(request)
    }
    fn recover_finish(&self, request: &RecoverFinishRequest) -> Result<SessionResponse, ApiError> {
        self.reach_route(Route::RecoverFinish)?;
        self.inner.recover_finish(request)
    }
    fn logout(&self, token: &SessionToken) -> Result<(), ApiError> {
        self.reach()?;
        self.faults.logouts.fetch_add(1, Ordering::SeqCst);
        let gate = self.faults.logout_gate.lock().unwrap().take();
        if let Some((entered, release)) = gate {
            entered.send(()).unwrap();
            release.recv().unwrap();
        }
        self.inner.logout(token)
    }
    fn account(&self, token: &SessionToken) -> Result<AccountResponse, ApiError> {
        self.reach()?;
        self.inner.account(token)
    }
    fn change_password(
        &self,
        token: &SessionToken,
        request: &PasswordRequest,
    ) -> Result<PasswordResponse, ApiError> {
        self.reach_route(Route::Password)?;
        self.faults.passwords.lock().unwrap().push(request.clone());
        self.inner.change_password(token, request)
    }
    fn replace_recovery_key(
        &self,
        token: &SessionToken,
        request: &RecoveryKeyRequest,
    ) -> Result<RecoveryKeyResponse, ApiError> {
        self.reach_route(Route::RecoveryKey)?;
        self.inner.replace_recovery_key(token, request)
    }
    fn delete_account(
        &self,
        token: &SessionToken,
        request: &DeleteRequest,
    ) -> Result<DeleteResponse, ApiError> {
        self.reach()?;
        self.inner.delete_account(token, request)
    }
    fn set_device_name(
        &self,
        token: &SessionToken,
        request: &DeviceNameRequest,
    ) -> Result<(), ApiError> {
        self.reach()?;
        self.inner.set_device_name(token, request)
    }
    fn devices(&self, token: &SessionToken) -> Result<DevicesResponse, ApiError> {
        self.reach()?;
        self.inner.devices(token)
    }
    fn revoke_device(&self, token: &SessionToken, device_id: &str) -> Result<(), ApiError> {
        self.reach()?;
        self.inner.revoke_device(token, device_id)
    }
    fn push(&self, token: &SessionToken, request: &PushRequest) -> Result<PushResponse, ApiError> {
        self.reach()?;
        self.inner.push(token, request)
    }
    fn changes(
        &self,
        token: &SessionToken,
        since: Option<i64>,
        limit: Option<usize>,
    ) -> Result<WirePage, ApiError> {
        self.reach()?;
        self.inner.changes(token, since, limit)
    }
    fn put_file(
        &self,
        token: &SessionToken,
        wire_id: &str,
        sealed: &[u8],
    ) -> Result<PutFileResponse, ApiError> {
        self.reach()?;
        self.inner.put_file(token, wire_id, sealed)
    }
    fn get_file(&self, token: &SessionToken, wire_id: &str) -> Result<Option<Vec<u8>>, ApiError> {
        self.reach()?;
        self.inner.get_file(token, wire_id)
    }
}

fn hostile() -> Hostile {
    Hostile {
        server: MemoryServer::new(),
        faults: Faults::default(),
    }
}

#[test]
fn a_taken_account_id_is_replaced_and_the_keys_rewrapped() {
    let b = hostile();
    b.faults.taken.store(2, Ordering::SeqCst);
    let up = sign_up(&b, "alice", &mac(1));
    assert_eq!(
        b.server.account_id("alice").as_deref(),
        Some(up.session.account_id.as_str())
    );
    // The wraps were made for the account ID that was stored: sign-in unwraps.
    sign_in(&b, "alice", "correct horse battery", &mac(2)).unwrap();
    b.faults.taken.store(10, Ordering::SeqCst);
    assert!(matches!(
        plain::sign_up(
            &b,
            SERVER,
            "bob",
            &password("correct horse battery"),
            &mac(3),
            &quiet
        ),
        Err(AccountError::AccountIdTaken)
    ));
}

#[test]
fn a_different_key_id_is_a_key_mismatch_and_logs_the_session_out() {
    let b = hostile();
    sign_up(&b, "alice", &mac(1));
    *b.faults.login_key_id.lock().unwrap() = Some("0000000000000000".into());
    assert!(matches!(
        sign_in(&b, "alice", "correct horse battery", &mac(2)),
        Err(AccountError::KeyMismatch)
    ));
    assert_eq!(b.faults.logouts.load(Ordering::SeqCst), 1);
    let rows = b.server.devices("alice");
    let second = rows
        .iter()
        .find(|d| d.id == mac(2).device_id(&b.server.account_id("alice").unwrap()))
        .unwrap();
    assert_eq!(second.revoked, Some(SignOutReason::SignedOut));
}

#[test]
fn an_outdated_server_and_an_unreachable_one() {
    let b = hostile();
    b.faults.outdated.store(true, Ordering::SeqCst);
    assert!(matches!(
        kanso_sync::server_info(&b, SERVER),
        Err(AccountError::ServerOutdated)
    ));
    assert!(matches!(
        plain::sign_up(
            &b,
            SERVER,
            "alice",
            &password("correct horse battery"),
            &mac(1),
            &quiet
        ),
        Err(AccountError::ServerOutdated)
    ));
    b.faults.outdated.store(false, Ordering::SeqCst);
    b.faults.unreachable.store(true, Ordering::SeqCst);
    assert!(matches!(
        kanso_sync::server_info(&b, SERVER),
        Err(AccountError::Transport(_))
    ));
}

#[test]
fn the_production_floor_refuses_test_parameters() {
    let b = hostile();
    sign_up(&b, "alice", &mac(1));
    *b.faults.floor.lock().unwrap() = Some(KdfParams::FLOOR);
    assert!(matches!(
        sign_in(&b, "alice", "correct horse battery", &mac(2)),
        Err(AccountError::UnsafeKdf)
    ));
    assert_eq!(b.faults.logins.load(Ordering::SeqCst), 0);
    // A default below the floor is refused before anything is derived.
    assert!(matches!(
        plain::sign_up(
            &b,
            SERVER,
            "bob",
            &password("correct horse battery"),
            &mac(2),
            &quiet
        ),
        Err(AccountError::UnsafeKdf)
    ));
}

fn kdf() -> impl Strategy<Value = (WireKdf, bool)> {
    // Outside anywhere, or inside and cheap enough to derive in a test.
    let outside = (0u32..4_200_000, 0u32..20, 0u32..20).prop_map(|(m_kib, t, p)| {
        let params = KdfParams { m_kib, t, p };
        (params, params.is_within(&KdfParams::TEST))
    });
    let inside = (64u32..=512, 1u32..=2, 1u32..=2)
        .prop_map(|(m_kib, t, p)| (KdfParams { m_kib, t, p }, true));
    prop_oneof![
        outside.prop_filter("cheap when inside", |(p, ok)| !ok || p.m_kib <= 4096),
        inside
    ]
    .prop_map(|(params, ok)| (params.into(), ok))
}

proptest! {
    #![proptest_config(config(48))]

    /// Invariant: prelogin parameters outside the backend's floor or the
    /// ceilings are refused as `UnsafeKdf` before any key is derived or
    /// sent (no login request); inside, the client derives and logs in.
    #[test]
    fn prop_prelogin_outside_floor_or_ceiling_is_refused((wire, inside) in kdf()) {
        let b = hostile();
        sign_up(&b, "alice", &mac(1));
        *b.faults.prelogin_kdf.lock().unwrap() = Some(wire);
        let result = sign_in(&b, "alice", "correct horse battery", &mac(2));
        let logins = b.faults.logins.load(Ordering::SeqCst);
        if inside {
            prop_assert_eq!(logins, 1);
            // Derived with other parameters than the account's: wrong auth key.
            prop_assert!(result.is_ok() || matches!(result, Err(AccountError::InvalidCredentials)));
        } else {
            prop_assert!(matches!(result, Err(AccountError::UnsafeKdf)));
            prop_assert_eq!(logins, 0);
        }
    }
}

// ---------------------------------------------------------------------------
// Keychain items (MemorySecrets only)
// ---------------------------------------------------------------------------

#[test]
fn keychain_items_round_trip_and_ignore_other_accounts() {
    let server = MemoryServer::new();
    let up = sign_up(&server, "alice", &mac(1));
    let secrets = MemorySecrets::new();
    let library = mac(1).library_id;
    keychain::save_session(&secrets, &library, &up.session).unwrap();
    let loaded = keychain::load_session(&secrets, &library, ORIGIN, &up.session.account_id)
        .unwrap()
        .unwrap();
    assert_eq!(loaded.account_id, up.session.account_id);
    assert_eq!(loaded.device_id, up.session.device_id);
    assert_eq!(loaded.library_id(), Some(library.as_str()));
    assert!(check(&server, &loaded).is_ok());
    // Another origin or account: ignored.
    assert!(
        keychain::load_session(
            &secrets,
            &library,
            "https://other.example",
            &up.session.account_id
        )
        .unwrap()
        .is_none()
    );
    assert!(
        keychain::load_session(
            &secrets,
            &library,
            ORIGIN,
            "a_00000000000000000000000000000000"
        )
        .unwrap()
        .is_none()
    );
    // A damaged item is ignored too.
    secrets
        .set(&keychain::account_item(&library), "{\"v\":1}")
        .unwrap();
    assert!(
        keychain::load_session(&secrets, &library, ORIGIN, &up.session.account_id)
            .unwrap()
            .is_none()
    );

    let pending = PendingRecoveryKey {
        origin: ORIGIN.into(),
        server: SERVER.into(),
        unconfirmed: false,
        account_id: up.session.account_id.clone(),
        username: "alice".into(),
        recovery_key: up.recovery_key.clone(),
    };
    keychain::save_recovery_key(&secrets, &library, &pending).unwrap();
    let back = keychain::load_recovery_key(&secrets, &library, ORIGIN, &up.session.account_id)
        .unwrap()
        .unwrap();
    assert_eq!(back.recovery_key, up.recovery_key);
    assert!(
        keychain::load_recovery_key(
            &secrets,
            &library,
            ORIGIN,
            "a_00000000000000000000000000000000"
        )
        .unwrap()
        .is_none()
    );
}

#[test]
fn discarding_a_session_keeps_the_librarys_items_and_queues_a_failed_logout() {
    let b = hostile();
    let alice = sign_up(&b, "alice", &mac(1));
    let secrets = MemorySecrets::new();
    let library = mac(1).library_id;
    // This Mac is signed in to alice, with her new recovery key unsaved.
    keychain::save_session(&secrets, &library, &alice.session).unwrap();
    keychain::save_recovery_key(
        &secrets,
        &library,
        &PendingRecoveryKey {
            origin: ORIGIN.into(),
            server: SERVER.into(),
            unconfirmed: false,
            account_id: alice.session.account_id.clone(),
            username: "alice".into(),
            recovery_key: alice.recovery_key.clone(),
        },
    )
    .unwrap();
    let items = secrets.items();
    // A session for bob that the binding question then cancels.
    let bob = sign_up(&b, "bob", &mac(1)).session;

    kanso_sync::discard_session(&b, &secrets, &bob).unwrap();
    assert_eq!(secrets.items(), items, "alice's items stay");
    assert_eq!(
        check(&b.server, &bob).unwrap_err(),
        ApiError::Unauthorized {
            reason: SignOutReason::SignedOut
        }
    );
    assert!(check(&b.server, &alice.session).is_ok());
    assert!(
        keychain::load_session(&secrets, &library, ORIGIN, &alice.session.account_id)
            .unwrap()
            .is_some()
    );

    // Unreachable: the logout is queued, and only that changes.
    let carol = sign_up(&b, "carol", &mac(1)).session;
    b.faults.unreachable.store(true, Ordering::SeqCst);
    kanso_sync::discard_session(&b, &secrets, &carol).unwrap();
    let queued = keychain::pending_logouts(&secrets, &library).unwrap();
    assert_eq!(queued.len(), 1);
    assert_eq!(queued[0].server, SERVER);
    b.faults.unreachable.store(false, Ordering::SeqCst);
    kanso_sync::flush_pending_logouts(&b, &secrets, &library);
    assert_eq!(secrets.items(), items);
    assert!(check(&b.server, &carol).is_err());
}

#[test]
fn sign_out_deletes_the_items_and_queues_a_failed_logout() {
    let b = hostile();
    let up = sign_up(&b, "alice", &mac(1));
    let secrets = MemorySecrets::new();
    let library = mac(1).library_id;
    keychain::save_session(&secrets, &library, &up.session).unwrap();
    keychain::save_recovery_key(
        &secrets,
        &library,
        &PendingRecoveryKey {
            origin: ORIGIN.into(),
            server: SERVER.into(),
            unconfirmed: false,
            account_id: up.session.account_id.clone(),
            username: "alice".into(),
            recovery_key: up.recovery_key.clone(),
        },
    )
    .unwrap();
    secrets.set("unrelated", "kept").unwrap();

    b.faults.unreachable.store(true, Ordering::SeqCst);
    kanso_sync::sign_out(&b, &secrets, &up.session).unwrap();
    assert_eq!(
        secrets.items(),
        vec![keychain::logout_item(&library), "unrelated".to_owned()]
    );
    let queued = keychain::pending_logouts(&secrets, &library).unwrap();
    assert_eq!(queued.len(), 1);
    assert_eq!(queued[0].server, SERVER);
    // Still unreachable: it stays queued.
    kanso_sync::flush_pending_logouts(&b, &secrets, &library);
    assert_eq!(
        keychain::pending_logouts(&secrets, &library).unwrap().len(),
        1
    );
    assert!(
        b.server
            .devices("alice")
            .iter()
            .all(|d| d.revoked.is_none())
    );
    // Reachable again: the logout goes through and the queue empties.
    b.faults.unreachable.store(false, Ordering::SeqCst);
    kanso_sync::flush_pending_logouts(&b, &secrets, &library);
    assert!(
        keychain::pending_logouts(&secrets, &library)
            .unwrap()
            .is_empty()
    );
    assert_eq!(secrets.items(), vec!["unrelated".to_owned()]);
    assert!(
        b.server
            .devices("alice")
            .iter()
            .all(|d| d.revoked == Some(SignOutReason::SignedOut))
    );

    // A 401 counts as done.
    let again = sign_in(&b, "alice", "correct horse battery", &mac(1))
        .unwrap()
        .session;
    keychain::set_pending_logouts(
        &secrets,
        &library,
        &[PendingLogout {
            server: SERVER.into(),
            token: SessionToken::new("kanso_unknown"),
        }],
    )
    .unwrap();
    kanso_sync::flush_pending_logouts(&b, &secrets, &library);
    assert!(
        keychain::pending_logouts(&secrets, &library)
            .unwrap()
            .is_empty()
    );
    // Signing out normally logs out at once.
    kanso_sync::sign_out(&b, &secrets, &again).unwrap();
    assert!(
        keychain::pending_logouts(&secrets, &library)
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        check(&b.server, &again).unwrap_err(),
        ApiError::Unauthorized {
            reason: SignOutReason::SignedOut
        }
    );
}

// ---------------------------------------------------------------------------
// New recovery keys are never lost (sec-client 1)
// ---------------------------------------------------------------------------

thread_local! {
    /// The test server's clock: every probe and every step moves it an hour
    /// on, past `recover-ip` and `password-account`.
    static CLOCK: std::cell::Cell<i64> = const { std::cell::Cell::new(1_800_000_000_000) };
}

fn tick(server: &MemoryServer) {
    let now = CLOCK.with(|clock| {
        clock.set(clock.get() + 3_600_001);
        clock.get()
    });
    server.set_now(now);
}

/// Whether `key` is the account's recovery key on the server now.
fn holds(server: &MemoryServer, username: &str, key: &RecoveryKey) -> bool {
    tick(server);
    let auth = kanso_sync::crypto::recovery_keys(key, ORIGIN, username).auth_key;
    server
        .api(SERVER)
        .recover_start(&RecoverStartRequest {
            username: username.into(),
            recovery_auth_key: auth.to_base64url(),
        })
        .is_ok()
}

/// The library's unsaved key, as the Keychain holds it.
fn kept(secrets: &MemorySecrets) -> Option<PendingRecoveryKey> {
    keychain::load_any_recovery_key(secrets, &mac(1).library_id).unwrap()
}

/// Every rotation saves its key in the Keychain, unconfirmed, before the
/// request goes; the server's yes confirms it.
#[test]
fn each_new_key_is_in_the_keychain_before_its_request_is_sent() {
    struct Watching<'a> {
        server: MemoryServer,
        secrets: &'a MemorySecrets,
        /// What the Keychain held as each rotating request was sent.
        seen: Mutex<Vec<(Route, Option<PendingRecoveryKey>, String)>>,
    }
    struct WatchingApi<'a> {
        inner: Box<dyn Api + 'a>,
        watching: &'a Watching<'a>,
    }
    impl Backend for Watching<'_> {
        fn api(&self, server: &str) -> Box<dyn Api + '_> {
            Box::new(WatchingApi {
                inner: self.server.api(server),
                watching: self,
            })
        }
        fn kdf_floor(&self) -> KdfParams {
            KdfParams::TEST
        }
        fn kdf_default(&self) -> KdfParams {
            KdfParams::TEST
        }
    }
    impl WatchingApi<'_> {
        fn saw(&self, route: Route, recovery_auth_key: &str) {
            self.watching.seen.lock().unwrap().push((
                route,
                kept(self.watching.secrets),
                recovery_auth_key.to_owned(),
            ));
        }
    }
    impl Api for WatchingApi<'_> {
        fn health(&self) -> Result<Health, ApiError> {
            self.inner.health()
        }
        fn prelogin(&self, r: &PreloginRequest) -> Result<PreloginResponse, ApiError> {
            self.inner.prelogin(r)
        }
        fn signup(&self, r: &SignupRequest) -> Result<SessionResponse, ApiError> {
            self.saw(Route::Signup, &r.recovery_auth_key);
            self.inner.signup(r)
        }
        fn login(&self, r: &LoginRequest) -> Result<LoginResponse, ApiError> {
            self.inner.login(r)
        }
        fn recover_start(&self, r: &RecoverStartRequest) -> Result<RecoverStartResponse, ApiError> {
            self.inner.recover_start(r)
        }
        fn recover_finish(&self, r: &RecoverFinishRequest) -> Result<SessionResponse, ApiError> {
            self.saw(Route::RecoverFinish, &r.recovery_auth_key_new);
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
            self.saw(Route::Password, &r.recovery_auth_key);
            self.inner.change_password(t, r)
        }
        fn replace_recovery_key(
            &self,
            t: &SessionToken,
            r: &RecoveryKeyRequest,
        ) -> Result<RecoveryKeyResponse, ApiError> {
            self.saw(Route::RecoveryKey, &r.recovery_auth_key);
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
        fn revoke_device(&self, t: &SessionToken, id: &str) -> Result<(), ApiError> {
            self.inner.revoke_device(t, id)
        }
        fn push(&self, t: &SessionToken, r: &PushRequest) -> Result<PushResponse, ApiError> {
            self.inner.push(t, r)
        }
        fn changes(
            &self,
            t: &SessionToken,
            since: Option<i64>,
            limit: Option<usize>,
        ) -> Result<WirePage, ApiError> {
            self.inner.changes(t, since, limit)
        }
        fn put_file(
            &self,
            t: &SessionToken,
            id: &str,
            sealed: &[u8],
        ) -> Result<PutFileResponse, ApiError> {
            self.inner.put_file(t, id, sealed)
        }
        fn get_file(&self, t: &SessionToken, id: &str) -> Result<Option<Vec<u8>>, ApiError> {
            self.inner.get_file(t, id)
        }
    }

    let secrets = MemorySecrets::new();
    let b = Watching {
        server: MemoryServer::new(),
        secrets: &secrets,
        seen: Mutex::new(Vec::new()),
    };
    let pw = password("correct horse battery");
    let up = kanso_sync::sign_up(&b, &secrets, SERVER, "alice", &pw, &mac(1), &quiet).unwrap();
    assert!(!kept(&secrets).unwrap().unconfirmed, "confirmed once taken");
    let newer = kanso_sync::new_recovery_key(&b, &secrets, &up.session, &pw, &quiet).unwrap();
    assert_eq!(newer, up.recovery_key, "the unsaved key is set again");
    keychain::delete_recovery_key(&secrets, &mac(1).library_id).unwrap();
    let fresh = kanso_sync::new_recovery_key(&b, &secrets, &up.session, &pw, &quiet).unwrap();
    assert_ne!(
        fresh, up.recovery_key,
        "a saved key is replaced by a new one"
    );
    keychain::delete_recovery_key(&secrets, &mac(1).library_id).unwrap();
    let changed = kanso_sync::change_password(
        &b,
        &secrets,
        &up.session,
        &pw,
        &password("a changed passphrase"),
        &quiet,
    )
    .unwrap();
    assert_ne!(
        changed.recovery_key, fresh,
        "a password change replaces the key"
    );
    let fresh = changed.recovery_key;
    let recovered = kanso_sync::recover(
        &b,
        &secrets,
        SERVER,
        "alice",
        &fresh,
        &password("a brand new passphrase"),
        &mac(1),
        &quiet,
    )
    .unwrap();
    assert_ne!(recovered.recovery_key, fresh, "the used key always changes");

    let seen = b.seen.lock().unwrap();
    let routes: Vec<Route> = seen.iter().map(|(route, ..)| *route).collect();
    assert_eq!(
        routes,
        [
            Route::Signup,
            Route::RecoveryKey,
            Route::RecoveryKey,
            Route::Password,
            Route::RecoverFinish
        ]
    );
    for (route, item, sent) in seen.iter() {
        let item = item
            .as_ref()
            .unwrap_or_else(|| panic!("{route:?}: no item"));
        assert!(item.unconfirmed, "{route:?}");
        assert_eq!(item.username, "alice");
        assert_eq!(item.account_id, up.session.account_id);
        let auth = kanso_sync::crypto::recovery_keys(&item.recovery_key, ORIGIN, "alice").auth_key;
        assert_eq!(
            &auth.to_base64url(),
            sent,
            "{route:?}: the kept key is the sent one"
        );
    }
}

/// Three lost answers in a row: every flow returns its key with the error,
/// the Keychain keeps it unconfirmed, and it is the key the server holds.
#[test]
fn lost_answers_keep_the_new_key_and_return_it() {
    let quick = Quick(MemoryServer::new());
    let server = &quick.0;
    let secrets = MemorySecrets::new();
    let pw = password("correct horse battery");

    server.drop_next_responses(Route::Signup, 3);
    let lost =
        kanso_sync::sign_up(&quick, &secrets, SERVER, "alice", &pw, &mac(1), &quiet).unwrap_err();
    assert!(matches!(lost.error, AccountError::Transport(_)));
    let key = lost.recovery_key.expect("the key comes back");
    assert!(key.unconfirmed);
    assert_eq!(kept(&secrets).unwrap().recovery_key, key.recovery_key);
    assert!(holds(server, "alice", &key.recovery_key));
    // The account exists: its password signs in.
    let session = sign_in(&quick, "alice", "correct horse battery", &mac(1))
        .unwrap()
        .session;

    // A new key whose answer is lost is the server's.
    server.drop_next_responses(Route::RecoveryKey, 3);
    let lost = kanso_sync::new_recovery_key(&quick, &secrets, &session, &pw, &quiet).unwrap_err();
    let shown = lost.recovery_key.unwrap();
    assert!(holds(server, "alice", &shown.recovery_key));
    assert_eq!(
        shown.recovery_key, key.recovery_key,
        "the unsaved key again"
    );
    keychain::delete_recovery_key(&secrets, &mac(1).library_id).unwrap();
    server.drop_next_responses(Route::RecoveryKey, 3);
    let lost = kanso_sync::new_recovery_key(&quick, &secrets, &session, &pw, &quiet).unwrap_err();
    let second = lost.recovery_key.unwrap().recovery_key;
    assert_ne!(second, key.recovery_key);
    assert!(holds(server, "alice", &second));
    assert_eq!(kept(&secrets).unwrap().recovery_key, second);

    // Recovery: the password changed and the key rotated, unseen.
    server.drop_next_responses(Route::RecoverFinish, 3);
    let lost = kanso_sync::recover(
        &quick,
        &secrets,
        SERVER,
        "alice",
        &second,
        &password("a brand new passphrase"),
        &mac(1),
        &quiet,
    )
    .unwrap_err();
    let third = lost.recovery_key.unwrap();
    assert!(third.unconfirmed);
    assert_ne!(third.recovery_key, second);
    assert!(holds(server, "alice", &third.recovery_key));
    assert!(sign_in(&quick, "alice", "a brand new passphrase", &mac(1)).is_ok());
}

/// A refused rotation, or one that never left this Mac, puts the item back
/// as it was: gone, or the unsaved key from before.
#[test]
fn a_refused_or_unsent_rotation_puts_the_item_back() {
    let b = hostile();
    let secrets = MemorySecrets::new();
    let pw = password("correct horse battery");
    let up = kanso_sync::sign_up(&b, &secrets, SERVER, "alice", &pw, &mac(1), &quiet).unwrap();
    keychain::delete_recovery_key(&secrets, &mac(1).library_id).unwrap();

    let wrong = password("wrong horse battery");
    let refused =
        kanso_sync::new_recovery_key(&b, &secrets, &up.session, &wrong, &quiet).unwrap_err();
    assert!(matches!(refused.error, AccountError::InvalidCredentials));
    assert!(refused.recovery_key.is_none());
    assert!(kept(&secrets).is_none(), "no item was left behind");
    assert!(holds(&b.server, "alice", &up.recovery_key));

    // With an unsaved key from before, that one stays and comes back.
    let unsaved = kanso_sync::new_recovery_key(&b, &secrets, &up.session, &pw, &quiet).unwrap();
    let refused =
        kanso_sync::new_recovery_key(&b, &secrets, &up.session, &wrong, &quiet).unwrap_err();
    assert_eq!(refused.recovery_key.unwrap().recovery_key, unsaved);
    let item = kept(&secrets).unwrap();
    assert_eq!(item.recovery_key, unsaved);
    assert!(!item.unconfirmed);

    // Unsent: the server never saw it.
    keychain::delete_recovery_key(&secrets, &mac(1).library_id).unwrap();
    b.faults.unsent.store(true, Ordering::SeqCst);
    let unsent = kanso_sync::new_recovery_key(&b, &secrets, &up.session, &pw, &quiet);
    b.faults.unsent.store(false, Ordering::SeqCst);
    let unsent = unsent.unwrap_err();
    assert!(matches!(unsent.error, AccountError::Transport(_)));
    assert!(unsent.recovery_key.is_none());
    assert!(kept(&secrets).is_none());
    assert!(holds(&b.server, "alice", &unsaved));
}

/// What a rotation's error must hold when the server may have taken its key:
/// the key, unconfirmed, also in the Keychain, and the server's.
fn kept_and_held(
    server: &MemoryServer,
    secrets: &MemorySecrets,
    flow: &str,
    lost: kanso_sync::RotationError,
    refused: bool,
) -> RecoveryKey {
    if refused {
        assert!(
            matches!(lost.error, AccountError::RateLimited { .. }),
            "{flow}: {:?}",
            lost.error
        );
    } else {
        assert!(
            matches!(lost.error, AccountError::Transport(_)),
            "{flow}: {:?}",
            lost.error
        );
    }
    let key = lost
        .recovery_key
        .unwrap_or_else(|| panic!("{flow}: the key comes back"));
    assert!(key.unconfirmed, "{flow}: not known to be taken");
    assert_eq!(
        kept(secrets).map(|k| k.recovery_key),
        Some(key.recovery_key.clone()),
        "{flow}: the Keychain keeps it"
    );
    assert!(
        holds(server, "alice", &key.recovery_key),
        "{flow}: the server holds it"
    );
    key.recovery_key
}

/// Review-verify 1: the first attempt reaches the server and its answer is lost,
/// then the retries never leave the Mac (offline), or the server refuses
/// them (a 429). An earlier attempt may have been carried out, so every
/// flow keeps the key and returns it, and it is the one the server holds.
#[test]
fn a_lost_answer_then_unsent_or_refused_retries_keep_the_key() {
    let later: [(fn() -> ApiError, bool); 2] = [
        (|| ApiError::unsent("no route to host"), false),
        (
            || ApiError::RateLimited {
                retry_after: 60,
                limit: "session-account".into(),
            },
            true,
        ),
    ];
    for (make, refused) in later {
        let b = hostile();
        let secrets = MemorySecrets::new();
        let pw = password("correct horse battery");
        let lose_first = |route: Route| {
            b.server.drop_next_responses(route, 1);
            *b.faults.after_first.lock().unwrap() = Some((route, make, 0));
        };
        let heal = || {
            let (route, _, sent) = b.faults.after_first.lock().unwrap().take().unwrap();
            assert!(sent > 1, "{route:?}: it was retried");
        };
        // The user saves the key on screen.
        let save = || keychain::delete_recovery_key(&secrets, &mac(1).library_id).unwrap();

        lose_first(Route::Signup);
        let lost = kanso_sync::sign_up(&b, &secrets, SERVER, "alice", &pw, &mac(1), &quiet);
        heal();
        let first = kept_and_held(&b.server, &secrets, "sign-up", lost.unwrap_err(), refused);
        let session = sign_in(&b, "alice", "correct horse battery", &mac(1))
            .expect("the account exists")
            .session;
        save();

        lose_first(Route::RecoveryKey);
        let lost = kanso_sync::new_recovery_key(&b, &secrets, &session, &pw, &quiet);
        heal();
        let second = kept_and_held(&b.server, &secrets, "new key", lost.unwrap_err(), refused);
        assert_ne!(second, first);
        save();

        lose_first(Route::Password);
        let changed = password("a changed passphrase");
        let lost = kanso_sync::change_password(&b, &secrets, &session, &pw, &changed, &quiet);
        heal();
        let third = kept_and_held(&b.server, &secrets, "password", lost.unwrap_err(), refused);
        assert_ne!(third, second);
        assert!(sign_in(&b, "alice", "a changed passphrase", &mac(1)).is_ok());
        save();

        lose_first(Route::RecoverFinish);
        let lost = kanso_sync::recover(
            &b,
            &secrets,
            SERVER,
            "alice",
            &third,
            &password("a brand new passphrase"),
            &mac(1),
            &quiet,
        );
        heal();
        let fourth = kept_and_held(&b.server, &secrets, "recovery", lost.unwrap_err(), refused);
        assert_ne!(fourth, third);
        assert!(sign_in(&b, "alice", "a brand new passphrase", &mac(1)).is_ok());
    }
}

/// The Keychain refusing before the request means nothing is sent; refusing
/// to mark the key confirmed afterwards still returns the key, which stays
/// in the Keychain unconfirmed.
#[test]
fn the_keychain_refusing_never_loses_a_key() {
    let quick = Quick(MemoryServer::new());
    let server = &quick.0;
    let secrets = MemorySecrets::new();
    let pw = password("correct horse battery");

    secrets.fail_write(0);
    let refused =
        kanso_sync::sign_up(&quick, &secrets, SERVER, "alice", &pw, &mac(1), &quiet).unwrap_err();
    assert!(matches!(refused.error, AccountError::Keychain(_)));
    assert!(refused.recovery_key.is_none());
    assert_eq!(server.stats().count(Route::Signup), 0, "nothing was sent");
    assert!(server.account_id("alice").is_none());

    secrets.fail_write(1);
    let up = kanso_sync::sign_up(&quick, &secrets, SERVER, "alice", &pw, &mac(1), &quiet).unwrap();
    let item = kept(&secrets).unwrap();
    assert_eq!(item.recovery_key, up.recovery_key);
    assert!(
        item.unconfirmed,
        "the confirmation didn't save, the key did"
    );
    assert!(holds(server, "alice", &up.recovery_key));

    // A refused request whose putting back fails still returns the key the
    // server holds.
    secrets.fail_write(1);
    let wrong = password("wrong horse battery");
    let refused =
        kanso_sync::new_recovery_key(&quick, &secrets, &up.session, &wrong, &quiet).unwrap_err();
    assert_eq!(refused.recovery_key.unwrap().recovery_key, up.recovery_key);
}

/// An unsaved key of another account is never replaced: the rotation is
/// refused before anything is sent.
#[test]
fn another_accounts_unsaved_key_is_never_replaced() {
    let quick = Quick(MemoryServer::new());
    let secrets = MemorySecrets::new();
    let bob = kanso_sync::sign_up(
        &quick,
        &secrets,
        SERVER,
        "bob",
        &Password::new("correct horse battery", "bob").unwrap(),
        &mac(1),
        &quiet,
    )
    .unwrap();
    let before = quick.0.stats().count(Route::Signup);
    let refused = kanso_sync::sign_up(
        &quick,
        &secrets,
        SERVER,
        "alice",
        &password("correct horse battery"),
        &mac(1),
        &quiet,
    )
    .unwrap_err();
    assert!(matches!(
        refused.error,
        AccountError::UnsavedRecoveryKey { ref username } if username == "bob"
    ));
    assert_eq!(quick.0.stats().count(Route::Signup), before);
    assert_eq!(kept(&secrets).unwrap().recovery_key, bob.recovery_key);
    let shown = refused
        .recovery_key
        .expect("bob's key comes back to be shown");
    assert_eq!(shown.recovery_key, bob.recovery_key);
    assert_eq!(shown.username, "bob");
}

#[derive(Clone, Debug)]
enum KeyOp {
    /// "New recovery key", its answer lost `drops` times, the Keychain
    /// refusing its write number `fails`, maybe with a wrong password.
    NewKey {
        drops: usize,
        fails: Option<usize>,
        wrong: bool,
    },
    /// Recovery with the key the server holds.
    Recover { drops: usize, fails: Option<usize> },
    /// A password change, which sets a new recovery key too.
    ChangePassword {
        drops: usize,
        fails: Option<usize>,
        wrong: bool,
    },
    /// The user saves the key on screen ("I've saved my recovery key").
    Save,
}

fn key_op() -> impl Strategy<Value = KeyOp> {
    let fails = prop::option::of(0usize..3);
    prop_oneof![
        3 => (0usize..4, fails.clone(), prop::bool::weighted(0.2))
            .prop_map(|(drops, fails, wrong)| KeyOp::NewKey { drops, fails, wrong }),
        2 => (0usize..4, fails.clone()).prop_map(|(drops, fails)| KeyOp::Recover { drops, fails }),
        2 => (0usize..4, fails, prop::bool::weighted(0.2))
            .prop_map(|(drops, fails, wrong)| KeyOp::ChangePassword { drops, fails, wrong }),
        1 => Just(KeyOp::Save),
    ]
}

proptest! {
    #![proptest_config(config(32))]

    /// Invariant: whatever fails and wherever, the recovery key the server
    /// holds is one the user can still get: in the Keychain item, on screen
    /// (the last key a call returned, with or without an error), or saved
    /// by the user.
    #[test]
    fn prop_a_key_the_server_holds_is_always_kept_or_shown(ops in prop::collection::vec(key_op(), 1..7)) {
        let quick = Quick(MemoryServer::new());
        let server = &quick.0;
        let secrets = MemorySecrets::new();
        tick(server);
        let mut passwords = vec!["correct horse battery".to_owned()];
        let up = kanso_sync::sign_up(&quick, &secrets, SERVER, "alice", &password(&passwords[0]), &mac(1), &quiet).unwrap();
        let mut session = up.session;
        let mut shown = Some(up.recovery_key.clone());
        let mut saved: Vec<RecoveryKey> = Vec::new();
        let mut known = vec![up.recovery_key];
        for (n, op) in ops.into_iter().enumerate() {
            tick(server);
            let result = match op {
                KeyOp::Save => {
                    if let Some(key) = shown.take() {
                        saved.push(key);
                    }
                    keychain::delete_recovery_key(&secrets, &mac(1).library_id).unwrap();
                    None
                }
                KeyOp::NewKey { drops, fails, wrong } => {
                    server.drop_next_responses(Route::RecoveryKey, drops);
                    if let Some(at) = fails {
                        secrets.fail_write(at);
                    }
                    let text = if wrong { "wrong horse battery" } else { passwords.last().unwrap().as_str() };
                    Some(kanso_sync::new_recovery_key(&quick, &secrets, &session, &password(text), &quiet)
                        .map_err(|err| err.recovery_key.map(|k| k.recovery_key)))
                }
                KeyOp::ChangePassword { drops, fails, wrong } => {
                    server.drop_next_responses(Route::Password, drops);
                    if let Some(at) = fails {
                        secrets.fail_write(at);
                    }
                    let current = if wrong { "wrong horse battery" } else { passwords.last().unwrap().as_str() };
                    let text = format!("changed passphrase {n} here");
                    let result = kanso_sync::change_password(&quick, &secrets, &session, &password(current), &password(&text), &quiet)
                        .map(|changed| changed.recovery_key)
                        .map_err(|err| err.recovery_key.map(|k| k.recovery_key));
                    passwords.push(text);
                    Some(result)
                }
                KeyOp::Recover { drops, fails } => {
                    let current = known.iter().chain(&saved).find(|k| holds(server, "alice", k)).cloned();
                    let current = current.expect("the server's key is known");
                    server.drop_next_responses(Route::RecoverFinish, drops);
                    if let Some(at) = fails {
                        secrets.fail_write(at);
                    }
                    let text = format!("passphrase number {n} here");
                    let result = kanso_sync::recover(&quick, &secrets, SERVER, "alice", &current, &password(&text), &mac(1), &quiet)
                        .map(|r| { session = r.session; r.recovery_key })
                        .map_err(|err| err.recovery_key.map(|k| k.recovery_key));
                    passwords.push(text);
                    Some(result)
                }
            };
            server.drop_next_responses(Route::RecoveryKey, 0);
            server.drop_next_responses(Route::RecoverFinish, 0);
            server.drop_next_responses(Route::Password, 0);
            secrets.heal();
            match result {
                Some(Ok(key)) => shown = Some(key),
                Some(Err(Some(key))) => shown = Some(key),
                Some(Err(None)) | None => {}
            }
            // A recovery may have ended this session: sign in again with
            // whichever password is now the account's.
            if check(server, &session).is_err() {
                session = passwords.iter().rev()
                    .find_map(|text| sign_in(&quick, "alice", text, &mac(1)).ok())
                    .expect("one of the passwords works")
                    .session;
            }
            let item = kept(&secrets).map(|k| k.recovery_key);
            known.extend(item.clone());
            known.extend(shown.clone());
            let current = known.iter().chain(&saved).find(|k| holds(server, "alice", k)).cloned();
            prop_assert!(current.is_some(), "after {op:?}: the server holds a key nobody has");
            let current = current.unwrap();
            prop_assert!(
                item.as_ref() == Some(&current) || shown.as_ref() == Some(&current) || saved.contains(&current),
                "after {op:?}: the server's key is neither kept, shown nor saved"
            );
        }
    }
}

/// A password change sets a new recovery key with it: the old key stops
/// working at once, and the new one is in the Keychain, then returned.
#[test]
fn a_password_change_replaces_the_recovery_key() {
    let quick = Quick(MemoryServer::new());
    let server = &quick.0;
    let secrets = MemorySecrets::new();
    let pw = password("correct horse battery");
    let up = kanso_sync::sign_up(&quick, &secrets, SERVER, "alice", &pw, &mac(1), &quiet).unwrap();
    keychain::delete_recovery_key(&secrets, &mac(1).library_id).unwrap();

    tick(server);
    let changed = kanso_sync::change_password(
        &quick,
        &secrets,
        &up.session,
        &pw,
        &password("new long passphrase"),
        &quiet,
    )
    .unwrap();
    assert_ne!(changed.recovery_key, up.recovery_key);
    assert!(
        !holds(server, "alice", &up.recovery_key),
        "the old key stops working"
    );
    assert!(holds(server, "alice", &changed.recovery_key));
    let item = kept(&secrets).unwrap();
    assert_eq!(item.recovery_key, changed.recovery_key);
    assert!(!item.unconfirmed);
    assert!(
        check(server, &up.session).is_ok(),
        "this Mac stays signed in"
    );

    // Lost three times: the password and the key changed all the same, and
    // the error carries the key.
    keychain::delete_recovery_key(&secrets, &mac(1).library_id).unwrap();
    tick(server);
    server.drop_next_responses(Route::Password, 3);
    let lost = kanso_sync::change_password(
        &quick,
        &secrets,
        &up.session,
        &password("new long passphrase"),
        &password("third long passphrase"),
        &quiet,
    )
    .unwrap_err();
    let shown = lost.recovery_key.expect("the key comes back");
    assert!(shown.unconfirmed);
    assert!(holds(server, "alice", &shown.recovery_key));
    assert!(sign_in(&quick, "alice", "third long passphrase", &mac(2)).is_ok());
}

/// The password route's recovery fields: malformed ones are refused before
/// the current key is checked; the same request again after it was carried
/// out is answered 200 with no devices (its answer was lost); other
/// recovery values with the old password are refused.
#[test]
fn the_password_route_checks_its_recovery_fields_and_retries() {
    let b = hostile();
    let secrets = MemorySecrets::new();
    let pw = password("correct horse battery");
    let up = kanso_sync::sign_up(&b, &secrets, SERVER, "alice", &pw, &mac(1), &quiet).unwrap();
    tick(&b.server);
    let changed = kanso_sync::change_password(
        &b,
        &secrets,
        &up.session,
        &pw,
        &password("new long passphrase"),
        &quiet,
    )
    .unwrap();
    let sent = b.faults.passwords.lock().unwrap().last().cloned().unwrap();
    let api = b.server.api(SERVER);
    let token = &token_of(&b.server, &up.session);

    tick(&b.server);
    let again = api.change_password(token, &sent).unwrap();
    assert_eq!(again.signed_out_devices, 0);

    let other_key = RecoveryKey::generate();
    let other = kanso_sync::crypto::recovery_keys(&other_key, ORIGIN, "alice");
    let with = |change: &dyn Fn(&mut PasswordRequest)| {
        let mut request = sent.clone();
        change(&mut request);
        request
    };
    let bad_auth = with(&|r| r.recovery_auth_key = "AAAA".into());
    let bad_wrap = with(&|r| r.recovery_wrapped_key = "AAAA".into());
    for (field, bad) in [
        ("recovery_auth_key", bad_auth),
        ("recovery_wrapped_key", bad_wrap),
    ] {
        tick(&b.server);
        assert_eq!(
            api.change_password(token, &bad).unwrap_err(),
            ApiError::BadRequest,
            "{field}"
        );
    }
    tick(&b.server);
    let different = with(&|r| r.recovery_auth_key = other.auth_key.to_base64url());
    assert_eq!(
        api.change_password(token, &different).unwrap_err(),
        ApiError::InvalidCredentials
    );
    assert!(
        holds(&b.server, "alice", &changed.recovery_key),
        "the stored key stays"
    );
    assert!(!holds(&b.server, "alice", &other_key));
}

/// The Keychain refusing to delete the session's item doesn't stop the
/// logout (A14): the server ends the session, and the refusal is the error.
#[test]
fn signing_out_logs_out_even_when_the_keychain_refuses() {
    let b = hostile();
    let up = sign_up(&b, "alice", &mac(1));
    let secrets = MemorySecrets::new();
    let library = mac(1).library_id;
    keychain::save_session(&secrets, &library, &up.session).unwrap();
    secrets.fail_writes_to(Some("sync-account:"));
    let refused = kanso_sync::sign_out(&b, &secrets, &up.session).unwrap_err();
    assert!(matches!(refused, AccountError::Keychain(_)));
    assert_eq!(b.faults.logouts.load(Ordering::SeqCst), 1);
    assert!(matches!(
        check(&b.server, &up.session).unwrap_err(),
        ApiError::Unauthorized { .. }
    ));

    // Unreachable as well: the token is queued all the same.
    let again = sign_in(&b, "alice", "correct horse battery", &mac(1))
        .unwrap()
        .session;
    b.faults.unreachable.store(true, Ordering::SeqCst);
    let refused = kanso_sync::sign_out(&b, &secrets, &again).unwrap_err();
    b.faults.unreachable.store(false, Ordering::SeqCst);
    secrets.heal();
    assert!(matches!(refused, AccountError::Keychain(_)));
    assert_eq!(
        keychain::pending_logouts(&secrets, &library).unwrap().len(),
        1
    );
    kanso_sync::flush_pending_logouts(&b, &secrets, &library);
    assert!(
        keychain::pending_logouts(&secrets, &library)
            .unwrap()
            .is_empty()
    );
    assert!(check(&b.server, &again).is_err());
}

/// A token queued while a flush waits on the network stays queued: the
/// flush takes out only what it sent (sync 9).
#[test]
fn a_logout_queued_during_a_flush_is_kept() {
    let b = hostile();
    let secrets = MemorySecrets::new();
    let library = mac(1).library_id;
    let first = sign_up(&b, "alice", &mac(1)).session;
    let second = sign_in(&b, "alice", "correct horse battery", &mac(1))
        .unwrap()
        .session;
    b.faults.unreachable.store(true, Ordering::SeqCst);
    kanso_sync::discard_session(&b, &secrets, &first).unwrap();
    b.faults.unreachable.store(false, Ordering::SeqCst);
    assert_eq!(
        keychain::pending_logouts(&secrets, &library).unwrap().len(),
        1
    );

    let (entered_tx, entered) = std::sync::mpsc::channel();
    let (release, release_rx) = std::sync::mpsc::channel();
    *b.faults.logout_gate.lock().unwrap() = Some((entered_tx, release_rx));
    std::thread::scope(|scope| {
        let flush = scope.spawn(|| kanso_sync::flush_pending_logouts(&b, &secrets, &library));
        entered.recv().unwrap();
        // The flush is on the network with the first token; the second is
        // queued meanwhile.
        b.faults.unreachable.store(true, Ordering::SeqCst);
        kanso_sync::discard_session(&b, &secrets, &second).unwrap();
        b.faults.unreachable.store(false, Ordering::SeqCst);
        release.send(()).unwrap();
        flush.join().unwrap();
    });
    let queued = keychain::pending_logouts(&secrets, &library).unwrap();
    assert_eq!(queued.len(), 1, "the second token stays queued");
    assert!(
        check(&b.server, &first).is_err(),
        "the first was logged out"
    );
    assert!(check(&b.server, &second).is_ok());
    kanso_sync::flush_pending_logouts(&b, &secrets, &library);
    assert!(
        keychain::pending_logouts(&secrets, &library)
            .unwrap()
            .is_empty()
    );
    assert!(check(&b.server, &second).is_err());
}
