//! The streaming client: agent-runtime's `Llm`, driven from one small tokio
//! runtime on a named background thread, exposed to the app as a blocking
//! iterator of [`Event`]s.
//!
//! Dropping the iterator aborts its tokio task, which drops the provider
//! request and its response body. Stop therefore cancels immediately, and a
//! server that never answers fails with a typed timeout instead of hanging.

use agent_runtime::{
    ChatMessage, EventSink, Llm, ProviderError, RuntimeEvent, SharedHttpClient, Timeouts,
    ToolRegistry, ToolSessionRequest,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock, mpsc};

use crate::Provider;
use crate::prompt::Prompt;

/// The most tokens an answer may take.
const MAX_TOKENS: u32 = 1024;
/// The most tool calls one question may make.
pub const MAX_TOOL_CALLS: usize = 6;
/// What the panel says when a stream stopped without its end marker.
pub const CUT_OFF: &str = "The answer was cut off before it finished.";

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

/// What went wrong before a stream opened, or while it ran.
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
    /// The server did not answer within its limits.
    Timeout(String),
}

impl std::fmt::Display for AiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiError::MissingKey => write!(f, "Add an API key in Settings › AI."),
            AiError::MissingModel => write!(f, "Set a model in Settings › AI."),
            AiError::MissingUrl => write!(f, "Set the server's address in Settings › AI."),
            AiError::Status { status, message } => {
                write!(f, "The model's server answered {status}: {message}")
            }
            AiError::Provider(message) => write!(f, "The model reported an error: {message}"),
            AiError::Transport(message) => write!(f, "Can't reach the model's server: {message}"),
            AiError::Timeout(message) => {
                write!(f, "The model's server didn't answer in time ({message}).")
            }
        }
    }
}

impl std::error::Error for AiError {}

/// One earlier exchange kept by the panel for its lifetime: the question and
/// the answer the person saw. It is never written to disk.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Turn {
    pub question: String,
    pub answer: String,
}

/// One provider, ready to answer. Everything above this trait (retrieval, the
/// panel, the tools) sees only this, so a later ACP-agent backend can answer
/// instead without those features changing.
pub trait ChatProvider: Send {
    /// Asks `prompt`, returning the answer as it arrives. Dropping the
    /// iterator stops the request.
    fn stream(&self, prompt: &Prompt) -> Result<Box<dyn Iterator<Item = Event> + Send>, AiError>;

    /// Asks `prompt` as the next turn of `history`, with `tools` available.
    /// `on_tool_completed` runs after each tool call, on the runtime thread,
    /// so the panel can refresh the notes it has shown as used. The default
    /// keeps single-turn providers working by ignoring history and tools.
    fn ask(
        &self,
        prompt: &Prompt,
        _history: &[Turn],
        _tools: agent_runtime::ToolRegistry<()>,
        _on_tool_completed: Box<dyn FnMut() + Send>,
        _cancel: Arc<AtomicBool>,
    ) -> Result<Box<dyn Iterator<Item = Event> + Send>, AiError> {
        self.stream(prompt)
    }
}

/// What the app's settings decide a request uses.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientSettings {
    pub provider: Provider,
    pub base_url: String,
    pub model: String,
    pub api_key: Option<String>,
    /// The AWS region, for Bedrock only.
    pub region: String,
}

impl ClientSettings {
    fn validate(&self) -> Result<(), AiError> {
        if self.model.trim().is_empty() {
            return Err(AiError::MissingModel);
        }
        if self.provider != Provider::Bedrock && self.base_url.trim().is_empty() {
            return Err(AiError::MissingUrl);
        }
        if self.provider.needs_key()
            && self
                .api_key
                .as_deref()
                .is_none_or(|key| key.trim().is_empty())
        {
            return Err(AiError::MissingKey);
        }
        Ok(())
    }

