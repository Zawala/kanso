//! The ACP client against the fake agent: in process over pipes, and as a
//! real child process launched the way Kanso launches an agent.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use kanso_acp::fake::{self, Script};
use kanso_acp::{
    AcpError, AcpEvent, AgentProcess, AgentSpec, Limits, PermissionPolicy, StopReason, reject_all,
};

fn quick() -> Limits {
    Limits {
        initialize: Duration::from_secs(10),
        initialize_cold: Duration::from_secs(10),
        session_new: Duration::from_secs(10),
        first_text: Duration::from_secs(30),
        answer: Duration::from_secs(30),
        cancel_grace: Duration::from_millis(500),
        stuck: Duration::from_secs(5),
        ..Limits::default()
    }
}

fn connect(script: Script, limits: Limits, permission: PermissionPolicy) -> AgentProcess {
    let (from_agent, to_agent) = fake::in_process(script).unwrap();
    AgentProcess::connect("Fake Agent", from_agent, to_agent, limits, permission).unwrap()
}

fn workspace() -> tempfile::TempDir {
    tempfile::tempdir().unwrap()
}

/// Every event of a turn, until it ends.
fn collect(turn: impl Iterator<Item = AcpEvent>) -> Vec<AcpEvent> {
    turn.collect()
}

fn text_of(events: &[AcpEvent]) -> String {
    events
        .iter()
        .filter_map(|event| match event {
            AcpEvent::Text(text) => Some(text.as_str()),
            _ => None,
        })
        .collect()
}

fn spec(script: &[&str], dir: &std::path::Path) -> AgentSpec {
    AgentSpec {
        label: "Fake Agent".to_owned(),
        command: PathBuf::from(env!("CARGO_BIN_EXE_kanso-acp-fake")),
        args: script.iter().map(|arg| (*arg).to_owned()).collect(),
        env: kanso_acp::env::agent_env(
            std::env::vars().chain([
                ("KANSO_DATA_DIR".to_owned(), "/secret".to_owned()),
                ("KANSO_API_PASSWORD".to_owned(), "pw".to_owned()),
            ]),
            None,
            &[],
        ),
        workspace: dir.join("agents").join("workspace"),
    }
}

#[test]
fn text_streams_in_order_and_ends_the_turn() {
    let agent = connect(
        Script::Answer("The plan is yearly.".into()),
        quick(),
        reject_all(),
    );
    assert_eq!(agent.info().name, "fake-agent");
    assert_eq!(agent.info().version.as_deref(), Some("1.2.3"));
    assert!(agent.info().mcp_http);
    let dir = workspace();
    let session = agent.new_session(dir.path()).unwrap();
    let events = collect(session.prompt("What did we decide?", None).unwrap());
    assert_eq!(text_of(&events), "The plan is yearly.");
    assert!(
        events
            .iter()
            .filter(|event| matches!(event, AcpEvent::Text(_)))
            .count()
            >= 2
    );
    assert_eq!(events.last(), Some(&AcpEvent::Done(StopReason::EndTurn)));
}

#[test]
fn acp_never_advertises_fs_or_terminal() {
    let agent = connect(Script::EchoInit, quick(), reject_all());
    let dir = workspace();
    let session = agent.new_session(dir.path()).unwrap();
    let init: serde_json::Value =
        serde_json::from_str(&text_of(&collect(session.prompt("x", None).unwrap()))).unwrap();
    assert_eq!(init["protocolVersion"], 1);
    assert_eq!(init["clientCapabilities"], serde_json::json!({}));
    assert_eq!(init["clientInfo"]["name"], "kanso");
}

#[test]
fn the_most_restrictive_mode_is_chosen() {
    let agent = connect(Script::Answer("OK".into()), quick(), reject_all());
    let dir = workspace();
    let session = agent.new_session(dir.path()).unwrap();
    assert_eq!(session.mode(), Some("read-only"));
}

#[test]
fn fs_terminal_and_elicitation_requests_are_refused() {
    let agent = connect(Script::Fs, quick(), reject_all());
    let dir = workspace();
    let session = agent.new_session(dir.path()).unwrap();
    let events = collect(session.prompt("x", None).unwrap());
    assert_eq!(
        text_of(&events),
        "fs=-32601 terminal=-32601 elicitation=-32601"
    );
    let diagnostics = agent.diagnostics();
    assert!(diagnostics.contains("refused fs/read_text_file"));
    assert!(diagnostics.contains("refused terminal/create"));
    assert!(diagnostics.contains("refused elicitation/create"));
}

#[test]
fn a_permission_request_with_no_person_is_rejected() {
    let agent = connect(Script::Permission, quick(), reject_all());
    let dir = workspace();
    let session = agent.new_session(dir.path()).unwrap();
    let events = collect(session.prompt("x", None).unwrap());
    assert_eq!(text_of(&events), "permission=no");
    assert!(events.iter().any(|event| matches!(
        event,
        AcpEvent::Notice(notice) if notice.contains("Read ~/.ssh/id_ed25519") && notice.contains("Kanso said no")
    )));
}

