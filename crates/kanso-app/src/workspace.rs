//! The main window: sidebar, note list and editor, plus the operations that
//! need more than one of them (dialogs, moving notes, deleting notebooks).

use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use gpui_kit::base::{GlobalState, resize_handle};
use gpui_kit::component::notification::Notification;
use gpui_kit::component::{ActiveTheme as _, WindowExt as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_core::model::TRASH_BOOK_ID;
use kanso_core::{DocType, TagColor};
use kanso_store::{Origin, Put, Store};
use serde_json::{Value, json};

use crate::account_page::SignedOutForm;
use crate::commands::Picker;
use crate::config::{AppConfig, ViewMode};
use crate::dialogs::{confirm, prompt_text};
use crate::editor::{EditorEvent, NoteEditor};
use crate::layout::{BandAction, BandDrag, PaneEdge, PaneWidths, inset_after, rail_width};
use crate::library::{Filter, Library, LibraryEvent};
use crate::note_list::{NoteList, NoteListEvent};
use crate::palette::{Mode, Palette};
use crate::paths;
use crate::settings::{SettingsPage, SettingsReturn, SettingsView};
use crate::sidebar::{Sidebar, SidebarEvent};
use crate::theme::Colors;

actions!(
    kanso,
    [
        NewNote,
        NewNotebook,
        FocusSearch,
        SearchAllNotes,
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
        ShowSignIn,
        ShowCreateAccount,
        ChangePassword,
        NewRecoveryKey,
        SignOut,
        ImportInkdrop,
        Quit,
        OpenCommandPalette,
        OpenNotePicker,
    ]
);

// Focus mode: ⌘⇧D in and out, and Escape out once nothing inside wants it.
actions!(kanso, [ToggleFocusMode, ExitFocusMode]);

/// Process-wide state: the store, the data folder and the settings.
pub struct AppState {
    pub store: Arc<Store>,
    pub data_dir: PathBuf,
    pub config: AppConfig,
}

impl Global for AppState {}

/// How long settings that change often wait before they are written.
const CONFIG_SAVE_DELAY: Duration = Duration::from_secs(2);

/// A settings write waiting for [`CONFIG_SAVE_DELAY`].
#[derive(Default)]
struct PendingConfigSave(Option<Task<()>>);

impl Global for PendingConfigSave {}

impl AppState {
    /// Changes the settings and writes them to disk.
    pub fn update_config(cx: &mut App, change: impl FnOnce(&mut AppConfig)) {
        // This write includes anything still waiting.
        cx.default_global::<PendingConfigSave>().0 = None;
        let state = cx.global_mut::<AppState>();
        change(&mut state.config);
        let path = paths::config_path(&state.data_dir);
        if let Err(err) = state.config.save(&path) {
            eprintln!("kanso: couldn't save settings: {err:#}");
        }
    }

    /// Changes the settings now and writes them once they stop changing for
    /// a moment, for things that change all the time, like the selected note.
    pub fn update_config_soon(cx: &mut App, change: impl FnOnce(&mut AppConfig)) {
        change(&mut cx.global_mut::<AppState>().config);
        let save = cx.spawn(async move |cx| {
            cx.background_executor().timer(CONFIG_SAVE_DELAY).await;
            cx.update(AppState::save_pending_config);
        });
        cx.default_global::<PendingConfigSave>().0 = Some(save);
    }

    /// Writes a waiting settings change now, as when quitting.
    pub fn save_pending_config(cx: &mut App) {
        if cx.default_global::<PendingConfigSave>().0.take().is_some() {
            AppState::update_config(cx, |_| {});
        }
    }
}

pub fn bind_keys(cx: &mut App) {
    // `secondary` is ⌘ on macOS and Ctrl on Windows and Linux.
    cx.bind_keys([
        KeyBinding::new("secondary-n", NewNote, None),
        KeyBinding::new("secondary-shift-n", NewNotebook, None),
        // ⌘L searches the list as it is; ⇧⌘F, global search, all notes.
        KeyBinding::new("secondary-l", FocusSearch, None),
        KeyBinding::new("secondary-shift-f", SearchAllNotes, None),
        // ⌘L goes to the list's search box; ⇧⌘L to the list itself.
        KeyBinding::new("secondary-shift-l", FocusNoteList, None),
        KeyBinding::new("secondary-\\", ToggleSidebar, None),
        KeyBinding::new("secondary-shift-d", ToggleFocusMode, None),
        // After anything inside that takes Escape (an open menu, extra
        // cursors, the palette), so the first Escape left over leaves.
        KeyBinding::new("escape", ExitFocusMode, Some("FocusMode")),
        KeyBinding::new("secondary-1", ShowEditor, None),
        KeyBinding::new("secondary-2", ShowSplit, None),
        KeyBinding::new("secondary-3", ShowPreview, None),
        KeyBinding::new("secondary-s", SaveNote, None),
        KeyBinding::new("secondary-shift-p", TogglePinned, None),
        KeyBinding::new("secondary-alt-down", NextNote, None),
        KeyBinding::new("secondary-alt-up", PreviousNote, None),
        KeyBinding::new("secondary-,", OpenSettings, None),
        KeyBinding::new("secondary-q", Quit, None),
        KeyBinding::new("enter", FocusEditor, Some("NoteList")),
        KeyBinding::new("secondary-k", OpenCommandPalette, None),
        KeyBinding::new("secondary-p", OpenNotePicker, None),
    ]);
    crate::palette::bind_keys(cx);
    crate::settings::bind_keys(cx);
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
            MenuItem::action("Export Note…", ExportNote),
            MenuItem::action("Export All Notes…", ExportLibrary),
        ]),
        Menu::new("Edit").items([MenuItem::os_submenu("Services", SystemMenuType::Services)]),
        Menu::new("Note").items([
            MenuItem::action("Pin / Unpin Note", TogglePinned),
            MenuItem::action("Move to Trash", TrashNote),
            MenuItem::action("Show History…", ShowHistory),
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
            MenuItem::action("Focus Mode", ToggleFocusMode),
            MenuItem::action("Search All Notes", SearchAllNotes),
            MenuItem::action("Search This List", FocusSearch),
            MenuItem::action("Go to Note List", FocusNoteList),
        ]),
    ]);
}

