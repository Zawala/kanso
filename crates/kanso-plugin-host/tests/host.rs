//! Drives the real `kanso-plugin-host` binary over its pipes, the way the
//! app does: empty environment, a temporary folder as its working
//! directory, newline-delimited JSON in and out.

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::{Value, json};

struct Host {
    child: Child,
    input: Option<ChildStdin>,
    output: BufReader<ChildStdout>,
    _dir: tempdir::Dir,
}

/// A throwaway working directory, removed on drop.
mod tempdir {
    pub struct Dir(pub std::path::PathBuf);

    impl Dir {
        pub fn new() -> Dir {
            static N: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
            let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let path =
                std::env::temp_dir().join(format!("kanso-plugin-host-{}-{n}", std::process::id()));
            std::fs::create_dir_all(&path).unwrap();
            Dir(path)
        }
    }

    impl Drop for Dir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
}

const THEME: &str = r##"{"dark":false,"background":"#ffffff","text":"#000000","muted":"#666666","accent":"#0066ff","border":"#cccccc","surface":"#f4f4f4"}"##;

fn budgets() -> Value {
    json!({ "commandMs": 200, "renderMs": 200, "startupMs": 1000, "memory": 16 << 20, "stack": 1 << 20 })
}

impl Host {
    fn spawn() -> Host {
        let dir = tempdir::Dir::new();
        let mut child = Command::new(env!("CARGO_BIN_EXE_kanso-plugin-host"))
            .env_clear()
            .current_dir(&dir.0)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let input = child.stdin.take();
        let output = BufReader::new(child.stdout.take().unwrap());
        Host {
            child,
            input,
            output,
            _dir: dir,
        }
    }

    /// Spawns a host and starts `code`; returns it with the `started` message.
    fn started(code: &str) -> (Host, Value) {
        let mut host = Host::spawn();
        host.send(json!({
            "op": "start",
            "plugin": { "id": "sample", "name": "Sample", "version": "1.0.0" },
            "code": code,
            "budgets": budgets(),
        }));
        let started = host.receive();
        (host, started)
    }

    fn send(&mut self, message: Value) {
        let input = self.input.as_mut().unwrap();
        writeln!(input, "{message}").unwrap();
        input.flush().unwrap();
    }

    fn receive(&mut self) -> Value {
        let mut line = String::new();
        let read = self.output.read_line(&mut line).unwrap();
        assert!(read > 0, "the host closed its output");
        serde_json::from_str(&line).unwrap_or_else(|e| panic!("{e}: {line}"))
    }

    /// Runs a command that makes no host calls.
    fn invoke(&mut self, id: u64, command: &str, args: Value) -> Value {
        self.send(json!({ "op": "invoke", "id": id, "command": command, "args": args }));
        self.receive()
    }

    fn stop(mut self) -> std::process::ExitStatus {
        self.send(json!({ "op": "stop" }));
        self.child.wait().unwrap()
    }
}

impl Drop for Host {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn count_words_asks_the_app_for_the_note() {
    let code = r#"
        kanso.commands.register({
            id: 'count-words', title: 'Count words',
            run() {
                const id = kanso.editor.noteId();
                const note = kanso.notes.get(id);
                const words = note.body.split(/\s+/).filter(Boolean).length;
                kanso.log('info', 'counted');
                return { text: 'Words: ' + words };
            },
        });
    "#;
    let (mut host, started) = Host::started(code);
    assert_eq!(started["op"], "started");
    assert_eq!(started["ok"], true);
    assert_eq!(started["registered"]["commands"][0]["id"], "count-words");