#[test]
fn an_allowed_permission_is_allowed_once_never_always() {
    let asked = Arc::new(std::sync::Mutex::new(Vec::new()));
    let policy: PermissionPolicy = {
        let asked = asked.clone();
        Arc::new(move |request| {
            asked.lock().unwrap().push(request.title.clone());
            true
        })
    };
    let agent = connect(Script::Permission, quick(), policy);
    let dir = workspace();
    let session = agent.new_session(dir.path()).unwrap();
    let events = collect(session.prompt("x", None).unwrap());
    assert_eq!(text_of(&events), "permission=yes");
    assert_eq!(
        *asked.lock().unwrap(),
        vec!["Read ~/.ssh/id_ed25519".to_owned()]
    );
}

#[test]
fn cancel_ends_the_turn_with_cancelled() {
    let agent = connect(Script::Hang, quick(), reject_all());
    let dir = workspace();
    let session = agent.new_session(dir.path()).unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    let mut turn = session.prompt("x", Some(cancel.clone())).unwrap();
    cancel.store(true, Ordering::Relaxed);
    assert_eq!(turn.next(), Some(AcpEvent::Done(StopReason::Cancelled)));
    assert_eq!(turn.next(), None);
    assert!(agent.is_alive());
}

#[test]
fn an_ignored_cancel_kills_the_agent_after_the_grace() {
    let agent = connect(Script::IgnoreCancel, quick(), reject_all());
    let dir = workspace();
    let session = agent.new_session(dir.path()).unwrap();
    let mut turn = session.prompt("x", None).unwrap();
    let started = Instant::now();
    turn.cancel();
    assert_eq!(turn.next(), Some(AcpEvent::Error(AcpError::CancelIgnored)));
    assert!(started.elapsed() < Duration::from_secs(5));
    assert!(!agent.is_alive());
}

#[test]
fn no_words_yet_says_still_working_and_goes_on() {
    let limits = Limits {
        first_text: Duration::from_millis(100),
        ..quick()
    };
    let agent = connect(Script::Slow(400), limits, reject_all());
    let dir = workspace();
    let session = agent.new_session(dir.path()).unwrap();
    let events = collect(session.prompt("x", None).unwrap());
    assert_eq!(events.first(), Some(&AcpEvent::StillWorking));
    assert_eq!(text_of(&events), "late");
    assert_eq!(events.last(), Some(&AcpEvent::Done(StopReason::EndTurn)));
}

#[test]
fn an_answer_over_its_time_is_cancelled() {
    let limits = Limits {
        answer: Duration::from_millis(150),
        ..quick()
    };
    let agent = connect(Script::Hang, limits, reject_all());
    let dir = workspace();
    let session = agent.new_session(dir.path()).unwrap();
    let events = collect(session.prompt("x", None).unwrap());
    assert_eq!(
        events.last(),
        Some(&AcpEvent::Error(AcpError::Timeout(
            kanso_acp::Stage::Answer
        )))
    );
}

#[test]
fn authentication_required_is_its_own_error_and_authenticate_clears_it() {
    let agent = connect(Script::Auth, quick(), reject_all());
    let dir = workspace();
    assert!(matches!(
        agent.new_session(dir.path()),
        Err(AcpError::AuthRequired(_))
    ));
    assert_eq!(agent.info().auth_methods[0].id, "fake-login");
    agent.authenticate("fake-login").unwrap();
    let session = agent.new_session(dir.path()).unwrap();
    assert_eq!(text_of(&collect(session.prompt("x", None).unwrap())), "OK");
}

#[test]
fn another_protocol_version_is_refused() {
    let (from_agent, to_agent) = fake::in_process(Script::V2).unwrap();
    let result = AgentProcess::connect("Fake Agent", from_agent, to_agent, quick(), reject_all());
    let Err(err) = result else {
        panic!("a version 2 agent was accepted");
    };
    assert_eq!(err, AcpError::ProtocolVersion(2));
    assert!(err.describe("Fake Agent").contains("Kanso needs version 1"));
}

#[test]
fn bad_json_is_a_protocol_error_not_a_panic() {
    let agent = connect(Script::BadJson, quick(), reject_all());
    let dir = workspace();
    let session = agent.new_session(dir.path()).unwrap();
    let events = collect(session.prompt("x", None).unwrap());
    assert!(matches!(
        events.last(),
        Some(AcpEvent::Error(AcpError::Protocol(_)))
    ));
    assert!(!agent.is_alive());
}

#[test]
fn a_flood_of_words_arrives_whole() {
    let agent = connect(Script::Flood(5_000), quick(), reject_all());
    let dir = workspace();
    let session = agent.new_session(dir.path()).unwrap();
    let mut turn = session.prompt("x", None).unwrap();
    // Read slowly at first, so the queue fills and the reader has to wait.
    std::thread::sleep(Duration::from_millis(200));
    let events: Vec<AcpEvent> = turn.by_ref().collect();
    assert_eq!(text_of(&events), "word ".repeat(5_000));
    assert_eq!(events.last(), Some(&AcpEvent::Done(StopReason::EndTurn)));
}