    fn llm_builder(&self) -> agent_runtime::LlmBuilder {
        let mut builder = Llm::builder()
            .provider(self.provider.agent_kind())
            .api_key(self.api_key.clone().unwrap_or_default())
            .max_tokens(MAX_TOKENS)
            .max_completion_tokens(MAX_TOKENS as u64)
            .timeouts(Timeouts::default());
        if self.provider == Provider::Bedrock {
            if !self.region.trim().is_empty() {
                builder = builder.region(self.region.trim());
            }
        } else if !self.base_url.trim().is_empty() {
            builder = builder.base_url(self.base_url.trim());
        }
        builder
    }

    fn llm(&self) -> Result<Llm, AiError> {
        self.validate()?;
        self.llm_builder().build().map_err(map_error)
    }
}

/// A client for `settings`, or why one can't be made.
pub fn client(settings: &ClientSettings) -> Result<Box<dyn ChatProvider>, AiError> {
    settings.validate()?;
    Ok(Box::new(AgentProviderClient {
        llm: settings.llm()?,
        model: settings.model.clone(),
    }))
}

/// A client over a specific `HttpClient`, for tests and advanced embedders.
pub fn client_with(
    settings: &ClientSettings,
    http: SharedHttpClient,
) -> Result<Box<dyn ChatProvider>, AiError> {
    client_with_timeouts(settings, http, Timeouts::default())
}

/// [`client_with`] with explicit timeouts.
pub fn client_with_timeouts(
    settings: &ClientSettings,
    http: SharedHttpClient,
    timeouts: Timeouts,
) -> Result<Box<dyn ChatProvider>, AiError> {
    settings.validate()?;
    let llm = settings
        .llm_builder()
        .http_client(http)
        .timeouts(timeouts)
        .build()
        .map_err(map_error)?;
    Ok(Box::new(AgentProviderClient {
        llm,
        model: settings.model.clone(),
    }))
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

/// Asks a question with tools: the initial excerpts are in `system`/`user`,
/// `history` holds earlier exchanges, and the model may call `tools` before
/// its final answer. Tool calls are capped at [`MAX_TOOL_CALLS`].
pub fn answer(
    settings: &ClientSettings,
    system: &str,
    user: &str,
    history: &[Turn],
    tools: ToolRegistry<()>,
    on_tool_completed: impl FnMut() + Send + 'static,
) -> Result<Box<dyn Iterator<Item = Event> + Send>, AiError> {
    let provider = client(settings)?;
    provider.ask(
        &Prompt {
            system: system.to_owned(),
            user: user.to_owned(),
        },
        history,
        tools,
        Box::new(on_tool_completed),
        Arc::new(AtomicBool::new(false)),
    )
}

/// The chat messages sent for a turn: earlier questions and answers, then the
/// new user message carrying this turn's excerpts and question. Earlier
/// excerpts are deliberately not resent; retrieval runs fresh for each turn.
fn chat_messages(history: &[Turn], user: &str) -> Vec<ChatMessage> {
    let mut messages = Vec::with_capacity(history.len() * 2 + 1);
    for turn in history {
        if !turn.question.trim().is_empty() {
            messages.push(ChatMessage::user(turn.question.clone()));
        }
        if !turn.answer.trim().is_empty() {
            messages.push(ChatMessage::assistant(turn.answer.clone()));
        }
    }
    messages.push(ChatMessage::user(user.to_owned()));
    messages
}

/// A streamed answer bound to its tokio task. Dropping it aborts that task,
/// which cancels the request.
struct EventStream {
    rx: mpsc::Receiver<Event>,
    task: Option<tokio::task::JoinHandle<()>>,
    cancel: Arc<AtomicBool>,
}

impl Iterator for EventStream {
    type Item = Event;

    fn next(&mut self) -> Option<Event> {
        loop {
            // Waiting with a short timeout lets Stop abort the request even
            // while a silent server sends nothing at all.
            match self.rx.recv_timeout(std::time::Duration::from_millis(25)) {
                Ok(event) => return Some(event),
                Err(mpsc::RecvTimeoutError::Disconnected) => return None,
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if self.cancel.load(Ordering::Relaxed) {
                        if let Some(task) = self.task.take() {
                            task.abort();
                        }
                        return None;
                    }
                }
            }
        }
    }
}

impl Drop for EventStream {
    fn drop(&mut self) {
        if let Some(task) = self.task.take() {
            task.abort();
        }
    }
}

