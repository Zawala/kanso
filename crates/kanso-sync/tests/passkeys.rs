//! Passkey sign-up and sign-in, fresh proofs, adding and removing passkeys,
//! the first password, and the proof rotations, against `MemoryServer` as a
//! verifying relying party and `TestAuthenticator` as the platform
//! (docs/ACCOUNTS.md › Account flows, › Changing a way in needs a fresh
//! proof). Plus properties of proof construction, the sign-in decision and
//! the PRF wrap. CI pins the seed with `PROPTEST_RNG_SEED`.

use kanso_sync::api::{Backend as _, LoginDevice, PasswordRequest};
use kanso_sync::crypto::{DataKey, key_id};
use kanso_sync::keychain;
use kanso_sync::passkey::{open_passkey_session_key, passkey_wrapped_key};
use kanso_sync::{
    AccountError, ApiError, DeviceIdentity, MemorySecrets, MemoryServer, PasskeyCeremony,
    PasskeySignIn, PasskeyUnlock, Password, Proof, ProofWire, SecretStore, Session, SessionToken,
    SignedUp, Step, TestAuthenticator, passkey_unlock,
};
use proptest::prelude::*;

const SERVER: &str = "http://127.0.0.1:8787";

fn mac(n: u8) -> DeviceIdentity {
    DeviceIdentity::from_parts(&format!("l_{:032x}", n), &format!("Mac {n}"), [n; 32])
}

fn quiet(_: Step) {}

fn b64(bytes: &[u8]) -> String {
    use base64::Engine as _;
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

fn token_of(session: &Session) -> SessionToken {
    let secrets = MemorySecrets::new();
    let library = "l_00000000000000000000000000000000";
    keychain::save_session(&secrets, library, session).unwrap();
    let json = secrets
        .get(&keychain::account_item(library))
        .unwrap()
        .unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    SessionToken::new(value["token"].as_str().unwrap())
}

fn passkey_sign_up(
    server: &MemoryServer,
    auth: &TestAuthenticator,
    mac: &DeviceIdentity,
) -> SignedUp {
    let start = kanso_sync::start_passkey_sign_up(server, SERVER, "alice", mac).unwrap();
    let registration = auth.register(&start.challenge).unwrap();
    let secrets = MemorySecrets::new();
    kanso_sync::sign_up_with_passkey(server, &secrets, &start, mac, registration, &quiet)
        .map_err(|err| err.error)
        .unwrap()
}

fn passkey_sign_in(
    server: &MemoryServer,
    secrets: &dyn SecretStore,
    auth: &TestAuthenticator,
    mac: &DeviceIdentity,
) -> Result<PasskeySignIn, AccountError> {
    let start = kanso_sync::start_passkey_sign_in(server, SERVER, None, &LoginDevice::ephemeral())?;
    let assertion = auth.assert(&start.challenge).unwrap();
    kanso_sync::sign_in_with_passkey(server, secrets, &start, mac, assertion, &quiet)
}

/// A passkey proof by `auth` for `session`.
fn passkey_proof(
    server: &MemoryServer,
    session: &Session,
    auth: &TestAuthenticator,
) -> Proof<'static> {
    let start = kanso_sync::start_passkey_proof(server, session, None).unwrap();
    assert!(!start.challenge.request_prf, "a proof never asks for PRF");
    Proof::passkey(&start, auth.assert(&start.challenge).unwrap())
}

#[test]
fn a_passkey_account_signs_in_with_its_local_key_then_prf_then_locked() {
    let server = MemoryServer::new();
    let auth = TestAuthenticator::new(1);
    let first = passkey_sign_up(&server, &auth, &mac(1));
    let key_id = first.session.key_id();

    // The library that holds the key uses it.
    let secrets = MemorySecrets::new();
    keychain::save_session(&secrets, &mac(1).library_id, &first.session).unwrap();
    let PasskeySignIn::Unlocked(again) =
        passkey_sign_in(&server, &secrets, &auth, &mac(1)).unwrap()
    else {
        panic!("the local key unlocks")
    };
    assert_eq!(again.session.key_id(), key_id);

    // A new Mac opens the PRF wrap with the same credential.
    let PasskeySignIn::Unlocked(second) =
        passkey_sign_in(&server, &MemorySecrets::new(), &auth, &mac(2)).unwrap()
    else {
        panic!("the PRF wrap unlocks")
    };
    assert_eq!(second.session.key_id(), key_id);

    // A credential without PRF signs in but can't unlock.
    let no_prf = TestAuthenticator::new(3).without_prf();
    let start = kanso_sync::start_add_passkey(
        &server,
        &first.session,
        &passkey_proof(&server, &first.session, &auth),
    )
    .unwrap();
    let info = kanso_sync::add_passkey(
        &server,
        &first.session,
        &start,
        no_prf.register(&start.challenge).unwrap(),
    )
    .unwrap();
    assert!(!info.prf);
    let locked = passkey_sign_in(&server, &MemorySecrets::new(), &no_prf, &mac(3)).unwrap();
    assert!(matches!(locked, PasskeySignIn::Locked(_)));
}

