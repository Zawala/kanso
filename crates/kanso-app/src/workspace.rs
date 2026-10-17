//! The main window: sidebar, note list and editor, plus the operations that
//! need more than one of them (dialogs, moving notes, deleting notebooks).

use std::path::PathBuf;
use std::sync::Arc;

use gpui_kit::component::notification::Notification;
use gpui_kit::component::resizable::{h_resizable, resizable_panel};
use gpui_kit::component::{ActiveTheme as _, Theme, ThemeMode, WindowExt as _, h_flex, v_flex};
use gpui_kit::*;
use kanso_core::model::TRASH_BOOK_ID;
use kanso_core::{DocType, TagColor};
use kanso_store::{Origin, Put, Store};
use serde_json::{Value, json};

use crate::config::{AppConfig, ThemePreference, ViewMode};
use crate::dialogs::{confirm, prompt_text};
use crate::editor::{EditorEvent, NoteEditor};
use crate::library::{Filter, Library, LibraryEvent};
use crate::note_list::{NoteList, NoteListEvent};
use crate::paths;
use crate::services::Services;
use crate::sidebar::{Sidebar, SidebarEvent};

actions!(
    kanso,
    [
        NewNote,
        NewNotebook,
        FocusSearch,
        FocusNoteList,
        FocusEditor,
        ToggleSidebar,
        ShowEditor,
        ShowSplit,
        ShowPreview,
        TogglePinned,
        TrashNote,
        SaveNote,
        NextNote,
        PreviousNote,
        ShowHistory,
        OpenSettings,
        ExportNote,
        ExportLibrary,
        SyncNow,
        ImportInkdrop,
        Quit,
    ]
);

/// Process-wide state: the store, the data folder and the settings.
pub struct AppState {
    pub store: Arc<Store>,
    pub data_dir: PathBuf,
    pub config: AppConfig,
}

impl Global for AppState {}

impl AppState {
    /// Changes the settings and writes them to disk.
    pub fn update_config(cx: &mut App, change: impl FnOnce(&mut AppConfig)) {
        let state = cx.global_mut::<AppState>();
        change(&mut state.config);
        let path = paths::config_path(&state.data_dir);
        if let Err(err) = state.config.save(&path) {
            eprintln!("kanso: couldn't save settings: {err:#}");
        }
    }
}

pub fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("cmd-n", NewNote, None),
        KeyBinding::new("cmd-shift-n", NewNotebook, None),
        KeyBinding::new("cmd-shift-f", FocusSearch, None),
        KeyBinding::new("cmd-l", FocusSearch, None),
        KeyBinding::new("cmd-\\", ToggleSidebar, None),
        KeyBinding::new("cmd-1", ShowEditor, None),
        KeyBinding::new("cmd-2", ShowSplit, None),
        KeyBinding::new("cmd-3", ShowPreview, None),
        KeyBinding::new("cmd-s", SaveNote, None),
        KeyBinding::new("cmd-shift-p", TogglePinned, None),
        KeyBinding::new("cmd-alt-down", NextNote, None),
        KeyBinding::new("cmd-alt-up", PreviousNote, None),
        KeyBinding::new("cmd-,", OpenSettings, None),
        KeyBinding::new("cmd-q", Quit, None),
        KeyBinding::new("enter", FocusEditor, Some("NoteList")),
        KeyBinding::new("escape", FocusNoteList, Some("NoteEditor")),
    ]);
}

pub fn set_menus(cx: &mut App) {
    cx.set_menus([
        Menu::new("Kanso").items([
            MenuItem::action("Settings…", OpenSettings),
            MenuItem::separator(),
            MenuItem::action("Import from Inkdrop…", ImportInkdrop),
            MenuItem::action("Sync Now", SyncNow),
            MenuItem::separator(),
            MenuItem::action("Quit Kanso", Quit),
        ]),
        Menu::new("File").items([
            MenuItem::action("New Note", NewNote),
            MenuItem::action("New Notebook", NewNotebook),
            MenuItem::separator(),
            MenuItem::action("Save", SaveNote),
            MenuItem::action("Export Note as Markdown…", ExportNote),
            MenuItem::action("Export All Notes as Markdown…", ExportLibrary),
        ]),
        Menu::new("Edit").items([MenuItem::os_submenu("Services", SystemMenuType::Services)]),
        Menu::new("Note").items([
            MenuItem::action("Pin / Unpin", TogglePinned),
            MenuItem::action("Move to Trash", TrashNote),
            MenuItem::action("History…", ShowHistory),
            MenuItem::separator(),
            MenuItem::action("Next Note", NextNote),
            MenuItem::action("Previous Note", PreviousNote),
        ]),
        Menu::new("View").items([
            MenuItem::action("Editor Only", ShowEditor),
            MenuItem::action("Editor and Preview", ShowSplit),
            MenuItem::action("Preview Only", ShowPreview),
            MenuItem::separator(),
            MenuItem::action("Toggle Sidebar", ToggleSidebar),
            MenuItem::action("Search Notes", FocusSearch),
        ]),
    ]);
}

