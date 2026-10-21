//! `kanso mcp`: Kanso's MCP server over standard input and output, for AI
//! agents that start a program rather than connect to a URL (Claude
//! Desktop, say). Each JSON-RPC message read from a line of input goes to
//! the running app's `/mcp` with the local API's sign-in, and each answer
//! is written back on a line of its own. It never opens the library
//! itself: an agent keeps this process running, and holding the library
//! would stop Kanso from opening.

use std::io::{BufRead as _, Write as _};
use std::path::Path;
use std::time::Duration;

use anyhow::{Context as _, Result};
use serde_json::{Value, json};

use crate::config::ServerSettings;

const USAGE: &str = "\
Usage: kanso mcp

Serves Kanso's MCP tools over standard input and output, for AI agents
that start a program (Claude Desktop, Codex). It talks to the running app,
so Kanso must be open with Settings › Local API on.
";

/// The JSON-RPC error code for a request the app couldn't answer.
const UNAVAILABLE: i64 = -32000;

pub fn run_cli(data_dir: &Path, rest: &[String]) -> Result<()> {
    if rest.iter().any(|arg| arg == "help" || arg == "--help") {
        println!("{USAGE}");
        return Ok(());
    }
    let config = crate::load_config(data_dir)?;
    let relay = Relay::new(&config.server);
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout().lock();
    for line in stdin.lock().lines() {
        let line = line.context("reading standard input")?;
        if line.trim().is_empty() {
            continue;
        }
        if let Some(answer) = relay.send(&config.server, &line) {
            writeln!(stdout, "{answer}")?;
            stdout.flush()?;
        }
    }
    Ok(())
}

/// The running app's MCP endpoint and how to sign in to it.
struct Relay {
    endpoint: String,
    authorization: String,
    agent: ureq::Agent,
}

impl Relay {
    fn new(server: &ServerSettings) -> Relay {
        Relay {
            endpoint: crate::settings::mcp_endpoint(&server.bind_address, server.port),
            authorization: kanso_server::basic_header(&server.username, &server.password),
            agent: ureq::Agent::config_builder()
                .http_status_as_error(false)
                .max_redirects(0)
                .timeout_global(Some(Duration::from_secs(120)))
                .build()
                .into(),
        }
    }

    /// Passes one message on, and gives back the line to answer with:
    /// the app's answer, an error for a request it couldn't answer, or
    /// nothing for a notification.
    fn send(&self, server: &ServerSettings, message: &str) -> Option<String> {
        let id = request_id(message);
        if !server.enabled {
            return id.map(|id| {
                error(
                    id,
                    "Kanso's local API is off. Turn it on in Kanso's Settings › Local API.",
                )
            });
        }
        let response = self
            .agent
            .post(&self.endpoint)
            .header("authorization", &self.authorization)
            .header("content-type", "application/json")
            .header("accept", "application/json, text/event-stream")
            .send(message.as_bytes());
        let mut response = match response {
            Ok(response) => response,
            Err(_) => {
                return id.map(|id| error(id, "Kanso isn't running. Open Kanso, then try again."));
            }
        };
        let status = response.status().as_u16();
        let body = response.body_mut().read_to_string().unwrap_or_default();
        answer(id, status, &body)
    }
}

/// The ID of a JSON-RPC request; `None` for a notification (or a line that
/// isn't JSON-RPC, which the app answers itself).
fn request_id(message: &str) -> Option<Value> {
    let value: Value = serde_json::from_str(message).ok()?;
    value.get("id").filter(|id| !id.is_null()).cloned()
}

/// The line to write for the app's answer to a request with `id`: its body
/// on one line, or an error saying what went wrong; nothing when there is
/// nothing to answer.
fn answer(id: Option<Value>, status: u16, body: &str) -> Option<String> {
    if let Ok(value) = serde_json::from_str::<Value>(body) {
        return Some(value.to_string());
    }
    let id = id?;
    Some(match status {
        202 | 204 => return None,
        401 => error(
            id,
            "Kanso didn't accept the sign-in in its settings. Check Settings › Local API.",
        ),
        _ => error(id, &format!("Kanso answered with HTTP {status}.")),
    })
}

fn error(id: Value, message: &str) -> String {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": UNAVAILABLE, "message": message}})
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn answer_examples() {
        let ok = r#"{"jsonrpc":"2.0","id":1,"result":{}}"#;
        assert_eq!(answer(Some(json!(1)), 200, ok).as_deref(), Some(ok));
        // A pretty-printed answer still goes back on one line.
        let pretty = "{\n  \"jsonrpc\": \"2.0\",\n  \"id\": 1,\n  \"result\": {}\n}";
        assert_eq!(answer(Some(json!(1)), 200, pretty).as_deref(), Some(ok));
        assert_eq!(answer(None, 202, ""), None);
        assert_eq!(answer(Some(json!(1)), 202, ""), None);
        let refused: Value =
            serde_json::from_str(&answer(Some(json!("a")), 401, "").unwrap()).unwrap();
        assert_eq!(refused["id"], "a");
        assert_eq!(refused["error"]["code"], UNAVAILABLE);
        assert_eq!(
            request_id(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#),
            None
        );
        assert_eq!(
            request_id(r#"{"jsonrpc":"2.0","id":7,"method":"ping"}"#),
            Some(json!(7))
        );
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: 256, ..ProptestConfig::default() })]

        /// Whatever the app answers, what goes back is one line of JSON,
        /// and a request whose answer was lost still gets an error with
        /// its own ID.
        #[test]
        fn prop_answers_are_one_line_of_json(
            id in prop_oneof![any::<i64>().prop_map(|n| json!(n)), "[a-z0-9-]{1,12}".prop_map(|s| json!(s))],
            status in prop_oneof![Just(200u16), Just(400), Just(401), Just(500), Just(502)],
            body in prop_oneof![Just(String::new()), "[ -~]{0,40}", Just("{\n \"x\": 1\n}".to_owned())],
        ) {
            let line = answer(Some(id.clone()), status, &body).expect("a request is answered");
            prop_assert!(!line.contains('\n'));
            let value: Value = serde_json::from_str(&line).unwrap();
            if serde_json::from_str::<Value>(&body).is_err() {
                prop_assert_eq!(&value["id"], &id);
                prop_assert_eq!(&value["error"]["code"], &json!(UNAVAILABLE));
            }
        }
    }
}