#[test]
fn a_sign_up_without_prf_sends_no_wrap_and_a_new_mac_is_locked() {
    let server = MemoryServer::new();
    let auth = TestAuthenticator::new(2).without_prf();
    let up = passkey_sign_up(&server, &auth, &mac(1));
    let methods = kanso_sync::methods(&server, &up.session).unwrap();
    assert!(!methods.has_password);
    assert_eq!(methods.passkeys.len(), 1);
    assert!(!methods.passkeys[0].prf);
    let locked = passkey_sign_in(&server, &MemorySecrets::new(), &auth, &mac(2)).unwrap();
    assert!(matches!(locked, PasskeySignIn::Locked(_)));
}

#[test]
fn an_authenticator_without_user_verification_is_refused() {
    let server = MemoryServer::new();
    let auth = TestAuthenticator::new(4).without_user_verification();
    let start = kanso_sync::start_passkey_sign_up(&server, SERVER, "alice", &mac(1)).unwrap();
    let registration = auth.register(&start.challenge).unwrap();
    let err = kanso_sync::sign_up_with_passkey(
        &server,
        &MemorySecrets::new(),
        &start,
        &mac(1),
        registration,
        &quiet,
    )
    .unwrap_err();
    assert!(
        matches!(err.error, AccountError::InvalidCredentials),
        "{:?}",
        err.error
    );
}

#[test]
fn a_passkey_proof_is_single_use_and_replaces_the_recovery_key() {
    let server = MemoryServer::new();
    let auth = TestAuthenticator::new(5);
    let up = passkey_sign_up(&server, &auth, &mac(1));
    let secrets = MemorySecrets::new();
    let proof = passkey_proof(&server, &up.session, &auth);
    let new_key = kanso_sync::new_recovery_key_with_proof(
        &server,
        &secrets,
        &up.session,
        &proof,
        None,
        &quiet,
    )
    .map_err(|err| err.error)
    .unwrap();
    let _ = new_key;
    let again = kanso_sync::new_recovery_key_with_proof(
        &server,
        &secrets,
        &up.session,
        &proof,
        None,
        &quiet,
    )
    .unwrap_err();
    assert!(
        matches!(
            again.error,
            AccountError::FlowExpired | AccountError::Protocol(_)
        ),
        "{:?}",
        again.error
    );
    // Another account's credential doesn't prove this one.
    let other = TestAuthenticator::new(6);
    let start = kanso_sync::start_passkey_proof(&server, &up.session, None).unwrap();
    let forged = Proof::passkey(
        &start,
        other.assertion(
            &start.challenge.rp_id,
            &start.challenge.origin,
            &start.challenge.challenge,
            false,
        ),
    );
    let err =
        kanso_sync::delete_account_with_proof(&server, &up.session, &forged, None, "alice", &quiet)
            .unwrap_err();
    assert!(matches!(err, AccountError::InvalidCredentials), "{err:?}");
    // The real one deletes.
    let proof = passkey_proof(&server, &up.session, &auth);
    kanso_sync::delete_account_with_proof(&server, &up.session, &proof, None, "alice", &quiet)
        .unwrap();
    assert!(server.api(SERVER).account(&token_of(&up.session)).is_err());
}

#[test]
fn the_last_passkey_needs_the_recovery_key_to_go() {
    let server = MemoryServer::new();
    let auth = TestAuthenticator::new(7);
    let up = passkey_sign_up(&server, &auth, &mac(1));
    let id = auth.credential_id().to_vec();
    let proof = passkey_proof(&server, &up.session, &auth);
    assert!(matches!(
        kanso_sync::remove_passkey(&server, &up.session, &id, &proof, None),
        Err(AccountError::LastMethod)
    ));
    let proof = passkey_proof(&server, &up.session, &auth);
    kanso_sync::remove_passkey(&server, &up.session, &id, &proof, Some(&up.recovery_key)).unwrap();
    assert!(
        kanso_sync::methods(&server, &up.session)
            .unwrap()
            .passkeys
            .is_empty()
    );
}

