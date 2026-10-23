//! Mermaid diagrams drawn by Pele (Obsidian's renderer, MIT), run in
//! QuickJS on a thread of its own: the alternative to `mermaid-svg` chosen
//! in Settings; see [`crate::mermaid::render_with`].
//!
//! Pele themes a drawing with CSS variables, which resvg doesn't read, so
//! each `var(…)` is replaced with the palette's value before it's flattened.

use std::collections::HashMap;
use std::io::Read as _;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use crate::{Failure, Palette, Rgb};

static BUNDLE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/pele.js.deflate"));

/// The longest a diagram may take.
const TIME_LIMIT: Duration = Duration::from_secs(3);
/// The most memory the engine may hold.
const MEMORY_LIMIT: usize = 128 << 20;
/// The fonts labels are drawn in: the same as `mermaid-svg`'s.
const FONT: &str =
    "'Helvetica Neue', 'Segoe UI', Inter, 'Noto Sans', 'DejaVu Sans', Arial, sans-serif";

/// What Pele answered: its SVG, or why it didn't draw.
type Answer = Result<String, Failure>;

struct Request {
    source: String,
    reply: mpsc::Sender<Answer>,
}

/// The engine's thread, started on first use.
fn engine() -> &'static Mutex<mpsc::Sender<Request>> {
    static ENGINE: OnceLock<Mutex<mpsc::Sender<Request>>> = OnceLock::new();
    ENGINE.get_or_init(|| {
        let (send, receive) = mpsc::channel::<Request>();
        std::thread::Builder::new()
            .name("kanso-pele".into())
            .spawn(move || run(receive))
            .expect("starting the Pele thread");
        Mutex::new(send)
    })
}

fn run(requests: mpsc::Receiver<Request>) {
    let deadline = Arc::new(AtomicU64::new(u64::MAX));
    let started = Instant::now();
    let engine = start(deadline.clone(), started);
    for request in requests {
        let answer = match &engine {
            Ok((_, context)) => {
                let limit = started.elapsed() + TIME_LIMIT;
                deadline.store(limit.as_millis() as u64, Ordering::Relaxed);
                let answer = draw(context, &request.source);
                deadline.store(u64::MAX, Ordering::Relaxed);
                answer
            }
            Err(message) => Err(Failure::Invalid(message.clone())),
        };
        request.reply.send(answer).ok();
    }
}

type Engine = (rquickjs::Runtime, rquickjs::Context);

fn start(deadline: Arc<AtomicU64>, started: Instant) -> Result<Engine, String> {
    let unavailable = |err: &dyn std::fmt::Display| format!("Diagrams are unavailable: {err}");
    let mut js = String::new();
    flate2::read::DeflateDecoder::new(BUNDLE)
        .read_to_string(&mut js)
        .map_err(|e| unavailable(&e))?;
    let runtime = rquickjs::Runtime::new().map_err(|e| unavailable(&e))?;
    runtime.set_memory_limit(MEMORY_LIMIT);
    runtime.set_max_stack_size(2 << 20);
    runtime.set_interrupt_handler(Some(Box::new(move || {
        started.elapsed().as_millis() as u64 > deadline.load(Ordering::Relaxed)
    })));
    let context = rquickjs::Context::full(&runtime).map_err(|e| unavailable(&e))?;
    context.with(|ctx| {
        ctx.eval::<(), _>(js).map_err(|e| {
            let _ = e;
            unavailable(&"Pele didn't start")
        })
    })?;
    Ok((runtime, context))
}

fn draw(context: &rquickjs::Context, source: &str) -> Answer {
    let failed = |_| Failure::Invalid("The diagram took too long or used too much memory.".into());
    let json: String = context.with(|ctx| {
        let draw: rquickjs::Function = ctx.globals().get("kansoPele").map_err(failed)?;
        draw.call((source, FONT)).map_err(failed)
    })?;
    let answer: serde_json::Value =
        serde_json::from_str(&json).map_err(|e| Failure::Invalid(e.to_string()))?;
    if let Some(svg) = answer["svg"].as_str() {
        return Ok(svg.to_owned());
    }
    let message = answer["error"].as_str().unwrap_or("Pele drew nothing.");
    Err(match answer["code"].as_str() {
        Some("unsupported-diagram") => {
            Failure::Unsupported(answer["type"].as_str().unwrap_or("").to_owned())
        }
        _ => Failure::Invalid(message.to_owned()),
    })
}

/// Pele's raw SVG for `source`, before it's themed. Blocks on its thread.
pub(crate) fn render(source: &str) -> Answer {
    let unavailable = || Failure::Invalid("Diagrams are unavailable.".into());
    let (reply, answer) = mpsc::channel();
    engine()
        .lock()
        .map_err(|_| unavailable())?
        .send(Request {
            source: source.to_owned(),
            reply,
        })
        .map_err(|_| unavailable())?;
    answer.recv().map_err(|_| unavailable())?
}

