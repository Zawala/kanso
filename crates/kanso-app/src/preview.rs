//! The rendered note. It is its own view so the editor can cache it:
//! re-laying out a large note on every keystroke frame doubles edit latency.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use gpui_kit::base::text::TextView;
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::*;
use kanso_store::Store;

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

impl Render for Preview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let links = self.links.clone();
        div()
            .id("preview")
            .size_full()
            .px_6()
            .py_4()
            .overflow_y_scroll()
            .track_scroll(&self.scroll)
            .bg(cx.theme().background)
            .child(
                TextView::markdown("preview-text", self.text.clone())
                    .selectable(true)
                    .image_source(self.image_resolver())
                    .on_link_click(move |url, _, _, cx| match linked_note(url) {
                        Some(id) => {
                            links.try_send(id).ok();
                        }
                        None => cx.open_url(url),
                    }),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::linked_note;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Round-trip: a note link built from an ID resolves to that ID.
        #[test]
        fn prop_note_links_resolve_to_their_id(suffix in "[A-Za-z0-9_-]{1,20}", tail in prop::option::of("[?#/][a-z=]{0,8}")) {
            let url = format!("inkdrop://note/{suffix}{}", tail.unwrap_or_default());
            prop_assert_eq!(linked_note(&url), Some(format!("note:{suffix}")));
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
}
