//! A scriptable ACP agent for tests. It speaks the same newline-delimited
//! JSON-RPC as a real agent, over stdio (`kanso-acp-fake`) or in process
//! over pipes ([`in_process`]), and can stream, hang, crash, ask
//! permission, request the file system or a terminal, or misbehave.

use std::io::{BufRead, BufReader, Read, Write};
use std::time::Duration;

use serde_json::{Value, json};

/// What the fake agent does when prompted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Script {
    /// Streams `answer` in three pieces, then ends the turn.
    Answer(String),
    /// Streams back the prompt it was sent.
    Echo,
    /// Streams back the `initialize` params it got.
    EchoInit,
    /// Streams back the names of its environment and its working folder.
    Env,
    /// Never answers; stops when cancelled.
    Hang,
    /// Never answers and ignores cancel.
    IgnoreCancel,
    /// Sends one piece, then exits with code 3.
    Crash,
    /// Asks permission for a tool, then answers with what it was told.
    Permission,
    /// Asks Kanso to read a file and open a terminal, then answers with the
    /// error codes it got.
    Fs,
    /// Sends one line of 9 MB.
    Huge,
    /// Sends a line that isn't JSON.
    BadJson,
    /// Answers `session/new` with "authentication required" until
    /// `authenticate` is called.
    Auth,
    /// Answers `initialize` with protocol version 2.
    V2,
    /// Starts a child process that sleeps, and answers with its pid.
    SpawnChild,
    /// Waits `ms` before answering "late".
    Slow(u64),
    /// Sends `count` pieces of text as fast as it can.
    Flood(usize),
}

impl Script {
    /// The script named by the fake binary's arguments.
    pub fn from_args(args: &[String]) -> Script {
        let arg = |ix: usize| args.get(ix).cloned().unwrap_or_default();
        match arg(0).as_str() {
            "echo" => Script::Echo,
            "echo-init" => Script::EchoInit,
            "env" => Script::Env,
            "hang" => Script::Hang,
            "ignore-cancel" => Script::IgnoreCancel,
            "crash" => Script::Crash,
            "permission" => Script::Permission,
            "fs" => Script::Fs,
            "huge" => Script::Huge,
            "bad-json" => Script::BadJson,
            "auth" => Script::Auth,
            "v2" => Script::V2,
            "spawn-child" => Script::SpawnChild,
            "slow" => Script::Slow(arg(1).parse().unwrap_or(1000)),
            "flood" => Script::Flood(arg(1).parse().unwrap_or(1000)),
            "sleep" => {
                std::thread::sleep(Duration::from_secs(600));
                Script::Hang
            }
            _ => Script::Answer(if arg(1).is_empty() {
                "OK".to_owned()
            } else {
                arg(1)
            }),
        }
    }
}

struct Fake<W: Write> {
    out: W,
    script: Script,
    init: Value,
    signed_in: bool,
    /// The id of a prompt still open (hang scripts).
    open_prompt: Option<Value>,
    next_id: u64,
    children: Vec<std::process::Child>,
}

impl<W: Write> Fake<W> {
    fn send(&mut self, message: Value) {
        let mut line = serde_json::to_vec(&message).unwrap_or_default();
        line.push(b'\n');
        self.out.write_all(&line).ok();
        self.out.flush().ok();
    }

    fn reply(&mut self, id: &Value, result: Value) {
        self.send(json!({ "jsonrpc": "2.0", "id": id, "result": result }));
    }

    fn error(&mut self, id: &Value, code: i64, message: &str) {
        self.send(json!({
            "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message },
        }));
    }

    fn chunk(&mut self, session: &str, text: &str) {
        self.send(json!({
            "jsonrpc": "2.0",
            "method": "session/update",
            "params": {
                "sessionId": session,
                "update": {
                    "sessionUpdate": "agent_message_chunk",
                    "content": { "type": "text", "text": text },
                },
            },
        }));
    }

