//! Exporting the open note as a web page or a PDF, in the preview's theme.
//!
//! The page is `kanso_render::html::page`: one self-contained file, with
//! math, diagrams and drawings inline as SVG and attached images as data
//! URIs, and no scripts. On macOS a PDF is made from that page by WebKit
//! (WKWebView's createPDF), offscreen; elsewhere the page opens in the
//! browser, to print to PDF from there.

#[cfg(not(target_os = "macos"))]
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use gpui_kit::component::WindowExt as _;
use gpui_kit::component::notification::Notification;
use gpui_kit::*;
use kanso_render::Palette;
use kanso_store::Store;

use crate::export::sanitize_file_name;

/// The page for a note: its title and body, in `palette`, with its
/// attachments read from `store`. Blocks while math and diagrams are drawn.
pub fn page(store: &Store, title: &str, body: &str, palette: &Palette) -> String {
    let attachments = |id: &str| {
        store
            .attachment(id)
            .ok()
            .flatten()
            .map(|(content_type, attachment)| (content_type, attachment.data))
    };
    kanso_render::html::page(title, body, palette, &attachments)
}

/// A file name for the note's export. Pure.
pub fn file_name(title: &str, extension: &str) -> String {
    format!("{}.{extension}", sanitize_file_name(title))
}

fn documents() -> PathBuf {
    let home = crate::paths::home();
    let documents = home.join("Documents");
    if documents.is_dir() { documents } else { home }
}

fn tell(window: &mut Window, cx: &mut App, notification: Notification) {
    window.push_notification(notification, cx);
}

/// Asks where to save the page, then writes it there and shows it.
pub fn export_html(
    store: Arc<Store>,
    title: String,
    body: String,
    window: &mut Window,
    cx: &mut App,
) {
    let palette = crate::preview_rich::palette(cx);
    let picked = cx.prompt_for_new_path(&documents(), Some(&file_name(&title, "html")));
    window
        .spawn(cx, async move |cx| {
            let Ok(Ok(Some(path))) = picked.await else {
                return;
            };
            let written = cx
                .background_executor()
                .spawn({
                    let path = path.clone();
                    async move { std::fs::write(&path, page(&store, &title, &body, &palette)) }
                })
                .await;
            cx.update(|window, cx| match written {
                Ok(()) => cx.reveal_path(&path),
                Err(err) => tell(
                    window,
                    cx,
                    Notification::error(format!("Couldn't save the page: {err}")),
                ),
            })
            .ok();
        })
        .detach();
}

/// Makes a PDF of the page: on macOS saved where asked, by WebKit; elsewhere
/// the page opens in the browser, to print to PDF.
pub fn export_pdf(
    store: Arc<Store>,
    title: String,
    body: String,
    window: &mut Window,
    cx: &mut App,
) {
    let palette = crate::preview_rich::palette(cx);
    #[cfg(target_os = "macos")]
    {
        let picked = cx.prompt_for_new_path(&documents(), Some(&file_name(&title, "pdf")));
        window
            .spawn(cx, async move |cx| {
                let Ok(Ok(Some(path))) = picked.await else {
                    return;
                };
                let html = cx
                    .background_executor()
                    .spawn(async move { page(&store, &title, &body, &palette) })
                    .await;
                let Ok(view) = cx.update(|_, _| mac::load(&html)) else {
                    return;
                };
                let result = match view {
                    Ok(view) => {
                        // WebKit lays the page out on this thread: look in
                        // on it every 50 ms, for up to 30 s.
                        let mut loaded = false;
                        for _ in 0..600 {
                            cx.background_executor()
                                .timer(std::time::Duration::from_millis(50))
                                .await;
                            if !cx.update(|_, _| mac::is_loading(&view)).unwrap_or(false) {
                                loaded = true;
                                break;
                            }
                        }
                        if loaded {
                            let (done, finished) = async_channel::bounded(1);
                            cx.update(|_, _| mac::create_pdf(&view, path.clone(), done))
                                .ok();
                            let result = finished
                                .recv()
                                .await
                                .unwrap_or_else(|_| Err("WebKit stopped".into()));
                            drop(view);
                            result
                        } else {
                            Err("the page took too long to lay out".into())
                        }
                    }
                    Err(err) => Err(err),
                };
                cx.update(|window, cx| match result {
                    Ok(()) => cx.reveal_path(&path),
                    Err(err) => tell(
                        window,
                        cx,
                        Notification::error(format!("Couldn't make the PDF: {err}")),
                    ),
                })
                .ok();
            })
            .detach();
    }
    #[cfg(not(target_os = "macos"))]
    {
        window
            .spawn(cx, async move |cx| {
                let written = cx
                    .background_executor()
                    .spawn(async move {
                        let dir = std::env::temp_dir().join("kanso-export");
                        std::fs::create_dir_all(&dir)?;
                        let path = dir.join(file_name(&title, "html"));
                        std::fs::write(&path, page(&store, &title, &body, &palette))?;
                        Ok::<_, std::io::Error>(path)
                    })
                    .await;
                cx.update(|window, cx| match written {
                    Ok(path) => {
                        open_in_browser(&path, cx);
                        tell(
                            window,
                            cx,
                            Notification::info(
                                "The note opened in your browser. Print it and choose Save as PDF.",
                            ),
                        );
                    }
                    Err(err) => tell(
                        window,
                        cx,
                        Notification::error(format!("Couldn't make the page: {err}")),
                    ),
                })
                .ok();
            })
            .detach();
    }
}