/// One provider, ready to answer a prompt without tools.
struct AgentProviderClient {
    llm: Llm,
    model: String,
}

impl ChatProvider for AgentProviderClient {
    fn stream(&self, prompt: &Prompt) -> Result<Box<dyn Iterator<Item = Event> + Send>, AiError> {
        let llm = self.llm.clone();
        let system = prompt.system.clone();
        let user = prompt.user.clone();
        run_on_runtime(
            self.model.clone(),
            Arc::new(AtomicBool::new(false)),
            Box::new(|| {}),
            move |model, mut sink| async move {
                let messages = [ChatMessage::user(user)];
                llm.stream_message(&model, &system, &messages, &mut sink)
                    .await
                    .map(|_| ())
            },
        )
    }

    fn ask(
        &self,
        prompt: &Prompt,
        history: &[Turn],
        tools: ToolRegistry<()>,
        on_tool_completed: Box<dyn FnMut() + Send>,
        cancel: Arc<AtomicBool>,
    ) -> Result<Box<dyn Iterator<Item = Event> + Send>, AiError> {
        let llm = self.llm.clone();
        let system = prompt.system.clone();
        let messages = chat_messages(history, &prompt.user);
        run_on_runtime(
            self.model.clone(),
            cancel,
            on_tool_completed,
            move |model, mut sink| async move {
                llm.execute_tool_session(
                    ToolSessionRequest {
                        model: &model,
                        decision_system_prompt: &system,
                        followup_system_prompt: &system,
                        history: &messages,
                        tool_registry: &tools,
                        tool_context: (),
                        max_tool_calls: MAX_TOOL_CALLS,
                    },
                    &mut sink,
                )
                .await
                .map(|_| ())
            },
        )
    }
}

/// Runs `call` on the AI runtime and returns its events as a blocking
/// iterator.
fn run_on_runtime<F, Fut>(
    model: String,
    cancel: Arc<AtomicBool>,
    on_tool_completed: impl FnMut() + Send + 'static,
    call: F,
) -> Result<Box<dyn Iterator<Item = Event> + Send>, AiError>
where
    F: FnOnce(String, ChannelSink) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = anyhow::Result<()>> + Send,
{
    let runtime = runtime()?;
    let (tx, rx) = mpsc::channel();
    let finished = Arc::new(AtomicBool::new(false));
    let sink = ChannelSink {
        tx,
        cancel: cancel.clone(),
        on_tool: Box::new(on_tool_completed),
        finished: finished.clone(),
    };
    let task = runtime.spawn(async move {
        let tx = sink.tx.clone();
        let result = call(model, sink).await;
        if !finished.load(Ordering::Relaxed) {
            finished.store(true, Ordering::Relaxed);
            let event = match result {
                Ok(()) => Event::Done,
                Err(err) => Event::Error(error_text(&err)),
            };
            tx.send(event).ok();
        }
    });
    Ok(Box::new(EventStream {
        rx,
        task: Some(task),
        cancel,
    }))
}

/// Maps agent-runtime's events onto Kanso's, aborting when `cancel` is set.
struct ChannelSink {
    tx: mpsc::Sender<Event>,
    cancel: Arc<AtomicBool>,
    on_tool: Box<dyn FnMut() + Send>,
    finished: Arc<AtomicBool>,
}

#[async_trait::async_trait]
impl EventSink for ChannelSink {
    async fn emit(&mut self, event: RuntimeEvent) -> anyhow::Result<()> {
        if self.cancel.load(Ordering::Relaxed) {
            anyhow::bail!("cancelled");
        }
        match event {
            RuntimeEvent::AssistantDelta { delta } => {
                self.tx.send(Event::Text(delta)).ok();
            }
            RuntimeEvent::AssistantEnded { complete } => {
                self.finished.store(true, Ordering::Relaxed);
                let event = if complete {
                    Event::Done
                } else {
                    Event::Error(CUT_OFF.to_owned())
                };
                self.tx.send(event).ok();
            }
            RuntimeEvent::ToolCompleted { .. } => (self.on_tool)(),
            RuntimeEvent::AssistantStarted { .. } | RuntimeEvent::ToolStarted { .. } => {}
        }
        Ok(())
    }
}

