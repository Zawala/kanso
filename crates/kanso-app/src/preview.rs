//! The rendered note. It is its own view so the editor can cache it:
//! re-laying out a large note on every keystroke frame doubles edit latency.
//!
//! The note is shown in chunks (see [`crate::preview_chunks`]), each its own
//! text view in a list: an edit re-parses and re-lays out only the chunk it
//! falls in, and only the chunks on screen are laid out at all. Finding the
//! chunks of a long note parses it, which runs off the main thread.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::base::text::{
    CodeBlock, MarkdownExtensions, MarkdownNode, MarkdownParseContext, TextView, TextViewState,
    TextViewStyle, markdown_ast,
};
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{ActiveTheme as _, Icon, Sizable as _, h_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_store::Store;

use crate::preview_chunks::{self, End, Prepared, SPLIT_SYNC_BYTES, Splice, Start};
use crate::theme::Colors;
use crate::theme_file::PreviewColors;

/// The rendered text's size and line height.
const TEXT_SIZE: f32 = 16.;
const LINE_HEIGHT: f32 = 1.65;
/// The room above the first chunk and below the last.
const TEXT_TOP: f32 = 24.;
/// How far past the pane the list lays chunks out, so a scroll shows them
/// laid out already.
const OVERDRAW: f32 = 800.;
/// The widest text runs, in ems of its size: about 75 characters a line,
/// however wide the window. SF Pro averages about 0.44 em a character in
/// prose, SF Mono 0.62.
const READABLE_EMS: f32 = 35.;
const MONO_READABLE_EMS: f32 = 47.;
/// How far text sits from the pane's left edge: under the note's title, like
/// the source in Edit mode.
pub const TEXT_INSET: f32 = 28.;

/// The readable measure for text set at `size`, in the monospaced face or
/// the proportional one. The preview keeps to it, and so does the source in
/// Edit mode, at its own size.
pub fn readable_width(size: f32, mono: bool) -> f32 {
    size * if mono {
        MONO_READABLE_EMS
    } else {
        READABLE_EMS
    }
}

/// Where clicks on `inkdrop://note/<id>` links go.
pub type LinkSender = async_channel::Sender<String>;

/// A chunk on show.
struct Shown {
    key: u64,
    start: Start,
    end: End,
    state: Entity<TextViewState>,
    _watch: Subscription,
}

pub struct Preview {
    /// The text handed over last.
    text: SharedString,
    /// The text the chunks show: `text`, or an older one while it is split.
    shown: SharedString,
    chunks: Vec<Shown>,
    list: ListState,
    /// Splitting a long text, off the main thread.
    splitting: Option<Task<()>>,
    /// Counts the texts shown, so the editor can tell the preview changed.
    generation: u64,
    store: Arc<Store>,
    images: Arc<Mutex<HashMap<String, Option<Arc<Image>>>>>,
    links: LinkSender,
    /// Whether text keeps to the readable measure (the setting), or fills
    /// the pane.
    readable: bool,
    /// Draws quotes in their own color (see [`quote_extensions`]).
    extensions: MarkdownExtensions,
}

impl Preview {
    pub fn new(store: Arc<Store>, links: LinkSender) -> Self {
        let images: Arc<Mutex<HashMap<String, Option<Arc<Image>>>>> = Arc::default();
        let resolver: ImageResolver = Arc::new(image_resolver(store.clone(), images.clone()));
        let extensions = quote_extensions(Arc::new(Links(links.clone())), resolver);
        Preview {
            text: SharedString::default(),
            shown: SharedString::default(),
            chunks: Vec::new(),
            list: ListState::new(0, ListAlignment::Top, px(OVERDRAW)).measure_all(),
            splitting: None,
            generation: 0,
            store,
            images,
            links,
            readable: false,
            extensions,
        }
    }

    /// Shows `text`. A short one shows at once; a long one once it has been
    /// split, off the main thread, with the text before it shown until then.
    /// While one is being split, a newer text waits for it and then follows.
    pub fn set_text(&mut self, text: SharedString, cx: &mut Context<Self>) {
        if self.text == text {
            return;
        }
        self.text = text;
        if self.splitting.is_some() {
            return;
        }
        if self.text.len() <= SPLIT_SYNC_BYTES {
            let prepared = preview_chunks::prepare(&self.text);
            self.apply(self.text.clone(), prepared, cx);
        } else {
            self.split_soon(cx);
        }
    }

    fn split_soon(&mut self, cx: &mut Context<Self>) {
        let text = self.text.clone();
        self.splitting = Some(cx.spawn(async move |this, cx| {
            let (text, prepared) = cx
                .background_spawn(async move {
                    let prepared = preview_chunks::prepare(&text);
                    (text, prepared)
                })
                .await;
            this.update(cx, |this, cx| {
                this.splitting = None;
                this.apply(text, prepared, cx);
                if this.shown != this.text {
                    if this.text.len() <= SPLIT_SYNC_BYTES {
                        let prepared = preview_chunks::prepare(&this.text);
                        this.apply(this.text.clone(), prepared, cx);
                    } else {
                        this.split_soon(cx);
                    }
                }
            })
            .ok();
        }));
    }

    /// Shows `text`, split into `prepared`: the chunks it shares with what
    /// is shown keep their parse and layout, the rest are parsed and
    /// measured again.
    fn apply(&mut self, text: SharedString, prepared: Vec<Prepared>, cx: &mut Context<Self>) {
        let old: Vec<u64> = self.chunks.iter().map(|c| c.key).collect();
        let new: Vec<u64> = prepared.iter().map(|c| c.key).collect();
        let Splice {
            old_range,
            new_range,
        } = preview_chunks::diff(&old, &new);
        let mut prepared = prepared;
        let mut changed = prepared.drain(new_range.clone());
        if old_range.len() == new_range.len() {
            // Typing changes a chunk in place: it keeps its view and its
            // measured height until it is measured again.
            for (shown, chunk) in self.chunks[old_range.clone()].iter_mut().zip(&mut changed) {
                shown.key = chunk.key;
                shown.start = chunk.start;
                shown.end = chunk.end;
                shown
                    .state
                    .update(cx, |state, cx| state.set_text(&chunk.text, cx));
            }
            drop(changed);
            if !old_range.is_empty() {
                self.list.remeasure_items(old_range);
            }
        } else {
            let added: Vec<Shown> = changed.map(|chunk| self.show_chunk(chunk, cx)).collect();
            self.chunks.splice(old_range.clone(), added);
            self.list.splice(old_range, new_range.len());
            // Measures the new chunks, and only them, at the next layout.
            self.list.remeasure_items(new_range);
        }
        self.shown = text;
        self.generation += 1;
        cx.notify();
    }

    fn show_chunk(&mut self, chunk: Prepared, cx: &mut Context<Self>) -> Shown {
        let state = cx.new(|cx| TextViewState::markdown(&chunk.text, cx).selectable(true));
        // A long chunk parses in the background and changes height when it
        // is done, as does one whose images load: it is measured again.
        let watch = cx.observe(&state, |this, state, _| {
            if let Some(ix) = this.chunks.iter().position(|c| c.state == state) {
                this.list.remeasure_items(ix..ix + 1);
            }
        });
        Shown {
            key: chunk.key,
            start: chunk.start,
            end: chunk.end,
            state,
            _watch: watch,
        }
    }

    /// Counts the texts shown: it changes whenever the preview shows
    /// another.
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// The text the preview shows.
    #[cfg(test)]
    pub fn shown(&self) -> &SharedString {
        &self.shown
    }

    /// How many chunks the text is shown in.
    #[cfg(test)]
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    /// The chunks shown, by key, with their views.
    #[cfg(test)]
    fn chunk_views(&self) -> Vec<(u64, EntityId)> {
        self.chunks
            .iter()
            .map(|c| (c.key, c.state.entity_id()))
            .collect()
    }

    /// Keeps text to the readable measure, or lets it fill the pane.
    pub fn set_readable(&mut self, readable: bool, cx: &mut Context<Self>) {
        if self.readable != readable {
            self.readable = readable;
            cx.notify();
        }
    }

    /// How far the preview can scroll, as of its last layout.
    pub fn scroll_max(&self) -> f32 {
        f32::from(self.list.max_offset_for_scrollbar().y).max(0.)
    }

    /// Scrolls `top` points down from the start.
    pub fn scroll_to(&mut self, top: f32, cx: &mut Context<Self>) {
        let top = top.max(0.);
        if (self.scroll_top() - top).abs() >= 0.5 {
            self.list.set_offset_from_scrollbar(point(px(0.), px(-top)));
            cx.notify();
        }
    }

    /// Where the preview is scrolled to, in points from the start.
    pub fn scroll_top(&self) -> f32 {
        -f32::from(self.list.scroll_px_offset_for_scrollbar().y)
    }

    /// Resolves `inkdrop://file:<id>` from the store, caching decoded images.
    fn image_resolver(&self) -> impl Fn(&SharedUri) -> ImageSource + Send + Sync + 'static {
        image_resolver(self.store.clone(), self.images.clone())
    }
}

