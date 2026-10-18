//! The rendered note. It is its own view so the editor can cache it:
//! re-laying out a large note on every keystroke frame doubles edit latency.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use gpui_kit::base::text::{TextView, TextViewStyle};
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::*;
use kanso_store::Store;

use crate::theme::Colors;

/// The rendered text's size and line height.
const TEXT_SIZE: f32 = 16.;
const LINE_HEIGHT: f32 = 1.65;
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

pub struct Preview {
    pub text: SharedString,
    store: Arc<Store>,
    images: Arc<Mutex<HashMap<String, Option<Arc<Image>>>>>,
    links: LinkSender,
    scroll: ScrollHandle,
}

impl Preview {
    pub fn new(store: Arc<Store>, links: LinkSender) -> Self {
        Preview {
            text: SharedString::default(),
            store,
            images: Arc::default(),
            links,
            scroll: ScrollHandle::new(),
        }
    }

    pub fn set_text(&mut self, text: SharedString, cx: &mut Context<Self>) {
        if self.text != text {
            self.text = text;
            cx.notify();
        }
    }

    /// Resolves `inkdrop://file:<id>` from the store, caching decoded images.
    fn image_resolver(&self) -> impl Fn(&SharedUri) -> ImageSource + Send + Sync + 'static {
        let store = self.store.clone();
        let images = self.images.clone();
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

/// Body text in the theme's colors; code is set apart by a quiet fill rather
/// than by color, and so is a table's header row, which is also set heavier.
/// Table rows sit on the page itself; a table wider than the column scrolls
/// sideways rather than cutting off its last columns. Quotes and finished
/// tasks, which the text view draws in its muted color, get the secondary
/// text color: they are still read. `dark` picks the check mark that reads
/// on the dark theme's task boxes.
// no-prop-test: maps theme colors onto a style; the snapshots show it.
fn text_style(colors: &Colors, dark: bool) -> TextViewStyle {
    let mut table = StyleRefinement::default().bg(colors.editor);
    table.overflow.x = Some(Overflow::Scroll);
    TextViewStyle::default()
        .with_dark(dark)
        .with_foreground(colors.text.into())
        .with_muted_foreground(colors.text_secondary.into())
        .with_link(colors.link.into())
        .with_selection(Hsla::from(colors.accent).opacity(0.3))
        .with_code_background(colors.code.into())
        .with_border(colors.divider.into())
        .with_paragraph_gap(rems(0.9))
        .with_code_block(StyleRefinement::default().rounded(px(6.)))
        .with_inline_code(HighlightStyle {
            background_color: Some(colors.code.into()),
            ..Default::default()
        })
        .with_table(table)
        .with_table_head(StyleRefinement::default().font_weight(FontWeight::SEMIBOLD))
}

impl Render for Preview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let links = self.links.clone();
        let colors = Colors::get(cx);
        let dark = cx.theme().is_dark();
        div()
            .id("preview")
            .size_full()
            .pl(px(TEXT_INSET))
            .pr_8()
            .py_6()
            .overflow_y_scroll()
            .track_scroll(&self.scroll)
            .bg(colors.editor)
            .child(
                div()
                    .w_full()
                    .max_w(px(readable_width(TEXT_SIZE, false)))
                    .child(
                        TextView::markdown("preview-text", self.text.clone())
                            .style(text_style(colors, dark))
                            .text_size(px(TEXT_SIZE))
                            .line_height(relative(LINE_HEIGHT))
                            .text_color(colors.text)
                            .selectable(true)
                            .image_source(self.image_resolver())
                            .on_link_click(move |url, _, _, cx| match linked_note(url) {
                                Some(id) => {
                                    links.try_send(id).ok();
                                }
                                None if is_openable(url) => cx.open_url(url),
                                None => {}
                            }),
                    ),
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
