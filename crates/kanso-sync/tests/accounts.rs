//! Account flows against `MemoryServer` and `MemorySecrets`
//! (docs/ACCOUNTS.md › Account flows, › Keychain items): sign-up, sign-in,
//! recovery, password change, new recovery key, deletion, devices, the
//! documented retries after lost responses, the KDF floor and ceilings, and
//! the Keychain items. Tests never touch the real Keychain. CI pins the seed
//! with `PROPTEST_RNG_SEED`.

use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

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

fn mac(n: u8) -> DeviceIdentity {
    DeviceIdentity::from_parts(&format!("l_{:032x}", n), &format!("Mac {n}"), [n; 32])
}

fn password(text: &str) -> Password {
    Password::new(text, "alice").unwrap()
}

fn quiet(_: Step) {}

fn sign_up(b: &dyn Backend, username: &str, device: &DeviceIdentity) -> kanso_sync::SignedUp {
    kanso_sync::sign_up(
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
    let up = kanso_sync::sign_up(
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
        kanso_sync::sign_up(
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
        kanso_sync::sign_up(
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
        kanso_sync::sign_up(
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
        kanso_sync::sign_up(
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
        kanso_sync::sign_up(
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
        kanso_sync::sign_up(
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
        kanso_sync::sign_up(
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
    let recovered = kanso_sync::recover(
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
        kanso_sync::recover(
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
    kanso_sync::recover(
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
        kanso_sync::change_password(
            &server,
            &up.session,
            &password("wrong horse battery"),
            &password("new long passphrase"),
            &quiet
        ),
        Err(AccountError::InvalidCredentials)
    ));
    let n = kanso_sync::change_password(
        &server,
        &up.session,
        &password("correct horse battery"),
        &password("new long passphrase"),
        &quiet,
    )
    .unwrap();
    assert_eq!(n, 2);
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
    let key = kanso_sync::new_recovery_key(
        &server,
        &up.session,
        &password("correct horse battery"),
        &quiet,
    )
    .unwrap();
    assert!(check(&server, &up.session).is_ok());
    assert!(matches!(
        kanso_sync::recover(
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
    kanso_sync::recover(
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
    let server = MemoryServer::new();
    let up = sign_up(&server, "alice", &mac(1));
    let other = sign_in(&server, "alice", "correct horse battery", &mac(2))
        .unwrap()
        .session;
    // A typed username that isn't the account's is refused before sending.
    assert!(matches!(
        kanso_sync::delete_account(
            &server,
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
            &server,
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
        &server,
        &up.session,
        &password("correct horse battery"),
        " ALICE ",
        &quiet,
    )
    .unwrap();
    assert_eq!(server.stats().count(Route::Delete), before + 2);
    assert_eq!(
        check(&server, &other).unwrap_err(),
        ApiError::Unauthorized {
            reason: SignOutReason::Unknown
        }
    );
    assert!(server.account_id("alice").is_none());
    // Any other 401 means the session ended first: that is reported.
    let again = sign_up(&server, "alice", &mac(1));
    server.revoke_all("alice", SignOutReason::Revoked);
    assert!(matches!(
        kanso_sync::delete_account(
            &server,
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
    assert!(matches!(
        kanso_sync::revoke_device(&server, &eleventh.session, &sessions[1].device_id),
        Err(AccountError::NotFound)
    ));
}

#[test]
fn lost_responses_are_recovered_by_the_documented_retry() {
    let server = MemoryServer::new();
    server.drop_next_responses(Route::Signup, 1);
    let up = sign_up(&server, "alice", &mac(1));
    assert_eq!(server.stats().count(Route::Signup), 2);
    assert!(check(&server, &up.session).is_ok());

    server.drop_next_responses(Route::Password, 1);
    kanso_sync::change_password(
        &server,
        &up.session,
        &password("correct horse battery"),
        &password("new long passphrase"),
        &quiet,
    )
    .unwrap();
    assert_eq!(server.stats().count(Route::Password), 2);
    assert!(sign_in(&server, "alice", "new long passphrase", &mac(2)).is_ok());

    server.drop_next_responses(Route::RecoveryKey, 1);
    let key = kanso_sync::new_recovery_key(
        &server,
        &up.session,
        &password("new long passphrase"),
        &quiet,
    )
    .unwrap();
    assert_eq!(server.stats().count(Route::RecoveryKey), 2);

    server.drop_next_responses(Route::RecoverFinish, 1);
    let recovered = kanso_sync::recover(
        &server,
        SERVER,
        "alice",
        &key,
        &password("third long passphrase"),
        &mac(3),
        &quiet,
    )
    .unwrap();
    assert_eq!(server.stats().count(Route::RecoverFinish), 2);
    assert!(check(&server, &recovered.session).is_ok());
    kanso_sync::recover(
        &server,
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
        kanso_sync::sign_up(
            &server,
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
    /// Health answers 404, like a v1 Worker.
    outdated: std::sync::atomic::AtomicBool,
    logins: AtomicUsize,
    logouts: AtomicUsize,
    /// A server with the backend's floor and default.
    floor: Mutex<Option<KdfParams>>,
}

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
}

impl Wrapped<'_> {
    fn reach(&self) -> Result<(), ApiError> {
        if self.faults.unreachable.load(Ordering::SeqCst) {
            Err(ApiError::Transport("unreachable".into()))
        } else {
            Ok(())
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
        self.reach()?;
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
        self.reach()?;
        self.inner.recover_finish(request)
    }
    fn logout(&self, token: &SessionToken) -> Result<(), ApiError> {
        self.reach()?;
        self.faults.logouts.fetch_add(1, Ordering::SeqCst);
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
        self.reach()?;
        self.inner.change_password(token, request)
    }
    fn replace_recovery_key(
        &self,
        token: &SessionToken,
        request: &RecoveryKeyRequest,
    ) -> Result<RecoveryKeyResponse, ApiError> {
        self.reach()?;
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
        kanso_sync::sign_up(
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
        kanso_sync::sign_up(
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
        kanso_sync::sign_up(
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