/// Runs `f` on the workspace in every main window.
pub fn for_each_workspace(
    cx: &mut App,
    mut f: impl FnMut(&mut Workspace, &mut Context<Workspace>),
) {
    for window in cx.windows() {
        window
            .update(cx, |root, _, cx| {
                let Ok(root) = root.downcast::<gpui_kit::base::Root>() else {
                    return;
                };
                let view = root.read(cx).view().clone();
                if let Ok(workspace) = view.downcast::<Workspace>() {
                    workspace.update(cx, |workspace, cx| f(workspace, cx));
                }
            })
            .ok();
    }
}

/// Shows the themes the settings choose, in every window.
pub fn apply_theme(window: Option<&mut Window>, cx: &mut App) {
    crate::themes::apply(window, cx);
}

pub struct Workspace {
    store: Arc<Store>,
    pub(crate) library: Entity<Library>,
    pub(crate) sidebar: Entity<Sidebar>,
    pub(crate) note_list: Entity<NoteList>,
    pub editor: Entity<NoteEditor>,
    /// The command palette, drawn over the rest when open.
    pub(crate) palette: Entity<Palette>,
    /// Settings, drawn in place of the list and the editor while open.
    pub(crate) settings: Entity<SettingsView>,
    /// While Settings are open: where the notes were, to go back to.
    pub(crate) settings_return: Option<SettingsReturn>,
    /// The full sidebar, or the icon rail it collapses to.
    pub(crate) show_sidebar: bool,
    /// Only the note, centred: no sidebar and no list.
    pub(crate) focus_mode: bool,
    /// The sidebar's and the list's widths; the editor takes the rest.
    pub(crate) pane_widths: PaneWidths,
    /// The workspace's own focus. It always has a place in the window, so
    /// the window's shortcuts work whenever nothing inside has the keyboard:
    /// at launch, or after the focused element stops being drawn.
    pub(crate) focus: FocusHandle,
    /// A press in a top band that may become a window drag.
    band_drag: BandDrag,
    /// Draws stand-ins for the traffic lights, which the system draws over
    /// the window and a headless snapshot can't show.
    #[cfg(test)]
    pub(crate) placeholder_traffic_lights: bool,
    _subscriptions: Vec<Subscription>,
    _links: Task<()>,
}

impl Workspace {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let state = cx.global::<AppState>();
        let store = state.store.clone();
        let appearance = state.config.appearance.clone();
        let last_note = state.config.last_note.clone();
        let focus_mode = state.config.layout.focus_mode;