    fn answer(&mut self, id: &Value, session: &str, text: &str) {
        let chars: Vec<char> = text.chars().collect();
        let third = chars.len().div_ceil(3).max(1);
        for piece in chars.chunks(third) {
            let piece: String = piece.iter().collect();
            self.chunk(session, &piece);
        }
        self.reply(id, json!({ "stopReason": "end_turn" }));
    }

    /// Sends a request and reads lines until its response, answering
    /// nothing else in between.
    fn ask<R: BufRead>(&mut self, input: &mut R, method: &str, params: Value) -> Value {
        self.next_id += 1;
        let id = self.next_id + 900;
        self.send(json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }));
        let mut line = String::new();
        loop {
            line.clear();
            if input.read_line(&mut line).unwrap_or(0) == 0 {
                return Value::Null;
            }
            if let Ok(message) = serde_json::from_str::<Value>(&line)
                && message.get("id").and_then(Value::as_u64) == Some(id)
                && message.get("method").is_none()
            {
                return message;
            }
        }
    }

    fn prompt<R: BufRead>(&mut self, input: &mut R, id: Value, params: &Value) {
        let session = params
            .get("sessionId")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned();
        let text: String = params
            .get("prompt")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|block| block.get("text").and_then(Value::as_str))
            .collect();
        match self.script.clone() {
            Script::Answer(answer) => self.answer(&id, &session, &answer),
            Script::Echo => self.answer(&id, &session, &text),
            Script::EchoInit => {
                let init = self.init.to_string();
                self.answer(&id, &session, &init);
            }
            Script::Env => {
                let mut names: Vec<String> = std::env::vars().map(|(name, _)| name).collect();
                names.sort();
                let cwd = std::env::current_dir().unwrap_or_default();
                let text = format!("cwd={} env={}", cwd.display(), names.join(","));
                self.answer(&id, &session, &text);
            }
            Script::Hang | Script::IgnoreCancel => self.open_prompt = Some(id),
            Script::Crash => {
                self.chunk(&session, "partial ");
                std::process::exit(3);
            }
            Script::Permission => {
                let response = self.ask(
                    input,
                    "session/request_permission",
                    json!({
                        "sessionId": session,
                        "toolCall": { "toolCallId": "t1", "title": "Read ~/.ssh/id_ed25519", "kind": "read" },
                        "options": [
                            { "optionId": "yes", "name": "Allow", "kind": "allow_once" },
                            { "optionId": "always", "name": "Always", "kind": "allow_always" },
                            { "optionId": "no", "name": "Reject", "kind": "reject_once" },
                        ],
                    }),
                );
                let outcome = response
                    .pointer("/result/outcome/optionId")
                    .and_then(Value::as_str)
                    .unwrap_or("cancelled")
                    .to_owned();
                self.answer(&id, &session, &format!("permission={outcome}"));
            }
            Script::Fs => {
                let fs = self.ask(
                    input,
                    "fs/read_text_file",
                    json!({ "sessionId": session, "path": "/etc/hosts" }),
                );
                let terminal = self.ask(
                    input,
                    "terminal/create",
                    json!({ "sessionId": session, "command": "id" }),
                );
                let elicit = self.ask(input, "elicitation/create", json!({ "sessionId": session }));
                let code = |message: &Value| {
                    message
                        .pointer("/error/code")
                        .and_then(Value::as_i64)
                        .map_or("ok".to_owned(), |code| code.to_string())
                };
                let text = format!(
                    "fs={} terminal={} elicitation={}",
                    code(&fs),
                    code(&terminal),
                    code(&elicit)
                );
                self.answer(&id, &session, &text);
            }
            Script::Huge => {
                let huge = "x".repeat(9 * 1024 * 1024);
                self.chunk(&session, &huge);
                self.reply(&id, json!({ "stopReason": "end_turn" }));
            }
            Script::BadJson => {
                self.out.write_all(b"this is not json\n").ok();
                self.out.flush().ok();
            }
            Script::SpawnChild => {
                let me = std::env::current_exe().unwrap_or_default();
                match std::process::Command::new(me).arg("sleep").spawn() {
                    Ok(child) => {
                        let pid = child.id().to_string();
                        self.children.push(child);
                        self.answer(&id, &session, &pid);
                    }
                    Err(err) => self.answer(&id, &session, &format!("no child: {err}")),
                }
            }
            Script::Slow(ms) => {
                std::thread::sleep(Duration::from_millis(ms));
                self.answer(&id, &session, "late");
            }
            Script::Flood(count) => {
                for _ in 0..count {
                    self.chunk(&session, "word ");
                }
                self.reply(&id, json!({ "stopReason": "end_turn" }));
            }
            Script::Auth | Script::V2 => self.answer(&id, &session, "OK"),
        }
    }

    fn handle<R: BufRead>(&mut self, input: &mut R, message: Value) {
        let method = message.get("method").and_then(Value::as_str).unwrap_or("");
        let id = message.get("id").cloned();
        let params = message.get("params").cloned().unwrap_or(Value::Null);
        let Some(id) = id else {
            if method == "session/cancel"
                && self.script != Script::IgnoreCancel
                && let Some(open) = self.open_prompt.take()
            {
                self.reply(&open, json!({ "stopReason": "cancelled" }));
            }
            return;
        };
        match method {
            "initialize" => {
                self.init = params;
                let version = if self.script == Script::V2 { 2 } else { 1 };
                self.reply(
                    &id,
                    json!({
                        "protocolVersion": version,
                        "agentCapabilities": {
                            "mcpCapabilities": { "http": true },
                            "sessionCapabilities": { "close": {} },
                        },
                        "agentInfo": { "name": "fake-agent", "title": "Fake Agent", "version": "1.2.3" },
                        "authMethods": [{ "id": "fake-login", "name": "Log in", "description": "Opens a browser" }],
                    }),
                );
            }
            "authenticate" => {
                self.signed_in = true;
                self.reply(&id, json!({}));
            }
            "session/new" => {
                if self.script == Script::Auth && !self.signed_in {
                    self.error(&id, crate::AUTH_REQUIRED, "Authentication required");
                    return;
                }
                self.reply(
                    &id,
                    json!({
                        "sessionId": "fake-session",
                        "modes": {
                            "currentModeId": "code",
                            "availableModes": [
                                { "id": "code", "name": "Code" },
                                { "id": "ask", "name": "Ask" },
                                { "id": "read-only", "name": "Read only" },
                            ],
                        },
                    }),
                );
            }
            "session/set_mode" | "session/close" => self.reply(&id, json!({})),
            "session/prompt" => self.prompt(input, id, &params),
            _ => self.error(&id, crate::METHOD_NOT_FOUND, "Unknown method"),
        }
    }
}

