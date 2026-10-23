//! Exporting the open note as a web page or a PDF, in the preview's theme.
//!
//! The page is `kanso_render::html::page`: one self-contained file, with
//! math, diagrams and drawings inline as SVG and attached images as data
//! URIs, and no scripts. On macOS a PDF is printed from that page by
//! WebKit, offscreen, in pages; elsewhere the page opens in the browser, to
//! print to PDF from there.

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

/// The page for a note: its title and body, in `palette` and `style`, with
/// its attachments read from `store` and its query views drawn. Blocks while
/// math and diagrams are drawn.
pub fn page(
    store: &Store,
    title: &str,
    body: &str,
    palette: &Palette,
    self_id: Option<&str>,
    style: &crate::export_style::PageStyle,
) -> String {
    let body = with_queries(store, body, self_id);
    page_without_queries(store, title, &body, palette, style)
}

/// The page as it was before query views: a `query` fence stays code.
pub fn page_without_queries(
    store: &Store,
    title: &str,
    body: &str,
    palette: &Palette,
    style: &crate::export_style::PageStyle,
) -> String {
    let attachments = |id: &str| {
        store
            .attachment(id)
            .ok()
            .flatten()
            .map(|(content_type, attachment)| (content_type, attachment.data))
    };
    kanso_render::html::page_with(title, body, palette, &attachments, &style.options())
}

