//! The note editor: title, metadata toolbar, Markdown body and preview.
//!
//! Edits autosave after a short pause. A change to the open note from
//! elsewhere (the API, sync) reloads it quietly when there are no unsaved
//! edits, and otherwise asks which version to keep. The losing version is
//! kept in the note's history either way.

use std::sync::Arc;
use std::time::Duration;

use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{Editor, EditorState, Input, InputEvent, InputState, Position};
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenuItem};
use gpui_kit::component::{
    ActiveTheme as _, IconName, Selectable as _, Sizable as _, StyledExt as _, h_flex, v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_core::model::TRASH_BOOK_ID;
use kanso_core::{DocType, Note, NoteStatus, now_ms};
use kanso_store::{Change, Origin, Put, Store, StoreError};

use crate::config::ViewMode;
use crate::library::Library;
use crate::preview::{LinkSender, Preview};

const AUTOSAVE_DELAY: Duration = Duration::from_millis(600);
const PREVIEW_DELAY: Duration = Duration::from_millis(150);

pub enum EditorEvent {
    /// Something worth telling the user, such as a failed save.
    Notice(String),
}

pub struct NoteEditor {
    store: Arc<Store>,
    library: Entity<Library>,
    /// The note as last loaded or saved, with its current `_rev`.
    note: Option<Note>,
    title: Entity<InputState>,
    body: Entity<EditorState>,
    preview: Entity<Preview>,
    pub view_mode: ViewMode,
    pub font_size: f32,
    autosave: Option<Task<()>>,
    preview_task: Option<Task<()>>,
    /// A newer version from elsewhere that arrived over unsaved edits.
    incoming: Option<Note>,
    /// Notes that link here: (id, title).
    backlinks: Vec<(String, String)>,
    backlinks_task: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<EditorEvent> for NoteEditor {}

impl NoteEditor {
    pub fn new(
        store: Arc<Store>,
        library: Entity<Library>,
        links: LinkSender,
        view_mode: ViewMode,
        font_size: f32,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let title = cx.new(|cx| InputState::new(window, cx).placeholder("Untitled"));
        let body = cx.new(|cx| {
            EditorState::new(window, cx)
                .language("markdown")
                .line_number(false)
                .folding(false)
                .indent_guides(false)
                .soft_wrap(true)
        });
        let preview = cx.new(|_| Preview::new(store.clone(), links));
        let subscriptions = vec![
            cx.subscribe_in(&title, window, |this, _, event, window, cx| match event {
                InputEvent::Change => this.schedule_autosave(cx),
                InputEvent::PressEnter { .. } => {
                    this.body.update(cx, |body, cx| body.focus(window, cx));
                }
                _ => {}
            }),
            cx.subscribe_in(&body, window, |this, _, event, window, cx| {
                if matches!(event, InputEvent::Change) {
                    this.schedule_autosave(cx);
                    this.schedule_preview(window, cx);
                }
            }),
        ];
        NoteEditor {
            store,
            library,
            note: None,
            title,
            body,
            preview,
            view_mode,
            font_size,
            autosave: None,
            preview_task: None,
            incoming: None,
            backlinks: Vec::new(),
            backlinks_task: None,
            _subscriptions: subscriptions,
        }
    }

    pub fn note_id(&self) -> Option<&str> {
        self.note.as_ref().map(|n| n.id.as_str())
    }

    /// Saves the open note, then opens `id` (or nothing).
    pub fn open(&mut self, id: Option<&str>, window: &mut Window, cx: &mut Context<Self>) {
        if self.note_id() == id && self.incoming.is_none() {
            return;
        }
        self.flush(cx);
        let note = match id.map(|id| self.store.note(id)) {
            Some(Ok(note)) => note,
            Some(Err(err)) => {
                cx.emit(EditorEvent::Notice(format!(
                    "Couldn't open the note: {err}"
                )));
                None
            }
            None => None,
        };
        self.show(note, false, window, cx);
    }

    /// Puts `note` in the editor. With `keep_place`, the cursor and scroll
    /// position survive, for reloading the same note.
    fn show(
        &mut self,
        note: Option<Note>,
        keep_place: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.incoming = None;
        let (title, body) = note.as_ref().map_or((String::new(), String::new()), |n| {
            (n.title.clone(), n.body.clone())
        });
        self.note = note;
        self.title.update(cx, |state, cx| {
            if state.value().as_ref() != title {
                state.set_value(title, window, cx);
            }
        });
        let had_focus = self.body.read(cx).focus_handle(cx).is_focused(window);
        self.body.update(cx, |state, cx| {
            if state.value().as_ref() == body {
                return;
            }
            let cursor = state.cursor_position();
            let scroll = state.scroll_offset();
            state.set_value(body.clone(), window, cx);
            if keep_place {
                if had_focus {
                    let row = cursor
                        .line
                        .min(body.lines().count().saturating_sub(1) as u32);
                    state.set_cursor_position(Position::new(row, cursor.character), window, cx);
                }
                state.set_scroll_offset(scroll, cx);
            }
        });
        let text: SharedString = body.into();
        self.preview
            .update(cx, |preview, cx| preview.set_text(text, cx));
        self.refresh_backlinks(cx);
        cx.notify();
    }

    pub fn focus_body(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.body.update(cx, |body, cx| body.focus(window, cx));
    }

    pub fn focus_title(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.title.update(cx, |title, cx| title.focus(window, cx));
    }

    fn current_text(&self, cx: &App) -> (String, String) {
        (
            self.title.read(cx).value().to_string(),
            self.body.read(cx).value().to_string(),
        )
    }

    pub fn has_unsaved_edits(&self, cx: &App) -> bool {
        self.note.as_ref().is_some_and(|note| {
            let (title, body) = self.current_text(cx);
            title != note.title || body != note.body
        })
    }

    fn schedule_autosave(&mut self, cx: &mut Context<Self>) {
        if self.note.is_none() {
            return;
        }
        self.autosave = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(AUTOSAVE_DELAY).await;
            this.update(cx, |this, cx| {
                this.autosave = None;
                this.save(cx);
            })
            .ok();
        }));
    }

    fn schedule_preview(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.view_mode == ViewMode::Edit {
            return;
        }
        self.preview_task = Some(cx.spawn_in(window, async move |this, cx| {
            cx.background_executor().timer(PREVIEW_DELAY).await;
            this.update(cx, |this, cx| this.refresh_preview(cx)).ok();
        }));
    }

    fn refresh_preview(&mut self, cx: &mut Context<Self>) {
        let text = self.body.read(cx).value();
        self.preview
            .update(cx, |preview, cx| preview.set_text(text, cx));
    }

    /// Saves any pending edits now.
    pub fn flush(&mut self, cx: &mut Context<Self>) {
        self.autosave = None;
        if self.incoming.is_none() {
            self.save(cx);
        }
    }

    /// Writes the title and body if they changed since the last save.
    fn save(&mut self, cx: &mut Context<Self>) {
        if self.incoming.is_some() || !self.has_unsaved_edits(cx) {
            return;
        }
        self.write(|_, _| {}, Origin::Ui, cx);
    }

    /// Writes the open note with the editor's text and `change` applied.
    fn write(
        &mut self,
        change: impl FnOnce(&mut Note, &App),
        origin: Origin,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(mut note) = self.note.clone() else {
            return false;
        };
        let (title, body) = self.current_text(cx);
        note.title = title;
        note.body = body;
        change(&mut note, cx);
        note.updated_at = now_ms();
        let value = match serde_json::to_value(&note) {
            Ok(value) => value,
            Err(err) => {
                cx.emit(EditorEvent::Notice(format!("Couldn't save: {err}")));
                return false;
            }
        };
        match self.store.put(DocType::Note, value, Put::Upsert, origin) {
            Ok(result) => {
                note.rev = Some(result.rev);
                self.note = Some(note);
                cx.notify();
                true
            }
            Err(StoreError::Conflict) => {
                // Someone else saved first; show their version next to ours.
                if let Ok(Some(latest)) = self.store.note(&note.id) {
                    self.incoming = Some(latest);
                    cx.notify();
                }
                false
            }
            Err(err) => {
                cx.emit(EditorEvent::Notice(format!("Couldn't save: {err}")));
                false
            }
        }
    }

    /// Reacts to a write to the open note made outside this editor.
    pub fn on_store_change(
        &mut self,
        change: &Change,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(note) = &self.note else {
            return;
        };
        if change.id != note.id || note.rev.as_deref() == Some(change.rev.as_str()) {
            return;
        }
        if change.deleted {
            self.note = None;
            self.show(None, false, window, cx);
            return;
        }
        let Ok(Some(latest)) = self.store.note(&change.id) else {
            return;
        };
        if self.has_unsaved_edits(cx) {
            self.autosave = None;
            self.incoming = Some(latest);
            cx.notify();
        } else {
            self.show(Some(latest), true, window, cx);
        }
    }

    /// Keeps the editor's text over the incoming version, which stays in history.
    pub fn keep_mine(&mut self, cx: &mut Context<Self>) {
        let Some(theirs) = self.incoming.take() else {
            return;
        };
        let mut note = theirs;
        let (title, body) = self.current_text(cx);
        note.title = title;
        note.body = body;
        self.note = Some(note);
        // An API-origin write always snapshots the version it replaces.
        self.write(|_, _| {}, Origin::Api, cx);
    }

    /// Takes the incoming version; the editor's unsaved text goes to history.
    pub fn take_theirs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(theirs) = self.incoming.take() else {
            return;
        };
        if let Some(mine) = &self.note {
            let (title, body) = self.current_text(cx);
            let mut fields = serde_json::Map::new();
            fields.insert("title".into(), title.into());
            fields.insert("body".into(), body.into());
            fields.insert("bookId".into(), mine.book_id.clone().into());
            let rev = mine.rev.clone().unwrap_or_default();
            if let Err(err) = self
                .store
                .record_revision(&mine.id, &rev, &fields, Origin::Ui)
            {
                cx.emit(EditorEvent::Notice(format!(
                    "Couldn't keep your draft: {err}"
                )));
            }
        }
        self.show(Some(theirs), true, window, cx);
    }

    pub fn set_book(&mut self, book_id: String, cx: &mut Context<Self>) {
        self.write(move |note, _| note.book_id = book_id, Origin::Ui, cx);
    }

    pub fn set_status(&mut self, status: NoteStatus, cx: &mut Context<Self>) {
        self.write(move |note, _| note.status = status, Origin::Ui, cx);
    }

    pub fn toggle_tag(&mut self, tag_id: String, cx: &mut Context<Self>) {
        self.write(
            move |note, _| match note.tags.iter().position(|t| *t == tag_id) {
                Some(ix) => {
                    note.tags.remove(ix);
                }
                None => note.tags.push(tag_id),
            },
            Origin::Ui,
            cx,
        );
    }

    pub fn toggle_pinned(&mut self, cx: &mut Context<Self>) {
        self.write(|note, _| note.pinned = !note.pinned, Origin::Ui, cx);
    }

    /// Moves the open note to the trash.
    pub fn trash(&mut self, cx: &mut Context<Self>) {
        self.set_book(TRASH_BOOK_ID.into(), cx);
    }

    pub fn set_view_mode(&mut self, mode: ViewMode, cx: &mut Context<Self>) {
        self.view_mode = mode;
        if mode != ViewMode::Edit {
            self.refresh_preview(cx);
        }
        cx.notify();
    }

    /// Finds the notes that link to the open one, in the background.
    pub fn refresh_backlinks(&mut self, cx: &mut Context<Self>) {
        let Some(id) = self.note_id().map(str::to_owned) else {
            self.backlinks.clear();
            self.backlinks_task = None;
            return;
        };
        let store = self.store.clone();
        let search = cx
            .background_executor()
            .spawn(async move { backlinks(&store, &id) });
        self.backlinks_task = Some(cx.spawn(async move |this, cx| {
            let found = search.await;
            this.update(cx, |this, cx| {
                if this.backlinks != found {
                    this.backlinks = found;
                    cx.notify();
                }
            })
            .ok();
        }));
    }

    /// Replaces the editor's text with an earlier version, as a normal edit.
    /// The text it replaces is kept in history first.
    pub fn restore(
        &mut self,
        title: String,
        body: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.flush(cx);
        if let Some(note) = &self.note {
            let (current_title, current_body) = self.current_text(cx);
            let mut fields = serde_json::Map::new();
            fields.insert("title".into(), current_title.into());
            fields.insert("body".into(), current_body.into());
            let rev = note.rev.clone().unwrap_or_default();
            self.store
                .record_revision(&note.id, &rev, &fields, Origin::Ui)
                .ok();
        }
        self.title
            .update(cx, |state, cx| state.set_value(title, window, cx));
        self.body
            .update(cx, |state, cx| state.set_value(body.clone(), window, cx));
        self.preview
            .update(cx, |preview, cx| preview.set_text(body.into(), cx));
        self.save(cx);
    }

    fn render_toolbar(&self, note: &Note, cx: &mut Context<Self>) -> impl IntoElement {
        let library = self.library.read(cx);
        let book_name = if note.book_id == TRASH_BOOK_ID {
            "Trash".to_string()
        } else {
            library
                .book(&note.book_id)
                .map_or_else(|| "No notebook".to_string(), |b| b.name.clone())
        };
        let books: Vec<(String, String, usize)> = library
            .book_tree()
            .into_iter()
            .map(|(depth, b)| (b.id.clone(), b.name.clone(), depth))
            .collect();
        let tags: Vec<(String, String, u32)> = library
            .tags
            .iter()
            .map(|t| (t.id.clone(), t.name.clone(), t.color.rgb()))
            .collect();
        let note_tags = note.tags.clone();
        let current_book = note.book_id.clone();
        let status = note.status;
        let view = cx.entity();

        let tag_chips = note_tags.iter().filter_map(|id| {
            let (_, name, rgb) = tags.iter().find(|(tid, _, _)| tid == id)?;
            Some(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .px_2()
                    .py_0p5()
                    .rounded_md()
                    .text_xs()
                    .bg(cx.theme().muted)
                    .child(div().size_2().rounded_full().bg(rgb_color(*rgb)))
                    .child(name.clone()),
            )
        });

        h_flex()
            .gap_1()
            .px_4()
            .py_1()
            .border_b_1()
            .border_color(cx.theme().border)
            .child(
                Button::new("book")
                    .ghost()
                    .small()
                    .icon(IconName::BookOpen)
                    .label(book_name)
                    .dropdown_caret(true)
                    .dropdown_menu({
                        let view = view.clone();
                        move |mut menu, _, _| {
                            for (id, name, depth) in &books {
                                let view = view.clone();
                                let book_id = id.clone();
                                let label = format!("{}{name}", "   ".repeat(*depth));
                                menu = menu.item(
                                    PopupMenuItem::new(label)
                                        .checked(*id == current_book)
                                        .on_click(move |_, _, cx| {
                                            view.update(cx, |this, cx| {
                                                this.set_book(book_id.clone(), cx)
                                            });
                                        }),
                                );
                            }
                            menu
                        }
                    }),
            )
            .child(
                Button::new("status")
                    .ghost()
                    .small()
                    .label(if status == NoteStatus::None {
                        "No status"
                    } else {
                        status.label()
                    })
                    .dropdown_caret(true)
                    .dropdown_menu({
                        let view = view.clone();
                        move |mut menu, _, _| {
                            for option in NoteStatus::ALL {
                                let view = view.clone();
                                menu = menu.item(
                                    PopupMenuItem::new(option.label())
                                        .checked(option == status)
                                        .on_click(move |_, _, cx| {
                                            view.update(cx, |this, cx| this.set_status(option, cx));
                                        }),
                                );
                            }
                            menu
                        }
                    }),
            )
            .children(tag_chips)
            .child(
                Button::new("tags")
                    .ghost()
                    .xsmall()
                    .icon(IconName::Plus)
                    .tooltip("Tags")
                    .dropdown_menu({
                        let view = view.clone();
                        move |mut menu, _, _| {
                            if tags.is_empty() {
                                return menu.item(PopupMenuItem::new("No tags yet").disabled(true));
                            }
                            for (id, name, _) in &tags {
                                let view = view.clone();
                                let tag_id = id.clone();
                                menu = menu.item(
                                    PopupMenuItem::new(name.clone())
                                        .checked(note_tags.contains(id))
                                        .on_click(move |_, _, cx| {
                                            view.update(cx, |this, cx| {
                                                this.toggle_tag(tag_id.clone(), cx)
                                            });
                                        }),
                                );
                            }
                            menu
                        }
                    }),
            )
            .child(div().flex_1())
            .child(
                Button::new("pin")
                    .ghost()
                    .small()
                    .icon(if note.pinned {
                        IconName::StarFill
                    } else {
                        IconName::Star
                    })
                    .tooltip(if note.pinned { "Unpin" } else { "Pin to top" })
                    .on_click(cx.listener(|this, _, _, cx| this.toggle_pinned(cx))),
            )
            .child(self.view_mode_button(
                "mode-edit",
                IconName::FileText,
                ViewMode::Edit,
                "Editor only",
                cx,
            ))
            .child(self.view_mode_button(
                "mode-split",
                IconName::PanelRight,
                ViewMode::Split,
                "Editor and preview",
                cx,
            ))
            .child(self.view_mode_button(
                "mode-preview",
                IconName::Eye,
                ViewMode::Preview,
                "Preview only",
                cx,
            ))
            .child(
                Button::new("note-menu")
                    .ghost()
                    .small()
                    .icon(IconName::Ellipsis)
                    .tooltip("More")
                    .dropdown_menu(|menu, _, _| {
                        menu.menu("History…", Box::new(crate::workspace::ShowHistory))
                            .menu(
                                "Export as Markdown…",
                                Box::new(crate::workspace::ExportNote),
                            )
                            .separator()
                            .menu("Move to Trash", Box::new(crate::workspace::TrashNote))
                    }),
            )
    }

    fn view_mode_button(
        &self,
        id: &'static str,
        icon: IconName,
        mode: ViewMode,
        tooltip: &'static str,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        Button::new(id)
            .ghost()
            .small()
            .icon(icon)
            .tooltip(tooltip)
            .selected(self.view_mode == mode)
            .on_click(cx.listener(move |this, _, _, cx| this.set_view_mode(mode, cx)))
    }

    fn render_backlinks(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let chips = self.backlinks.iter().enumerate().map(|(ix, (id, title))| {
            let library = self.library.clone();
            let id = id.clone();
            div()
                .id(("backlink", ix))
                .px_2()
                .rounded_md()
                .cursor_pointer()
                .text_color(cx.theme().link)
                .hover(|el| el.bg(cx.theme().muted))
                .child(if title.trim().is_empty() {
                    "Untitled".to_owned()
                } else {
                    title.clone()
                })
                .on_click(move |_, _, cx| {
                    library.update(cx, |library, cx| library.select(Some(id.clone()), cx));
                })
        });
        h_flex()
            .flex_wrap()
            .gap_1()
            .px_4()
            .py_1()
            .text_xs()
            .border_b_1()
            .border_color(cx.theme().border)
            .child(
                div()
                    .text_color(cx.theme().muted_foreground)
                    .child("Linked from"),
            )
            .children(chips)
    }

    fn render_incoming_banner(&self, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .gap_2()
            .px_4()
            .py_2()
            .bg(cx.theme().warning.opacity(0.15))
            .border_b_1()
            .border_color(cx.theme().border)
            .text_sm()
            .child(div().flex_1().child(
                "This note was changed elsewhere while you were editing. The version you don't keep stays in its history.",
            ))
            .child(
                Button::new("keep-mine")
                    .small()
                    .primary()
                    .label("Keep mine")
                    .on_click(cx.listener(|this, _, _, cx| this.keep_mine(cx))),
            )
            .child(
                Button::new("take-theirs")
                    .small()
                    .label("Use theirs")
                    .on_click(cx.listener(|this, _, window, cx| this.take_theirs(window, cx))),
            )
    }
}