/// Pele's theme tokens in the palette's colors, chosen to match how
/// `mermaid-svg` is themed: nodes tinted with the accent.
fn tokens(p: &Palette) -> HashMap<&'static str, String> {
    let node_fill = p
        .accent
        .over(p.background, if p.dark { 0.16 } else { 0.10 });
    let node_stroke = p.accent.over(p.background, 0.6);
    let line = p.muted.over(p.text, 0.5);
    // Charts: the accent first, then muted earthy tones that sit with it.
    let series = [
        p.accent,
        Rgb(0x6A8CAF),
        Rgb(0x7FA37A),
        Rgb(0xC0765A),
        Rgb(0x9A7FB0),
        Rgb(0xC9A227),
        Rgb(0x5E9C9A),
        Rgb(0xA0806A),
    ];
    let mut t = HashMap::from([
        ("--pele-bg", p.background.hex()),
        ("--pele-fg", p.text.hex()),
        ("--pele-muted", p.muted.hex()),
        ("--pele-line", line.hex()),
        ("--pele-surface", node_fill.hex()),
        ("--pele-surface-alt", p.surface.hex()),
        ("--pele-border", node_stroke.hex()),
        ("--pele-accent", p.accent.hex()),
        ("--pele-critical", Rgb(0xC0504D).hex()),
        ("--pele-error", Rgb(0xC0504D).hex()),
        ("--pele-radius", "4px".into()),
        ("--pele-font", FONT.into()),
        (
            "--pele-font-mono",
            "Menlo, 'DejaVu Sans Mono', monospace".into(),
        ),
    ]);
    const SERIES: [&str; 8] = [
        "--pele-series-1",
        "--pele-series-2",
        "--pele-series-3",
        "--pele-series-4",
        "--pele-series-5",
        "--pele-series-6",
        "--pele-series-7",
        "--pele-series-8",
    ];
    for (name, color) in SERIES.iter().zip(series) {
        t.insert(name, color.hex());
    }
    t
}

/// `svg` as resvg can draw it, in the palette's colors: every `var(…)`
/// replaced with its value, and corner radii moved from `style` to `rx`
/// attributes. Pure.
pub(crate) fn theme(svg: &str, palette: &Palette) -> String {
    let tokens = tokens(palette);
    // Pele's private aliases (`--_fg:var(--pele-fg,#1f1f1f)`), set in
    // `style` attributes, the root's first.
    let mut aliases: HashMap<String, String> = HashMap::new();
    for style in attribute_values(svg, "style") {
        for declaration in style.split(';') {
            if let Some((name, value)) = declaration.split_once(':') {
                let name = name.trim();
                if name.starts_with("--") && !aliases.contains_key(name) {
                    aliases.insert(name.to_owned(), value.trim().to_owned());
                }
            }
        }
    }
    let lookup = |name: &str| -> Option<String> {
        tokens
            .get(name)
            .cloned()
            .or_else(|| aliases.get(name).cloned())
    };
    let resolved = substitute(svg, &lookup, 0);
    lift_radii(&resolved)
}

/// Every value of attribute `name` in `svg`, in order.
fn attribute_values<'a>(svg: &'a str, name: &str) -> Vec<&'a str> {
    let needle = format!(" {name}=\"");
    let mut out = Vec::new();
    let mut rest = svg;
    while let Some(at) = rest.find(&needle) {
        let after = &rest[at + needle.len()..];
        let Some(end) = after.find('"') else { break };
        out.push(&after[..end]);
        rest = &after[end..];
    }
    out
}

/// The deepest `var()` chain followed; deeper is left as it is.
const MAX_DEPTH: usize = 8;

/// `text` with each `var(--name[, fallback])` replaced: by `lookup(name)`
/// (itself substituted), else its fallback, else left out. Pure.
fn substitute(text: &str, lookup: &dyn Fn(&str) -> Option<String>, depth: usize) -> String {
    if depth > MAX_DEPTH {
        return text.to_owned();
    }
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find("var(") {
        out.push_str(&rest[..at]);
        let inner_start = at + 4;
        let Some(len) = closing_paren(&rest[inner_start..]) else {
            out.push_str(&rest[at..]);
            return out;
        };
        let inner = &rest[inner_start..inner_start + len];
        let (name, fallback) = match top_level_comma(inner) {
            Some(c) => (inner[..c].trim(), Some(inner[c + 1..].trim())),
            None => (inner.trim(), None),
        };
        let value = lookup(name).or_else(|| fallback.map(str::to_owned));
        if let Some(value) = value {
            out.push_str(&substitute(&value, lookup, depth + 1));
        }
        rest = &rest[inner_start + len + 1..];
    }
    out.push_str(rest);
    out
}

/// Where the `)` closing an already opened paren is in `text`.
fn closing_paren(text: &str) -> Option<usize> {
    let mut depth = 0usize;
    for (i, c) in text.char_indices() {
        match c {
            '(' => depth += 1,
            ')' if depth == 0 => return Some(i),
            ')' => depth -= 1,
            _ => {}
        }
    }
    None
}