/// Finds a note's images: `inkdrop://file:<id>` from the store, decoded once
/// and cached; anything else as the URI says.
type ImageResolver = Arc<dyn Fn(&SharedUri) -> ImageSource + Send + Sync>;

fn image_resolver(
    store: Arc<Store>,
    images: Arc<Mutex<HashMap<String, Option<Arc<Image>>>>>,
) -> impl Fn(&SharedUri) -> ImageSource + Send + Sync + 'static {
    move |uri| {
        let Some(suffix) = uri.strip_prefix("inkdrop://file:") else {
            return ImageSource::from(uri.clone());
        };
        let id = format!("file:{suffix}");
        let mut cache = images.lock().expect("image cache poisoned");
        let image = cache.entry(id.clone()).or_insert_with(|| {
            let (content_type, attachment) = store.attachment(&id).ok()??;
            let format = ImageFormat::from_mime_type(&content_type)?;
            Some(Arc::new(Image::from_bytes(format, attachment.data)))
        });
        match image {
            Some(image) => ImageSource::Image(image.clone()),
            None => ImageSource::from(uri.clone()),
        }
    }
}

/// Where a clicked link goes: a note link to the editor, a web or mail link
/// to the system, anything else nowhere.
struct Links(LinkSender);

impl Links {
    fn open(&self, url: &str, cx: &mut App) {
        match linked_note(url) {
            Some(id) => {
                self.0.try_send(id).ok();
            }
            None if is_openable(url) => cx.open_url(url),
            None => {}
        }
    }
}