#[cfg(not(target_os = "macos"))]
fn open_in_browser(path: &Path, cx: &mut App) {
    cx.open_with_system(path);
}

#[cfg(target_os = "macos")]
mod mac {
    //! WebKit, offscreen: a WKWebView loads the page, and once it has laid
    //! it out, createPDF gives the PDF. All on the main thread, as WebKit
    //! requires.

    use std::path::PathBuf;

    use block2::RcBlock;
    use objc2::MainThreadMarker;
    use objc2::rc::Retained;
    use objc2_foundation::{NSData, NSError, NSPoint, NSRect, NSSize, NSString};
    use objc2_web_kit::{WKWebView, WKWebViewConfiguration};

    /// The page's width in points: A4 at 96 points an inch, roughly.
    const PAGE_WIDTH: f64 = 794.;

    /// A web view, offscreen, loading `html`.
    pub fn load(html: &str) -> Result<Retained<WKWebView>, String> {
        let mtm = MainThreadMarker::new().ok_or("not on the main thread")?;
        // SAFETY: on the main thread (checked above); the configuration is
        // copied by the view, which the caller keeps.
        unsafe {
            let configuration = WKWebViewConfiguration::new(mtm);
            let frame = NSRect::new(NSPoint::new(0., 0.), NSSize::new(PAGE_WIDTH, 1123.));
            let view = WKWebView::initWithFrame_configuration(
                mtm.alloc::<WKWebView>(),
                frame,
                &configuration,
            );
            view.loadHTMLString_baseURL(&NSString::from_str(html), None);
            Ok(view)
        }
    }

    pub fn is_loading(view: &WKWebView) -> bool {
        // SAFETY: a property read, on the main thread (GPUI's foreground).
        unsafe { view.isLoading() }
    }

    /// Asks WebKit for the PDF of the whole page and writes it to `path`;
    /// `done` hears how it went.
    pub fn create_pdf(
        view: &WKWebView,
        path: PathBuf,
        done: async_channel::Sender<Result<(), String>>,
    ) {
        let handler = RcBlock::new(move |data: *mut NSData, error: *mut NSError| {
            // SAFETY: WebKit passes a valid NSData or NSError, or null.
            let result = unsafe {
                match (data.as_ref(), error.as_ref()) {
                    (Some(data), _) => {
                        std::fs::write(&path, data.to_vec()).map_err(|err| err.to_string())
                    }
                    (None, Some(error)) => Err(error.localizedDescription().to_string()),
                    (None, None) => Err("WebKit made no PDF".into()),
                }
            };
            done.try_send(result).ok();
        });
        // SAFETY: on the main thread; WebKit retains the handler until it
        // has called it.
        unsafe { view.createPDFWithConfiguration_completionHandler(None, &handler) };
    }
}

#[cfg(test)]
mod tests {
    use super::{Palette, Store, file_name, page};
    use proptest::prelude::*;

    #[test]
    fn a_note_exports_with_its_images() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("k.db")).unwrap();
        let png = b"\x89PNG\r\n\x1a\nnot really".to_vec();
        let id = crate::smart_paste::attach_image(&store, &png, "png", Some("cat.png")).unwrap();
        let body = format!("Area $\\pi r^2$.\n\n![cat](inkdrop://{id})\n");
        let html = page(&store, "Circles", &body, &Palette::DARK);
        assert!(html.contains("<title>Circles</title>"));
        assert!(html.contains("src=\"data:image/png;base64,"));
        assert!(html.contains("kanso-math"));
        assert!(!html.contains("<script"));
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(128))]

        /// Invariant: an export's file name is safe on every platform and
        /// ends in its extension.
        #[test]
        fn prop_file_names_are_safe(title in "\\PC{0,80}") {
            let name = file_name(&title, "html");
            prop_assert!(name.ends_with(".html"));
            prop_assert!(!name.contains(['/', '\\', ':', '*', '?', '"', '<', '>', '|']));
        }
    }
}