#[test]
fn a_passkey_account_sets_its_first_password_once() {
    let server = MemoryServer::new();
    let auth = TestAuthenticator::new(8);
    let up = passkey_sign_up(&server, &auth, &mac(1));
    let password = Password::new("correct horse battery", "alice").unwrap();

    // `account/password` is for an account that has one.
    let err = server
        .api(SERVER)
        .change_password(
            &token_of(&up.session),
            &PasswordRequest {
                current_auth_key: b64(&[1; 32]),
                kdf: kanso_sync::KdfParams::TEST.into(),
                salt: b64(&[2; 16]),
                auth_key: b64(&[3; 32]),
                wrapped_key: b64(&[4; 73]),
                recovery_auth_key: b64(&[5; 32]),
                recovery_wrapped_key: b64(&[6; 73]),
            },
        )
        .unwrap_err();
    assert_eq!(err, ApiError::NoPassword);

    let secrets = MemorySecrets::new();
    let proof = passkey_proof(&server, &up.session, &auth);
    let changed = kanso_sync::set_password(
        &server,
        &secrets,
        &up.session,
        &password,
        &proof,
        None,
        &quiet,
    )
    .map_err(|err| err.error)
    .unwrap();
    assert_eq!(changed.signed_out_devices, 0);
    assert!(
        kanso_sync::methods(&server, &up.session)
            .unwrap()
            .has_password
    );

    // The new password signs in to the same key.
    let signed_in =
        kanso_sync::sign_in(&server, SERVER, "alice", &password, &mac(2), &quiet).unwrap();
    assert_eq!(signed_in.session.key_id(), up.session.key_id());

    // A second first password is refused.
    let proof = passkey_proof(&server, &up.session, &auth);
    let other = Password::new("another horse battery", "alice").unwrap();
    let err =
        kanso_sync::set_password(&server, &secrets, &up.session, &other, &proof, None, &quiet)
            .unwrap_err();
    assert!(
        matches!(err.error, AccountError::AlreadyHasPassword),
        "{:?}",
        err.error
    );
}

#[test]
fn a_password_account_adds_a_passkey_with_a_password_proof() {
    let server = MemoryServer::new();
    let password = Password::new("correct horse battery", "alice").unwrap();
    let up = kanso_sync::sign_up(
        &server,
        &MemorySecrets::new(),
        SERVER,
        "alice",
        &password,
        &mac(1),
        &quiet,
    )
    .map_err(|err| err.error)
    .unwrap();
    let auth_key = kanso_sync::password_proof(&server, &up.session, &password, &quiet).unwrap();
    let auth = TestAuthenticator::new(9);
    let start =
        kanso_sync::start_add_passkey(&server, &up.session, &Proof::Password(&auth_key)).unwrap();
    let info = kanso_sync::add_passkey(
        &server,
        &up.session,
        &start,
        auth.register(&start.challenge).unwrap(),
    )
    .unwrap();
    assert!(info.prf);
    // A provider proof can't add a way in.
    let err = kanso_sync::start_add_passkey(
        &server,
        &up.session,
        &Proof::Provider {
            flow: "A".repeat(43),
        },
    )
    .unwrap_err();
    assert!(matches!(err, AccountError::ReauthRequired), "{err:?}");
    // The new passkey unlocks a new Mac.
    let PasskeySignIn::Unlocked(second) =
        passkey_sign_in(&server, &MemorySecrets::new(), &auth, &mac(2)).unwrap()
    else {
        panic!("the PRF wrap unlocks")
    };
    assert_eq!(second.session.key_id(), up.session.key_id());
}

// ---------------------------------------------------------------- properties

fn config() -> ProptestConfig {
    ProptestConfig::with_cases(64)
}