/// A quote's text without its `>` markers, to be shown as Markdown of its
/// own. Each line loses the whitespace before its `>`, the `>` and one space
/// or tab after it; a line without one (a lazy continuation) stays as it is.
/// Pure.
pub(crate) fn unquote(source: &str) -> String {
    source
        .split('\n')
        .map(
            |line| match line.trim_start_matches([' ', '\t']).strip_prefix('>') {
                Some(rest) => rest
                    .strip_prefix(' ')
                    .or_else(|| rest.strip_prefix('\t'))
                    .unwrap_or(rest),
                None => line,
            },
        )
        .collect::<Vec<_>>()
        .join("\n")
}

/// The custom node a quote becomes.
const QUOTE: &str = "kanso-quote";
/// Changes when [`quote_extensions`] parses differently, so a text view
/// re-parses its note.
const QUOTE_PARSER_REVISION: u64 = 1;

/// A quote's Markdown, without its markers.
struct Quote(SharedString);

/// Quotes drawn by Kanso rather than the text view: the text view draws a
/// quote in its one muted color, the one it also gives finished tasks and
/// list markers, so a theme couldn't make quotes stand out without making
/// finished tasks shout too. Each quote becomes a text view of its own in
/// the theme's quote color, beside a bar.
fn quote_extensions(links: Arc<Links>, images: ImageResolver) -> MarkdownExtensions {
    let parse = |node: &markdown_ast::Node, cx: &MarkdownParseContext<'_>| {
        let markdown_ast::Node::Blockquote(_) = node else {
            return None;
        };
        let source = cx.node_source(node)?;
        let body = unquote(source);
        Some(
            MarkdownNode::new(QUOTE, Quote(body.clone().into()))
                .text(body)
                .markdown(source),
        )
    };
    MarkdownExtensions::default()
        .parser_revision(QUOTE_PARSER_REVISION)
        .block_parser(parse)
        .block_renderer(QUOTE, move |node, _window, cx| {
            let Some(Quote(body)) = node.data::<Quote>() else {
                return div().into_any_element();
            };
            let colors = Colors::get(cx);
            let preview = crate::themes::preview_colors(cx);
            let dark = cx.theme().is_dark();
            let style = text_style(colors, &preview, dark).with_foreground(preview.quote.into());
            let bar = Hsla::from(preview.quote).opacity(0.45);
            let id = ElementId::NamedInteger(
                QUOTE.into(),
                node.source_range().map_or(0, |r| r.start) as u64,
            );
            let links = links.clone();
            let resolver = images.clone();
            div()
                .w_full()
                .min_w_0()
                .border_l(px(3.))
                .border_color(bar)
                .pl(px(14.))
                .child(
                    TextView::markdown(id, body.clone())
                        .style(style)
                        .text_size(px(TEXT_SIZE))
                        .line_height(relative(LINE_HEIGHT))
                        .text_color(preview.quote)
                        .selectable(true)
                        .markdown_extensions(quote_extensions(links.clone(), resolver.clone()))
                        .code_block_actions(code_block_actions)
                        .image_source(move |uri| resolver(uri))
                        .on_link_click(move |url, _, _, cx| links.open(url, cx)),
                )
                .into_any_element()
        })
}

