//! Math, Mermaid diagrams and Excalidraw drawings in the preview.
//!
//! The parsers here pick the nodes out as the text view parses (on any
//! thread); the renderers show a drawing from `kanso_render`'s cache, or ask
//! for one and show a quiet placeholder until it is ready. Drawing happens
//! on `kanso_render`'s threads, never this one; a finished drawing wakes
//! the preview, which measures its chunks again. A source that can't be
//! drawn shows as its code, with a badge saying why.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock, Mutex};

use gpui_kit::base::text::{
    InlineElement, InlineRenderContext, MarkdownExtensions, MarkdownNode, MarkdownParseContext,
    MarkdownPlugin, markdown_ast,
};
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::notification::Notification;
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::component::{ActiveTheme as _, Sizable as _, WindowExt as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_render::{Failure, Job, Kind, Outcome, Palette, Rgb};
use kanso_store::Store;

use crate::preview::{PreviewEdit, ResizeSender};
use crate::theme::Colors;

/// The custom nodes.
const MATH: &str = "kanso-math";
const BLOCK: &str = "kanso-rich-block";

/// What the renderers need from the preview: who to wake when a drawing is
/// ready, the store (for attached drawings), and where edits go.
pub struct Rich {
    pub wake: async_channel::Sender<()>,
    pub store: Arc<Store>,
    pub edits: ResizeSender,
}

impl Rich {
    fn waker(&self) -> impl FnOnce() + Send + 'static {
        let wake = self.wake.clone();
        move || {
            wake.try_send(()).ok();
        }
    }
}

/// The preview theme as `kanso_render` draws in it.
pub fn palette(cx: &App) -> Palette {
    let colors = Colors::get(cx);
    let preview = crate::themes::preview_colors(cx);
    let rgb = |c: crate::theme::Rgb| Rgb(c.0);
    Palette {
        dark: cx.theme().is_dark(),
        background: rgb(colors.editor),
        text: rgb(preview.text),
        muted: rgb(preview.task_done),
        accent: rgb(preview.link),
        border: rgb(preview.border),
        surface: rgb(preview.code_background),
    }
}

/// A block to draw.
#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub kind: Kind,
    /// The source to draw; for an attached drawing, empty until read.
    pub source: Arc<str>,
    /// The attachment an attached drawing is.
    pub file: Option<String>,
    /// The block's Markdown as written: shown when it can't be drawn, and
    /// what an edit replaces.
    pub markdown: SharedString,
}

/// Whether a fence's info string names `want`.
fn fence_is(lang: &Option<String>, want: &str) -> bool {
    lang.as_deref()
        .and_then(|l| l.split_whitespace().next())
        .is_some_and(|l| l.eq_ignore_ascii_case(want))
}

/// The block `node` draws as, if any. Pure.
pub fn block(node: &markdown_ast::Node, markdown: &str) -> Option<Block> {
    use markdown_ast::Node;
    let (kind, source, file) = match node {
        Node::Math(math) => (Kind::DisplayMath, math.value.clone(), None),
        Node::Code(code) if fence_is(&code.lang, "math") => {
            (Kind::DisplayMath, code.value.clone(), None)
        }
        Node::Code(code) if fence_is(&code.lang, "mermaid") => {
            (Kind::Mermaid, code.value.clone(), None)
        }
        Node::Code(code) if fence_is(&code.lang, "excalidraw") => {
            (Kind::Excalidraw, code.value.clone(), None)
        }
        Node::Paragraph(_) => (
            Kind::Excalidraw,
            String::new(),
            Some(kanso_render::html::attached_scene(node)?),
        ),
        _ => return None,
    };
    Some(Block {
        kind,
        source: source.into(),
        file,
        markdown: markdown.trim_end().to_owned().into(),
    })
}

/// The block parser: math blocks, diagrams and drawings.
pub fn parse_block(
    node: &markdown_ast::Node,
    cx: &MarkdownParseContext<'_>,
) -> Option<MarkdownNode> {
    let markdown = cx.node_source(node)?;
    let block = block(node, markdown)?;
    let text = block.source.to_string();
    Some(
        MarkdownNode::new(BLOCK, block)
            .text(text)
            .markdown(markdown),
    )
}

/// An inline formula.
struct InlineMath(Arc<str>);

/// `$…$` in a line: typeset in place, sized to the text around it.
struct MathPlugin {
    rich: Arc<Rich>,
}

impl MarkdownPlugin for MathPlugin {
    fn name(&self) -> &str {
        MATH
    }

