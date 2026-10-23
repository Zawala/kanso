//! The JavaScript Kanso draws with: MathJax 3 (TeX as SVG) and Pele
//! (Mermaid as SVG), loaded into one QuickJS engine on a thread of its own.
//! Sharing it lets Pele typeset `$$…$$` in diagram labels with MathJax.
//!
//! MathJax starts on the first formula or diagram, Pele on the first
//! diagram, and then each answers in a few milliseconds. Each call has a time and memory limit, so
//! a runaway macro or a huge diagram can't hang or swell the app.

use std::io::Read as _;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

static MATHJAX: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/mathjax-tex-svg.js.deflate"));
static PELE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/pele.js.deflate"));

/// The longest a call may take.
pub(crate) const TIME_LIMIT: Duration = Duration::from_secs(3);
/// The most memory the engine may hold.
const MEMORY_LIMIT: usize = 256 << 20;

/// A function the bundles define, with what it's called with. Each answers
/// with JSON.
pub(crate) enum Call {
    /// `kansoTex2Svg(source, display)`.
    Tex { source: String, display: bool },
    /// `kansoPele(source, font)`.
    Pele { source: String, font: &'static str },
}

struct Request {
    call: Call,
    reply: mpsc::Sender<Result<String, String>>,
}

/// The engine's thread, started on first use.
fn engine() -> &'static Mutex<mpsc::Sender<Request>> {
    static ENGINE: OnceLock<Mutex<mpsc::Sender<Request>>> = OnceLock::new();
    ENGINE.get_or_init(|| {
        let (send, receive) = mpsc::channel::<Request>();
        std::thread::Builder::new()
            .name("kanso-js".into())
            .spawn(move || run(receive))
            .expect("starting the JavaScript thread");
        Mutex::new(send)
    })
}

/// The engine's loop: starts the bundles, then answers calls as they come.
fn run(requests: mpsc::Receiver<Request>) {
    let deadline = Arc::new(AtomicU64::new(u64::MAX));
    let started = Instant::now();
    let engine = start(deadline.clone(), started);
    let mut pele: Option<Result<(), String>> = None;
    for request in requests {
        let answer = match &engine {
            Ok((_, context)) => {
                // Pele is loaded for the first diagram, outside its limit.
                let loaded = match (&request.call, &mut pele) {
                    (Call::Tex { .. }, _) => Ok(()),
                    (Call::Pele { .. }, Some(loaded)) => loaded.clone(),
                    (Call::Pele { .. }, pele) => pele.insert(load(context, PELE)).clone(),
                };
                let limit = started.elapsed() + TIME_LIMIT;
                deadline.store(limit.as_millis() as u64, Ordering::Relaxed);
                let answer = loaded.and_then(|()| answer(context, &request.call));
                deadline.store(u64::MAX, Ordering::Relaxed);
                answer
            }
            Err(message) => Err(message.clone()),
        };
        request.reply.send(answer).ok();
    }
}

type Engine = (rquickjs::Runtime, rquickjs::Context);

fn start(deadline: Arc<AtomicU64>, started: Instant) -> Result<Engine, String> {
    let unavailable = |err: &dyn std::fmt::Display| format!("Drawing is unavailable: {err}");
    let runtime = rquickjs::Runtime::new().map_err(|e| unavailable(&e))?;
    runtime.set_memory_limit(MEMORY_LIMIT);
    runtime.set_max_stack_size(2 << 20);
    runtime.set_interrupt_handler(Some(Box::new(move || {
        started.elapsed().as_millis() as u64 > deadline.load(Ordering::Relaxed)
    })));
    let context = rquickjs::Context::full(&runtime).map_err(|e| unavailable(&e))?;
    // MathJax first: Pele's entry hands it `$$…$$` from labels.
    load(&context, MATHJAX)?;
    Ok((runtime, context))
}

/// Inflates `bundle` and runs it in `context`.
fn load(context: &rquickjs::Context, bundle: &[u8]) -> Result<(), String> {
    let unavailable = |err: &dyn std::fmt::Display| format!("Drawing is unavailable: {err}");
    let mut js = String::new();
    flate2::read::DeflateDecoder::new(bundle)
        .read_to_string(&mut js)
        .map_err(|e| unavailable(&e))?;
    context.with(|ctx| {
        ctx.eval::<(), _>(js)
            .map_err(|e| unavailable(&describe(&ctx, e)))
    })
}

/// What went wrong in QuickJS, with the thrown value's message if any.
fn describe(ctx: &rquickjs::Ctx<'_>, error: rquickjs::Error) -> String {
    if matches!(error, rquickjs::Error::Exception) {
        let thrown = ctx.catch();
        if let Some(message) = thrown.as_exception().and_then(|e| e.message()) {
            return message;
        }
        return "it took too long or used too much memory".into();
    }
    error.to_string()
}

fn answer(context: &rquickjs::Context, call: &Call) -> Result<String, String> {
    context.with(|ctx| {
        let get = |name: &str| -> Result<rquickjs::Function, String> {
            ctx.globals().get(name).map_err(|e| describe(&ctx, e))
        };
        match call {
            Call::Tex { source, display } => get("kansoTex2Svg")?
                .call((source.as_str(), *display))
                .map_err(|e| describe(&ctx, e)),
            Call::Pele { source, font } => get("kansoPele")?
                .call((source.as_str(), *font))
                .map_err(|e| describe(&ctx, e)),
        }
    })
}

/// The JSON `call` answers with, or why there's none: the engine didn't
/// start, or the call threw, ran out of time or out of memory. Blocks on
/// the engine's thread.
pub(crate) fn call(call: Call) -> Result<String, String> {
    let unavailable = || "Drawing is unavailable.".to_owned();
    let (reply, answer) = mpsc::channel();
    engine()
        .lock()
        .map_err(|_| unavailable())?
        .send(Request { call, reply })
        .map_err(|_| unavailable())?;
    answer.recv().map_err(|_| unavailable())?
}
