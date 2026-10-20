//! Quick capture: a small window over every other app for jotting a note
//! down, opened by the global shortcut (`hotkey`) or the Quick Capture
//! command. It has a Markdown text area and a notebook picker that starts
//! at the Inbox. ⌘↩ saves the note and closes it; Escape closes it, asking
//! first when there is text.
//!
//! On macOS it is a panel that takes the keyboard without bringing Kanso
//! forward, so closing it leaves you in the app you were in; it works with
//! the main window closed. On Windows it is a topmost tool window, and on
//! Linux an ordinary small window (X11 gives pop-ups no keyboard).
//!
//! The note's title is its first line ([`split_capture`]).

use std::sync::Arc;

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{Editor, EditorState, InputEvent};
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenuItem};
use gpui_kit::component::{ActiveTheme as _, Icon, Sizable as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_core::DocType;
use kanso_core::roles::Role;
use kanso_store::{Origin, Put, Store};
use serde_json::json;

use crate::config::ViewMode;
use crate::theme::Colors;
use crate::workspace::AppState;

actions!(kanso, [SaveCapture, CancelCapture]);

/// The window's key context.
pub const CONTEXT: &str = "QuickCapture";
/// The window's size in points.
pub const WIDTH: f32 = 560.;
pub const HEIGHT: f32 = 300.;
/// A first line longer than this is cut for the title.
const TITLE_WORDS_LEN: usize = 80;

pub fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        // Over the text area's own ⌘↩, which would add a line.
        KeyBinding::new("secondary-enter", SaveCapture, Some("QuickCapture > Input")),
        KeyBinding::new("secondary-enter", SaveCapture, Some(CONTEXT)),
        KeyBinding::new("secondary-s", SaveCapture, Some(CONTEXT)),
        KeyBinding::new("escape", CancelCapture, Some(CONTEXT)),
    ]);
    // From the palette and the File menu too, though the system-wide
    // shortcut is the usual way in.
    cx.on_action(|_: &crate::journal::QuickCapture, cx| open(cx));
}

/// A capture's title and body. A heading or a short first line is the
/// title, and the lines after it the body. A long first line, or one that
/// starts a list or a quote, is cut short for the title and the body keeps
/// all of the text, so nothing typed is lost. `None` for text that is only
/// spaces. Pure.
pub fn split_capture(text: &str) -> Option<(String, String)> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }
    let (first, rest) = text.split_once('\n').unwrap_or((text, ""));
    let first = first.trim();
    let rest = without_blank_lead(rest.trim_end());
    let heading = first
        .strip_prefix('#')
        .map(|h| h.trim_start_matches('#'))
        .filter(|h| h.starts_with(' '))
        .map(str::trim)
        .filter(|h| !h.is_empty());
    if let Some(heading) = heading {
        return Some((cut_title(heading), rest.to_owned()));
    }
    let marker = first.starts_with(['-', '*', '>', '+', '|', '`', '#']);
    if !marker && first.chars().count() <= TITLE_WORDS_LEN {
        return Some((first.to_owned(), rest.to_owned()));
    }
    let plain = first.trim_start_matches(['-', '*', '>', '+', '|', '`', '#', ' ']);
    let plain = if plain.is_empty() { first } else { plain };
    Some((cut_title(plain), text.to_owned()))
}

/// `text` from its first line with more than spaces in it.
fn without_blank_lead(mut text: &str) -> &str {
    while let Some((line, rest)) = text.split_once('\n') {
        if !line.trim().is_empty() {
            break;
        }
        text = rest;
    }
    if text.trim().is_empty() { "" } else { text }
}

/// At most [`TITLE_WORDS_LEN`] characters, cut at a space, with "…" when
/// cut.
fn cut_title(line: &str) -> String {
    if line.chars().count() <= TITLE_WORDS_LEN {
        return line.to_owned();
    }
    let cut: String = line.chars().take(TITLE_WORDS_LEN).collect();
    let cut = match cut.rfind(' ') {
        Some(at) if at > TITLE_WORDS_LEN / 2 => &cut[..at],
        _ => cut.as_str(),
    };
    format!("{}…", cut.trim_end())
}

/// Saves `text` as a note in `book`, or the Inbox (made if needed).
/// Returns the note's ID, or `None` for empty text.
pub fn save_capture(
    store: &Store,
    text: &str,
    book: Option<&str>,
    origin: Origin,
) -> anyhow::Result<Option<String>> {
    let Some((title, body)) = split_capture(text) else {
        return Ok(None);
    };
    let book = match book {
        Some(book) if store.get(book)?.is_some() => book.to_owned(),
        _ => store.role_book(Role::Inbox, origin)?,
    };
    let created = store.put(
        DocType::Note,
        json!({"title": title, "body": body, "bookId": book}),
        Put::Upsert,
        origin,
    )?;
    Ok(Some(created.id))
}