/// The one runtime every provider call runs on: a single worker thread owned
/// by this crate, so GPUI's executor is never blocked.
fn runtime() -> Result<&'static tokio::runtime::Runtime, AiError> {
    static RUNTIME: OnceLock<Result<tokio::runtime::Runtime, String>> = OnceLock::new();
    RUNTIME
        .get_or_init(|| {
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(1)
                .thread_name("kanso-ai")
                .enable_all()
                .build()
                .map_err(|err| err.to_string())
        })
        .as_ref()
        .map_err(|err| AiError::Transport(format!("Couldn't start the AI runtime: {err}")))
}

/// A provider error's plain words, without credentials or request bodies.
fn error_text(error: &anyhow::Error) -> String {
    if let Some(provider) = error
        .chain()
        .find_map(|error| error.downcast_ref::<ProviderError>())
    {
        return provider.to_string();
    }
    error.to_string()
}

fn map_error(error: anyhow::Error) -> AiError {
    let Some(provider) = error
        .chain()
        .find_map(|error| error.downcast_ref::<ProviderError>())
    else {
        return AiError::Transport(error_text(&error));
    };
    match provider {
        ProviderError::Status { status, body, .. } => AiError::Status {
            status: *status,
            message: crate::prompt::truncate(body.trim(), 300),
        },
        ProviderError::Timeout { stage, after, .. } => {
            AiError::Timeout(format!("{} after {}s", stage.as_str(), after.as_secs_f64()))
        }
        ProviderError::Transport { source, .. } => AiError::Transport(source.to_string()),
        other => AiError::Provider(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::{Duration, Instant};

    use agent_runtime::{
        HttpClient, HttpRequest, HttpResponse, HttpStreamResponse, Timeouts, Tool, ToolCall,
        ToolDefinition, ToolOutput, ToolRegistry,
    };
    use async_trait::async_trait;
    use futures_util::stream;

    use super::{AiError, CUT_OFF, ClientSettings, Event, client_with_timeouts};
    use crate::Provider;
    use crate::prompt::Prompt;

    const OPENAI: &str = include_str!("../tests/fixtures/openai.sse");
    const ANTHROPIC: &str = include_str!("../tests/fixtures/anthropic.sse");
    const ANTHROPIC_ERROR: &str = include_str!("../tests/fixtures/anthropic_error.sse");

    /// Replays a recorded body in chunks, recording the request.
    #[derive(Clone, Default)]
    struct Replay {
        body: Arc<std::sync::Mutex<Option<String>>>,
        requests: Arc<std::sync::Mutex<Vec<HttpRequest>>>,
    }

    impl Replay {
        fn new(body: &str) -> Self {
            Self {
                body: Arc::new(std::sync::Mutex::new(Some(body.to_owned()))),
                requests: Arc::default(),
            }
        }
    }

    #[async_trait]
    impl HttpClient for Replay {
        async fn send(&self, request: HttpRequest) -> anyhow::Result<HttpResponse> {
            self.requests.lock().unwrap().push(request);
            Ok(HttpResponse {
                status: 200,
                body: b"{\"choices\":[{\"message\":{\"content\":\"ok\"}}]}".to_vec(),
            })
        }

        async fn send_streaming(&self, request: HttpRequest) -> anyhow::Result<HttpStreamResponse> {
            self.requests.lock().unwrap().push(request);
            let body = self.body.lock().unwrap().take().expect("one recorded body");
            // Split at every character so the runtime's decoder has to cope
            // with reads ending anywhere.
            let chunks: Vec<anyhow::Result<Vec<u8>>> = body
                .char_indices()
                .map(|(at, ch)| {
                    let end = at + ch.len_utf8();
                    Ok(body.as_bytes()[at..end].to_vec())
                })
                .collect();
            Ok(HttpStreamResponse {
                status: 200,
                body: Box::pin(stream::iter(chunks)),
            })
        }
    }

    /// Never answers.
    struct Silent;

    #[async_trait]
    impl HttpClient for Silent {
        async fn send(&self, _request: HttpRequest) -> anyhow::Result<HttpResponse> {
            std::future::pending().await
        }

        async fn send_streaming(
            &self,
            _request: HttpRequest,
        ) -> anyhow::Result<HttpStreamResponse> {
            std::future::pending().await
        }
    }

    fn settings(provider: Provider) -> ClientSettings {
        ClientSettings {
            provider,
            base_url: provider.default_base_url().to_owned(),
            model: provider.default_model().to_owned(),
            api_key: Some("test-key".to_owned()),
            region: provider.default_region().to_owned(),
        }
    }

    fn events(provider: Provider, body: &str) -> Vec<Event> {
        let client = client_with_timeouts(
            &settings(provider),
            Arc::new(Replay::new(body)),
            Timeouts::default(),
        )
        .unwrap();
        client
            .stream(&Prompt {
                system: "be brief".into(),
                user: "hello".into(),
            })
            .unwrap()
            .collect()
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

    #[test]
    fn a_recorded_openai_answer_streams_and_stops_at_done() {
        let events = events(Provider::OpenAi, OPENAI);
        assert_eq!(answer(&events), "Your notes say hello.");
        assert_eq!(events.last(), Some(&Event::Done));
    }

    #[test]
    fn a_recorded_anthropic_answer_streams_to_message_stop() {
        let events = events(Provider::Anthropic, ANTHROPIC);
        assert_eq!(answer(&events), "Your notes say hello.");
        assert_eq!(events.last(), Some(&Event::Done));
    }

    #[test]
    fn an_anthropic_error_is_plain_words() {
        let events = events(Provider::Anthropic, ANTHROPIC_ERROR);
        assert!(matches!(
            events.first(),
            Some(Event::Error(message)) if message.contains("overloaded")
        ));
    }

    #[test]
    fn a_stream_without_done_is_shown_as_cut_off() {
        let events = events(
            Provider::OpenAi,
            "data: {\"choices\":[{\"delta\":{\"content\":\"hi\"}}]}\n\n",
        );
        assert_eq!(answer(&events), "hi");
        assert_eq!(events.last(), Some(&Event::Error(CUT_OFF.to_owned())));
    }

    #[test]
    fn a_cloud_provider_needs_a_key_and_a_model() {
        let mut openai = settings(Provider::OpenAi);
        openai.api_key = None;
        assert!(matches!(
            client_with_timeouts(&openai, Arc::new(Silent), Timeouts::default()),
            Err(AiError::MissingKey)
        ));
        openai.api_key = Some("key".into());
        openai.model = String::new();
        assert!(matches!(
            client_with_timeouts(&openai, Arc::new(Silent), Timeouts::default()),
            Err(AiError::MissingModel)
        ));
        openai.model = "gpt-4o-mini".into();
        assert!(client_with_timeouts(&openai, Arc::new(Silent), Timeouts::default()).is_ok());
        let mut local = settings(Provider::LocalModel);
        local.api_key = None;
        assert!(client_with_timeouts(&local, Arc::new(Silent), Timeouts::default()).is_ok());
    }

    #[test]
    fn a_silent_server_times_out_instead_of_hanging() {
        let client = client_with_timeouts(
            &settings(Provider::OpenAi),
            Arc::new(Silent),
            Timeouts::default().response_headers(Some(Duration::from_millis(30))),
        )
        .expect("client builds");
        let events: Vec<Event> = client
            .stream(&Prompt {
                system: "be brief".into(),
                user: "hello".into(),
            })
            .expect("the request starts")
            .take(50)
            .collect();
        assert!(
            matches!(
                events.last(),
                Some(Event::Error(message)) if message.contains("timed out")
            ),
            "{events:?}"
        );
    }

    #[test]
    fn the_request_is_the_one_the_provider_speaks() {
        let replay = Arc::new(Replay::new(OPENAI));
        let client = client_with_timeouts(
            &settings(Provider::OpenAi),
            replay.clone(),
            Timeouts::default(),
        )
        .unwrap();
        let _ = client
            .stream(&Prompt {
                system: "sys".into(),
                user: "hello".into(),
            })
            .unwrap()
            .collect::<Vec<_>>();
        let requests = replay.requests.lock().unwrap();
        let request = requests.first().expect("a request");
        assert!(
            request.url.ends_with("/chat/completions"),
            "{}",
            request.url
        );
        let body = String::from_utf8_lossy(&request.body);
        assert!(body.contains("\"max_completion_tokens\":1024"), "{body}");
        assert!(body.contains("\"stream\":true"), "{body}");
    }
    #[test]
    fn a_follow_up_sends_the_earlier_exchange_and_the_new_question() {
        let replay = Arc::new(Replay::default());
        let client = client_with_timeouts(
            &settings(Provider::OpenAi),
            replay.clone(),
            Timeouts::default(),
        )
        .expect("client builds");
        let history = [super::Turn {
            question: "what did we decide?".to_owned(),
            answer: "the yearly plan".to_owned(),
        }];
        let events: Vec<Event> = client
            .ask(
                &Prompt {
                    system: "sys".into(),
                    user: "and when?".into(),
                },
                &history,
                ToolRegistry::new(),
                Box::new(|| {}),
                Arc::new(AtomicBool::new(false)),
            )
            .expect("the request starts")
            .take(20)
            .collect();
        assert!(matches!(events.last(), Some(Event::Done)), "{events:?}");
        let requests = replay.requests.lock().unwrap();
        let body = String::from_utf8_lossy(&requests[0].body);
        assert!(body.contains("what did we decide?"), "{body}");
        assert!(body.contains("the yearly plan"), "{body}");
        assert!(body.contains("and when?"), "{body}");
    }

    struct NoopTool {
        definition: ToolDefinition,
    }

    #[async_trait]
    impl Tool<()> for NoopTool {
        fn definition(&self) -> &ToolDefinition {
            &self.definition
        }

        async fn execute(&self, _ctx: (), _call: &ToolCall) -> anyhow::Result<ToolOutput> {
            Ok(ToolOutput {
                content: serde_json::json!({ "ok": true }),
            })
        }
    }

    #[test]
    fn the_tools_reach_the_request() {
        let replay = Arc::new(Replay::default());
        let client = client_with_timeouts(
            &settings(Provider::OpenAi),
            replay.clone(),
            Timeouts::default(),
        )
        .expect("client builds");
        let mut tools = ToolRegistry::new();
        tools.register(NoopTool {
            definition: ToolDefinition {
                name: "search_notes".to_owned(),
                description: "Search the notes.".to_owned(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": { "query": { "type": "string" } },
                    "required": ["query"]
                }),
            },
        });
        let _ = client
            .ask(
                &Prompt {
                    system: "sys".into(),
                    user: "hello".into(),
                },
                &[],
                tools,
                Box::new(|| {}),
                Arc::new(AtomicBool::new(false)),
            )
            .expect("the request starts")
            .take(20)
            .collect::<Vec<_>>();
        let requests = replay.requests.lock().unwrap();
        let body = String::from_utf8_lossy(&requests[0].body);
        assert!(body.contains("search_notes"), "{body}");
        assert!(body.contains("\"tools\""), "{body}");
    }

    #[test]
    fn stopping_cancels_a_silent_request() {
        let cancel = Arc::new(AtomicBool::new(false));
        let client = client_with_timeouts(
            &settings(Provider::OpenAi),
            Arc::new(Silent),
            Timeouts::none(),
        )
        .expect("client builds");
        let events = client
            .ask(
                &Prompt {
                    system: "be brief".into(),
                    user: "hello".into(),
                },
                &[],
                ToolRegistry::new(),
                Box::new(|| {}),
                cancel.clone(),
            )
            .expect("the request starts");
        let stopper = {
            let cancel = cancel.clone();
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_millis(50));
                cancel.store(true, Ordering::SeqCst);
            })
        };
        let started = Instant::now();
        let got: Vec<Event> = events.collect();
        stopper.join().unwrap();
        assert!(got.is_empty(), "{got:?}");
        assert!(
            started.elapsed() < Duration::from_secs(2),
            "Stop must abort a silent request promptly"
        );
    }
}
