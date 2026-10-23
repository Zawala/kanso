//! The streaming clients: OpenAI-compatible Chat Completions, which covers
//! OpenAI, OpenRouter, Ollama, LM Studio and any other server that speaks it,
//! and Anthropic's Messages.
//!
//! A request is a blocking iterator of [`Event`]s. Dropping it closes the
//! response, so a stopped answer stops reading.

use std::collections::VecDeque;
use std::fmt;
use std::io::{BufRead, BufReader, Read};
use std::time::Duration;

use serde_json::{Value, json};

use crate::Provider;
use crate::prompt::{Prompt, truncate};
use crate::sse::{SseDecoder, SseEvent};

/// How long a connection may take to open, and a read to return.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(20);
const READ_TIMEOUT: Duration = Duration::from_secs(300);
/// The most of a failed answer read for its message.
const ERROR_BODY: u64 = 64 * 1024;
/// The most tokens an answer may take.
const MAX_TOKENS: u32 = 1024;

/// Something a stream says.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    /// A piece of the answer, to add to what came before.
    Text(String),
    /// The answer is complete.
    Done,
    /// The answer stopped, with a plain reason.
    Error(String),
}

/// What went wrong before a stream opened.
#[derive(Debug)]
pub enum AiError {
    /// The provider needs a key and none is saved.
    MissingKey,
    /// No model name is set.
    MissingModel,
    /// The base URL is empty.
    MissingUrl,
    /// The server answered with an error.
    Status { status: u16, message: String },
    /// The provider answered with an error in its stream.
    Provider(String),
    /// The request couldn't reach the server.
    Transport(String),
}

impl fmt::Display for AiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AiError::MissingKey => write!(f, "Add an API key in Settings › AI."),
            AiError::MissingModel => write!(f, "Set a model in Settings › AI."),
            AiError::MissingUrl => write!(f, "Set the server's address in Settings › AI."),
            AiError::Status { status, message } => {
                write!(f, "The model's server answered {status}: {message}")
            }
            AiError::Provider(message) => write!(f, "The model reported an error: {message}"),
            AiError::Transport(message) => write!(f, "Can't reach the model's server: {message}"),
        }
    }
}

impl std::error::Error for AiError {}

/// One provider, ready to answer.
pub trait ChatProvider: Send {
    /// Asks `prompt`, returning the answer as it arrives. Dropping the
    /// iterator stops the request.
    fn stream(&self, prompt: &Prompt) -> Result<Box<dyn Iterator<Item = Event> + Send>, AiError>;
}

/// What the app's settings decide a request uses.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientSettings {
    pub provider: Provider,
    pub base_url: String,
    pub model: String,
    pub api_key: Option<String>,
}

/// A client for `settings`, or why one can't be made.
pub fn client(settings: &ClientSettings) -> Result<Box<dyn ChatProvider>, AiError> {
    if settings.base_url.trim().is_empty() {
        return Err(AiError::MissingUrl);
    }
    if settings.model.trim().is_empty() {
        return Err(AiError::MissingModel);
    }
    if settings.provider.needs_key()
        && settings
            .api_key
            .as_deref()
            .is_none_or(|key| key.trim().is_empty())
    {
        return Err(AiError::MissingKey);
    }
    Ok(if settings.provider == Provider::Anthropic {
        Box::new(Anthropic::new(settings.clone()))
    } else {
        Box::new(OpenAiCompatible::new(settings.clone()))
    })
}

/// Asks a tiny question and returns the first word of the answer, so Settings
/// can say the settings work.
pub fn probe(settings: &ClientSettings) -> Result<String, AiError> {
    let provider = client(settings)?;
    let prompt = Prompt {
        system: "You answer with one short word.".to_owned(),
        user: "Reply with OK.".to_owned(),
    };
    let mut stream = provider.stream(&prompt)?;
    let mut said = String::new();
    for event in stream.by_ref().take(200) {
        match event {
            Event::Text(text) => {
                said.push_str(&text);
                if said.trim().chars().count() >= 2 {
                    return Ok(said.trim().to_owned());
                }
            }
            Event::Done => break,
            Event::Error(message) => return Err(AiError::Provider(message)),
        }
    }
    let said = said.trim().to_owned();
    if said.is_empty() {
        Err(AiError::Transport(
            "The model answered without any text.".to_owned(),
        ))
    } else {
        Ok(said)
    }
}

/// An OpenAI-compatible Chat Completions server.
struct OpenAiCompatible {
    settings: ClientSettings,
}

