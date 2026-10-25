//! A provider link or proof flow belongs to the session that started it, as
//! the Worker's (security review of PR #67): its completion needs that
//! session still live and the account enabled; a proof flow is only its
//! device's; ending the session or disabling the account ends its flows.
//! Against `MemoryServer` with the fake GitHub device flow.

use kanso_sync::api::LoginDevice;
use kanso_sync::provider::{IdentityIntent, LinkOutcome, ProviderStart};
use kanso_sync::{
    AccountError, DeviceIdentity, MemorySecrets, MemoryServer, PasskeyCeremony, PasskeySignIn,
    Proof, Provider, Session, Step, TestAuthenticator,
};

const SERVER: &str = kanso_sync::MEMORY_PASSKEY_SERVER;
const START: i64 = 1_800_000_000_000;

fn mac(n: u8) -> DeviceIdentity {
    DeviceIdentity::from_parts(&format!("l_{:032x}", n), &format!("Mac {n}"), [n; 32])
}

fn quiet(_: Step) {}

struct World {
    server: MemoryServer,
    auth: TestAuthenticator,
    /// Mac 1's session, which starts the flows.
    a: Session,
    /// Mac 2's session.
    b: Session,
    recovery: kanso_sync::RecoveryKey,
}

fn world() -> World {
    let server = MemoryServer::new();
    server.set_now(START);
    server.set_providers(vec![Provider::Github]);
    server.set_github_device_flow("4242", "WXYZ-1234", 5);
    let auth = TestAuthenticator::new(1);
    let start = kanso_sync::start_passkey_sign_up(&server, SERVER, "alice", &mac(1)).unwrap();
    let registration = auth.register(&start.challenge).unwrap();
    let up = kanso_sync::sign_up_with_passkey(
        &server,
        &MemorySecrets::new(),
        &start,
        &mac(1),
        registration,
        &quiet,
    )
    .map_err(|err| err.error)
    .unwrap();
    let (a, recovery) = (up.session, up.recovery_key);
    let start = kanso_sync::start_passkey_sign_in(&server, SERVER, None, &LoginDevice::ephemeral())
        .unwrap();
    let assertion = auth.assert(&start.challenge).unwrap();
    let PasskeySignIn::Unlocked(b) = kanso_sync::sign_in_with_passkey(
        &server,
        &MemorySecrets::new(),
        &start,
        &mac(2),
        assertion,
        &quiet,
    )
    .unwrap() else {
        panic!("the PRF opens the wrap");
    };
    World {
        server,
        auth,
        a,
        b: b.session,
        recovery,
    }
}

fn device(n: u8, session: &Session) -> LoginDevice {
    LoginDevice::for_identity(&mac(n), &session.origin, &session.account_id)
}

/// Starts a GitHub link from Mac 1 with a passkey proof; the fake GitHub
/// will authorize the next poll.
fn start_link(w: &World) -> String {
    let start = kanso_sync::start_passkey_proof(&w.server, &w.a, None).unwrap();
    let proof = Proof::passkey(&start, w.auth.assert(&start.challenge).unwrap());
    let started = kanso_sync::start_identity(
        &w.server,
        &w.a,
        &Provider::Github,
        IdentityIntent::Link,
        Some(proof.to_wire()),
        None,
        None,
        &device(1, &w.a),
    )
    .unwrap();
    let ProviderStart::Github(github) = started else {
        panic!("a GitHub start");
    };
    w.server.push_github_answer("authorized");
    w.server.set_now(START + 10_000);
    github.flow_id
}

fn poll(w: &World, flow: &str) -> Result<LinkOutcome, AccountError> {
    kanso_sync::finish_identity(
        &w.server,
        &w.a,
        &Provider::Github,
        flow,
        None,
        &device(1, &w.a),
    )
}

fn linked(w: &World) -> usize {
    kanso_sync::methods(&w.server, &w.b).unwrap().linked.len()
}

#[test]
fn a_link_completes_while_its_session_is_live() {
    let w = world();
    let flow = start_link(&w);
    assert!(matches!(poll(&w, &flow), Ok(LinkOutcome::Linked { .. })));
    assert_eq!(linked(&w), 1);
}

#[test]
fn a_link_whose_session_ended_links_nothing() {
    let w = world();
    let flow = start_link(&w);
    let a_device = mac(1).device_id(&w.a.account_id);
    kanso_sync::revoke_device(&w.server, &w.b, &a_device).unwrap();
    let out = poll(&w, &flow);
    assert!(matches!(out, Err(AccountError::FlowExpired)), "{out:?}");
    assert_eq!(linked(&w), 0);
}

#[test]
fn a_link_on_a_disabled_account_links_nothing() {
    let w = world();
    let flow = start_link(&w);
    w.server.disable("alice");
    let out = poll(&w, &flow);
    assert!(matches!(out, Err(AccountError::FlowExpired)), "{out:?}");
    w.server.enable("alice");
    assert_eq!(linked(&w), 0);
}