/// What sits in a code block's corner: its language, quietly, and a button
/// that copies the code.
fn code_block_actions(block: &CodeBlock, _: &mut Window, cx: &mut App) -> AnyElement {
    let colors = Colors::get(cx);
    let code = block.code();
    h_flex()
        .gap_1()
        .items_center()
        .children(block.lang().filter(|lang| !lang.is_empty()).map(|lang| {
            div()
                .pl_1()
                .font_family(cx.theme().font_family.clone())
                .text_size(px(12.))
                .text_color(colors.text_muted)
                .child(lang)
        }))
        .child(
            Button::new("copy")
                .ghost()
                .xsmall()
                .icon(Icon::new(Lucide::Copy).text_color(colors.text_muted))
                .tooltip("Copy code")
                .on_click(move |_, _, cx| {
                    cx.write_to_clipboard(ClipboardItem::new_string(code.to_string()));
                }),
        )
        .into_any_element()
}

/// A heading's size relative to the body, and the gap above it relative to
/// its size, as the text view sets them.
fn heading_metrics(level: u8) -> (f32, f32) {
    match level {
        1 => (1.8, 1.1),
        2 => (4. / 3., 1.6),
        3 => (17. / 15., 1.4),
        4 => (1., 1.4),
        5 => (14. / 15., 1.4),
        _ => (13. / 15., 1.4),
    }
}

/// The gap below a heading, relative to its size, as the text view sets it.
const HEADING_BOTTOM_GAP: f32 = 0.35;
/// The gap below ordinary blocks, as [`text_style`] sets it.
const PARAGRAPH_GAP: f32 = 0.9;

/// The room above a chunk that starts like `start` (`first` for the note's
/// first): what the text view would leave above that block in the whole
/// note, less the gap the chunk before already leaves below itself.
fn room_above(start: Start, first: bool, rem: Pixels) -> Pixels {
    if first {
        return px(TEXT_TOP);
    }
    match start {
        Start::Flush => px(0.),
        Start::Heading(level) => {
            let (scale, gap) = heading_metrics(level);
            (px(TEXT_SIZE * scale * gap) - rem * PARAGRAPH_GAP).max(px(0.))
        }
    }
}