impl OpenAiCompatible {
    fn new(settings: ClientSettings) -> OpenAiCompatible {
        OpenAiCompatible { settings }
    }
}

impl ChatProvider for OpenAiCompatible {
    fn stream(&self, prompt: &Prompt) -> Result<Box<dyn Iterator<Item = Event> + Send>, AiError> {
        let url = endpoint(&self.settings.base_url, "/chat/completions");
        let body = json!({
            "model": self.settings.model,
            "stream": true,
            "max_tokens": MAX_TOKENS,
            "messages": [
                {"role": "system", "content": prompt.system},
                {"role": "user", "content": prompt.user},
            ],
        });
        let agent = agent();
        let mut request = agent.post(&url).header("content-type", "application/json");
        if let Some(key) = self
            .settings
            .api_key
            .as_deref()
            .filter(|key| !key.trim().is_empty())
        {
            request = request.header("authorization", format!("Bearer {key}"));
        }
        let body = serde_json::to_vec(&body).map_err(|err| AiError::Transport(err.to_string()))?;
        let response = request
            .config()
            .timeout_recv_body(Some(READ_TIMEOUT))
            .build()
            .send(&body[..])
            .map_err(transport)?;
        open_stream(response, Kind::OpenAi)
    }
}

/// Anthropic's Messages.
struct Anthropic {
    settings: ClientSettings,
}

impl Anthropic {
    fn new(settings: ClientSettings) -> Anthropic {
        Anthropic { settings }
    }
}

impl ChatProvider for Anthropic {
    fn stream(&self, prompt: &Prompt) -> Result<Box<dyn Iterator<Item = Event> + Send>, AiError> {
        let url = endpoint(&self.settings.base_url, "/v1/messages");
        let body = json!({
            "model": self.settings.model,
            "max_tokens": MAX_TOKENS,
            "stream": true,
            "system": prompt.system,
            "messages": [{"role": "user", "content": prompt.user}],
        });
        let agent = agent();
        let mut request = agent
            .post(&url)
            .header("content-type", "application/json")
            .header("anthropic-version", "2023-06-01");
        if let Some(key) = self
            .settings
            .api_key
            .as_deref()
            .filter(|key| !key.trim().is_empty())
        {
            request = request.header("x-api-key", key);
        }
        let body = serde_json::to_vec(&body).map_err(|err| AiError::Transport(err.to_string()))?;
        let response = request
            .config()
            .timeout_recv_body(Some(READ_TIMEOUT))
            .build()
            .send(&body[..])
            .map_err(transport)?;
        open_stream(response, Kind::Anthropic)
    }
}

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .http_status_as_error(false)
        .max_redirects(3)
        .timeout_connect(Some(CONNECT_TIMEOUT))
        .user_agent(concat!("Kanso/", env!("CARGO_PKG_VERSION")))
        .build()
        .into()
}

/// The full address of `path` under `base`, unless `base` is already the
/// endpoint itself.
fn endpoint(base: &str, path: &str) -> String {
    let base = base.trim().trim_end_matches('/');
    if base.ends_with(path) {
        base.to_owned()
    } else {
        format!("{base}{path}")
    }
}

fn transport(err: ureq::Error) -> AiError {
    AiError::Transport(err.to_string())
}

/// Checks the answer's status and turns its body into a stream.
fn open_stream(
    mut response: ureq::http::Response<ureq::Body>,
    kind: Kind,
) -> Result<Box<dyn Iterator<Item = Event> + Send>, AiError> {
    let status = response.status().as_u16();
    if !(200..300).contains(&status) {
        let body = response
            .body_mut()
            .with_config()
            .limit(ERROR_BODY)
            .read_to_vec()
            .unwrap_or_default();
        return Err(AiError::Status {
            status,
            message: error_message(&body),
        });
    }
    let reader: Box<dyn Read + Send> = Box::new(response.into_body().into_reader());
    Ok(Box::new(ChatStream::new(reader, kind)))
}