pub fn apply_theme(preference: ThemePreference, window: Option<&mut Window>, cx: &mut App) {
    match preference {
        ThemePreference::System => Theme::sync_system_appearance(window, cx),
        ThemePreference::Light => Theme::change(ThemeMode::Light, window, cx),
        ThemePreference::Dark => Theme::change(ThemeMode::Dark, window, cx),
    }
}

pub struct Workspace {
    store: Arc<Store>,
    library: Entity<Library>,
    sidebar: Entity<Sidebar>,
    note_list: Entity<NoteList>,
    pub editor: Entity<NoteEditor>,
    show_sidebar: bool,
    _subscriptions: Vec<Subscription>,
    _links: Task<()>,
}

impl Workspace {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let state = cx.global::<AppState>();
        let store = state.store.clone();
        let appearance = state.config.appearance.clone();
        let last_note = state.config.last_note.clone();

        let library = cx.new(|cx| Library::new(store.clone(), cx));
        let sidebar = cx.new(|cx| Sidebar::new(library.clone(), cx));
        let note_list = cx.new(|cx| NoteList::new(library.clone(), window, cx));
        let (links_tx, links_rx) = async_channel::unbounded::<String>();
        let editor = cx.new(|cx| {
            NoteEditor::new(
                store.clone(),
                library.clone(),
                links_tx,
                appearance.view_mode,
                appearance.editor_font_size,
                window,
                cx,
            )
        });

        let subscriptions = vec![
            cx.subscribe_in(&library, window, |this, _, event, window, cx| match event {
                LibraryEvent::Selected(id) => {
                    let id = id.clone();
                    this.editor
                        .update(cx, |editor, cx| editor.open(id.as_deref(), window, cx));
                    AppState::update_config(cx, |config| config.last_note = id);
                }
                LibraryEvent::Changed(change) => {
                    let change = change.clone();
                    this.editor
                        .update(cx, |editor, cx| editor.on_store_change(&change, window, cx));
                }
            }),
            cx.subscribe_in(&sidebar, window, |this, _, event, window, cx| {
                this.on_sidebar_event(event, window, cx);
            }),
            cx.subscribe_in(&note_list, window, |this, _, event, window, cx| {
                this.on_note_list_event(event, window, cx);
            }),
            cx.subscribe_in(&editor, window, |_, _, event, window, cx| match event {
                EditorEvent::Notice(message) => {
                    window.push_notification(Notification::error(message.clone()), cx);
                }
            }),
            cx.observe_window_appearance(window, |_, window, cx| {
                if cx.global::<AppState>().config.appearance.theme == ThemePreference::System {
                    Theme::sync_system_appearance(Some(window), cx);
                }
            }),
        ];

        let links = cx.spawn_in(window, async move |this, cx| {
            while let Ok(id) = links_rx.recv().await {
                let opened = this.update(cx, |this, cx| {
                    this.library
                        .update(cx, |library, cx| library.select(Some(id), cx));
                });
                if opened.is_err() {
                    break;
                }
            }
        });

        if let Some(id) = last_note.filter(|id| store.note(id).ok().flatten().is_some()) {
            library.update(cx, |library, cx| library.select(Some(id), cx));
        }