    fn parse(
        &self,
        node: &markdown_ast::Node,
        cx: &MarkdownParseContext<'_>,
    ) -> Option<MarkdownNode> {
        let markdown_ast::Node::InlineMath(math) = node else {
            return None;
        };
        let position = node.position()?;
        let span = cx
            .source()
            .get(position.start.offset..position.end.offset)?;
        let next = cx
            .source()
            .get(position.end.offset..)
            .and_then(|rest| rest.chars().next());
        // "$5 and $10" is prose: unclaimed, the text view shows it as such.
        if !kanso_render::is_inline_math(span, next) {
            return None;
        }
        // Copied, or shown while it is typeset, the formula reads as typed.
        Some(
            MarkdownNode::new(MATH, InlineMath(math.value.clone().into()))
                .text(span)
                .markdown(span),
        )
    }

    fn render_inline(
        &self,
        node: &MarkdownNode,
        context: &InlineRenderContext,
        _window: &mut Window,
        cx: &mut App,
    ) -> Option<InlineElement> {
        let InlineMath(source) = node.data::<InlineMath>()?;
        let job = Job::new(Kind::InlineMath, source.clone(), palette(cx));
        let outcome = drawing(&job, &self.rich)?;
        let (image, svg) = match outcome.as_ref() {
            Ok(svg) => (image(&job, svg), svg),
            // A mistake shows as its source, set apart.
            Err(_) => return None,
        };
        let scale = f32::from(context.font_size()) / kanso_render::TEXT_SIZE;
        // Whole points tall: the text view drops an inline element whose
        // baseline falls below its (pixel-snapped) bottom, as a formula
        // without descenders can by a fraction.
        let height = (svg.height * scale).ceil();
        let fit = height / (svg.height * scale);
        let width = svg.width * scale * fit;
        let baseline = (svg.baseline.unwrap_or(svg.height) * scale * fit)
            .min(height - 0.5)
            .max(0.);
        Some(
            InlineElement::new(img(image).w(px(width)).h(px(height)).flex_none())
                .with_baseline(px(baseline)),
        )
    }
}

/// Adds the math plugin and the block renderer to `extensions`.
pub fn register(extensions: MarkdownExtensions, rich: Arc<Rich>) -> MarkdownExtensions {
    let blocks = rich.clone();
    extensions
        .plugin(MathPlugin { rich })
        .block_renderer(BLOCK, move |node, window, cx| {
            render_block(node, &blocks, window, cx)
        })
}

/// The drawing for `job` if it is ready; otherwise asks for it (once) and
/// says so with `None`.
fn drawing(job: &Job, rich: &Rich) -> Option<Arc<Outcome>> {
    if let Some(outcome) = kanso_render::cached(job) {
        return Some(outcome);
    }
    kanso_render::request(job.clone(), rich.waker());
    None
}

/// Decoded images by job: GPUI caches a rasterized image by the image, so
/// a drawing keeps one.
static IMAGES: LazyLock<Mutex<HashMap<Job, Arc<Image>>>> = LazyLock::new(Default::default);
const IMAGES_KEPT: usize = 512;

fn image(job: &Job, svg: &kanso_render::Svg) -> Arc<Image> {
    let Ok(mut images) = IMAGES.lock() else {
        return Arc::new(Image::from_bytes(
            ImageFormat::Svg,
            svg.markup.clone().into_bytes(),
        ));
    };
    if let Some(image) = images.get(job) {
        return image.clone();
    }
    if images.len() >= IMAGES_KEPT {
        images.clear();
    }
    let image = Arc::new(Image::from_bytes(
        ImageFormat::Svg,
        svg.markup.clone().into_bytes(),
    ));
    images.insert(job.clone(), image.clone());
    image
}

/// Attached drawings' JSON by attachment ID, read off the main thread;
/// `None` once read if it couldn't be.
static SCENE_FILES: LazyLock<Mutex<HashMap<String, Option<Arc<str>>>>> =
    LazyLock::new(Default::default);
static SCENES_READING: LazyLock<Mutex<HashSet<String>>> = LazyLock::new(Default::default);