        let library = cx.new(|cx| Library::new(store.clone(), cx));
        let sidebar = cx.new(|cx| Sidebar::new(library.clone(), cx));
        let note_list = cx.new(|cx| NoteList::new(library.clone(), window, cx));
        let (links_tx, links_rx) = async_channel::unbounded::<String>();
        let editor = cx.new(|cx| {
            NoteEditor::new(
                store.clone(),
                library.clone(),
                links_tx,
                &appearance,
                window,
                cx,
            )
        });

        let focus = cx.focus_handle();
        let list_focus = note_list.read(cx).focus_handle(cx);
        let palette = cx.new(|cx| {
            Palette::new(
                store.clone(),
                library.clone(),
                editor.clone(),
                focus.clone(),
                list_focus,
                window,
                cx,
            )
        });

        let settings = cx.new(|cx| SettingsView::new(library.clone(), window, cx));

        let subscriptions = vec![
            cx.subscribe_in(&library, window, |this, _, event, window, cx| match event {
                // A place asked for: the list shows, whatever hid it.
                LibraryEvent::Shown => {
                    this.leave_settings(window, cx);
                    this.set_focus_mode(false, window, cx);
                }
                // A note someone picked shows; one the app picked itself
                // (the last note at launch, the first of a new list) waits
                // behind Settings.
                LibraryEvent::Selected { id, asked } => {
                    if *asked {
                        this.leave_settings(window, cx);
                    }
                    let id = id.clone();
                    this.editor
                        .update(cx, |editor, cx| editor.open(id.as_deref(), window, cx));
                    AppState::update_config_soon(cx, |config| config.last_note = id);
                }
                LibraryEvent::Changed(change) => {
                    let change = change.clone();
                    this.editor.update(cx, |editor, cx| {
                        editor.on_store_change(&change, window, cx);
                        if change.id.starts_with("note:") {
                            editor.refresh_backlinks(cx);
                        }
                    });
                }
            }),
            cx.subscribe_in(&sidebar, window, |this, _, event, window, cx| {
                this.on_sidebar_event(event, window, cx);
            }),
            cx.subscribe_in(&note_list, window, |this, _, event, window, cx| {
                this.on_note_list_event(event, window, cx);
            }),
            cx.subscribe_in(&editor, window, |this, _, event, window, cx| match event {
                EditorEvent::Notice(message) => {
                    window.push_notification(Notification::error(message.clone()), cx);
                }
                EditorEvent::ShowHistory => this.show_history(window, cx),
                EditorEvent::Pick(picker) => {
                    let mode = Mode::Pick(*picker);
                    this.palette
                        .update(cx, |palette, cx| palette.open(mode, window, cx));
                }
            }),
            cx.observe_window_appearance(window, |_, window, cx| {
                if cx
                    .global::<AppState>()
                    .config
                    .appearance
                    .themes
                    .follow_system
                {
                    apply_theme(Some(window), cx);
                }
            }),
            // The preview alone hides the text that had the keyboard, say:
            // the keyboard comes back here rather than going nowhere.
            cx.on_focus_lost(window, |this, window, cx| window.focus(&this.focus, cx)),
        ];
        window.focus(&focus, cx);

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
            library.update(cx, |library, cx| library.select_quietly(Some(id), cx));
        }
        // Focus mode kept from last time: once the note is open, the
        // keyboard goes into its text, as turning focus mode on does.
        if focus_mode {
            cx.defer_in(window, |this: &mut Workspace, window, cx| {
                if this.focus_mode && !this.settings_open() {
                    this.editor
                        .update(cx, |editor, cx| editor.focus_for_writing(window, cx));
                }
            });
        }

        Workspace {
            store,
            library,
            sidebar,
            note_list,
            editor,
            palette,
            settings,
            settings_return: None,
            show_sidebar: appearance.show_sidebar,
            focus_mode,
            pane_widths: PaneWidths::default(),
            focus,
            band_drag: BandDrag::default(),
            #[cfg(test)]
            placeholder_traffic_lights: false,
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
        // The list's own narrowing too, so the new note shows in it.
        if let Some(status) = library.status_only {
            note["status"] = json!(status.as_str());
        }
        if library.pinned_only {
            note["pinned"] = json!(true);
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
            SidebarEvent::NewNote => self.new_note(window, cx),
            SidebarEvent::GoToNotebook => {
                let mode = Mode::Pick(Picker::GoToNotebook);
                self.palette
                    .update(cx, |palette, cx| palette.open(mode, window, cx));
            }
            SidebarEvent::GoToTag => {
                let mode = Mode::Pick(Picker::GoToTag);
                self.palette
                    .update(cx, |palette, cx| palette.open(mode, window, cx));
            }
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

    /// Settings › Account with `form` open: Sign in, or Create account.
    pub fn show_account_form(
        &mut self,
        form: SignedOutForm,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.open_settings(Some(SettingsPage::Account), window, cx);
        let account = self.settings.read(cx).account.clone();
        account.update(cx, |account, cx| account.open_form(form, window, cx));
    }

    /// Asks for a folder, then exports `only` those notes, or all of them.
    fn export(&mut self, only: Option<Vec<String>>, window: &mut Window, cx: &mut Context<Self>) {
        self.flush(cx);
        let store = self.store.clone();
        let picked = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some("Export Here".into()),
        });
        cx.spawn_in(window, async move |_, cx| {
            let Ok(Ok(Some(paths))) = picked.await else {
                return;
            };
            let Some(root) = paths.into_iter().next() else {
                return;
            };
            let result = cx
                .background_executor()
                .spawn({
                    let root = root.clone();
                    async move { crate::export::export(&store, &root, only.as_deref()) }
                })
                .await;
            cx.update(|window, cx| match result {
                Ok(report) => {
                    window.push_notification(
                        Notification::success(format!(
                            "Exported {} notes and {} attachments.",
                            report.notes, report.attachments
                        )),
                        cx,
                    );
                    cx.reveal_path(&root);
                }
                Err(err) => window
                    .push_notification(Notification::error(format!("Export failed: {err:#}")), cx),
            })
            .ok();
        })
        .detach();
    }

    fn set_view_mode(&mut self, mode: ViewMode, cx: &mut Context<Self>) {
        self.editor
            .update(cx, |editor, cx| editor.set_view_mode(mode, cx));
        AppState::update_config(cx, |config| config.appearance.view_mode = mode);
    }

    pub(crate) fn show_history(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.flush(cx);
        crate::history::open_history(self.editor.clone(), window, cx);
    }

    /// Global search: the search box over every note, whatever place or
    /// narrowing the list showed. Text already in the box stays.
    fn search_all_notes(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.library.update(cx, |library, cx| {
            library.set_filter(Filter::All, cx);
            library.clear_refinements(cx);
        });
        self.note_list
            .update(cx, |list, cx| list.focus_search(window, cx));
    }

    /// Acts on a band drag's verdict.
    fn on_band(&mut self, action: BandAction, window: &mut Window) {
        match action {
            BandAction::None => {}
            BandAction::StartMove => window.start_window_move(),
            BandAction::TitlebarDoubleClick => window.titlebar_double_click(),
        }
    }

    /// Turns focus mode on or off, and keeps it for next time. Going in,
    /// the keyboard goes to the note's text, for writing. (An open palette
    /// closes as its field loses the keyboard.)
    pub(crate) fn set_focus_mode(&mut self, on: bool, window: &mut Window, cx: &mut Context<Self>) {
        if self.focus_mode == on {
            return;
        }
        self.focus_mode = on;
        AppState::update_config(cx, |config| config.layout.focus_mode = on);
        if on {
            self.editor
                .update(cx, |editor, cx| editor.focus_for_writing(window, cx));
        }
        cx.notify();
    }

    /// The rail's width while the sidebar is folded to it.
    fn rail(&self, window: &Window) -> Option<f32> {
        (!self.show_sidebar).then(|| rail_width(window.is_fullscreen()))
    }

    /// The panes' widths as drawn: as set, or narrower when the window
    /// leaves the editor less than its minimum.
    fn fitted_widths(&self, window: &Window) -> PaneWidths {
        let width = f32::from(window.viewport_size().width);
        self.pane_widths.fitted(width, self.rail(window))
    }

    /// The width of what is left of the note list: the sidebar or its rail.
    fn leading_width(&self, window: &Window) -> f32 {
        self.rail(window)
            .unwrap_or_else(|| self.fitted_widths(window).sidebar)
    }

    /// A pane's edge followed the pointer.
    fn on_pane_drag(
        &mut self,
        event: &DragMoveEvent<Rc<PaneEdge>>,
        window: &Window,
        cx: &mut Context<Self>,
    ) {
        let edge = **event.drag(cx);
        let x = f32::from(event.event.position.x - event.bounds.left());
        let rail = self.rail(window);
        // From the widths as drawn, so the edge doesn't jump to the pointer.
        let from = self.fitted_widths(window);
        let window = f32::from(event.bounds.size.width);
        let widths = from.dragged(edge, x, window, rail);
        if widths != self.pane_widths {
            self.pane_widths = widths;
            cx.notify();
        }
    }

    /// The three traffic lights where the system draws them, for snapshots.
    #[cfg(test)]
    fn render_placeholder_traffic_lights(&self) -> Option<impl IntoElement> {
        use crate::layout::TRAFFIC_LIGHTS;
        self.placeholder_traffic_lights.then(|| {
            div()
                .flex()
                .absolute()
                .left(px(TRAFFIC_LIGHTS.0))
                .top(px(TRAFFIC_LIGHTS.1))
                .gap(px(8.))
                .children(
                    [0xFF5F57, 0xFEBC2E, 0x28C840]
                        .map(|color| div().size(px(12.)).rounded_full().bg(gpui_kit::rgb(color))),
                )
        })
    }

    #[cfg(not(test))]
    fn render_placeholder_traffic_lights(&self) -> Option<Div> {
        None
    }
}