/// The room below a chunk that ends like `end`: the gap the text view
/// leaves below that block when another follows it.
fn room_below(end: End, rem: Pixels) -> Pixels {
    match end {
        End::Last => px(TEXT_TOP),
        End::Gap => rem * PARAGRAPH_GAP,
        End::Heading(level) => px(TEXT_SIZE * heading_metrics(level).0 * HEADING_BOTTOM_GAP),
    }
}

/// The note ID an in-app link points at.
pub fn linked_note(url: &str) -> Option<String> {
    let suffix = url
        .strip_prefix("inkdrop://note/")
        .or_else(|| url.strip_prefix("inkdrop://note:"))?;
    let suffix = suffix.split(['?', '#', '/']).next()?;
    (!suffix.is_empty()).then(|| format!("note:{suffix}"))
}

/// Whether a clicked link may be handed to macOS to open. Only web and mail
/// links are: a note can come from sync or the API, and other schemes can
/// launch apps or open local files.
pub fn is_openable(url: &str) -> bool {
    let Some((scheme, rest)) = url.split_once(':') else {
        return false;
    };
    match scheme.to_ascii_lowercase().as_str() {
        "http" | "https" => rest.starts_with("//") && rest.len() > 2,
        "mailto" => !rest.is_empty(),
        _ => false,
    }
}

/// Body text in the preview theme's colors; code is set apart by a quiet
/// fill rather than by color, and so is a table's header row, which is also
/// set heavier. Table rows sit on the page itself; a table wider than the
/// column scrolls sideways rather than cutting off its last columns.
/// Finished tasks and list markers, which the text view draws in its muted
/// color, take the theme's `task_done` color (Kanso's secondary text), not
/// the quote's: quotes are drawn by [`quote_extensions`]. Code blocks leave
/// room at the top for their language and copy button. Headings, code and
/// a table's header take colors of their own only when the theme gives
/// them; a first-level heading is set bold. `dark` picks the check mark
/// that reads on the dark theme's task boxes.
// no-prop-test: maps theme colors onto a style; the snapshots show it.
pub(crate) fn text_style(colors: &Colors, p: &PreviewColors, dark: bool) -> TextViewStyle {
    let mut table = StyleRefinement::default().bg(colors.editor);
    table.overflow.x = Some(Overflow::Scroll);
    let selection = match p.selection {
        Some(color) => Hsla::from(color.rgb).opacity(f32::from(color.alpha) / 255.),
        None => Hsla::from(colors.accent).opacity(0.3),
    };
    let mut code_block = StyleRefinement::default()
        .rounded(px(8.))
        .pt(px(CODE_TOP))
        .border_color(Hsla::from(p.border).opacity(0.6));
    if let Some(code) = p.code_text {
        code_block = code_block.text_color(code);
    }
    let mut table_head = StyleRefinement::default().font_weight(FontWeight::SEMIBOLD);
    if let Some(head) = p.table_header_background {
        table_head = table_head.bg(head);
    }
    let heading = p.heading;
    TextViewStyle::default()
        .with_dark(dark)
        .with_foreground(p.text.into())
        .with_muted_foreground(p.task_done.into())
        .with_link(p.link.into())
        .with_selection(selection)
        .with_code_background(p.code_background.into())
        .with_border(p.border.into())
        .with_paragraph_gap(rems(0.9))
        .with_code_block(code_block)
        .with_inline_code(HighlightStyle {
            background_color: Some(p.inline_code_background.into()),
            color: p.inline_code_text.map(Into::into),
            ..Default::default()
        })
        .with_table(table)
        .with_table_head(table_head)
        .with_heading(move |level| {
            let mut style = StyleRefinement::default();
            if level == 1 {
                style = style.font_weight(FontWeight::BOLD);
            }
            match heading {
                Some(color) => style.text_color(color),
                None => style,
            }
        })
}