/// An attached drawing's JSON: `Some(Some)` when read, `Some(None)` when it
/// can't be, `None` while it is being read.
fn scene_file(id: &str, rich: &Rich, cx: &mut App) -> Option<Option<Arc<str>>> {
    if let Some(found) = SCENE_FILES.lock().ok()?.get(id) {
        return Some(found.clone());
    }
    if SCENES_READING.lock().ok()?.insert(id.to_owned()) {
        let (store, id, wake) = (rich.store.clone(), id.to_owned(), rich.waker());
        cx.background_executor()
            .spawn(async move {
                let scene = store
                    .attachment(&id)
                    .ok()
                    .flatten()
                    .and_then(|(_, attachment)| String::from_utf8(attachment.data).ok())
                    .map(Arc::from);
                if let Ok(mut files) = SCENE_FILES.lock() {
                    files.insert(id.clone(), scene);
                }
                if let Ok(mut reading) = SCENES_READING.lock() {
                    reading.remove(&id);
                }
                wake();
            })
            .detach();
    }
    None
}

fn render_block(
    node: &MarkdownNode,
    rich: &Arc<Rich>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let Some(block) = node.data::<Block>() else {
        return div().into_any_element();
    };
    let at = node.source_range().map_or(0, |r| r.start) as u64;
    let source = match &block.file {
        Some(id) => match scene_file(id, rich, cx) {
            Some(Some(scene)) => scene,
            Some(None) => {
                return fallback(
                    block,
                    &Failure::Invalid("The drawing's file is missing.".into()),
                    at,
                    window,
                    cx,
                );
            }
            None => return placeholder(block.kind, cx),
        },
        None => block.source.clone(),
    };
    let job = Job::new(block.kind, source.clone(), palette(cx));
    let Some(outcome) = drawing(&job, rich) else {
        return placeholder(block.kind, cx);
    };
    let svg = match outcome.as_ref() {
        Ok(svg) => svg,
        Err(failure) => return fallback(block, failure, at, window, cx),
    };
    let picture = img(image(&job, svg))
        .object_fit(ObjectFit::Contain)
        .max_w(relative(1.))
        .w(px(svg.width));
    match block.kind {
        Kind::DisplayMath => h_flex()
            .id(ElementId::NamedInteger(BLOCK.into(), at))
            .w_full()
            .justify_center()
            .py(px(4.))
            .child(picture)
            .into_any_element(),
        Kind::Excalidraw => drawing_block(block, source, picture, svg.width, rich, at, cx),
        _ => div()
            .id(ElementId::NamedInteger(BLOCK.into(), at))
            .w_full()
            .child(picture)
            .into_any_element(),
    }
}

/// Room held for a drawing being made.
fn placeholder(kind: Kind, cx: &App) -> AnyElement {
    let colors = Colors::get(cx);
    let height = if kind == Kind::DisplayMath { 32. } else { 96. };
    div()
        .w_full()
        .h(px(height))
        .rounded(px(8.))
        .bg(Hsla::from(colors.fill).opacity(0.5))
        .into_any_element()
}

/// A source that couldn't be drawn: its code as written, with a badge.
fn fallback(
    block: &Block,
    failure: &Failure,
    at: u64,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let colors = Colors::get(cx);
    let detail = failure.detail().to_owned();
    let badge = div()
        .id(ElementId::NamedInteger("kanso-rich-badge".into(), at))
        .absolute()
        .top(px(7.))
        .left(px(10.))
        .px(px(7.))
        .py(px(1.))
        .rounded(px(5.))
        .bg(colors.fill)
        .border_1()
        .border_color(colors.divider)
        .text_size(px(11.5))
        .text_color(colors.text_secondary)
        .child(failure.badge())
        .when(!detail.is_empty(), |el| {
            el.tooltip(move |window, cx| Tooltip::new(detail.clone()).build(window, cx))
        });
    div()
        .relative()
        .w_full()
        .child(crate::preview::code_view(
            ElementId::NamedInteger("kanso-rich-code".into(), at),
            block.markdown.clone(),
            window,
            cx,
        ))
        .child(badge)
        .into_any_element()
}

/// The fenced block a drawing becomes in a note.
pub fn scene_block(scene: &str) -> Result<String, Failure> {
    Ok(format!(
        "```excalidraw\n{}\n```",
        kanso_render::excalidraw::to_note(scene)?
    ))
}