pub fn rgb_color(rgb: u32) -> Hsla {
    gpui_kit::rgb(rgb).into()
}

impl Render for NoteEditor {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(note) = self.note.clone() else {
            return v_flex()
                .size_full()
                .items_center()
                .justify_center()
                .gap_2()
                .bg(cx.theme().background)
                .text_color(cx.theme().muted_foreground)
                .child("No note selected")
                .child(div().text_xs().child("⌘N creates a note"))
                .into_any_element();
        };
        let trashed = note.is_trashed();
        let mode = self.view_mode;
        v_flex()
            .size_full()
            .bg(cx.theme().background)
            .child(
                div().px_4().pt_3().pb_1().child(
                    Input::new(&self.title)
                        .appearance(false)
                        .text_xl()
                        .font_semibold(),
                ),
            )
            .child(self.render_toolbar(&note, cx))
            .when(trashed, |el| {
                el.child(
                    h_flex()
                        .px_4()
                        .py_1()
                        .gap_2()
                        .text_sm()
                        .bg(cx.theme().muted)
                        .child(div().flex_1().child("This note is in the trash."))
                        .child(
                            Button::new("restore-from-trash")
                                .small()
                                .label("Restore")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    let book = this.library.read(cx).default_book();
                                    if let Some(book) = book {
                                        this.set_book(book, cx);
                                    }
                                })),
                        ),
                )
            })
            .when(self.incoming.is_some(), |el| {
                el.child(self.render_incoming_banner(cx))
            })
            .when(!self.backlinks.is_empty(), |el| {
                el.child(self.render_backlinks(cx))
            })
            .child(
                h_flex()
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .when(mode != ViewMode::Preview, |el| {
                        el.child(
                            div().flex_1().h_full().min_w_0().px_4().py_2().child(
                                Editor::new(&self.body)
                                    .bordered(false)
                                    .h_full()
                                    .text_size(px(self.font_size)),
                            ),
                        )
                    })
                    .when(mode != ViewMode::Edit, |el| {
                        el.child(
                            div()
                                .flex_1()
                                .h_full()
                                .min_w_0()
                                .when(mode == ViewMode::Split, |el| {
                                    el.border_l_1().border_color(cx.theme().border)
                                })
                                .child(
                                    self.preview
                                        .clone()
                                        .cached(StyleRefinement::default().size_full()),
                                ),
                        )
                    }),
            )
            .into_any_element()
    }
}