/// Links GitHub, then finishes a GitHub proof flow started by Mac 1.
fn finished_proof(w: &World) -> String {
    let flow = start_link(w);
    assert!(matches!(poll(w, &flow), Ok(LinkOutcome::Linked { .. })));
    let started = kanso_sync::start_identity(
        &w.server,
        &w.a,
        &Provider::Github,
        IdentityIntent::Proof,
        None,
        None,
        None,
        &device(1, &w.a),
    )
    .unwrap();
    let ProviderStart::Github(github) = started else {
        panic!("a GitHub start");
    };
    w.server.push_github_answer("authorized");
    w.server.set_now(START + 30_000);
    assert!(matches!(
        poll(w, &github.flow_id),
        Ok(LinkOutcome::Proof { .. })
    ));
    github.flow_id
}

fn rotate(w: &World, session: &Session, flow: &str) -> Result<(), AccountError> {
    let proof = Proof::Provider { flow: flow.into() };
    kanso_sync::new_recovery_key_with_proof(
        &w.server,
        &MemorySecrets::new(),
        session,
        &proof,
        Some(&w.recovery),
        &quiet,
    )
    .map(|_| ())
    .map_err(|err| err.error)
}

#[test]
fn a_provider_proof_is_only_its_devices() {
    let w = world();
    let flow = finished_proof(&w);
    let out = rotate(&w, &w.b, &flow);
    assert!(matches!(out, Err(AccountError::FlowExpired)), "{out:?}");
    // Its own session may use it.
    rotate(&w, &w.a, &flow).unwrap();
}

/// Mac 1 signs in again after its session ended: the same device ID and proof.
fn sign_in_again(w: &World) -> Session {
    let start =
        kanso_sync::start_passkey_sign_in(&w.server, SERVER, None, &device(1, &w.a)).unwrap();
    let assertion = w.auth.assert(&start.challenge).unwrap();
    let PasskeySignIn::Unlocked(again) = kanso_sync::sign_in_with_passkey(
        &w.server,
        &MemorySecrets::new(),
        &start,
        &mac(1),
        assertion,
        &quiet,
    )
    .unwrap() else {
        panic!("the PRF opens the wrap");
    };
    again.session
}

/// As the Worker's migration 0010: ending a session ends the link, proof
/// and passkey-registration flows its device started, so none outlives it,
/// not even into the device's next session.
#[test]
fn ending_a_session_ends_its_flows() {
    let w = world();
    let flow = start_link(&w);
    let proof = kanso_sync::start_passkey_proof(&w.server, &w.a, None).unwrap();
    let proof = Proof::passkey(&proof, w.auth.assert(&proof.challenge).unwrap());
    let register = kanso_sync::start_add_passkey(&w.server, &w.a, &proof).unwrap();
    let a_device = mac(1).device_id(&w.a.account_id);
    kanso_sync::revoke_device(&w.server, &w.b, &a_device).unwrap();
    let again = sign_in_again(&w);

    // The link was used up before GitHub answered, by a session that is
    // live again: GitHub's answer no longer counts.
    let out = kanso_sync::finish_identity(
        &w.server,
        &again,
        &Provider::Github,
        &flow,
        None,
        &device(1, &again),
    );
    assert!(matches!(out, Err(AccountError::ProviderDenied)), "{out:?}");
    assert_eq!(linked(&w), 0);
    let other = TestAuthenticator::new(7);
    let out = kanso_sync::add_passkey(
        &w.server,
        &again,
        &register,
        other.register(&register.challenge).unwrap(),
    );
    assert!(matches!(out, Err(AccountError::FlowExpired)), "{out:?}");
}

/// Disabling an account ends all of its sessions' flows.
#[test]
fn disabling_an_account_ends_its_flows() {
    let w = world();
    let proof = kanso_sync::start_passkey_proof(&w.server, &w.a, None).unwrap();
    let proof = Proof::passkey(&proof, w.auth.assert(&proof.challenge).unwrap());
    let register = kanso_sync::start_add_passkey(&w.server, &w.a, &proof).unwrap();
    let pending = kanso_sync::start_passkey_proof(&w.server, &w.b, None).unwrap();
    w.server.disable("alice");
    w.server.enable("alice");
    let other = TestAuthenticator::new(7);
    let out = kanso_sync::add_passkey(
        &w.server,
        &w.a,
        &register,
        other.register(&register.challenge).unwrap(),
    );
    assert!(matches!(out, Err(AccountError::FlowExpired)), "{out:?}");
    let proof = Proof::passkey(&pending, w.auth.assert(&pending.challenge).unwrap());
    let out = kanso_sync::new_recovery_key_with_proof(
        &w.server,
        &MemorySecrets::new(),
        &w.b,
        &proof,
        None,
        &quiet,
    )
    .map(|_| ())
    .map_err(|err| err.error);
    assert!(matches!(out, Err(AccountError::FlowExpired)), "{out:?}");
}

/// A finished provider proof ends with its session too.
#[test]
fn a_finished_provider_proof_ends_with_its_session() {
    let w = world();
    let flow = finished_proof(&w);
    let a_device = mac(1).device_id(&w.a.account_id);
    kanso_sync::revoke_device(&w.server, &w.b, &a_device).unwrap();
    let again = sign_in_again(&w);
    let out = rotate(&w, &again, &flow);
    assert!(matches!(out, Err(AccountError::FlowExpired)), "{out:?}");
}