/// `body` with each query fence drawn as the table it makes.
fn with_queries(store: &Store, body: &str, self_id: Option<&str>) -> String {
    let Ok(rows) = store.query_view_notes() else {
        return body.to_owned();
    };
    let notes: Vec<kanso_core::query_view::ViewNote> = rows
        .into_iter()
        .map(kanso_core::query_view::ViewNote::from)
        .collect();
    kanso_core::query_view::render_export(body, &notes, self_id)
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
    self_id: Option<String>,
    window: &mut Window,
    cx: &mut App,
) {
    let palette = crate::preview_rich::palette(cx);
    let style = crate::export_style::PageStyle::from_cx(cx);
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
                    let self_id = self_id.clone();
                    let style = style.clone();
                    async move {
                        std::fs::write(
                            &path,
                            page(&store, &title, &body, &palette, self_id.as_deref(), &style),
                        )
                    }
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
///
/// Paper is printed on white, so a dark preview theme exports in Kanso's
/// light colors: printing leaves out backgrounds, and light text would be
/// lost on the page.
pub fn export_pdf(
    store: Arc<Store>,
    title: String,
    body: String,
    self_id: Option<String>,
    window: &mut Window,
    cx: &mut App,
) {
    let palette = print_palette(crate::preview_rich::palette(cx));
    let style = crate::export_style::PageStyle::from_cx(cx);
    #[cfg(target_os = "macos")]
    {
        let picked = cx.prompt_for_new_path(&documents(), Some(&file_name(&title, "pdf")));
        window
            .spawn(cx, async move |cx| {
                let Ok(Ok(Some(path))) = picked.await else {
                    return;
                };
                let html =
                    cx.background_executor()
                        .spawn({
                            let self_id = self_id.clone();
                            let style = style.clone();
                            async move {
                                page(&store, &title, &body, &palette, self_id.as_deref(), &style)
                            }
                        })
                        .await;
                let result = mac_pdf(&html, &path, cx).await;
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
                        std::fs::write(
                            &path,
                            page(&store, &title, &body, &palette, self_id.as_deref(), &style),
                        )?;
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

/// The colors to print in: `palette` when it is light, else Kanso's light
/// theme. Pure.
pub(crate) fn print_palette(palette: Palette) -> Palette {
    if palette.dark {
        Palette::LIGHT
    } else {
        palette
    }
}

/// How often, and how many times, to look in on WebKit: every 50 ms, for up
/// to a minute.
#[cfg(target_os = "macos")]
const POLL: std::time::Duration = std::time::Duration::from_millis(50);
#[cfg(target_os = "macos")]
const POLLS: usize = 1200;

/// Prints `html` to a paged PDF at `path`, on the user's default paper:
/// WebKit's print operation, run offscreen with no panels. If printing
/// fails, the PDF is made by createPDF instead, as one long page, so there
/// is still a PDF.
#[cfg(target_os = "macos")]
async fn mac_pdf(
    html: &str,
    path: &std::path::Path,
    cx: &mut AsyncWindowContext,
) -> Result<(), String> {
    let closed = |_| "the window closed".to_string();
    let page = cx.update(|_, _| mac::Page::load(html)).map_err(closed)??;
    let mut loaded = false;
    for _ in 0..POLLS {
        cx.background_executor().timer(POLL).await;
        if !cx.update(|_, _| page.is_loading()).unwrap_or(false) {
            loaded = true;
            break;
        }
    }
    if !loaded {
        return Err("the page took too long to lay out".into());
    }
    // In pages, by printing.
    std::fs::remove_file(path).ok();
    let (done, finished) = async_channel::bounded(1);
    let printed = match cx.update(|_, _| page.print(path, done)).map_err(closed)? {
        Ok(()) => {
            let mut success = false;
            for _ in 0..POLLS {
                if let Ok(answer) = finished.try_recv() {
                    success = answer;
                    break;
                }
                cx.background_executor().timer(POLL).await;
            }
            success && std::fs::metadata(path).is_ok_and(|m| m.len() > 0)
        }
        Err(err) => {
            eprintln!("kanso: couldn't print the PDF: {err}");
            false
        }
    };
    if printed {
        return Ok(());
    }
    // One long page, rather than none.
    eprintln!("kanso: printing the PDF failed; making it as one page");
    let (done, finished) = async_channel::bounded(1);
    cx.update(|_, _| page.create_pdf(path.to_path_buf(), done))
        .map_err(closed)?;
    finished
        .recv()
        .await
        .unwrap_or_else(|_| Err("WebKit stopped".into()))
}

#[cfg(not(target_os = "macos"))]
fn open_in_browser(path: &Path, cx: &mut App) {
    cx.open_with_system(path);
}

#[cfg(target_os = "macos")]
mod mac;

#[cfg(test)]
mod tests {
    use super::{Palette, Store, file_name, page, page_without_queries, print_palette};
    use proptest::prelude::*;

    #[test]
    fn a_note_exports_with_its_images() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("k.db")).unwrap();
        let png = b"\x89PNG\r\n\x1a\nnot really".to_vec();
        let id = crate::smart_paste::attach_image(&store, &png, "png", Some("cat.png")).unwrap();
        let body = format!("Area $\\pi r^2$.\n\n![cat](inkdrop://{id})\n");
        let html = page(
            &store,
            "Circles",
            &body,
            &Palette::DARK,
            None,
            &crate::export_style::PageStyle::default(),
        );
        assert!(html.contains("<title>Circles</title>"));
        assert!(html.contains("src=\"data:image/png;base64,"));
        assert!(html.contains("kanso-math"));
        assert!(!html.contains("<script"));
    }

    #[test]
    fn a_page_draws_query_views() {
        use kanso_core::DocType;
        use kanso_store::{Origin, Put};
        use serde_json::json;

        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("q.db")).unwrap();
        let book = store
            .put(
                DocType::Book,
                json!({"name": "Clients"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        let note = |title: &str, body: &str| {
            store
                .put(
                    DocType::Note,
                    json!({"title": title, "body": body, "bookId": book}),
                    Put::Upsert,
                    Origin::Ui,
                )
                .unwrap()
                .id
        };
        note("Ana", "---\nclient: Acme\n---\n# Ana\n");
        // Sealed by hand: the store won't take text beside a lock.
        store
            .put(
                DocType::Note,
                json!({"_id": "note:sealed", "title": "Sealed", "body": "",
                       "bookId": book, "kansoLock": {"salt": "s", "nonce": "n", "data": "d"}}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
        let self_id = note("Self", "```query\nfrom: has:client\ncolumns: client\n```\n");
        let body = "```query\nfrom: has:client\ncolumns: client\n```\n";
        let style = crate::export_style::PageStyle::default();
        let html = page(&store, "Self", body, &Palette::DARK, Some(&self_id), &style);
        assert!(html.contains("<table>"), "{html}");
        assert!(html.contains("Ana"), "{html}");
        assert!(html.contains("Acme"), "{html}");
        assert!(!html.contains("Sealed"), "{html}");
        assert!(!html.contains("Self</td>"), "{html}");
        assert!(
            page_without_queries(&store, "Self", body, &Palette::DARK, &style)
                .contains("language-query")
        );
    }

    #[test]
    fn pdfs_print_in_light_colors() {
        assert_eq!(print_palette(Palette::DARK), Palette::LIGHT);
        assert_eq!(print_palette(Palette::LIGHT), Palette::LIGHT);
        let sepia = Palette {
            background: kanso_render::Rgb(0xF4ECD8),
            ..Palette::LIGHT
        };
        assert_eq!(print_palette(sepia), sepia);
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