/// Room above a code block's first line, clear of its language and copy
/// button.
const CODE_TOP: f32 = 34.;

impl Render for Preview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let preview = crate::themes::preview_colors(cx);
        let style = text_style(colors, &preview, cx.theme().is_dark());
        let text_color = preview.text;
        let readable = self.readable;
        let chunks: Vec<(Entity<TextViewState>, Start, End)> = self
            .chunks
            .iter()
            .map(|c| (c.state.clone(), c.start, c.end))
            .collect();
        let resolver: ImageResolver = Arc::new(self.image_resolver());
        let links = Arc::new(Links(self.links.clone()));
        let extensions = self.extensions.clone();
        div().id("preview").size_full().bg(colors.editor).child(
            list(self.list.clone(), move |ix, window, _| {
                let Some((state, start, end)) = chunks.get(ix).cloned() else {
                    return div().into_any_element();
                };
                let rem = window.rem_size();
                let resolver = resolver.clone();
                let links = links.clone();
                let extensions = extensions.clone();
                div()
                    .w_full()
                    .pl(px(TEXT_INSET))
                    .pr_8()
                    .pt(room_above(start, ix == 0, rem))
                    .pb(room_below(end, rem))
                    .child(
                        div()
                            .w_full()
                            .when(readable, |el| {
                                el.max_w(px(readable_width(TEXT_SIZE, false)))
                            })
                            .child(
                                TextView::new(&state)
                                    .style(style.clone())
                                    .text_size(px(TEXT_SIZE))
                                    .line_height(relative(LINE_HEIGHT))
                                    .text_color(text_color)
                                    .selectable(true)
                                    .markdown_extensions(extensions)
                                    .code_block_actions(code_block_actions)
                                    .image_source(move |uri| resolver(uri))
                                    .on_link_click(move |url, _, _, cx| links.open(url, cx)),
                            ),
                    )
                    .into_any_element()
            })
            .size_full(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{is_openable, linked_note, readable_width};
    use crate::theme::SYSTEM_MONO;
    use gpui_kit::{TextSystem, font, px};
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    /// Ordinary note prose, to measure faces with.
    const PROSE: &str = "Kanso acts as an MCP host. Clients connect to Kanso, discover the \
        tools it offers, and ask before they act. Documents live in D1, attachments in R2. \
        Last writer wins by the update time; the loser stays in history. Rows, not cards. \
        The reason is not recorded in the inspected fields, so earlier QA cannot establish \
        current acceptance.";

    /// How many characters of [`PROSE`] fit in `width`, on average, set in
    /// `family` at `size`.
    fn characters_in(family: &str, size: f32, width: f32) -> f32 {
        let text_system = TextSystem::new(std::sync::Arc::new(gpui_apple::AppleTextSystem::new()));
        let id = text_system.resolve_font(&font(family));
        let advance = |c| f32::from(text_system.advance(id, px(size), c).unwrap().width);
        let total: f32 = PROSE.chars().map(advance).sum();
        width / (total / PROSE.chars().count() as f32)
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 24,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        })]

        /// Invariant: at every size the setting allows, the measure holds
        /// about 75 characters of prose in either face, so it grows with the
        /// text instead of crowding more words onto a line at small sizes.
        #[test]
        fn prop_readable_measure_holds_about_75_characters(size in 10f32..=28., mono in any::<bool>()) {
            let family = if mono { SYSTEM_MONO } else { ".SystemUIFont" };
            let characters = characters_in(family, size, readable_width(size, mono));
            prop_assert!((72.0..=82.0).contains(&characters), "{size} {mono}: {characters:.1}");
        }
    }

    /// The default body size gives the preview 560 points of text, and the
    /// wider monospaced face a wider measure.
    #[test]
    fn readable_width_examples() {
        assert_eq!(readable_width(16., false), 560.);
        assert!(readable_width(16., true) > readable_width(16., false));
        // The old fixed 720 points held far more than 75 characters.
        assert!(characters_in(".SystemUIFont", 16., 720.) > 95.);
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Round-trip: a note link built from an ID resolves to that ID.
        #[test]
        fn prop_note_links_resolve_to_their_id(suffix in "[A-Za-z0-9_-]{1,20}", tail in prop::option::of("[?#/][a-z=]{0,8}")) {
            let url = format!("inkdrop://note/{suffix}{}", tail.unwrap_or_default());
            prop_assert_eq!(linked_note(&url), Some(format!("note:{suffix}")));
        }

        /// Refinement: only web and mail links are opened, whatever the
        /// scheme's case.
        #[test]
        fn prop_only_web_and_mail_links_open(scheme in "[a-zA-Z][a-zA-Z0-9+.-]{0,10}", rest in "(//)?[a-z@./]{1,20}") {
            let url = format!("{scheme}:{rest}");
            let allowed = matches!(scheme.to_ascii_lowercase().as_str(), "http" | "https" | "mailto");
            if is_openable(&url) {
                prop_assert!(allowed, "{}", url);
            }
            // Metamorphic: changing the scheme's case changes nothing.
            let flipped = format!("{}:{rest}", scheme.to_ascii_uppercase());
            prop_assert_eq!(is_openable(&flipped), is_openable(&url));
        }

        /// Invariant: other URLs never resolve to a note.
        #[test]
        fn prop_other_urls_are_not_note_links(url in "(https?|file|inkdrop)://[a-z:/.]{0,20}") {
            prop_assume!(!url.starts_with("inkdrop://note"));
            prop_assert_eq!(linked_note(&url), None);
        }
    }

    /// Incremental equals full: however a run of edits arrives (each shown,
    /// or several while a split is under way), the chunks shown once it
    /// settles are the chunks of the last text, and a chunk no edit touched
    /// keeps its view.
    // no-prop-test: drives the view through seeded random edits; the splice
    // itself is a property in preview_chunks.rs.
    #[gpui_kit::test]
    fn edits_settle_on_the_chunks_of_the_last_text(cx: &mut gpui_kit::TestAppContext) {
        use super::Preview;
        use crate::preview_chunks::prepare;
        use gpui_kit::AppContext as _;

        let dir = tempfile::tempdir().unwrap();
        let store = std::sync::Arc::new(kanso_store::Store::open(dir.path().join("k.db")).unwrap());
        let (links, _rx) = async_channel::unbounded();
        let preview = cx.new(|_| Preview::new(store, links));
        let mut rng = fastrand::Rng::with_seed(20261017);
        let pieces = [
            "word ",
            "\n",
            "\n\n",
            "## Head\n\n",
            "- item\n",
            "```\n",
            "Para. ",
            "> q\n",
        ];
        let mut text: String = (0..200)
            .map(|n| format!("## Part {n}\n\nText {n}.\n\n"))
            .collect();
        let set = |cx: &mut gpui_kit::TestAppContext, text: &str| {
            let text = text.to_owned();
            preview.update(cx, |p, cx| p.set_text(text.into(), cx));
        };
        set(cx, &text);
        cx.run_until_parked();
        for round in 0..60 {
            let before = cx.read(|cx| preview.read(cx).chunk_views());
            // One to four edits before the preview gets to run.
            for _ in 0..rng.usize(1..=4) {
                let mut at = rng.usize(0..=text.len());
                while !text.is_char_boundary(at) {
                    at -= 1;
                }
                if rng.bool() || text.is_empty() {
                    text.insert_str(at, pieces[rng.usize(0..pieces.len())]);
                } else {
                    let end = (at + rng.usize(1..8)).min(text.len());
                    let end = (end..=text.len())
                        .find(|e| text.is_char_boundary(*e))
                        .unwrap();
                    text.replace_range(at..end, "");
                }
                set(cx, &text);
            }
            cx.run_until_parked();
            let (shown, views) = cx.read(|cx| {
                let p = preview.read(cx);
                (p.shown().to_string(), p.chunk_views())
            });
            assert_eq!(shown, text, "round {round}");
            let keys: Vec<u64> = views.iter().map(|(k, _)| *k).collect();
            let expected: Vec<u64> = prepare(&text).iter().map(|c| c.key).collect();
            assert_eq!(keys, expected, "round {round}");
            // The chunks before the first that changed keep their views.
            let kept = before
                .iter()
                .zip(&views)
                .take_while(|(a, b)| a.0 == b.0)
                .count();
            assert!(before[..kept].iter().zip(&views).all(|(a, b)| a.1 == b.1));
        }
    }

    /// Lines of a quote's body: anything but line breaks.
    fn quote_lines() -> impl Strategy<Value = Vec<String>> {
        prop::collection::vec("[^\r\n]{0,24}", 1..8)
    }

    fn quote(lines: &[String]) -> String {
        lines
            .iter()
            .map(|line| format!("> {line}"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        })]

        /// Round trip: quoting lines with `> ` and unquoting gives them
        /// back, leading spaces, tabs and inner `>` included.
        #[test]
        fn prop_unquote_inverts_quote(lines in quote_lines()) {
            prop_assert_eq!(super::unquote(&quote(&lines)), lines.join("\n"));
        }

        /// Round trip for a nested quote: one level comes off at a time.
        #[test]
        fn prop_unquote_peels_one_level(lines in quote_lines()) {
            let inner = quote(&lines);
            let outer = quote(&inner.split('\n').map(str::to_owned).collect::<Vec<_>>());
            prop_assert_eq!(super::unquote(&outer), inner);
        }

        /// Invariants: the line count holds, and a line without a marker (a
        /// lazy continuation) comes through as it was.
        #[test]
        fn prop_unquote_keeps_lines(lines in quote_lines(), indent in "[ \t]{0,3}") {
            let text: Vec<String> = lines
                .iter()
                .enumerate()
                .map(|(ix, line)| if ix % 2 == 0 { format!("{indent}> {line}") } else { line.clone() })
                .collect();
            let out = super::unquote(&text.join("\n"));
            prop_assert_eq!(out.split('\n').count(), text.len());
            for ((ix, line), got) in text.iter().enumerate().zip(out.split('\n')) {
                if ix % 2 == 1 && !line.trim_start_matches([' ', '\t']).starts_with('>') {
                    prop_assert_eq!(got, line.as_str());
                }
            }
        }
    }

    #[test]
    fn unquote_examples() {
        use super::unquote;
        assert_eq!(unquote("> a\n> b"), "a\nb");
        assert_eq!(unquote(">a\n>b"), "a\nb");
        assert_eq!(unquote("> a\nlazy"), "a\nlazy");
        assert_eq!(unquote("  > - item\n  > - two"), "- item\n- two");
        assert_eq!(unquote("> > nested"), "> nested");
        assert_eq!(unquote(">     code"), "    code");
        assert_eq!(unquote(">"), "");
    }

    #[test]
    fn note_links() {
        assert_eq!(
            linked_note("inkdrop://note/abc123"),
            Some("note:abc123".into())
        );
        assert_eq!(
            linked_note("inkdrop://note:abc123"),
            Some("note:abc123".into())
        );
        assert_eq!(linked_note("inkdrop://note/"), None);
        assert_eq!(linked_note("https://example.com"), None);
    }

    #[test]
    fn openable_links() {
        assert!(is_openable("https://example.com/a?b#c"));
        assert!(is_openable("HTTP://example.com"));
        assert!(is_openable("mailto:someone@example.com"));
        for url in [
            "file:///etc/passwd",
            "javascript:alert(1)",
            "x-apple.systempreferences:com.apple.preference",
            "smb://server/share",
            "https:",
            "https://",
            "example.com",
        ] {
            assert!(!is_openable(url), "{url}");
        }
    }
}