fn error_message(body: &[u8]) -> String {
    if let Ok(value) = serde_json::from_slice::<Value>(body)
        && let Some(message) = value.pointer("/error/message").and_then(Value::as_str)
    {
        return truncate(message.trim(), 300);
    }
    let text = String::from_utf8_lossy(body);
    let text = text.trim();
    if text.is_empty() {
        "No message.".to_owned()
    } else {
        truncate(text, 300)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    OpenAi,
    Anthropic,
}

/// The answer as it arrives, one event at a time.
struct ChatStream {
    reader: BufReader<Box<dyn Read + Send>>,
    decoder: SseDecoder,
    kind: Kind,
    pending: VecDeque<Event>,
    finished: bool,
    saw_done: bool,
}

impl ChatStream {
    fn new(reader: Box<dyn Read + Send>, kind: Kind) -> ChatStream {
        ChatStream {
            reader: BufReader::new(reader),
            decoder: SseDecoder::new(),
            kind,
            pending: VecDeque::new(),
            finished: false,
            saw_done: false,
        }
    }

    fn decode(&mut self, events: Vec<SseEvent>) {
        for event in events {
            if let Some(event) = self.parse(&event.data) {
                self.pending.push_back(event);
            }
        }
    }

    /// One SSE event's payload as a stream event, if it is one.
    fn parse(&self, data: &str) -> Option<Event> {
        let data = data.trim();
        if data.is_empty() {
            return None;
        }
        if matches!(self.kind, Kind::OpenAi) && data == "[DONE]" {
            return Some(Event::Done);
        }
        let value: Value = serde_json::from_str(data).ok()?;
        match self.kind {
            Kind::OpenAi => {
                if let Some(message) = value.pointer("/error/message").and_then(Value::as_str) {
                    return Some(Event::Error(truncate(message.trim(), 300)));
                }
                value
                    .pointer("/choices/0/delta/content")
                    .and_then(Value::as_str)
                    .filter(|text| !text.is_empty())
                    .map(|text| Event::Text(text.to_owned()))
            }
            Kind::Anthropic => match value.get("type").and_then(Value::as_str) {
                Some("content_block_delta") => value
                    .pointer("/delta/text")
                    .and_then(Value::as_str)
                    .filter(|text| !text.is_empty())
                    .map(|text| Event::Text(text.to_owned())),
                Some("message_stop") => Some(Event::Done),
                Some("error") => Some(Event::Error(
                    value
                        .pointer("/error/message")
                        .and_then(Value::as_str)
                        .map_or_else(
                            || "The model reported an error.".to_owned(),
                            |message| truncate(message.trim(), 300),
                        ),
                )),
                _ => None,
            },
        }
    }
}

impl Iterator for ChatStream {
    type Item = Event;

    fn next(&mut self) -> Option<Event> {
        loop {
            if let Some(event) = self.pending.pop_front() {
                if matches!(event, Event::Done) {
                    self.saw_done = true;
                    self.finished = true;
                }
                return Some(event);
            }
            if self.finished {
                return None;
            }
            let mut bytes = Vec::new();
            match self.reader.read_until(b'\n', &mut bytes) {
                Ok(0) => {
                    let events = self.decoder.finish();
                    self.decode(events);
                    if !self.saw_done {
                        self.pending.push_back(Event::Done);
                    }
                    self.finished = true;
                }
                Ok(_) => {
                    let events = self.decoder.push(&bytes);
                    self.decode(events);
                }
                Err(err) => {
                    self.pending
                        .push_back(Event::Error(format!("The answer stopped: {err}")));
                    self.finished = true;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AiError, ChatStream, ClientSettings, Event, Kind, endpoint};
    use crate::{Provider, client};
    use proptest::prelude::*;
    use std::io::{Cursor, Read};

    const OPENAI: &str = include_str!("../tests/fixtures/openai.sse");
    const ANTHROPIC: &str = include_str!("../tests/fixtures/anthropic.sse");
    const ANTHROPIC_ERROR: &str = include_str!("../tests/fixtures/anthropic_error.sse");

    fn events(kind: Kind, text: &str) -> Vec<Event> {
        let reader: Box<dyn std::io::Read + Send> = Box::new(Cursor::new(text.as_bytes().to_vec()));
        ChatStream::new(reader, kind).collect()
    }

    fn answer(events: &[Event]) -> String {
        events
            .iter()
            .filter_map(|event| match event {
                Event::Text(text) => Some(text.as_str()),
                _ => None,
            })
            .collect()
    }

    /// A reader that returns `text` in the pieces `cuts` make.
    struct Chunks {
        pieces: Vec<Vec<u8>>,
        at: usize,
        offset: usize,
    }

    impl Read for Chunks {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            loop {
                let Some(piece) = self.pieces.get(self.at) else {
                    return Ok(0);
                };
                if piece.is_empty() {
                    self.at += 1;
                    self.offset = 0;
                    continue;
                }
                let left = &piece[self.offset..];
                let n = left.len().min(buf.len());
                buf[..n].copy_from_slice(&left[..n]);
                self.offset += n;
                if self.offset == piece.len() {
                    self.at += 1;
                    self.offset = 0;
                }
                return Ok(n);
            }
        }
    }

    /// `text` cut at `cuts`, as a stream of events.
    fn events_in_chunks(kind: Kind, text: &str, cuts: &[usize]) -> Vec<Event> {
        let mut cuts = cuts.to_vec();
        cuts.sort_unstable();
        let mut pieces = Vec::new();
        let mut start = 0;
        for cut in cuts {
            let cut = cut.clamp(start, text.len());
            if text.is_char_boundary(cut) {
                pieces.push(text.as_bytes()[start..cut].to_vec());
                start = cut;
            }
        }
        pieces.push(text.as_bytes()[start..].to_vec());
        let reader: Box<dyn Read + Send> = Box::new(Chunks {
            pieces,
            at: 0,
            offset: 0,
        });
        ChatStream::new(reader, kind).collect()
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        /// A recorded OpenAI answer read in any pieces is the same answer,
        /// and still ends at its `[DONE]`.
        #[test]
        fn prop_an_openai_answer_read_in_pieces_is_the_same(cuts in prop::collection::vec(0usize..=OPENAI.len(), 0..20)) {
            prop_assert_eq!(events_in_chunks(Kind::OpenAi, OPENAI, &cuts), events(Kind::OpenAi, OPENAI));
        }

        /// The same for Anthropic's recording, which ends at `message_stop`.
        #[test]
        fn prop_an_anthropic_answer_read_in_pieces_is_the_same(cuts in prop::collection::vec(0usize..=ANTHROPIC.len(), 0..20)) {
            prop_assert_eq!(events_in_chunks(Kind::Anthropic, ANTHROPIC, &cuts), events(Kind::Anthropic, ANTHROPIC));
        }
    }

    #[test]
    fn openai_streams_and_stops_at_done() {
        let events = events(Kind::OpenAi, OPENAI);
        assert_eq!(answer(&events), "Your notes say hello.");
        assert_eq!(events.last(), Some(&Event::Done));
        assert_eq!(events.iter().filter(|e| **e == Event::Done).count(), 1);
    }

    #[test]
    fn anthropic_streams_to_message_stop() {
        let events = events(Kind::Anthropic, ANTHROPIC);
        assert_eq!(answer(&events), "Your notes say hello.");
        assert_eq!(events.last(), Some(&Event::Done));
    }

    #[test]
    fn anthropic_errors_are_plain_words() {
        let events = events(Kind::Anthropic, ANTHROPIC_ERROR);
        assert!(matches!(
            events.first(),
            Some(Event::Error(message)) if message.contains("overloaded")
        ));
    }

    #[test]
    fn a_stream_without_done_still_ends() {
        let events = events(
            Kind::OpenAi,
            "data: {\"choices\":[{\"delta\":{\"content\":\"hi\"}}]}\n\n",
        );
        assert_eq!(events, [Event::Text("hi".into()), Event::Done]);
    }

    #[test]
    fn the_endpoint_is_appended_once() {
        assert_eq!(
            endpoint("https://api.openai.com/v1/", "/chat/completions"),
            "https://api.openai.com/v1/chat/completions"
        );
        assert_eq!(
            endpoint(
                "http://localhost:1234/v1/chat/completions",
                "/chat/completions"
            ),
            "http://localhost:1234/v1/chat/completions"
        );
        assert_eq!(
            endpoint("https://api.anthropic.com/", "/v1/messages"),
            "https://api.anthropic.com/v1/messages"
        );
    }

    #[test]
    fn a_cloud_provider_needs_a_key_and_a_model() {
        let settings = ClientSettings {
            provider: Provider::OpenAi,
            base_url: Provider::OpenAi.default_base_url().to_owned(),
            model: Provider::OpenAi.default_model().to_owned(),
            api_key: None,
        };
        assert!(matches!(client(&settings), Err(AiError::MissingKey)));
        let settings = ClientSettings {
            api_key: Some("key".into()),
            model: String::new(),
            ..settings
        };
        assert!(matches!(client(&settings), Err(AiError::MissingModel)));
        let settings = ClientSettings {
            model: "gpt-4o-mini".into(),
            ..settings
        };
        assert!(client(&settings).is_ok());
        let local = ClientSettings {
            provider: Provider::LocalModel,
            base_url: Provider::LocalModel.default_base_url().to_owned(),
            model: "llama3.2".into(),
            api_key: None,
        };
        assert!(client(&local).is_ok());
    }
}
