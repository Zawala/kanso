//! An agent on this computer answering through `ChatProvider`, against the
//! fake ACP agent in process.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;

use kanso_acp::fake::{self, Script};
use kanso_acp::{AcpError, AgentProcess, Limits};
use kanso_ai::agent::{self, AgentCheck, AgentKind, AgentSettings, Starter};
use kanso_ai::{Event, Prompt, ToolRegistry};

/// The app keeps one agent at a time, so these tests take turns.
static ONE_AT_A_TIME: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn serial() -> std::sync::MutexGuard<'static, ()> {
    ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn settings(name: &str) -> AgentSettings {
    let workspace = std::env::temp_dir().join(format!("kanso-ai-agent-test-{name}"));
    AgentSettings {
        kind: AgentKind::Custom,
        // Any existing file passes the custom command check; the starter
        // below connects the fake instead of running it.
        command: Some(
            std::env::current_exe()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
        ),
        args: Vec::new(),
        env: Vec::new(),
        workspace,
    }
}

fn limits() -> Limits {
    Limits {
        cancel_grace: Duration::from_millis(500),
        ..Limits::default()
    }
}

/// Starts the fake with `script`, counting starts.
fn starter(script: Script, starts: Arc<AtomicUsize>) -> Starter {
    Arc::new(move |spec, limits| {
        starts.fetch_add(1, Ordering::Relaxed);
        let (from_agent, to_agent) =
            fake::in_process(script.clone()).map_err(|err| AcpError::Spawn(err.to_string()))?;
        AgentProcess::connect(
            &spec.label,
            from_agent,
            to_agent,
            limits,
            kanso_acp::reject_all(),
        )
    })
}

fn failing(starts: Arc<AtomicUsize>) -> Starter {
    Arc::new(move |_, _| {
        starts.fetch_add(1, Ordering::Relaxed);
        Err(AcpError::Spawn("no such file".to_owned()))
    })
}

fn ask(provider: &kanso_ai::SharedProvider, question: &str) -> Vec<Event> {
    let provider = provider.lock().unwrap();
    let prompt = Prompt {
        system: "Reply.".to_owned(),
        user: question.to_owned(),
    };
    provider
        .ask(
            &prompt,
            &[],
            ToolRegistry::new(),
            Box::new(|| {}),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap()
        .collect()
}

fn text(events: &[Event]) -> String {
    events
        .iter()
        .filter_map(|event| match event {
            Event::Text(text) => Some(text.as_str()),
            _ => None,
        })
        .collect()
}

#[test]
fn an_agent_answers_and_stays_warm_for_the_next_question() {
    let _serial = serial();
    let starts = Arc::new(AtomicUsize::new(0));
    let settings = settings("warm");
    let provider = agent::agent_client_with(
        &settings,
        limits(),
        starter(Script::Answer("From your notes.".into()), starts.clone()),
    );
    let first = ask(&provider, "one");
    assert_eq!(
        first.first(),
        Some(&Event::Status(format!("Starting {}…", settings.label())))
    );
    assert_eq!(text(&first), "From your notes.");
    assert_eq!(first.last(), Some(&Event::Done));
    let second = ask(&provider, "two");
    assert!(!second.iter().any(|event| matches!(event, Event::Status(_))));
    assert_eq!(text(&second), "From your notes.");
    assert_eq!(starts.load(Ordering::Relaxed), 1);
}

#[test]
fn stop_ends_the_answer_and_the_agent_answers_again() {
    let _serial = serial();
    let starts = Arc::new(AtomicUsize::new(0));
    let settings = settings("stop");
    let provider =
        agent::agent_client_with(&settings, limits(), starter(Script::Hang, starts.clone()));
    let cancel = Arc::new(AtomicBool::new(false));
    {
        let provider = provider.lock().unwrap();
        let prompt = Prompt {
            system: "Reply.".into(),
            user: "x".into(),
        };
        let mut events = provider
            .ask(
                &prompt,
                &[],
                ToolRegistry::new(),
                Box::new(|| {}),
                cancel.clone(),
            )
            .unwrap();
        assert!(matches!(events.next(), Some(Event::Status(_))));
        cancel.store(true, Ordering::Relaxed);
        assert_eq!(events.next(), None);
    }
    // The fake stopped on cancel, so the same process takes the next
    // question, which is stopped too.
    let cancel = Arc::new(AtomicBool::new(false));
    let stopper = {
        let cancel = cancel.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(300));
            cancel.store(true, Ordering::Relaxed);
        })
    };
    let events: Vec<Event> = {
        let provider = provider.lock().unwrap();
        let prompt = Prompt {
            system: "Reply.".into(),
            user: "again".into(),
        };
        provider
            .ask(&prompt, &[], ToolRegistry::new(), Box::new(|| {}), cancel)
            .unwrap()
            .collect()
    };
    stopper.join().unwrap();
    assert!(
        events.iter().all(|event| !matches!(event, Event::Error(_))),
        "{events:?}"
    );
    assert_eq!(starts.load(Ordering::Relaxed), 1);
}