        Workspace {
            store,
            library,
            sidebar,
            note_list,
            editor,
            show_sidebar: appearance.show_sidebar,
            _subscriptions: subscriptions,
            _links: links,
        }
    }

    fn notify_error(&self, message: impl Into<SharedString>, window: &mut Window, cx: &mut App) {
        window.push_notification(Notification::error(message), cx);
    }

    /// Saves the open note before an operation that may touch it.
    fn flush(&self, cx: &mut App) {
        self.editor.update(cx, |editor, cx| editor.flush(cx));
    }

    fn new_note(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.flush(cx);
        let library = self.library.read(cx);
        let book = match library.default_book() {
            Some(book) => book,
            None => match self.store.put(
                DocType::Book,
                json!({"name": "Notes"}),
                Put::Upsert,
                Origin::Ui,
            ) {
                Ok(created) => created.id,
                Err(err) => {
                    return self.notify_error(
                        format!("Couldn't create a notebook: {err}"),
                        window,
                        cx,
                    );
                }
            },
        };
        let mut note = json!({"title": "", "body": "", "bookId": book});
        match &library.filter {
            Filter::Tag(tag) => note["tags"] = json!([tag]),
            Filter::Status(status) => note["status"] = json!(status.as_str()),
            Filter::Pinned => note["pinned"] = json!(true),
            _ => {}
        }
        if matches!(library.filter, Filter::Trash) {
            self.library.update(cx, |library, cx| {
                library.set_filter(Filter::Book(book.clone()), cx)
            });
        }
        match self.store.put(DocType::Note, note, Put::Upsert, Origin::Ui) {
            Ok(created) => {
                self.library
                    .update(cx, |library, cx| library.select(Some(created.id), cx));
                self.editor
                    .update(cx, |editor, cx| editor.focus_title(window, cx));
            }
            Err(err) => self.notify_error(format!("Couldn't create the note: {err}"), window, cx),
        }
    }

    /// Writes `fields` into document `id` with a shallow merge.
    fn merge(
        &self,
        doc_type: DocType,
        id: &str,
        mut fields: Value,
        window: &mut Window,
        cx: &mut App,
    ) {
        fields["_id"] = json!(id);
        if let Err(err) = self.store.put(doc_type, fields, Put::Merge, Origin::Ui) {
            self.notify_error(err.to_string(), window, cx);
        }
    }

    fn with_open_note(
        &self,
        id: &str,
        cx: &mut App,
        f: impl FnOnce(&mut NoteEditor, &mut Context<NoteEditor>),
    ) -> bool {
        let open = self.editor.read(cx).note_id() == Some(id);
        if open {
            self.editor.update(cx, f);
        }
        open
    }

    fn on_note_list_event(
        &mut self,
        event: &NoteListEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            NoteListEvent::NewNote => self.new_note(window, cx),
            NoteListEvent::OpenSelected => self
                .editor
                .update(cx, |editor, cx| editor.focus_body(window, cx)),
            NoteListEvent::TogglePinned(id) => {
                if !self.with_open_note(id, cx, |editor, cx| editor.toggle_pinned(cx)) {
                    let pinned = self
                        .library
                        .read(cx)
                        .notes
                        .iter()
                        .any(|n| &n.id == id && n.pinned);
                    self.merge(DocType::Note, id, json!({"pinned": !pinned}), window, cx);
                }
            }
            NoteListEvent::Trash(id) => {
                if !self.with_open_note(id, cx, |editor, cx| editor.trash(cx)) {
                    self.merge(
                        DocType::Note,
                        id,
                        json!({"bookId": TRASH_BOOK_ID}),
                        window,
                        cx,
                    );
                }
            }
            NoteListEvent::Restore(id) => {
                let Some(book) = self.library.read(cx).books.first().map(|b| b.id.clone()) else {
                    return self.notify_error(
                        "Create a notebook to restore notes into.",
                        window,
                        cx,
                    );
                };
                if !self.with_open_note(id, cx, |editor, cx| editor.set_book(book.clone(), cx)) {
                    self.merge(DocType::Note, id, json!({"bookId": book}), window, cx);
                }
            }
            NoteListEvent::DeleteForever(id) => {
                let id = id.clone();
                let store = self.store.clone();
                confirm(
                    "Delete this note permanently?",
                    "It can't be restored afterwards.",
                    "Delete",
                    window,
                    cx,
                    move |window, cx| {
                        if let Err(err) = store.delete(&id, Origin::Ui) {
                            window.push_notification(Notification::error(err.to_string()), cx);
                        }
                    },
                );
            }
            NoteListEvent::Duplicate(id) => {
                self.flush(cx);
                let Ok(Some(note)) = self.store.note(id) else {
                    return;
                };
                let copy = json!({
                    "title": format!("{} (copy)", note.title),
                    "body": note.body,
                    "bookId": note.book_id,
                    "tags": note.tags,
                    "status": note.status.as_str(),
                });
                match self.store.put(DocType::Note, copy, Put::Upsert, Origin::Ui) {
                    Ok(created) => self
                        .library
                        .update(cx, |library, cx| library.select(Some(created.id), cx)),
                    Err(err) => self.notify_error(err.to_string(), window, cx),
                }
            }
        }
    }

    fn on_sidebar_event(
        &mut self,
        event: &SidebarEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let store = self.store.clone();
        let library = self.library.clone();
        let report =
            |result: Result<(), kanso_store::StoreError>, window: &mut Window, cx: &mut App| {
                if let Err(err) = result {
                    window.push_notification(Notification::error(err.to_string()), cx);
                }
            };
        match event {
            SidebarEvent::NewBook { parent } => {
                let parent = parent.clone();
                prompt_text(
                    "New Notebook",
                    "",
                    "Create",
                    window,
                    cx,
                    move |name, window, cx| {
                        let result = store.put(
                            DocType::Book,
                            json!({"name": name, "parentBookId": parent}),
                            Put::Upsert,
                            Origin::Ui,
                        );
                        match result {
                            Ok(created) => library.update(cx, |library, cx| {
                                library.set_filter(Filter::Book(created.id), cx)
                            }),
                            Err(err) => report(Err(err), window, cx),
                        }
                    },
                );
            }
            SidebarEvent::RenameBook(id) => {
                let current = self
                    .library
                    .read(cx)
                    .book(id)
                    .map(|b| b.name.clone())
                    .unwrap_or_default();
                let id = id.clone();
                prompt_text(
                    "Rename Notebook",
                    &current,
                    "Rename",
                    window,
                    cx,
                    move |name, window, cx| {
                        let result = store.put(
                            DocType::Book,
                            json!({"_id": id, "name": name}),
                            Put::Merge,
                            Origin::Ui,
                        );
                        report(result.map(drop), window, cx);
                    },
                );
            }
            SidebarEvent::DeleteBook(id) => {
                let name = self
                    .library
                    .read(cx)
                    .book(id)
                    .map(|b| b.name.clone())
                    .unwrap_or_default();
                let id = id.clone();
                let editor = self.editor.clone();
                confirm(
                    format!("Delete “{name}”?"),
                    "Its notes and the notes in its sub-notebooks move to the trash.",
                    "Delete",
                    window,
                    cx,
                    move |window, cx| {
                        editor.update(cx, |editor, cx| editor.flush(cx));
                        let result = store.delete(&id, Origin::Ui);
                        library.update(cx, |library, cx| {
                            if library.filter == Filter::Book(id.clone()) {
                                library.set_filter(Filter::All, cx);
                            }
                        });
                        report(result.map(drop), window, cx);
                    },
                );
            }
            SidebarEvent::NewTag => {
                prompt_text(
                    "New Tag",
                    "",
                    "Create",
                    window,
                    cx,
                    move |name, window, cx| {
                        let result =
                            store.put(DocType::Tag, json!({"name": name}), Put::Upsert, Origin::Ui);
                        report(result.map(drop), window, cx);
                    },
                );
            }
            SidebarEvent::RenameTag(id) => {
                let current = self
                    .library
                    .read(cx)
                    .tag(id)
                    .map(|t| t.name.clone())
                    .unwrap_or_default();
                let id = id.clone();
                prompt_text(
                    "Rename Tag",
                    &current,
                    "Rename",
                    window,
                    cx,
                    move |name, window, cx| {
                        let result = store.put(
                            DocType::Tag,
                            json!({"_id": id, "name": name}),
                            Put::Merge,
                            Origin::Ui,
                        );
                        report(result.map(drop), window, cx);
                    },
                );
            }
            SidebarEvent::RecolorTag(id, color) => {
                self.merge(
                    DocType::Tag,
                    id,
                    json!({"color": tag_color_name(*color)}),
                    window,
                    cx,
                );
            }
            SidebarEvent::DeleteTag(id) => {
                let name = self
                    .library
                    .read(cx)
                    .tag(id)
                    .map(|t| t.name.clone())
                    .unwrap_or_default();
                let id = id.clone();
                let editor = self.editor.clone();
                confirm(
                    format!("Delete the tag “{name}”?"),
                    "Notes keep their content; the tag is removed from them.",
                    "Delete",
                    window,
                    cx,
                    move |window, cx| {
                        editor.update(cx, |editor, cx| editor.flush(cx));
                        report(store.delete(&id, Origin::Ui).map(drop), window, cx);
                    },
                );
            }
            SidebarEvent::EmptyTrash => {
                let count = self.library.read(cx).counts.trash;
                confirm(
                    format!("Delete {count} notes in the trash permanently?"),
                    "They can't be restored afterwards.",
                    "Empty Trash",
                    window,
                    cx,
                    move |window, cx| {
                        let params = crate::library::search_params(
                            &Filter::Trash,
                            "",
                            Default::default(),
                            true,
                        );
                        let result = store.search_summaries(&params).and_then(|notes| {
                            notes
                                .iter()
                                .try_for_each(|n| store.delete(&n.id, Origin::Ui).map(drop))
                        });
                        report(result, window, cx);
                    },
                );
            }
        }
    }

    fn set_view_mode(&mut self, mode: ViewMode, cx: &mut Context<Self>) {
        self.editor
            .update(cx, |editor, cx| editor.set_view_mode(mode, cx));
        AppState::update_config(cx, |config| config.appearance.view_mode = mode);
    }

    fn render_status_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let services = cx.global::<Services>();
        let api = services.api_status();
        let sync = services.sync_status();
        h_flex()
            .h_6()
            .px_3()
            .gap_4()
            .text_xs()
            .text_color(cx.theme().muted_foreground)
            .border_t_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().sidebar)
            .child(api)
            .child(sync)
            .child(div().flex_1())
            .child(format!("{} notes", self.library.read(cx).counts.all))
    }
}