#[cfg(test)]
impl NoteEditor {
    /// Replaces the title and body as if typed, scheduling the autosave.
    pub fn edit_for_test(&mut self, body: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.body
            .update(cx, |state, cx| state.set_value(body.to_owned(), window, cx));
        self.schedule_autosave(cx);
    }

    pub fn body_for_test(&self, cx: &App) -> String {
        self.current_text(cx).1
    }

    pub fn has_incoming(&self) -> bool {
        self.incoming.is_some()
    }
}

/// Notes outside the trash whose body links to note `id`, as (id, title).
pub fn backlinks(store: &Store, id: &str) -> Vec<(String, String)> {
    let Some(suffix) = id.strip_prefix("note:") else {
        return Vec::new();
    };
    let target = format!("inkdrop://note/{suffix}");
    let query = kanso_core::Query {
        clauses: vec![kanso_core::Clause {
            negated: false,
            field: Some(kanso_core::Field::Body),
            value: target.clone(),
        }],
    };
    let params = kanso_store::SearchParams {
        query,
        ..Default::default()
    };
    store
        .search_summaries(&params)
        .unwrap_or_default()
        .into_iter()
        .filter(|n| n.id != id)
        .filter(|n| {
            // The search matches substrings; make sure the ID isn't just a
            // prefix of a longer one.
            store.note(&n.id).ok().flatten().is_some_and(|note| {
                kanso_core::tasks::linked_note_ids(&note.body)
                    .iter()
                    .any(|linked| linked == id)
            })
        })
        .map(|n| (n.id, n.title))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::backlinks;
    use kanso_core::DocType;
    use kanso_store::{Origin, Put, Store};
    use proptest::prelude::*;
    use serde_json::json;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(48))]

        /// Model-based: backlinks are exactly the other live notes whose
        /// links include the note, even when one ID is a prefix of another.
        #[test]
        fn prop_backlinks_match_the_link_graph(links in prop::collection::vec((prop::collection::vec(0usize..6, 0..4), any::<bool>()), 1..6)) {
            let dir = tempfile::tempdir().unwrap();
            let store = Store::open(dir.path().join("kanso.db")).unwrap();
            let book = store.put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui).unwrap().id;
            // IDs n, nn, nnn… so shorter IDs are prefixes of longer ones.
            let ids: Vec<String> = (0..6).map(|i| format!("note:{}", "n".repeat(i + 1))).collect();
            for (ix, (targets, trashed)) in links.iter().enumerate() {
                let body: String = targets.iter().map(|t| format!("[x](inkdrop://note/{}) ", &ids[*t][5..])).collect();
                let book_id = if *trashed { "trash".to_owned() } else { book.clone() };
                store.put(DocType::Note, json!({"_id": ids[ix], "title": format!("t{ix}"), "body": body, "bookId": book_id}), Put::Upsert, Origin::Ui).unwrap();
            }
            for (target, id) in ids.iter().enumerate() {
                let mut expected: Vec<String> = links.iter().enumerate()
                    .filter(|(ix, (targets, trashed))| *ix != target && !*trashed && targets.contains(&target))
                    .map(|(ix, _)| ids[ix].clone())
                    .collect();
                expected.sort();
                let mut found: Vec<String> = backlinks(&store, id).into_iter().map(|(id, _)| id).collect();
                found.sort();
                prop_assert_eq!(found, expected);
            }
        }
    }
}
