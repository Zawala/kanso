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
}

fn world() -> World {
    let server = MemoryServer::new();
    server.set_now(START);
    server.set_providers(vec![Provider::Github]);
    server.set_github_device_flow("4242", "WXYZ-1234", 5);
    let auth = TestAuthenticator::new(1);
    let start = kanso_sync::start_passkey_sign_up(&server, SERVER, "alice", &mac(1)).unwrap();
    let registration = auth.register(&start.challenge).unwrap();
    let a = kanso_sync::sign_up_with_passkey(
        &server,
        &MemorySecrets::new(),
        &start,
        &mac(1),
        registration,
        &quiet,
    )
    .map_err(|err| err.error)
    .unwrap()
    .session;
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