/// The first comma in `text` outside any parens.
fn top_level_comma(text: &str) -> Option<usize> {
    let mut depth = 0usize;
    for (i, c) in text.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => return Some(i),
            _ => {}
        }
    }
    None
}

/// `svg` with `style` attributes as resvg reads them: `rx` and `ry` written
/// as attributes (SVG 2 allows them in CSS, resvg doesn't), the `font`
/// shorthand as just its size, and custom properties left out. Pure.
fn lift_radii(svg: &str) -> String {
    const OPEN: &str = " style=\"";
    let mut out = String::with_capacity(svg.len());
    let mut rest = svg;
    while let Some(at) = rest.find(OPEN) {
        out.push_str(&rest[..at]);
        let after = &rest[at + OPEN.len()..];
        let Some(end) = after.find('"') else {
            out.push_str(&rest[at..]);
            return out;
        };
        let mut kept = Vec::new();
        let mut kept_owned: Vec<String> = Vec::new();
        let mut lifted = String::new();
        for declaration in after[..end].split(';') {
            match declaration.split_once(':') {
                Some((name, value)) if matches!(name.trim(), "rx" | "ry") => {
                    let value = value.trim().trim_end_matches("px");
                    lifted.push_str(&format!(" {}=\"{value}\"", name.trim()));
                }
                // `font:14px sans-serif` beside `font-family`: usvg takes
                // the shorthand's family as its default, Times.
                Some((name, value)) if name.trim() == "font" => {
                    let size = value.split_whitespace().next().unwrap_or("");
                    kept_owned.push(format!("font-size:{size}"));
                }
                // Custom properties, already substituted: usvg's CSS parser
                // stops at them.
                Some((name, _)) if name.trim().starts_with("--") => {}
                _ if declaration.trim().is_empty() => {}
                _ => kept.push(declaration),
            }
        }
        let mut kept: Vec<&str> = kept;
        kept.extend(kept_owned.iter().map(String::as_str));
        out.push_str(&lifted);
        if !kept.is_empty() {
            out.push_str(OPEN);
            out.push_str(&kept.join(";"));
            out.push('"');
        }
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn variables_take_the_palette_or_their_fallback() {
        let lookup = |name: &str| match name {
            "--_fg" => Some("var(--pele-fg,#1f1f1f)".to_owned()),
            "--pele-fg" => Some("#2b2a27".to_owned()),
            _ => None,
        };
        assert_eq!(
            substitute("fill=\"var(--_fg)\"", &lookup, 0),
            "fill=\"#2b2a27\""
        );
        assert_eq!(
            substitute("var(--nope, rgb(1,2,3))", &lookup, 0),
            "rgb(1,2,3)"
        );
        assert_eq!(substitute("a var(--nope) b", &lookup, 0), "a  b");
        assert_eq!(substitute("var(--_fg", &lookup, 0), "var(--_fg");
    }

    #[test]
    fn radii_move_out_of_style() {
        assert_eq!(lift_radii(r#"<rect style="rx:4px"/>"#), r#"<rect rx="4"/>"#);
        assert_eq!(
            lift_radii(r#"<rect style="rx:2;fill:red"/>"#),
            r#"<rect rx="2" style="fill:red"/>"#
        );
        assert_eq!(
            lift_radii(r#"<g style="color:red">"#),
            r#"<g style="color:red">"#
        );
    }

    #[test]
    fn a_flowchart_draws_themed() {
        let raw = render("flowchart TD\n  A[Start] --> B{Working?}\n").unwrap();
        let svg = theme(&raw, &Palette::LIGHT);
        assert!(!svg.contains("var("), "{svg}");
        assert!(svg.contains(&Palette::LIGHT.text.hex()));
        assert!(crate::svg::flatten(&svg, 8.).is_some());
    }

    #[test]
    fn unknown_kinds_and_mistakes_are_told_apart() {
        assert!(matches!(
            render("nonsense\n  a"),
            Err(Failure::Unsupported(_))
        ));
        assert!(matches!(
            render("flowchart TD\n  A -->"),
            Err(Failure::Invalid(_))
        ));
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Invariant: text with no `var(` is left exactly as it was, and
        /// once every name resolves, no `var(` is left.
        #[test]
        fn prop_substitution_is_total(plain in "[a-z#0-9 =\"():;,-]{0,40}", names in prop::collection::vec("[a-z]{1,6}", 0..4)) {
            let none = |_: &str| None;
            if !plain.contains("var(") {
                prop_assert_eq!(substitute(&plain, &none, 0), plain.clone());
            }
            let text: String = names.iter().map(|n| format!("x var(--{n}, #fff) ")).collect();
            let some = |_: &str| Some("#000".to_owned());
            prop_assert!(!substitute(&text, &some, 0).contains("var("));
        }
    }
}