    host.send(json!({ "op": "invoke", "id": 7, "command": "count-words", "args": {} }));
    let first = host.receive();
    assert_eq!(
        first,
        json!({ "op": "host", "id": 7, "call": "editor.noteId", "args": {} })
    );
    host.send(json!({ "op": "reply", "id": 7, "ok": true, "value": "note:abc" }));
    let second = host.receive();
    assert_eq!(
        second,
        json!({ "op": "host", "id": 7, "call": "notes.get", "args": { "id": "note:abc" } })
    );
    host.send(json!({ "op": "reply", "id": 7, "ok": true, "value": { "id": "note:abc", "body": "one two  three four" } }));
    assert_eq!(
        host.receive(),
        json!({ "op": "log", "level": "info", "text": "counted" })
    );
    assert_eq!(
        host.receive(),
        json!({ "op": "result", "id": 7, "ok": true, "value": { "text": "Words: 4" } })
    );
    assert!(host.stop().success());
}

#[test]
fn a_failed_host_call_throws_in_the_plugin() {
    let code = r#"
        kanso.commands.register({ id: 'read', title: 'Read', run() { return kanso.notes.get('note:x'); } });
    "#;
    let (mut host, _) = Host::started(code);
    host.send(json!({ "op": "invoke", "id": 1, "command": "read", "args": {} }));
    assert_eq!(host.receive()["op"], "host");
    host.send(json!({ "op": "reply", "id": 1, "ok": false, "error": { "code": "not_found", "message": "not found" } }));
    let result = host.receive();
    assert_eq!(result["ok"], false);
    assert_eq!(result["error"]["code"], "not_found");
}

#[test]
fn an_endless_loop_times_out_and_the_host_answers_again() {
    let code = r#"
        kanso.commands.register({ id: 'spin', title: 'Spin', run() { while (true) {} } });
        kanso.commands.register({ id: 'ok', title: 'Ok', run() { return 'fine'; } });
    "#;
    let (mut host, _) = Host::started(code);
    let began = Instant::now();
    let result = host.invoke(1, "spin", json!({}));
    assert!(began.elapsed() < Duration::from_secs(3));
    assert_eq!(result["ok"], false);
    assert_eq!(result["error"]["code"], "timeout");
    let next = host.invoke(2, "ok", json!({}));
    assert_eq!(next["value"], "fine");
    assert_eq!(next["id"], 2);
}

#[test]
fn allocating_too_much_is_a_limit_and_the_host_answers_again() {
    let code = r#"
        kanso.commands.register({ id: 'array', title: 'A', run() { return new Array(1e9).fill(1).length; } });
        kanso.commands.register({ id: 'string', title: 'S', run() { const a = []; for (let i = 0; ; i++) a.push('x'.repeat(1000) + i); } });
        kanso.commands.register({ id: 'ok', title: 'Ok', run() { return 'fine'; } });
    "#;
    let (mut host, _) = Host::started(code);
    for (id, command) in [(1, "array"), (2, "string")] {
        let result = host.invoke(id, command, json!({}));
        assert_eq!(result["error"]["code"], "limit", "{command}: {result}");
    }
    assert_eq!(host.invoke(3, "ok", json!({}))["value"], "fine");
}

#[test]
fn a_renderer_that_fetches_is_denied_and_the_app_hears_nothing() {
    let code = r#"
        kanso.fences.register('chart', () => kanso.fetch('https://example.com'));
        kanso.fences.register('plain', ({ source }) => '<svg>' + source + '</svg>');
    "#;
    let (mut host, started) = Host::started(code);
    assert_eq!(started["registered"]["fences"], json!(["chart", "plain"]));
    let theme: Value = serde_json::from_str(THEME).unwrap();
    host.send(
        json!({ "op": "render", "id": 5, "language": "chart", "source": "x", "theme": theme }),
    );
    // The very next message is the result: no `host` message came first.
    let result = host.receive();
    assert_eq!(result["op"], "result");
    assert_eq!(result["error"]["code"], "denied");
    assert_eq!(
        result["error"]["message"],
        "a renderer can't use kanso.fetch"
    );
    host.send(
        json!({ "op": "render", "id": 6, "language": "plain", "source": "hi", "theme": theme }),
    );
    assert_eq!(host.receive()["value"], "<svg>hi</svg>");
}

#[test]
fn a_button_press_reaches_its_handler() {
    let (mut host, started) = Host::started("kanso.ui.onPress('go', (v) => 'hello ' + v.name);");
    assert_eq!(started["registered"]["presses"], json!(["go"]));
    host.send(json!({ "op": "press", "id": 9, "button": "go", "values": { "name": "Ada" } }));
    assert_eq!(host.receive()["value"], "hello Ada");
}

#[test]
fn the_dangerous_globals_are_not_there() {
    let names = [
        "require",
        "console",
        "setTimeout",
        "setInterval",
        "process",
        "fetch",
        "eval",
        "__kansoCall",
        "__kansoLog",
        "std",
        "os",
        "XMLHttpRequest",
    ];
    let probe = names
        .iter()
        .map(|n| format!("typeof {n}"))
        .collect::<Vec<_>>()
        .join(", ");
    let code = format!(
        "kanso.commands.register({{ id: 'probe', title: 'P', run() {{ return [{probe}]; }} }});"
    );
    let (mut host, _) = Host::started(&code);
    let result = host.invoke(1, "probe", json!({}));
    assert_eq!(result["value"], json!(vec!["undefined"; names.len()]));
}

#[test]
fn a_script_that_fails_to_start_is_reported_and_the_host_stays_polite() {
    let (mut host, started) = Host::started("throw new Error('nope');");
    assert_eq!(started["ok"], false);
    assert_eq!(started["error"]["code"], "thrown");
    assert_eq!(started["error"]["message"], "nope");
    let result = host.invoke(1, "anything", json!({}));
    assert_eq!(result["ok"], false);
    assert!(host.stop().success());
}

#[test]
fn a_malformed_first_line_exits_with_an_error() {
    for first in [
        "not json at all",
        r#"{"op":"invoke","id":1,"command":"x"}"#,
        r#"{"op":"stop"}"#,
    ] {
        let mut host = Host::spawn();
        let input = host.input.as_mut().unwrap();
        writeln!(input, "{first}").unwrap();
        input.flush().unwrap();
        let status = host.child.wait().unwrap();
        assert_eq!(status.code(), Some(2), "{first}");
        let mut stdout = String::new();
        std::io::Read::read_to_string(&mut host.output, &mut stdout).unwrap();
        assert!(stdout.is_empty(), "{first}: {stdout}");
    }
}

#[test]
fn a_wrong_reply_ends_the_host() {
    let code = "kanso.commands.register({ id: 'read', title: 'R', run() { return kanso.settings.get(); } });";
    let (mut host, _) = Host::started(code);
    host.send(json!({ "op": "invoke", "id": 1, "command": "read", "args": {} }));
    assert_eq!(host.receive()["op"], "host");
    host.send(json!({ "op": "reply", "id": 99, "ok": true, "value": null }));
    assert_eq!(host.child.wait().unwrap().code(), Some(3));
}

#[test]
fn closing_the_input_ends_the_host_quietly() {
    let (mut host, _) = Host::started("");
    host.input = None;
    assert!(host.child.wait().unwrap().success());
}