proptest! {
    #![proptest_config(config())]

    /// A proof's wire form names its kind, carries the assertion's fields as
    /// base64url, and never the PRF output or the user handle.
    #[test]
    fn prop_a_passkey_proof_carries_exactly_the_assertion(
        credential in proptest::collection::vec(any::<u8>(), 16..64),
        data in proptest::collection::vec(any::<u8>(), 37..80),
        signature in proptest::collection::vec(any::<u8>(), 1..80),
        prf in any::<[u8; 32]>(),
        flow in "[A-Za-z0-9_-]{43}",
    ) {
        let proof = Proof::Passkey {
            flow: flow.clone(),
            assertion: kanso_sync::PasskeyAssertion {
                credential_id: credential.clone(),
                authenticator_data: data.clone(),
                client_data_json: b"{}".to_vec(),
                signature: signature.clone(),
                user_handle: Some(vec![1; 16]),
                prf: Some(prf),
            },
        };
        let json: serde_json::Value = serde_json::to_value(proof.to_wire()).unwrap();
        prop_assert_eq!(&json["kind"], "passkey");
        prop_assert_eq!(&json["flow"], flow.as_str());
        let assertion = &json["assertion"];
        prop_assert_eq!(assertion["credential_id"].as_str().map(str::to_owned), Some(b64(&credential)));
        prop_assert_eq!(assertion["authenticator_data"].as_str().map(str::to_owned), Some(b64(&data)));
        prop_assert_eq!(assertion["signature"].as_str().map(str::to_owned), Some(b64(&signature)));
        let text = json.to_string();
        prop_assert!(!text.contains(&b64(&prf)));
        let keys = |v: &serde_json::Value| {
            let mut k: Vec<String> = v.as_object().unwrap().keys().cloned().collect();
            k.sort();
            k
        };
        prop_assert_eq!(keys(&json), ["assertion", "flow", "kind"]);
        prop_assert_eq!(
            keys(assertion),
            ["authenticator_data", "client_data_json", "credential_id", "signature"]
        );
        let back: ProofWire = serde_json::from_value(json).unwrap();
        prop_assert_eq!(back, proof.to_wire());
    }

    /// The decision: the local key exactly when its ID is the server's;
    /// else PRF exactly when both a PRF output and a wrap are there (and
    /// then the key is the wrapped one, or a mismatch); else locked.
    #[test]
    fn prop_the_sign_in_decision(
        local in prop_oneof![Just(None), Just(Some(true)), Just(Some(false))],
        has_prf in any::<bool>(),
        right_prf in any::<bool>(),
        has_wrap in any::<bool>(),
        prf in any::<[u8; 32]>(),
        credential in proptest::collection::vec(any::<u8>(), 16..32),
        key_bytes in any::<[u8; 32]>(),
    ) {
        let key = DataKey::from_bytes(key_bytes);
        let id = key_id(&key);
        let local_key = match local {
            Some(true) => Some(DataKey::from_bytes(key_bytes)),
            Some(false) => Some(DataKey::generate()),
            None => None,
        };
        let wrapped = passkey_wrapped_key(&prf, "https://o.test", "a_x", &credential, &key);
        let mut wrong = prf;
        wrong[0] ^= 1;
        let used_prf = if right_prf { prf } else { wrong };
        let got = passkey_unlock(
            local_key.as_ref(),
            has_prf.then_some(&used_prf),
            has_wrap.then_some(wrapped.as_str()),
            "https://o.test",
            "a_x",
            &credential,
            &id,
        );
        if local == Some(true) {
            let (how, k) = got.unwrap();
            prop_assert_eq!(how, PasskeyUnlock::LocalKey);
            prop_assert_eq!(key_id(&k.unwrap()), id);
        } else if has_prf && has_wrap {
            if right_prf {
                let (how, k) = got.unwrap();
                prop_assert_eq!(how, PasskeyUnlock::Prf);
                prop_assert_eq!(key_id(&k.unwrap()), id);
            } else {
                prop_assert!(got.is_err());
            }
        } else {
            let (how, k) = got.unwrap();
            prop_assert_eq!(how, PasskeyUnlock::Locked);
            prop_assert!(k.is_none());
        }
    }

    /// The PRF wrap opens to its key under the same PRF, origin, account and
    /// credential, and under nothing else.
    #[test]
    fn prop_the_prf_wrap_round_trips_and_is_bound(
        prf in any::<[u8; 32]>(),
        credential in proptest::collection::vec(any::<u8>(), 16..64),
        change in 0usize..4,
    ) {
        let key = DataKey::generate();
        let id = key_id(&key);
        let wrapped = passkey_wrapped_key(&prf, "https://o.test", "a_x", &credential, &key);
        let opened = open_passkey_session_key(&prf, "https://o.test", "a_x", &credential, Some(&wrapped), &id).unwrap();
        prop_assert_eq!(key_id(&opened), id.clone());
        let mut other_prf = prf;
        let mut origin = "https://o.test";
        let mut account = "a_x";
        let mut other_credential = credential.clone();
        match change {
            0 => other_prf[31] ^= 0x80,
            1 => origin = "https://p.test",
            2 => account = "a_y",
            _ => other_credential[0] ^= 1,
        }
        prop_assert!(open_passkey_session_key(&other_prf, origin, account, &other_credential, Some(&wrapped), &id).is_err());
    }
}