/// Runs the fake agent on `input` and `output` until `input` closes.
pub fn serve(input: impl Read, output: impl Write, script: Script) {
    let mut input = BufReader::new(input);
    let mut fake = Fake {
        out: output,
        script,
        init: Value::Null,
        signed_in: false,
        open_prompt: None,
        next_id: 0,
        children: Vec::new(),
    };
    let mut line = String::new();
    loop {
        line.clear();
        match input.read_line(&mut line) {
            Ok(0) | Err(_) => break,
            Ok(_) => {
                if let Ok(message) = serde_json::from_str::<Value>(&line) {
                    fake.handle(&mut input, message);
                }
            }
        }
    }
    for mut child in fake.children.drain(..) {
        child.kill().ok();
        child.wait().ok();
    }
}

/// A fake agent on a thread, connected by pipes: what to read its messages
/// from, and what to write Kanso's to.
pub fn in_process(script: Script) -> std::io::Result<(std::io::PipeReader, std::io::PipeWriter)> {
    let (from_agent, agent_out) = std::io::pipe()?;
    let (agent_in, to_agent) = std::io::pipe()?;
    std::thread::Builder::new()
        .name("kanso-acp-fake".into())
        .spawn(move || serve(agent_in, agent_out, script))?;
    Ok((from_agent, to_agent))
}