/// The divider on a pane's left edge, which drags `edge`: the pane before
/// it, the sidebar or the list.
// no-prop-test: drawing; the widths a drag makes are tested in `layout`.
fn pane_handle(edge: PaneEdge) -> impl IntoElement {
    let id = match edge {
        PaneEdge::Sidebar => "sidebar-edge",
        PaneEdge::NoteList => "list-edge",
    };
    resize_handle::<PaneEdge, PaneEdge>(id, Axis::Horizontal).on_drag(edge, |edge, _, _, cx| {
        cx.stop_propagation();
        cx.new(|_| *edge)
    })
}

/// What a pane drag shows under the pointer: nothing, the divider moves.
impl Render for PaneEdge {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Empty
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
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        // The sidebar or its rail is leftmost and keeps the traffic lights'
        // room; the list's top band leaves whatever is left of it free.
        let leading = self.leading_width(window);
        let list_inset = px(inset_after(leading, window.is_fullscreen()));
        self.note_list
            .update(cx, |list, cx| list.set_leading_inset(list_inset, cx));
        self.settings.update(cx, |settings, cx| {
            settings.set_leading_inset(list_inset, cx)
        });
        let collapsed = !self.show_sidebar;
        self.sidebar
            .update(cx, |sidebar, cx| sidebar.set_collapsed(collapsed, cx));
        // Opening Settings leaves focus mode, so the two never show together;
        // Settings would win if they did.
        let settings_open = self.settings_open();
        let focus_mode = self.focus_mode && !settings_open;
        self.editor
            .update(cx, |editor, cx| editor.set_focus_mode(focus_mode, cx));
        let widths = self.fitted_widths(window);
        self.palette
            .update(cx, |palette, _| palette.set_notes_hidden(settings_open));
        let signed_in = cx
            .try_global::<crate::services::Services>()
            .is_some_and(|services| services.signed_in());
        v_flex()
            .size_full()
            .relative()
            // Escape leaves focus mode, or Settings, once nothing inside
            // (the palette, a menu, an input) has taken it.
            .key_context(if focus_mode {
                "Workspace FocusMode"
            } else if settings_open {
                "Workspace Settings"
            } else {
                "Workspace"
            })
            .track_focus(&self.focus)
            .bg(colors.app)
            .text_color(cx.theme().foreground)
            // Controls mark the presses they take; start each press clean.
            .capture_any_mouse_down(|_, _, cx| {
                if cx.has_global::<GlobalState>() {
                    GlobalState::reset_text_selection_suppression(cx);
                }
            })
            // Presses on empty parts of the top bands move the window. A
            // button prevents the default; an input claims the press for its
            // own text selection.
            .on_any_mouse_down(cx.listener(|this, event: &MouseDownEvent, window, cx| {
                let claimed = window.default_prevented()
                    || (cx.has_global::<GlobalState>()
                        && GlobalState::is_text_selection_suppressed(cx));
                if event.button == MouseButton::Left {
                    let action = this
                        .band_drag
                        .down(event.position, event.click_count, claimed);
                    this.on_band(action, window);
                }
                // The workspace's focus only gives the keyboard a place when
                // nothing else has it. A press on a band or any other bare
                // chrome, or on text selected for copying, leaves the
                // keyboard where it was, as on macOS: without this the focus
                // would move here and typing would go nowhere.
                window.prevent_default();
            }))
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, window, _| {
                if this.band_drag.is_armed() {
                    let pressed = event.pressed_button == Some(MouseButton::Left);
                    let action = this.band_drag.moved(event.position, pressed);
                    this.on_band(action, window);
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _: &MouseUpEvent, _, _| this.band_drag.up()),
            )
            .on_action(cx.listener(|this, _: &NewNote, window, cx| this.new_note(window, cx)))
            .on_action(cx.listener(|this, _: &NewNotebook, window, cx| {
                this.on_sidebar_event(&SidebarEvent::NewBook { parent: None }, window, cx);
            }))
            .on_action(cx.listener(|this, _: &FocusSearch, window, cx| {
                this.leave_settings(window, cx);
                // The search box is in the list, which focus mode hides.
                this.set_focus_mode(false, window, cx);
                this.note_list
                    .update(cx, |list, cx| list.focus_search(window, cx));
            }))
            .on_action(cx.listener(|this, _: &SearchAllNotes, window, cx| {
                this.search_all_notes(window, cx);
            }))
            .on_action(cx.listener(|this, _: &FocusNoteList, window, cx| {
                // The list is hidden behind Settings and in focus mode.
                this.leave_settings(window, cx);
                this.set_focus_mode(false, window, cx);
                this.note_list
                    .update(cx, |list, cx| list.focus_list(window, cx));
            }))
            .on_action(cx.listener(|this, _: &FocusEditor, window, cx| {
                this.editor
                    .update(cx, |editor, cx| editor.focus_body(window, cx));
            }))
            .on_action(cx.listener(|this, _: &ToggleSidebar, window, cx| {
                // In focus mode there is no sidebar to toggle: it comes back.
                let show = this.focus_mode || !this.show_sidebar;
                this.set_focus_mode(false, window, cx);
                this.show_sidebar = show;
                AppState::update_config(cx, |config| config.appearance.show_sidebar = show);
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &ToggleFocusMode, window, cx| {
                // From Settings, back to the note, to write in it alone.
                if this.settings_open() {
                    this.close_settings(window, cx);
                    this.set_focus_mode(true, window, cx);
                } else {
                    this.set_focus_mode(!this.focus_mode, window, cx);
                }
            }))
            .on_action(cx.listener(|this, _: &ExitFocusMode, window, cx| {
                this.set_focus_mode(false, window, cx);
            }))
            // What acts on the open note brings it back from behind
            // Settings first, so the note it changes is in view.
            .on_action(cx.listener(|this, _: &ShowEditor, window, cx| {
                this.close_settings(window, cx);
                this.set_view_mode(ViewMode::Edit, cx)
            }))
            .on_action(cx.listener(|this, _: &ShowSplit, window, cx| {
                this.close_settings(window, cx);
                this.set_view_mode(ViewMode::Split, cx)
            }))
            .on_action(cx.listener(|this, _: &ShowPreview, window, cx| {
                this.close_settings(window, cx);
                this.set_view_mode(ViewMode::Preview, cx)
            }))
            .on_action(cx.listener(|this, _: &SaveNote, _, cx| this.flush(cx)))
            .on_action(cx.listener(|this, _: &TogglePinned, window, cx| {
                this.close_settings(window, cx);
                this.editor
                    .update(cx, |editor, cx| editor.toggle_pinned(cx));
            }))
            .on_action(cx.listener(|this, _: &TrashNote, window, cx| {
                this.close_settings(window, cx);
                this.editor.update(cx, |editor, cx| editor.trash(cx));
            }))
            .on_action(cx.listener(|this, _: &ShowHistory, window, cx| {
                this.close_settings(window, cx);
                this.show_history(window, cx);
            }))
            .on_action(cx.listener(|this, _: &ExportNote, window, cx| {
                this.close_settings(window, cx);
                let id = this.editor.read(cx).note_id().map(str::to_owned);
                if let Some(id) = id {
                    this.export(Some(vec![id]), window, cx);
                }
            }))
            .on_action(
                cx.listener(|this, _: &ExportLibrary, window, cx| this.export(None, window, cx)),
            )
            .on_action(cx.listener(|_, _: &ImportInkdrop, window, cx| {
                crate::inkdrop::start_import(window, cx);
            }))
            .on_action(cx.listener(|this, _: &NextNote, _, cx| {
                this.library
                    .update(cx, |library, cx| library.select_relative(1, cx));
            }))
            .on_action(cx.listener(|this, _: &PreviousNote, _, cx| {
                this.library
                    .update(cx, |library, cx| library.select_relative(-1, cx));
            }))
            .on_action(cx.listener(|this, _: &OpenCommandPalette, window, cx| {
                this.palette
                    .update(cx, |palette, cx| palette.open(Mode::Commands, window, cx));
            }))
            .on_action(cx.listener(|this, _: &OpenNotePicker, window, cx| {
                this.palette
                    .update(cx, |palette, cx| palette.open(Mode::Notes, window, cx));
            }))
            // Only while signed in, so the menu's Sync Now is disabled
            // otherwise.
            .when(signed_in, |el| {
                el.on_action(cx.listener(|_, _: &SyncNow, _, cx| crate::services::sync_now(cx)))
                    .on_action(cx.listener(|this, _: &ChangePassword, window, cx| {
                        this.open_settings(Some(SettingsPage::Account), window, cx);
                        crate::account_page::change_password_dialog(window, cx);
                    }))
                    .on_action(cx.listener(|this, _: &NewRecoveryKey, window, cx| {
                        this.open_settings(Some(SettingsPage::Account), window, cx);
                        crate::account_page::new_recovery_key_dialog(window, cx);
                    }))
                    .on_action(cx.listener(|_, _: &SignOut, window, cx| {
                        crate::account_page::sign_out_dialog(window, cx);
                    }))
            })
            .when(!signed_in, |el| {
                el.on_action(cx.listener(|this, _: &ShowSignIn, window, cx| {
                    this.show_account_form(SignedOutForm::SignIn, window, cx)
                }))
                .on_action(cx.listener(
                    |this, _: &ShowCreateAccount, window, cx| {
                        this.show_account_form(SignedOutForm::CreateAccount, window, cx)
                    },
                ))
            })
            .map(|el| crate::settings::on_actions(el, cx))
            // The panes run the full height: each has its own top band
            // (the title bar's row) and bottom band. The sidebar and the
            // list keep their widths and the editor takes the rest; each
            // pane's left edge drags the pane before it.
            .child(
                h_flex()
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .on_drag_move(cx.listener(
                        |this, event: &DragMoveEvent<Rc<PaneEdge>>, window, cx| {
                            this.on_pane_drag(event, window, cx)
                        },
                    ))
                    // Focus mode leaves the note alone in the window.
                    .when(!focus_mode, |el| {
                        el.child(
                            div()
                                .h_full()
                                .w(px(leading))
                                .flex_none()
                                .debug_selector(|| "sidebar-pane".into())
                                .child(self.sidebar.clone()),
                        )
                    })
                    // Settings take the list's and the editor's place, beside
                    // the sidebar or its rail.
                    .when(settings_open, |el| {
                        el.child(
                            div()
                                .relative()
                                .h_full()
                                .flex_1()
                                .min_w_0()
                                .debug_selector(|| "settings-pane".into())
                                .child(self.settings.clone())
                                // The rail keeps its width.
                                .when(self.show_sidebar, |el| {
                                    el.child(pane_handle(PaneEdge::Sidebar))
                                }),
                        )
                    })
                    .when(!settings_open, |el| {
                        el.when(!focus_mode, |el| {
                            el.child(
                                div()
                                    .relative()
                                    .h_full()
                                    .w(px(widths.list))
                                    .flex_none()
                                    .debug_selector(|| "list-pane".into())
                                    .child(self.note_list.clone())
                                    // The rail keeps its width.
                                    .when(self.show_sidebar, |el| {
                                        el.child(pane_handle(PaneEdge::Sidebar))
                                    }),
                            )
                        })
                        .child(
                            div()
                                .relative()
                                .h_full()
                                .flex_1()
                                .min_w_0()
                                .debug_selector(|| "editor-pane".into())
                                .child(self.editor.clone())
                                .when(!focus_mode, |el| el.child(pane_handle(PaneEdge::NoteList))),
                        )
                    }),
            )
            // Over everything else when open.
            .child(self.palette.clone())
            // The system draws the real traffic lights above the palette too.
            .children(self.render_placeholder_traffic_lights())
    }
}
