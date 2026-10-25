//! The plugin host: reads requests from the app on standard input, runs them
//! in the plugin's sandboxed runtime and answers on standard output.
//!
//! The first message must be `start`. After that the host answers one
//! request at a time; while a request runs, the plugin's host calls go out as
//! `host` messages and are answered by `reply` messages with the same id.
//! Standard output carries protocol messages only. On a fatal error the host
//! writes one line to standard error and exits: 2 if the first message isn't
//! a good `start`, 3 if the app breaks the protocol afterwards. It reads no
//! files, environment variables or network.

use std::io::{BufReader, StdinLock, StdoutLock, Write};
use std::process::ExitCode;

use kanso_plugin_host::protocol::{
    self, CODE_LIMIT, CODE_UNAVAILABLE, Failure, FrameError, FromHost, MAX_FROM_HOST, MAX_TO_HOST,
    Registered, ToHost,
};
use kanso_plugin_host::runtime::{Link, Plugin};
use serde_json::Value;

/// The pipes, and the request whose host calls are going out.
struct Pipes {
    input: BufReader<StdinLock<'static>>,
    output: StdoutLock<'static>,
    request: u64,
}

/// Stops the process with a one-line reason on standard error.
fn fatal(code: u8, reason: &str) -> ! {
    let mut err = std::io::stderr().lock();
    let _ = writeln!(err, "kanso-plugin-host: {reason}");
    std::process::exit(i32::from(code));
}

impl Pipes {
    fn read(&mut self) -> Result<Option<ToHost>, FrameError> {
        protocol::read_message(&mut self.input, MAX_TO_HOST)
    }

    fn write(&mut self, message: &FromHost) -> Result<(), FrameError> {
        protocol::write_message(&mut self.output, message, MAX_FROM_HOST)
    }

    /// Writes a message the app must get; if the pipe is gone, so is the app.
    fn send(&mut self, message: &FromHost) {
        match self.write(message) {
            Ok(()) => {}
            Err(FrameError::TooLong) => fatal(3, "a message was too long to send"),
            Err(_) => std::process::exit(0),
        }
    }

    /// Answers request `id` with `outcome`. An answer too long for the pipe
    /// is sent as a `limit` failure instead.
    fn finish(&mut self, id: u64, outcome: Result<Value, Failure>) {
        let message = match outcome {
            Ok(value) => FromHost::Result {
                id,
                ok: true,
                value,
                error: None,
            },
            Err(error) => FromHost::Result {
                id,
                ok: false,
                value: Value::Null,
                error: Some(error),
            },
        };
        match self.write(&message) {
            Ok(()) => {}
            Err(FrameError::TooLong) => self.send(&FromHost::Result {
                id,
                ok: false,
                value: Value::Null,
                error: Some(Failure::new(CODE_LIMIT, "its answer was too large")),
            }),
            Err(_) => std::process::exit(0),
        }
    }
}

impl Link for Pipes {
    fn call(&mut self, call: &str, args: Value) -> Result<Value, Failure> {
        let message = FromHost::Host {
            id: self.request,
            call: call.to_owned(),
            args,
        };
        match self.write(&message) {
            Ok(()) => {}
            Err(FrameError::TooLong) => {
                return Err(Failure::new(CODE_LIMIT, "the message is too long"));
            }
            Err(_) => std::process::exit(0),
        }
        match self.read() {
            Ok(Some(ToHost::Reply {
                id,
                ok,
                value,
                error,
            })) if id == self.request => {
                if ok {
                    Ok(value)
                } else {
                    Err(error.unwrap_or_else(|| {
                        Failure::new(CODE_UNAVAILABLE, "the app couldn't do that")
                    }))
                }
            }
            Ok(Some(_)) => fatal(3, "expected a reply to the host call"),
            Ok(None) => std::process::exit(0),
            Err(err) => fatal(3, &err.to_string()),
        }
    }

    fn log(&mut self, level: &str, text: &str) {
        let message = FromHost::Log {
            level: level.to_owned(),
            text: text.to_owned(),
        };
        // A log line too long for the pipe is dropped, not an error.
        match self.write(&message) {
            Ok(()) | Err(FrameError::TooLong) => {}
            Err(_) => std::process::exit(0),
        }
    }
}

fn main() -> ExitCode {
    let mut pipes = Pipes {
        input: BufReader::new(std::io::stdin().lock()),
        output: std::io::stdout().lock(),
        request: 0,
    };

    let (plugin, budgets, code) = match pipes.read() {
        Ok(Some(ToHost::Start {
            plugin,
            code,
            budgets,
        })) => (plugin, budgets, code),
        Ok(Some(_)) => fatal(2, "the first message must be start"),
        Ok(None) => return ExitCode::SUCCESS,
        Err(err) => fatal(2, &format!("the first message is bad: {err}")),
    };

    let mut running = match Plugin::start(&plugin, &code, budgets, &mut pipes) {
        Ok((running, registered)) => {
            pipes.send(&FromHost::Started {
                ok: true,
                registered,
                error: None,
            });
            Some(running)
        }
        Err(error) => {
            pipes.send(&FromHost::Started {
                ok: false,
                registered: Registered::default(),
                error: Some(error),
            });
            None
        }
    };
    drop(code);

    loop {
        let message = match pipes.read() {
            Ok(Some(message)) => message,
            Ok(None) => return ExitCode::SUCCESS,
            Err(err) => fatal(3, &err.to_string()),
        };
        let (id, outcome) = match message {
            ToHost::Stop => return ExitCode::SUCCESS,
            ToHost::Invoke { id, command, args } => {
                pipes.request = id;
                let outcome = match running.as_mut() {
                    Some(plugin) => plugin.invoke(&command, args, &mut pipes),
                    None => Err(not_running()),
                };
                (id, outcome)
            }
            ToHost::Render {
                id,
                language,
                source,
                theme,
            } => {
                pipes.request = id;
                let outcome = match running.as_mut() {
                    Some(plugin) => plugin
                        .render(&language, &source, &theme, &mut pipes)
                        .map(Value::String),
                    None => Err(not_running()),
                };
                (id, outcome)
            }
            ToHost::Press { id, button, values } => {
                pipes.request = id;
                let outcome = match running.as_mut() {
                    Some(plugin) => plugin.press(&button, values, &mut pipes),
                    None => Err(not_running()),
                };
                (id, outcome)
            }
            ToHost::Start { .. } | ToHost::Reply { .. } => {
                fatal(3, "a message came when none was expected")
            }
        };
        pipes.finish(id, outcome);
    }
}

fn not_running() -> Failure {
    Failure::new(CODE_UNAVAILABLE, "the plugin didn't start")
}