pub struct CaptureView {
    store: Arc<Store>,
    body: Entity<EditorState>,
    /// The notebooks to choose from, `(id, name)`, the Inbox first.
    books: Vec<(String, String)>,
    /// `None`: the Inbox.
    book: Option<String>,
    /// Escape was pressed over text: asking whether to discard it.
    confirming: bool,
    error: Option<String>,
    focus: FocusHandle,
    _subscriptions: Vec<Subscription>,
}

impl CaptureView {
    pub fn new(store: Arc<Store>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let body = cx.new(|cx| {
            EditorState::new(window, cx)
                .language("markdown")
                .line_number(false)
                .folding(false)
                .indent_guides(false)
                .soft_wrap(true)
                .auto_close(false)
                .placeholder("Jot something down…")
        });
        let inbox = store.find_role_book(Role::Inbox).ok().flatten();
        let mut inbox_name = Role::Inbox.default_name().to_owned();
        let mut books: Vec<(String, String)> = Vec::new();
        for book in store.books().unwrap_or_default() {
            if Some(&book.id) == inbox.as_ref() {
                inbox_name = book.name;
            } else {
                books.push((book.id, book.name));
            }
        }
        books.sort_by_key(|(_, name)| name.to_lowercase());
        books.insert(0, (Role::Inbox.book_id().to_owned(), inbox_name));
        let subscriptions = vec![
            cx.subscribe_in(&body, window, |this, _, event, window, cx| match event {
                InputEvent::Change => {
                    if this.confirming || this.error.is_some() {
                        this.confirming = false;
                        this.error = None;
                        cx.notify();
                    }
                }
                InputEvent::PressEnter {
                    secondary: true, ..
                } => this.save(window, cx),
                _ => {}
            }),
        ];
        body.update(cx, |body, cx| body.focus(window, cx));
        CaptureView {
            store,
            body,
            books,
            book: None,
            confirming: false,
            error: None,
            focus: cx.focus_handle(),
            _subscriptions: subscriptions,
        }
    }

    pub fn text(&self, cx: &App) -> String {
        self.body.read(cx).value().to_string()
    }

    #[cfg(test)]
    pub fn set_text_for_test(&mut self, text: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.body
            .update(cx, |body, cx| body.set_value(text.to_owned(), window, cx));
    }

    #[cfg(test)]
    pub fn is_confirming(&self) -> bool {
        self.confirming
    }

    fn book_name(&self) -> &str {
        let id = self.book.as_deref().unwrap_or(Role::Inbox.book_id());
        self.books
            .iter()
            .find(|(book, _)| book == id)
            .map_or(Role::Inbox.default_name(), |(_, name)| name.as_str())
    }

    pub fn set_book(&mut self, book: Option<String>, cx: &mut Context<Self>) {
        self.book = book.filter(|id| id != Role::Inbox.book_id());
        cx.notify();
    }

    pub fn save(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let text = self.text(cx);
        match save_capture(&self.store, &text, self.book.as_deref(), Origin::Ui) {
            Ok(_) => window.remove_window(),
            Err(err) => {
                self.error = Some(format!("Couldn't save the note: {err:#}"));
                cx.notify();
            }
        }
    }