#[test]
fn a_crash_is_reported_and_the_next_question_starts_it_again() {
    let _serial = serial();
    let starts = Arc::new(AtomicUsize::new(0));
    let settings = settings("crash");
    let provider = agent::agent_client_with(
        &settings,
        limits(),
        // The in-process fake can't exit, so a crash is a bad line instead.
        starter(Script::BadJson, starts.clone()),
    );
    let events = ask(&provider, "x");
    let Some(Event::Error(message)) = events.last() else {
        panic!("no error: {events:?}");
    };
    assert!(message.contains("couldn't read"), "{message}");
    ask(&provider, "y");
    assert_eq!(starts.load(Ordering::Relaxed), 2);
}

#[test]
fn three_failed_starts_stop_the_tries_until_find_again() {
    let _serial = serial();
    let starts = Arc::new(AtomicUsize::new(0));
    let settings = settings("fails");
    let provider = agent::agent_client_with(&settings, limits(), failing(starts.clone()));
    for _ in 0..3 {
        let events = ask(&provider, "x");
        let Some(Event::Error(message)) = events.last() else {
            panic!("no error: {events:?}");
        };
        assert!(message.contains("didn't start"), "{message}");
    }
    let events = ask(&provider, "x");
    let Some(Event::Error(message)) = events.last() else {
        panic!("no error: {events:?}");
    };
    assert!(message.contains("Find again"), "{message}");
    assert_eq!(starts.load(Ordering::Relaxed), 3);
}

#[test]
fn test_reports_the_agent_and_its_answer() {
    let _serial = serial();
    let settings = settings("check");
    let check = agent::check_with(
        &settings,
        limits(),
        starter(Script::Answer("OK".into()), Arc::new(AtomicUsize::new(0))),
    )
    .unwrap();
    assert_eq!(
        check,
        AgentCheck::Ready {
            name: "Fake Agent".into(),
            version: Some("1.2.3".into()),
            said: "OK".into(),
        }
    );
}

#[test]
fn test_asks_for_a_sign_in_and_signing_in_clears_it() {
    let _serial = serial();
    let settings = settings("auth");
    let starts = Arc::new(AtomicUsize::new(0));
    let start = starter(Script::Auth, starts.clone());
    let check = agent::check_with(&settings, limits(), start.clone()).unwrap();
    assert_eq!(
        check,
        AgentCheck::SignIn {
            methods: vec![("fake-login".into(), "Log in".into())]
        }
    );
    agent::sign_in_with(&settings, limits(), start.clone(), "fake-login").unwrap();
    let check = agent::check_with(&settings, limits(), start).unwrap();
    assert!(matches!(check, AgentCheck::Ready { .. }), "{check:?}");
    assert_eq!(starts.load(Ordering::Relaxed), 1);
}

#[test]
fn a_question_needing_a_sign_in_says_how() {
    let _serial = serial();
    let settings = AgentSettings {
        kind: AgentKind::Custom,
        ..settings("auth-ask")
    };
    let provider = agent::agent_client_with(
        &settings,
        limits(),
        starter(Script::Auth, Arc::new(AtomicUsize::new(0))),
    );
    let events = ask(&provider, "x");
    let Some(Event::Error(message)) = events.last() else {
        panic!("no error: {events:?}");
    };
    assert!(message.contains("needs you to sign in"), "{message}");
    assert!(message.contains("Settings › AI"), "{message}");
}

/// The manual check against a real agent installed on this computer:
/// `KANSO_REAL_AGENT=opencode cargo test -p kanso-ai --test agent real -- --ignored`
/// (claudeAgent, codex, geminiCli or opencode). It asks "Reply with OK." and
/// uses a little of that agent's subscription or quota.
#[test]
#[ignore = "starts a real agent; run on demand with KANSO_REAL_AGENT set"]
fn real_agent_answers_the_test_question() {
    let _serial = serial();
    let kind = match std::env::var("KANSO_REAL_AGENT").as_deref() {
        Ok("claudeAgent") => AgentKind::ClaudeAgent,
        Ok("codex") => AgentKind::Codex,
        Ok("geminiCli") => AgentKind::GeminiCli,
        Ok("opencode") => AgentKind::OpenCode,
        other => panic!("set KANSO_REAL_AGENT, not {other:?}"),
    };
    let settings = AgentSettings {
        kind,
        ..settings("real")
    };
    let check = agent::check(&settings);
    eprintln!("{check:?}");
    agent::shutdown();
    assert!(check.is_ok(), "{check:?}");
}