/// The JSON name of a tag color.
pub fn tag_color_name(color: TagColor) -> String {
    serde_json::to_value(color)
        .ok()
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_else(|| "default".into())
}

impl Render for Workspace {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .key_context("Workspace")
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .on_action(cx.listener(|this, _: &NewNote, window, cx| this.new_note(window, cx)))
            .on_action(cx.listener(|this, _: &NewNotebook, window, cx| {
                this.on_sidebar_event(&SidebarEvent::NewBook { parent: None }, window, cx);
            }))
            .on_action(cx.listener(|this, _: &FocusSearch, window, cx| {
                this.note_list
                    .update(cx, |list, cx| list.focus_search(window, cx));
            }))
            .on_action(cx.listener(|this, _: &FocusNoteList, window, cx| {
                this.note_list
                    .update(cx, |list, cx| list.focus_list(window, cx));
            }))
            .on_action(cx.listener(|this, _: &FocusEditor, window, cx| {
                this.editor
                    .update(cx, |editor, cx| editor.focus_body(window, cx));
            }))
            .on_action(cx.listener(|this, _: &ToggleSidebar, _, cx| {
                this.show_sidebar = !this.show_sidebar;
                let show = this.show_sidebar;
                AppState::update_config(cx, |config| config.appearance.show_sidebar = show);
                cx.notify();
            }))
            .on_action(
                cx.listener(|this, _: &ShowEditor, _, cx| this.set_view_mode(ViewMode::Edit, cx)),
            )
            .on_action(
                cx.listener(|this, _: &ShowSplit, _, cx| this.set_view_mode(ViewMode::Split, cx)),
            )
            .on_action(
                cx.listener(|this, _: &ShowPreview, _, cx| {
                    this.set_view_mode(ViewMode::Preview, cx)
                }),
            )
            .on_action(cx.listener(|this, _: &SaveNote, _, cx| this.flush(cx)))
            .on_action(cx.listener(|this, _: &TogglePinned, _, cx| {
                this.editor
                    .update(cx, |editor, cx| editor.toggle_pinned(cx));
            }))
            .on_action(cx.listener(|this, _: &TrashNote, _, cx| {
                this.editor.update(cx, |editor, cx| editor.trash(cx));
            }))
            .on_action(cx.listener(|this, _: &NextNote, _, cx| {
                this.library
                    .update(cx, |library, cx| library.select_relative(1, cx));
            }))
            .on_action(cx.listener(|this, _: &PreviousNote, _, cx| {
                this.library
                    .update(cx, |library, cx| library.select_relative(-1, cx));
            }))
            .child(
                h_resizable("workspace")
                    .child(
                        resizable_panel()
                            .size(px(230.))
                            .size_range(px(160.)..px(380.))
                            .visible(self.show_sidebar)
                            .child(self.sidebar.clone()),
                    )
                    .child(
                        resizable_panel()
                            .size(px(320.))
                            .size_range(px(240.)..px(560.))
                            .child(self.note_list.clone()),
                    )
                    .child(resizable_panel().child(self.editor.clone())),
            )
            .child(self.render_status_bar(cx))
    }
}
