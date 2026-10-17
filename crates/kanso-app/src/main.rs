//! M0 spike: gpui-kit's editor and markdown preview on a real note.
//!
//! Usage: `kanso <note.md> [--files <dir>] [--no-preview] [--bench]`
//!
//! `--files` resolves `inkdrop://file:<id>` images to `<dir>/<id>`.
//! `--bench` types into the start, middle and end of the note, prints
//! edit-to-frame latencies, then quits.

use std::{
    cell::RefCell,
    path::PathBuf,
    rc::Rc,
    time::{Duration, Instant},
};

use anyhow::{Context as _, Result};
use gpui_kit::base::text::TextView;
use gpui_kit::component::input::{Editor, EditorState, InputEvent, Position};
use gpui_kit::component::{ActiveTheme, StyledExt as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

const PREVIEW_DEBOUNCE: Duration = Duration::from_millis(150);
const BENCH_EDITS_PER_SPOT: usize = 120;

struct Args {
    note: PathBuf,
    files: Option<PathBuf>,
    preview: bool,
    bench: bool,
}

impl Args {
    fn parse() -> Result<Self> {
        let mut note = None;
        let mut files = None;
        let mut preview = true;
        let mut bench = false;
        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--files" => files = Some(args.next().context("--files needs a directory")?.into()),
                "--no-preview" => preview = false,
                "--bench" => bench = true,
                _ => note = Some(PathBuf::from(arg)),
            }
        }
        Ok(Self {
            note: note
                .context("usage: kanso <note.md> [--files <dir>] [--no-preview] [--bench]")?,
            files,
            preview,
            bench,
        })
    }
}

struct Spike {
    editor: Entity<EditorState>,
    preview: Entity<Preview>,
    show_preview: bool,
    preview_task: Option<Task<()>>,
    _subscription: Subscription,
}

/// The rendered note, in its own view so it can be cached: re-laying out a
/// large note on every keystroke frame would double the edit latency.
struct Preview {
    text: SharedString,
    files_dir: Option<PathBuf>,
}

impl Preview {
    fn image_resolver(&self) -> impl Fn(&SharedUri) -> ImageSource + Send + Sync + 'static {
        let files_dir = self.files_dir.clone();
        move |uri| match (uri.strip_prefix("inkdrop://file:"), &files_dir) {
            (Some(id), Some(dir)) => ImageSource::from(dir.join(id)),
            _ => ImageSource::from(uri.clone()),
        }
    }
}

impl Render for Preview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("preview")
            .size_full()
            .p_4()
            .border_l_1()
            .border_color(cx.theme().border)
            .overflow_y_scroll()
            .child(
                TextView::markdown("preview-text", self.text.clone())
                    .image_source(self.image_resolver()),
            )
    }
}

impl Spike {
    fn new(args: &Args, body: String, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let editor = cx.new(|cx| {
            EditorState::new(window, cx)
                .language("markdown")
                .line_number(false)
                .folding(false)
                .indent_guides(false)
                .soft_wrap(true)
        });

        let loaded = Instant::now();
        let chars = body.len();
        editor.update(cx, |state, cx| state.set_value(body.clone(), window, cx));
        window.on_next_frame(move |_, _| {
            eprintln!("load: {chars} bytes rendered in {:?}", loaded.elapsed());
        });

        let subscription = cx.subscribe_in(&editor, window, |this, _, event, window, cx| {
            if matches!(event, InputEvent::Change) {
                this.schedule_preview(window, cx);
            }
        });

        let preview = cx.new(|_| Preview {
            text: body.into(),
            files_dir: args.files.clone(),
        });

        Self {
            editor,
            preview,
            show_preview: args.preview,
            preview_task: None,
            _subscription: subscription,
        }
    }