    /// Escape: closes an empty capture; over text, asks first, and a second
    /// Escape discards it.
    pub fn cancel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.text(cx).trim().is_empty() || self.confirming {
            window.remove_window();
        } else {
            self.confirming = true;
            cx.notify();
        }
    }

    fn keep_writing(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.confirming = false;
        self.body.update(cx, |body, cx| body.focus(window, cx));
        cx.notify();
    }

    fn render_header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let view = cx.entity();
        let books = self.books.clone();
        let current = self
            .book
            .clone()
            .unwrap_or_else(|| Role::Inbox.book_id().into());
        h_flex()
            .flex_none()
            .h(px(40.))
            .px(px(14.))
            .gap(px(8.))
            .child(
                Icon::new(Lucide::SquarePen)
                    .size(px(15.))
                    .text_color(colors.text_muted),
            )
            .child(
                div()
                    .flex_1()
                    .text_size(px(12.5))
                    .text_color(colors.text_muted)
                    .child("Quick note"),
            )
            .child(
                Button::new("capture-book")
                    .ghost()
                    .small()
                    .debug_selector(|| "capture-book".into())
                    .icon(Icon::new(Lucide::Inbox).text_color(colors.text_secondary))
                    .child(
                        div()
                            .text_size(px(12.5))
                            .text_color(colors.text_secondary)
                            .child(self.book_name().to_owned()),
                    )
                    .dropdown_caret(true)
                    .tooltip("Save in notebook")
                    .dropdown_menu(move |mut menu, _, _| {
                        for (ix, (id, name)) in books.iter().enumerate() {
                            if ix == 1 {
                                menu = menu.separator();
                            }
                            let view = view.clone();
                            let id = id.clone();
                            menu = menu.item(
                                PopupMenuItem::new(name.clone())
                                    .checked(id == current)
                                    .on_click(move |_, _, cx| {
                                        let id = id.clone();
                                        view.update(cx, |this, cx| this.set_book(Some(id), cx));
                                    }),
                            );
                        }
                        menu
                    }),
            )
    }

    fn render_footer(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let hint = |keys: &'static str, label: &'static str| {
            h_flex()
                .gap(px(5.))
                .child(
                    div()
                        .px(px(5.))
                        .py(px(1.))
                        .rounded(px(4.))
                        .bg(colors.fill)
                        .text_size(px(11.))
                        .text_color(colors.text_secondary)
                        .child(keys),
                )
                .child(
                    div()
                        .text_size(px(12.))
                        .text_color(colors.text_muted)
                        .child(label),
                )
        };
        let save_keys = if cfg!(target_os = "macos") {
            "⌘↩"
        } else {
            "Ctrl+Enter"
        };
        let bar = h_flex()
            .flex_none()
            .h(px(46.))
            .px(px(14.))
            .gap(px(14.))
            .border_t_1()
            .border_color(colors.divider);
        if self.confirming {
            return bar
                .child(
                    div()
                        .flex_1()
                        .text_size(px(13.))
                        .text_color(colors.text)
                        .child("Discard this note?"),
                )
                .child(
                    Button::new("capture-keep")
                        .ghost()
                        .small()
                        .label("Keep writing")
                        .on_click(cx.listener(|this, _, window, cx| this.keep_writing(window, cx))),
                )
                .child(
                    Button::new("capture-discard")
                        .danger()
                        .small()
                        .label("Discard")
                        .on_click(cx.listener(|_, _, window, _| window.remove_window())),
                );
        }
        bar.child(hint(save_keys, "Save"))
            .child(hint("esc", "Cancel"))
            .child(div().flex_1())
            .when_some(self.error.clone(), |el, error| {
                el.child(
                    div()
                        .min_w_0()
                        .truncate()
                        .text_size(px(12.))
                        .text_color(colors.danger)
                        .child(error),
                )
            })
            .child(
                Button::new("capture-save")
                    .primary()
                    .small()
                    .label("Save")
                    .debug_selector(|| "capture-save".into())
                    .on_click(cx.listener(|this, _, window, cx| this.save(window, cx))),
            )
    }
}

impl Focusable for CaptureView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for CaptureView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        // The editor's writing face and size, as Settings › Editor has them,
        // for a note being written on its own.
        let appearance = &cx.global::<AppState>().config.appearance;
        let size = appearance.editor_font_size;
        let family = if appearance.editor_font.face(ViewMode::Edit).mono {
            cx.theme().mono_font_family.clone()
        } else {
            cx.theme().font_family.clone()
        };
        v_flex()
            .size_full()
            .key_context(CONTEXT)
            .track_focus(&self.focus)
            .bg(colors.editor)
            .text_color(colors.text)
            .on_action(cx.listener(|this, _: &SaveCapture, window, cx| this.save(window, cx)))
            .on_action(cx.listener(|this, _: &CancelCapture, window, cx| this.cancel(window, cx)))
            .child(self.render_header(cx))
            .child(
                div().flex_1().min_h_0().px(px(6.)).child(
                    Editor::new(&self.body)
                        .bordered(false)
                        .h_full()
                        .font_family(family)
                        .text_size(px(size))
                        .line_height(relative(crate::editor::BODY_LINE_HEIGHT)),
                ),
            )
            .child(self.render_footer(cx))
    }
}

/// Opens the capture window, or brings the open one forward.
pub fn open(cx: &mut App) {
    if focus_existing(cx) {
        return;
    }
    let Some(store) = cx.try_global::<AppState>().map(|state| state.store.clone()) else {
        return;
    };
    let size = size(px(WIDTH), px(HEIGHT));
    let mut bounds = Bounds::centered(None, size, cx);
    // In the upper part of the screen, where the eye already is.
    bounds.origin.y = (bounds.origin.y - px(120.)).max(px(40.));
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    let kind = WindowKind::PopUp;
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let kind = WindowKind::Normal;
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: (!cfg!(target_os = "macos")).then(|| TitlebarOptions {
            title: Some("Quick Note".into()),
            appears_transparent: false,
            traffic_light_position: None,
        }),
        kind,
        is_resizable: false,
        is_minimizable: false,
        focus: true,
        show: true,
        #[cfg(target_os = "linux")]
        app_id: Some(crate::flavour::CURRENT.xdg_name.into()),
        ..Default::default()
    };
    let opened = gpui_kit::open_window(options, cx, move |window, cx| {
        crate::themes::apply(Some(window), cx);
        cx.new(|cx| CaptureView::new(store, window, cx))
    });
    if let Err(err) = opened {
        eprintln!("kanso: couldn't open the quick capture window: {err:#}");
    }
}