#[test]
fn request_ids_are_never_reused_across_sessions() {
    let agent = connect(Script::Echo, quick(), reject_all());
    let dir = workspace();
    for round in 0..5 {
        let session = agent.new_session(dir.path()).unwrap();
        let question = format!("question {round}");
        let events = collect(session.prompt(&question, None).unwrap());
        assert_eq!(text_of(&events), question);
    }
    assert!(!agent.diagnostics().contains("a response to no request"));
}

#[test]
fn a_launched_agent_answers_from_its_private_folder() {
    let dir = workspace();
    let spec = spec(&["answer", "Hello from a process"], dir.path());
    let agent = AgentProcess::start(&spec, quick(), reject_all()).unwrap();
    let session = agent.new_session(&spec.workspace).unwrap();
    let events = collect(session.prompt("x", None).unwrap());
    assert_eq!(text_of(&events), "Hello from a process");
    assert!(spec.workspace.join("README.txt").exists());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let mode = std::fs::metadata(&spec.workspace)
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o700);
    }
}

#[test]
fn env_allowlist_has_no_kanso_secrets_in_the_child() {
    let dir = workspace();
    let spec = spec(&["env"], dir.path());
    let agent = AgentProcess::start(&spec, quick(), reject_all()).unwrap();
    let session = agent.new_session(&spec.workspace).unwrap();
    let said = text_of(&collect(session.prompt("x", None).unwrap()));
    let (cwd, env) = said.split_once(" env=").unwrap();
    let names: Vec<&str> = env.split(',').collect();
    assert!(
        names
            .iter()
            .all(|name| !name.to_ascii_uppercase().starts_with("KANSO_"))
    );
    assert!(
        names.iter().all(|name| kanso_acp::env::passes(name)),
        "{names:?}"
    );
    let cwd = std::fs::canonicalize(cwd.trim_start_matches("cwd=")).unwrap();
    assert_eq!(cwd, std::fs::canonicalize(&spec.workspace).unwrap());
}

#[test]
fn a_crash_mid_answer_keeps_the_partial_text_and_says_so() {
    let dir = workspace();
    let spec = spec(&["crash"], dir.path());
    let agent = AgentProcess::start(&spec, quick(), reject_all()).unwrap();
    let session = agent.new_session(&spec.workspace).unwrap();
    let events = collect(session.prompt("x", None).unwrap());
    assert_eq!(text_of(&events), "partial ");
    let Some(AcpEvent::Error(err)) = events.last() else {
        panic!("the crash wasn't reported: {events:?}");
    };
    assert_eq!(err, &AcpError::Exited(Some(3)));
    assert!(err.describe("Fake Agent").contains("exit code 3"));
}

#[test]
fn a_line_over_the_cap_kills_the_agent() {
    let dir = workspace();
    let spec = spec(&["huge"], dir.path());
    let agent = AgentProcess::start(&spec, quick(), reject_all()).unwrap();
    let session = agent.new_session(&spec.workspace).unwrap();
    let events = collect(session.prompt("x", None).unwrap());
    assert_eq!(events.last(), Some(&AcpEvent::Error(AcpError::TooLarge)));
    assert!(!agent.is_alive());
}

#[test]
fn a_missing_command_is_a_spawn_error() {
    let dir = workspace();
    let mut spec = spec(&[], dir.path());
    spec.command = dir.path().join("no-such-agent");
    assert!(matches!(
        AgentProcess::start(&spec, quick(), reject_all()),
        Err(AcpError::Spawn(_))
    ));
}

#[cfg(unix)]
#[test]
fn acp_child_tree_is_killed_on_drop() {
    let dir = workspace();
    let spec = spec(&["spawn-child"], dir.path());
    let agent = AgentProcess::start(&spec, quick(), reject_all()).unwrap();
    let session = agent.new_session(&spec.workspace).unwrap();
    let pid: i32 = text_of(&collect(session.prompt("x", None).unwrap()))
        .parse()
        .unwrap();
    let alive = |pid: i32| unsafe { libc::kill(pid, 0) == 0 };
    assert!(alive(pid));
    drop(session);
    drop(agent);
    let until = Instant::now() + Duration::from_secs(5);
    // The grandchild is reaped by init once killed; until then it may linger
    // as a zombie, which `kill(pid, 0)` still sees.
    while alive(pid) && Instant::now() < until {
        std::thread::sleep(Duration::from_millis(20));
        unsafe {
            libc::waitpid(pid, std::ptr::null_mut(), libc::WNOHANG);
        }
    }
    let state = std::process::Command::new("ps")
        .args(["-o", "stat=", "-p", &pid.to_string()])
        .output()
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_owned())
        .unwrap_or_default();
    assert!(
        state.is_empty() || state.starts_with('Z'),
        "the grandchild survived: {state}"
    );
}