    fn schedule_preview(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.show_preview {
            return;
        }
        self.preview_task = Some(cx.spawn_in(window, async move |this, cx| {
            cx.background_executor().timer(PREVIEW_DEBOUNCE).await;
            this.update(cx, |this, cx| {
                let text = this.editor.read(cx).value();
                this.preview.update(cx, |preview, cx| {
                    preview.text = text;
                    cx.notify();
                });
            })
            .ok();
        }));
    }
}

impl Render for Spike {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .h_flex()
            .size_full()
            .bg(cx.theme().background)
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .p_4()
                    .child(Editor::new(&self.editor).bordered(false).h_full()),
            )
            .when(self.show_preview, |el| {
                el.child(
                    self.preview
                        .clone()
                        .cached(StyleRefinement::default().flex_1().h_full()),
                )
            })
    }
}

/// Types into the start, middle and end of the note, one edit per frame, and
/// records how long each edit takes to reach the screen.
fn run_bench(editor: Entity<EditorState>, window: &mut Window, cx: &mut App) {
    let rows = editor.read(cx).value().lines().count().saturating_sub(1) as u32;
    let spots = [("start", 0), ("middle", rows / 2), ("end", rows)];
    let samples: Rc<RefCell<Vec<(&'static str, Duration)>>> = Rc::default();
    let plan: Vec<(&'static str, u32)> = spots
        .into_iter()
        .flat_map(|spot| std::iter::repeat_n(spot, BENCH_EDITS_PER_SPOT))
        .collect();
    bench_step(editor, plan, 0, samples, window, cx);
}

fn bench_step(
    editor: Entity<EditorState>,
    plan: Vec<(&'static str, u32)>,
    index: usize,
    samples: Rc<RefCell<Vec<(&'static str, Duration)>>>,
    window: &mut Window,
    cx: &mut App,
) {
    let Some(&(spot, row)) = plan.get(index) else {
        report_bench(&samples.borrow());
        cx.quit();
        return;
    };
    let first_at_spot = index == 0 || plan[index - 1].0 != spot;
    let started = Instant::now();
    editor.update(cx, |state, cx| {
        if first_at_spot {
            state.set_cursor_position(Position::new(row, 0), window, cx);
        }
        state.replace_text_in_range(None, "x", window, cx);
    });
    window.on_next_frame(move |window, cx| {
        samples.borrow_mut().push((spot, started.elapsed()));
        bench_step(editor, plan, index + 1, samples, window, cx);
    });
}

fn report_bench(samples: &[(&'static str, Duration)]) {
    for spot in ["start", "middle", "end"] {
        let mut times: Vec<Duration> = samples
            .iter()
            .filter(|(s, _)| *s == spot)
            .map(|(_, d)| *d)
            .collect();
        times.sort();
        if times.is_empty() {
            continue;
        }
        let pct = |p: f64| times[((times.len() - 1) as f64 * p).round() as usize];
        eprintln!(
            "bench {spot:>6}: n={} p50={:?} p95={:?} max={:?}",
            times.len(),
            pct(0.5),
            pct(0.95),
            times[times.len() - 1]
        );
    }
}

fn main() -> Result<()> {
    let args = Args::parse()?;
    let body = std::fs::read_to_string(&args.note)
        .with_context(|| format!("reading {}", args.note.display()))?;

    application().run(move |cx| {
        gpui_kit::init(cx);
        let bounds = Bounds::centered(None, size(px(1400.), px(900.)), cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            ..Default::default()
        };
        let bench = args.bench;
        let (window, view) = gpui_kit::open_window(options, cx, |window, cx| {
            cx.new(|cx| Spike::new(&args, body, window, cx))
        })
        .expect("failed to open window");
        cx.activate(true);

        if bench {
            window
                .update(cx, |_, window, cx| {
                    let editor = view.read(cx).editor.clone();
                    // Let the initial layout settle before typing.
                    window.on_next_frame(move |window, cx| run_bench(editor, window, cx));
                })
                .ok();
        }
    });
    Ok(())
}