/// Brings an open capture window forward with the keyboard in it.
fn focus_existing(cx: &mut App) -> bool {
    for handle in cx.windows() {
        let found = handle
            .update(cx, |root, window, cx| {
                let Ok(root) = root.downcast::<gpui_kit::base::Root>() else {
                    return false;
                };
                let view = root.read(cx).view().clone();
                let Ok(capture) = view.downcast::<CaptureView>() else {
                    return false;
                };
                window.activate_window();
                capture.update(cx, |capture, cx| {
                    capture.body.update(cx, |body, cx| body.focus(window, cx))
                });
                true
            })
            .unwrap_or(false);
        if found {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::{TITLE_WORDS_LEN, save_capture, split_capture};
    use kanso_core::DocType;
    use kanso_core::model::MAX_TITLE_LEN;
    use kanso_store::{Origin, Put, Store};
    use proptest::prelude::*;
    use serde_json::json;

    #[test]
    fn split_examples() {
        let s = |t: &str| split_capture(t);
        assert_eq!(s("  \n "), None);
        assert_eq!(s("Buy milk"), Some(("Buy milk".into(), String::new())));
        assert_eq!(
            s("# Idea\n\nA longer thought.\n"),
            Some(("Idea".into(), "A longer thought.".into()))
        );
        assert_eq!(
            s("Call Sam\nabout the lease"),
            Some(("Call Sam".into(), "about the lease".into()))
        );
        assert_eq!(
            s("- milk\n- eggs"),
            Some(("milk".into(), "- milk\n- eggs".into()))
        );
        let long = "word ".repeat(40);
        let (title, body) = s(&long).unwrap();
        assert!(title.ends_with('…') && title.chars().count() <= TITLE_WORDS_LEN + 1);
        assert_eq!(body, long.trim());
        let (title, body) = s(&format!("{long}\nmore")).unwrap();
        assert!(title.ends_with('…'));
        assert!(body.starts_with("word word"));
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(512))]

        /// Invariants: nothing typed is lost (every word is in the title or
        /// the body), the title is one line a note can take, and a body
        /// never starts with a blank line or ends with spaces.
        #[test]
        fn prop_split_keeps_every_word(text in "[a-z#*> \n-]{0,200}") {
            match split_capture(&text) {
                None => prop_assert!(text.trim().is_empty()),
                Some((title, body)) => {
                    prop_assert!(!title.contains('\n'));
                    prop_assert!(title.chars().count() <= MAX_TITLE_LEN);
                    prop_assert_eq!(body.trim_end(), body.as_str());
                    prop_assert!(body.lines().next().is_none_or(|line| !line.trim().is_empty()));
                    let kept = format!("{title} {body}");
                    for word in text.split_whitespace() {
                        let word = word.trim_matches(|c: char| !c.is_alphanumeric());
                        prop_assert!(word.is_empty() || kept.contains(word), "{:?} lost from {:?}", word, text);
                    }
                }
            }
        }
    }

    /// Saving puts the note in the Inbox, made on first use, or in the
    /// notebook chosen.
    #[test]
    fn captures_go_to_the_inbox() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("kanso.db")).unwrap();
        assert_eq!(save_capture(&store, " \n", None, Origin::Ui).unwrap(), None);
        let id = save_capture(&store, "Buy milk\n- and eggs", None, Origin::Ui)
            .unwrap()
            .unwrap();
        let note = store.note(&id).unwrap().unwrap();
        assert_eq!(
            (
                note.title.as_str(),
                note.body.as_str(),
                note.book_id.as_str()
            ),
            ("Buy milk", "- and eggs", "book:inbox")
        );
        let book = store
            .put(
                DocType::Book,
                json!({"name": "Ideas"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        let id = save_capture(&store, "Kites", Some(&book), Origin::Ui)
            .unwrap()
            .unwrap();
        assert_eq!(store.note(&id).unwrap().unwrap().book_id, book);
        // A notebook deleted meanwhile: the Inbox instead.
        let id = save_capture(&store, "Late", Some("book:gone"), Origin::Ui)
            .unwrap()
            .unwrap();
        assert_eq!(store.note(&id).unwrap().unwrap().book_id, "book:inbox");
    }
}