/// A drawing with its actions over its corner, shown on hover: edit it in
/// Excalidraw, bring it back, or keep it as a file.
fn drawing_block(
    block: &Block,
    scene: Arc<str>,
    picture: Img,
    width: f32,
    rich: &Arc<Rich>,
    at: u64,
    cx: &App,
) -> AnyElement {
    let colors = Colors::get(cx);
    let group = SharedString::from(format!("{BLOCK}-{at}"));
    #[cfg(test)]
    let hidden = !crate::preview::REVEAL_IMAGE_SIZES.load(std::sync::atomic::Ordering::Relaxed);
    #[cfg(not(test))]
    let hidden = true;
    let action = |ix: usize, label: &'static str, tooltip: &'static str| {
        Button::new(ix)
            .ghost()
            .xsmall()
            .tab_stop(false)
            .tooltip(tooltip)
            .child(
                div()
                    .px_0p5()
                    .text_size(px(11.5))
                    .text_color(colors.text_secondary)
                    .child(label),
            )
    };
    let open = {
        let scene = scene.clone();
        action(
            0,
            "Edit in Excalidraw",
            "Copies the drawing and opens excalidraw.com: paste it there",
        )
        .on_click(move |_, window, cx| {
            cx.stop_propagation();
            edit_in_excalidraw(&scene, window, cx);
        })
    };
    let paste = {
        let (rich, markdown) = (rich.clone(), block.markdown.to_string());
        action(
            1,
            "Paste back",
            "Replaces this drawing with the one copied from Excalidraw",
        )
        .on_click(move |_, window, cx| {
            cx.stop_propagation();
            paste_back(&markdown, &rich, window, cx);
        })
    };
    let save = {
        let scene = scene.clone();
        action(
            2,
            "Save as file…",
            "Saves the drawing as an .excalidraw file and shows it",
        )
        .on_click(move |_, window, cx| {
            cx.stop_propagation();
            save_scene(scene.clone(), window, cx);
        })
    };
    let update = {
        let (rich, markdown) = (rich.clone(), block.markdown.to_string());
        action(
            3,
            "Update from file…",
            "Replaces this drawing with an edited .excalidraw file",
        )
        .on_click(move |_, window, cx| {
            cx.stop_propagation();
            update_from_file(markdown.clone(), rich.clone(), window, cx);
        })
    };
    let actions = h_flex()
        .id("drawing-actions")
        .absolute()
        .top(px(8.))
        .right(px(8.))
        .p(px(2.))
        .gap(px(2.))
        .rounded(px(7.))
        .border_1()
        .border_color(colors.divider)
        .bg(colors.elevated)
        .shadow_sm()
        .debug_selector(|| "drawing-actions".into())
        .when(hidden, |el| {
            el.opacity(0.)
                .group_hover(group.clone(), |style| style.opacity(1.))
        })
        .child(open)
        .child(paste)
        .child(save)
        .child(update);
    v_flex()
        .id(ElementId::NamedInteger(BLOCK.into(), at))
        .group(group)
        .relative()
        // As wide as the drawing, so its actions sit over its corner.
        .w(px(width.max(380.)))
        .max_w(relative(1.))
        .items_start()
        .child(picture)
        .child(actions)
        .into_any_element()
}

fn notify(window: &mut Window, cx: &mut App, notification: Notification) {
    window.push_notification(notification, cx);
}

/// Puts the drawing on the clipboard as Excalidraw copies it, and opens
/// excalidraw.com to paste it into. Nothing is uploaded.
fn edit_in_excalidraw(scene: &str, window: &mut Window, cx: &mut App) {
    match kanso_render::excalidraw::to_clipboard(scene) {
        Ok(clip) => {
            cx.write_to_clipboard(ClipboardItem::new_string(clip));
            cx.open_url("https://excalidraw.com");
            notify(
                window,
                cx,
                Notification::info(
                    "Drawing copied. Paste it into Excalidraw, then copy it back and choose Paste back.",
                ),
            );
        }
        Err(failure) => notify(window, cx, Notification::error(failure.badge())),
    }
}

/// Replaces the drawing with the scene on the clipboard.
fn paste_back(markdown: &str, rich: &Rich, window: &mut Window, cx: &mut App) {
    let text = cx.read_from_clipboard().and_then(|item| item.text());
    let replacement = text
        .filter(|text| kanso_render::excalidraw::is_scene(text))
        .and_then(|text| scene_block(&text).ok());
    match replacement {
        Some(replacement) => {
            rich.edits
                .try_send(PreviewEdit::Replace {
                    source: markdown.to_owned(),
                    replacement,
                })
                .ok();
        }
        None => notify(
            window,
            cx,
            Notification::warning(
                "The clipboard has no Excalidraw drawing. In Excalidraw, select it all and copy.",
            ),
        ),
    }
}

fn save_scene(scene: Arc<str>, window: &mut Window, cx: &mut App) {
    let file = match kanso_render::excalidraw::to_file(&scene) {
        Ok(file) => file,
        Err(failure) => return notify(window, cx, Notification::error(failure.badge())),
    };
    let start = crate::paths::home().join("Documents");
    let start = if start.is_dir() {
        start
    } else {
        crate::paths::home()
    };
    let picked = cx.prompt_for_new_path(&start, Some("Drawing.excalidraw"));
    window
        .spawn(cx, async move |cx| {
            let Ok(Ok(Some(path))) = picked.await else {
                return;
            };
            let written = std::fs::write(&path, file);
            cx.update(|window, cx| match written {
                Ok(()) => cx.reveal_path(&path),
                Err(err) => notify(
                    window,
                    cx,
                    Notification::error(format!("Couldn't save the drawing: {err}")),
                ),
            })
            .ok();
        })
        .detach();
}

fn update_from_file(markdown: String, rich: Arc<Rich>, window: &mut Window, cx: &mut App) {
    let picked = cx.prompt_for_paths(PathPromptOptions {
        files: true,
        directories: false,
        multiple: false,
        prompt: Some("Update Drawing".into()),
    });
    window
        .spawn(cx, async move |cx| {
            let Ok(Ok(Some(paths))) = picked.await else {
                return;
            };
            let Some(path) = paths.into_iter().next() else {
                return;
            };
            let read = std::fs::read_to_string(&path)
                .map_err(|err| format!("Couldn't read the file: {err}"))
                .and_then(|text| {
                    if kanso_render::excalidraw::is_scene(&text) {
                        scene_block(&text).map_err(|f| f.badge())
                    } else {
                        Err("That file isn't an Excalidraw drawing.".into())
                    }
                });
            cx.update(|window, cx| match read {
                Ok(replacement) => {
                    rich.edits
                        .try_send(PreviewEdit::Replace {
                            source: markdown,
                            replacement,
                        })
                        .ok();
                }
                Err(message) => notify(window, cx, Notification::error(message)),
            })
            .ok();
        })
        .detach();
}

#[cfg(test)]
mod tests {
    use super::{Block, block, scene_block};
    use kanso_render::Kind;
    use proptest::prelude::*;

    fn first_block(text: &str) -> Option<Block> {
        let mut options = markdown::ParseOptions::gfm();
        options.constructs.math_text = true;
        options.constructs.math_flow = true;
        let root = markdown::to_mdast(text, &options).unwrap();
        let node = root.children().unwrap()[0].clone();
        let at = node.position().unwrap();
        block(&node, &text[at.start.offset..at.end.offset])
    }

    #[test]
    fn rich_blocks_are_found() {
        let math = first_block("$$\nx^2\n$$\n").unwrap();
        assert_eq!((math.kind, &*math.source), (Kind::DisplayMath, "x^2"));
        let fenced = first_block("```math\n\\frac{1}{2}\n```\n").unwrap();
        assert_eq!(fenced.kind, Kind::DisplayMath);
        let diagram = first_block("```mermaid\ngraph TD\nA-->B\n```\n").unwrap();
        assert_eq!(diagram.kind, Kind::Mermaid);
        assert_eq!(&*diagram.markdown, "```mermaid\ngraph TD\nA-->B\n```");
        let drawing = first_block("```excalidraw\n{}\n```").unwrap();
        assert_eq!(drawing.kind, Kind::Excalidraw);
        let attached = first_block("[plan.excalidraw](inkdrop://file:abc)\n").unwrap();
        assert_eq!(attached.file.as_deref(), Some("file:abc"));
        assert_eq!(first_block("```rust\nfn x() {}\n```"), None);
        assert_eq!(first_block("[plan.png](inkdrop://file:abc)"), None);
        assert_eq!(first_block("Just words."), None);
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(128))]

        /// Round trip: a scene put into a note as a block is found again as
        /// a drawing with the same elements.
        #[test]
        fn prop_scene_blocks_round_trip(n in 0usize..5, seed in any::<i32>()) {
            let elements: Vec<serde_json::Value> = (0..n)
                .map(|i| serde_json::json!({"type": "rectangle", "x": i * 10, "y": 0, "width": 5, "height": 5, "seed": seed}))
                .collect();
            let scene = serde_json::json!({"type": "excalidraw/clipboard", "elements": elements}).to_string();
            let block_text = scene_block(&scene).unwrap();
            let found = first_block(&block_text).unwrap();
            prop_assert_eq!(found.kind, Kind::Excalidraw);
            let back: serde_json::Value = serde_json::from_str(&found.source).unwrap();
            prop_assert_eq!(back["elements"].as_array().unwrap().len(), n);
        }
    }
}
