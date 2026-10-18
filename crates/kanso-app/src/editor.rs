//! The note editor: a header band (where the note is, whether it is saved,
//! the view mode and note actions), the title, status and tags, the
//! formatting toolbar, the Markdown body and preview, and a footer band.
//!
//! Edits autosave after a short pause. A change to the open note from
//! elsewhere (the API, sync) reloads it quietly when there are no unsaved
//! edits, and otherwise asks which version to keep. The losing version is
//! kept in the note's history either way. Leaving the note or quitting
//! before answering keeps the editor's text.

use std::sync::Arc;
use std::time::Duration;

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonGroup, ButtonVariants as _};
use gpui_kit::component::input::{Editor, EditorState, Input, InputEvent, InputState, Position};
use gpui_kit::component::menu::{ContextMenuExt as _, DropdownMenu as _, PopupMenuItem};
use gpui_kit::component::tag::Tag;
use gpui_kit::component::{
    ActiveTheme as _, Icon, IconName, Selectable as _, Sizable as _, StyledExt as _, h_flex, v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_core::model::TRASH_BOOK_ID;
use kanso_core::{DocType, Note, NoteStatus, now_ms};
use kanso_store::{Change, Origin, Put, Store, StoreError};

use crate::clock::Clock;
use crate::commands::Picker;
use crate::config::{Appearance, EditorFont, ViewMode};
use crate::editor_footer::EditorFooter;
use crate::empty_state::EmptyNotebook;
use crate::layout::{BOTTOM_BAND_HEIGHT, TOP_BAND_HEIGHT};
use crate::library::{Filter, Library};
use crate::markdown_edit::{self, Format};
use crate::preview::{LinkSender, Preview, TEXT_INSET, readable_width};
use crate::theme::{Colors, muted_tag_color};
use crate::workspace::AppState;

const AUTOSAVE_DELAY: Duration = Duration::from_millis(600);
const PREVIEW_DELAY: Duration = Duration::from_millis(150);
/// Line height of the Markdown source, relative to its size. Every face and
/// view mode sets the source at the chosen size with this line height:
/// beside the preview it wraps sooner rather than getting smaller.
const BODY_LINE_HEIGHT: f32 = 1.6;
/// Where the source editor's text starts inside it, without line numbers:
/// its padding and the gap its gutter keeps. The pane's padding makes up the
/// rest of [`TEXT_INSET`].
const SOURCE_TEXT_LEFT: f32 = 12.;
/// What the source editor adds around its text beyond the readable measure:
/// that, its right padding, and the line number column when it shows.
const SOURCE_EDGES: f32 = SOURCE_TEXT_LEFT + 10.;
const LINE_NUMBER_COLUMN: f32 = 44.;
/// One line of the header's save state; a second line is clipped away.
const SAVE_LABEL_HEIGHT: f32 = 20.;

pub enum EditorEvent {
    /// Something worth telling the user, such as a failed save.
    Notice(String),
    /// The history button: the workspace opens the note's history.
    ShowHistory,
    /// The notebook path or "Add tag": the workspace opens the palette's
    /// list of notebooks or tags, which filters as you type and makes new
    /// tags.
    Pick(Picker),
}

/// What happened elsewhere to the open note while it had unsaved edits.
enum Incoming {
    /// A newer version was saved.
    Changed(Box<Note>),
    /// The note was deleted.
    Deleted,
}

pub struct NoteEditor {
    store: Arc<Store>,
    library: Entity<Library>,
    /// The note as last loaded or saved, with its current `_rev`.
    note: Option<Note>,
    title: Entity<InputState>,
    body: Entity<EditorState>,
    preview: Entity<Preview>,
    footer: Entity<EditorFooter>,
    pub view_mode: ViewMode,
    pub editor_font: EditorFont,
    pub font_size: f32,
    /// Whether monospace text may show line numbers (a setting).
    pub allow_line_numbers: bool,
    /// Whether the body shows line numbers now; see [`EditorFont::face`].
    line_numbers: bool,
    autosave: Option<Task<()>>,
    preview_task: Option<Task<()>>,
    /// A change from elsewhere that arrived over unsaved edits.
    incoming: Option<Incoming>,
    /// Notes that link here: (id, title).
    backlinks: Vec<(String, String)>,
    backlinks_task: Option<Task<()>>,
    /// Where the empty-notebook block sends its actions from while no note
    /// is open.
    pub(crate) empty_focus: FocusHandle,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<EditorEvent> for NoteEditor {}

impl NoteEditor {
    pub fn new(
        store: Arc<Store>,
        library: Entity<Library>,
        links: LinkSender,
        appearance: &Appearance,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let Appearance {
            view_mode,
            editor_font,
            editor_font_size: font_size,
            line_numbers: allow_line_numbers,
            ..
        } = *appearance;
        let title = cx.new(|cx| InputState::new(window, cx).placeholder("Untitled"));
        let line_numbers = editor_font
            .face(view_mode)
            .with_line_numbers(allow_line_numbers)
            .line_numbers;
        let body = cx.new(|cx| {
            EditorState::new(window, cx)
                .language("markdown")
                .line_number(line_numbers)
                // Folding comes with line numbers: its column is what keeps
                // the numbers clear of the text.
                .folding(line_numbers)
                .indent_guides(false)
                .soft_wrap(true)
        });
        let preview = cx.new(|_| Preview::new(store.clone(), links));
        let footer = cx.new(|cx| {
            let mut footer = EditorFooter::new(body.clone(), cx);
            footer.set_show_cursor(view_mode != ViewMode::Preview, cx);
            footer
        });
        let subscriptions = vec![
            cx.subscribe_in(&title, window, |this, _, event, window, cx| match event {
                // The breadcrumb shows the title as it's typed.
                InputEvent::Change => {
                    this.schedule_autosave(cx);
                    cx.notify();
                }
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
            // Quitting from the Dock, at logout or from the menu.
            cx.on_app_quit(|this, cx| {
                this.flush(cx);
                async {}
            }),
            // With no note open, the pane follows the list: an empty
            // notebook offers its first note.
            cx.observe(&library, |this, _, cx| {
                if this.note.is_none() {
                    cx.notify();
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
            footer,
            view_mode,
            editor_font,
            font_size,
            allow_line_numbers,
            line_numbers,
            autosave: None,
            preview_task: None,
            incoming: None,
            backlinks: Vec::new(),
            backlinks_task: None,
            empty_focus: cx.focus_handle(),
            _subscriptions: subscriptions,
        }
    }

    pub fn note_id(&self) -> Option<&str> {
        self.note.as_ref().map(|n| n.id.as_str())
    }

    /// Saves the open note, then opens `id` (or nothing).
    pub fn open(&mut self, id: Option<&str>, window: &mut Window, cx: &mut Context<Self>) {
        if self.note_id() == id {
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
        let has_note = self.note.is_some();
        self.footer
            .update(cx, |footer, cx| footer.note_opened(has_note, cx));
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
        // With the question about a change made elsewhere open, nothing is
        // written until it is answered.
        if self.note.is_none() || self.incoming.is_some() {
            return;
        }
        let was_saved = self.autosave.is_none();
        self.autosave = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(AUTOSAVE_DELAY).await;
            this.update(cx, |this, cx| {
                this.autosave = None;
                this.save(cx);
                // "Saving…" gives way even when there was nothing to write.
                cx.notify();
            })
            .ok();
        }));
        if was_saved {
            cx.notify();
        }
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

    /// Saves any pending edits now. With the question about a change made
    /// elsewhere still open, this keeps the editor's text, so that leaving
    /// the note or quitting never loses it; the other version stays in
    /// history.
    pub fn flush(&mut self, cx: &mut Context<Self>) {
        self.autosave = None;
        // Another writer can win the race between reading and writing; try
        // again on top of its version.
        for _ in 0..3 {
            if self.incoming.is_some() {
                self.keep_mine(cx);
            } else {
                self.save(cx);
            }
            if self.incoming.is_none() {
                return;
            }
        }
        self.stash_draft(cx);
    }

    /// Keeps the editor's text in the open note's history.
    fn stash_draft(&mut self, cx: &mut Context<Self>) {
        let Some(note) = &self.note else {
            return;
        };
        let (title, body) = self.current_text(cx);
        let mut fields = serde_json::Map::new();
        fields.insert("title".into(), title.into());
        fields.insert("body".into(), body.into());
        fields.insert("bookId".into(), note.book_id.clone().into());
        let rev = note.rev.clone().unwrap_or_default();
        if let Err(err) = self
            .store
            .record_revision(&note.id, &rev, &fields, Origin::Ui)
        {
            cx.emit(EditorEvent::Notice(format!(
                "Couldn't keep your draft: {err}"
            )));
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
        let Some(note) = self.note.clone() else {
            return false;
        };
        self.write_over(note, change, origin, cx)
    }

    /// Writes `base` with the editor's text and `change` applied. `self.note`
    /// only changes once the write succeeds.
    fn write_over(
        &mut self,
        mut note: Note,
        change: impl FnOnce(&mut Note, &App),
        origin: Origin,
        cx: &mut Context<Self>,
    ) -> bool {
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
                // The write was based on the current version, so there is
                // nothing left to choose between.
                self.incoming = None;
                cx.notify();
                true
            }
            Err(StoreError::Conflict) => {
                // Someone else saved first; show their version next to ours.
                if let Ok(Some(latest)) = self.store.note(&note.id) {
                    self.incoming = Some(Incoming::Changed(Box::new(latest)));
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
        let incoming = if change.deleted {
            Incoming::Deleted
        } else {
            let Ok(Some(latest)) = self.store.note(&change.id) else {
                return;
            };
            Incoming::Changed(Box::new(latest))
        };
        if self.has_unsaved_edits(cx) {
            self.autosave = None;
            self.incoming = Some(incoming);
            cx.notify();
        } else {
            match incoming {
                Incoming::Changed(latest) => self.show(Some(*latest), true, window, cx),
                Incoming::Deleted => self.show(None, false, window, cx),
            }
        }
    }

    /// Keeps the editor's text over the change made elsewhere. A newer
    /// version stays in history; a deleted note comes back.
    pub fn keep_mine(&mut self, cx: &mut Context<Self>) {
        let Some(incoming) = self.incoming.take() else {
            return;
        };
        let base = match &incoming {
            Incoming::Changed(theirs) => (**theirs).clone(),
            Incoming::Deleted => match self.note.clone() {
                Some(note) => note,
                None => return,
            },
        };
        // The note's notebook may have been deleted along with it.
        let store = self.store.clone();
        let fallback_book = self.library.read(cx).default_book();
        let rehome = move |note: &mut Note, _: &App| {
            if note.book_id != TRASH_BOOK_ID
                && store.get(&note.book_id).ok().flatten().is_none()
                && let Some(book) = fallback_book
            {
                note.book_id = book;
            }
        };
        // An API-origin write always snapshots the version it replaces.
        if !self.write_over(base, rehome, Origin::Api, cx) && self.incoming.is_none() {
            // Not a conflict: leave the question open rather than drop it.
            self.incoming = Some(incoming);
        }
    }

    /// Takes the change made elsewhere; the editor's unsaved text goes to
    /// history.
    pub fn take_theirs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(incoming) = self.incoming.take() else {
            return;
        };
        self.stash_draft(cx);
        match incoming {
            Incoming::Changed(theirs) => self.show(Some(*theirs), true, window, cx),
            Incoming::Deleted => self.show(None, false, window, cx),
        }
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
        self.footer.update(cx, |footer, cx| {
            footer.set_show_cursor(mode != ViewMode::Preview, cx)
        });
        cx.notify();
    }

    /// The view mode picked in the header, kept for next time.
    fn choose_view_mode(&mut self, mode: ViewMode, cx: &mut Context<Self>) {
        self.set_view_mode(mode, cx);
        AppState::update_config(cx, |config| config.appearance.view_mode = mode);
    }

    /// Applies a toolbar format to the body at its selection, as one edit
    /// that undoes in one step, and puts the cursor back in the text.
    pub fn apply_format(&mut self, format: Format, window: &mut Window, cx: &mut Context<Self>) {
        if self.note.is_none() {
            return;
        }
        self.body.update(cx, |state, cx| {
            let text = state.value();
            let edit = markdown_edit::format(format, &text, state.selected_range());
            state.set_selected_range(edit.range.clone(), cx);
            state.replace(edit.text, window, cx);
            state.set_selected_range(edit.selection, cx);
            state.focus(window, cx);
        });
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
        self.stash_draft(cx);
        self.title
            .update(cx, |state, cx| state.set_value(title, window, cx));
        self.body
            .update(cx, |state, cx| state.set_value(body.clone(), window, cx));
        self.preview
            .update(cx, |preview, cx| preview.set_text(body.into(), cx));
        self.save(cx);
    }

    /// Whether the open note's text is saved, as the header says it.
    pub(crate) fn save_state(&self, cx: &App) -> SaveState {
        save_state(self.incoming.is_some(), self.autosave.is_some(), || {
            self.has_unsaved_edits(cx)
        })
    }

    /// The top band: where the note lives (click it to move the note),
    /// whether it is saved, linked-from notes, the view mode and the note's
    /// actions. Its empty middle moves the window.
    fn render_band(&self, note: &Note, cx: &mut Context<Self>) -> impl IntoElement {
        let library = self.library.read(cx);
        let path: Vec<String> = if note.book_id == TRASH_BOOK_ID {
            vec!["Trash".to_owned()]
        } else {
            let names: Vec<String> = library
                .book_path(&note.book_id)
                .iter()
                .map(|b| b.name.clone())
                .collect();
            if names.is_empty() {
                vec!["No notebook".to_owned()]
            } else {
                names
            }
        };
        let title = self.title.read(cx).value();
        let title: SharedString = if title.trim().is_empty() {
            "Untitled".into()
        } else {
            title
        };
        let colors = Colors::get(cx);
        let (saved, saved_color) = match self.save_state(cx) {
            SaveState::Saving => ("Saving…".to_owned(), colors.text_muted),
            SaveState::NotSaved => ("Not saved".to_owned(), colors.warning),
            SaveState::Saved => (
                format!("Edited {}", Clock::edited_label(cx, note.updated_at)),
                colors.text_muted,
            ),
        };
        let view = cx.entity();
        // One separator for the notebook path and before the title, the
        // same size as the words and the same room on either side.
        let separator = || {
            div()
                .flex_none()
                .text_size(px(13.))
                .text_color(colors.text_muted)
                .child("/")
        };
        let mode = self.view_mode;
        let pinned = note.pinned;
        let status = note.status;

        let segments = path.into_iter().enumerate().flat_map(|(ix, name)| {
            let name = div()
                .min_w_0()
                .truncate()
                .text_size(px(13.))
                .text_color(colors.text_secondary)
                .child(name)
                .into_any_element();
            let before = (ix > 0).then(|| separator().mx(px(8.)).into_any_element());
            before.into_iter().chain([name])
        });
        let move_menu = Button::new("breadcrumb")
            .ghost()
            .small()
            // Buttons don't shrink by default; a long path truncates.
            .flex_shrink(1.)
            .min_w_0()
            .child(
                h_flex()
                    .min_w_0()
                    .max_w(px(240.))
                    .overflow_hidden()
                    .children(segments),
            )
            .tooltip("Move to notebook")
            .debug_selector(|| "breadcrumb".into())
            // The palette's list rather than a menu: a library can have more
            // notebooks than a menu shows, and typing finds one.
            .on_click(cx.listener(|_, _, _, cx| cx.emit(EditorEvent::Pick(Picker::Notebook))));

        let backlinks = (!self.backlinks.is_empty()).then(|| {
            let links = self.backlinks.clone();
            let count = links.len();
            let library = self.library.clone();
            Button::new("backlinks")
                .ghost()
                .small()
                .icon(Icon::new(Lucide::Link).text_color(colors.text_secondary))
                .child(
                    div()
                        .text_size(px(12.5))
                        .text_color(colors.text_secondary)
                        .child(count.to_string()),
                )
                .tooltip(if count == 1 {
                    "Linked from 1 note".to_owned()
                } else {
                    format!("Linked from {count} notes")
                })
                .dropdown_menu(move |mut menu, _, _| {
                    // Room for about a dozen; more scroll.
                    menu = menu.max_h(px(420.)).scrollable(true).label("Linked from");
                    for (id, title) in &links {
                        let library = library.clone();
                        let id = id.clone();
                        let title = if title.trim().is_empty() {
                            "Untitled".to_owned()
                        } else {
                            title.clone()
                        };
                        menu = menu.item(PopupMenuItem::new(title).on_click(move |_, _, cx| {
                            library.update(cx, |library, cx| library.select(Some(id.clone()), cx));
                        }));
                    }
                    menu
                })
        });

        let modes = [
            ("mode-edit", "Edit", ViewMode::Edit),
            ("mode-preview", "Preview", ViewMode::Preview),
            ("mode-split", "Split", ViewMode::Split),
        ];
        let mode_group = ButtonGroup::new("view-mode")
            .outline()
            .small()
            .children(modes.map(|(id, label, option)| {
                let selected = mode == option;
                Button::new(id).selected(selected).child(
                    div()
                        .px_0p5()
                        .text_size(px(12.5))
                        .text_color(if selected {
                            colors.text
                        } else {
                            colors.text_secondary
                        })
                        .child(label),
                )
            }))
            .on_click(cx.listener(move |this, clicked: &Vec<usize>, _, cx| {
                if let Some(&ix) = clicked.first() {
                    this.choose_view_mode(modes[ix].2, cx);
                }
            }));

        h_flex()
            .flex_none()
            .h(px(TOP_BAND_HEIGHT))
            // The ghost button's padding lines its text up with the title.
            .pl(px(TEXT_INSET - 8.))
            .pr(px(12.))
            .gap(px(4.))
            // When the pane is narrow the save state gives way first, then
            // the path and the title down to a few letters. The button's
            // own padding is the room before the separator.
            .child(
                h_flex()
                    .min_w_0()
                    .child(move_menu)
                    .child(separator().mr(px(8.)))
                    .child(
                        div()
                            .min_w(px(64.))
                            .truncate()
                            .text_size(px(13.))
                            .child(title),
                    ),
            )
            // The save state shows whole or not at all: when it doesn't fit
            // beside the title it wraps onto a second line the band clips.
            // Shrinking it instead left a stub such as "Edi…" while the
            // title was cut short.
            .child(
                h_flex()
                    .flex_1()
                    .min_w(px(12.))
                    .h(px(SAVE_LABEL_HEIGHT))
                    .flex_wrap()
                    .overflow_hidden()
                    .child(div().flex_1().h(px(SAVE_LABEL_HEIGHT)))
                    .child(
                        div()
                            .flex_none()
                            .mr(px(8.))
                            .whitespace_nowrap()
                            .line_height(px(SAVE_LABEL_HEIGHT))
                            .text_size(px(12.))
                            .text_color(saved_color)
                            .child(saved),
                    ),
            )
            .children(backlinks)
            .child(div().flex_none().mx_1().child(mode_group))
            .child(
                Button::new("pin")
                    .ghost()
                    .small()
                    .icon(Icon::new(Lucide::Pin).text_color(if pinned {
                        colors.text
                    } else {
                        colors.text_secondary
                    }))
                    .selected(pinned)
                    .tooltip(if pinned { "Unpin" } else { "Pin to top" })
                    .on_click(cx.listener(|this, _, _, cx| this.toggle_pinned(cx))),
            )
            .child(
                Button::new("history")
                    .ghost()
                    .small()
                    .icon(Icon::new(Lucide::Clock).text_color(colors.text_secondary))
                    .tooltip("History")
                    .on_click(cx.listener(|_, _, _, cx| cx.emit(EditorEvent::ShowHistory))),
            )
            .child(
                Button::new("note-menu")
                    .ghost()
                    .small()
                    .icon(Icon::new(IconName::Ellipsis).text_color(colors.text_secondary))
                    .tooltip("More")
                    .dropdown_menu(move |menu, window, cx| {
                        let view = view.clone();
                        menu.menu("History…", Box::new(crate::workspace::ShowHistory))
                            .menu(
                                "Export as Markdown…",
                                Box::new(crate::workspace::ExportNote),
                            )
                            .submenu("Status", window, cx, move |mut menu, _, _| {
                                for option in NoteStatus::ALL {
                                    let view = view.clone();
                                    menu = menu.item(
                                        PopupMenuItem::new(option.label())
                                            .checked(option == status)
                                            .on_click(move |_, _, cx| {
                                                view.update(cx, |this, cx| {
                                                    this.set_status(option, cx)
                                                });
                                            }),
                                    );
                                }
                                menu
                            })
                            .separator()
                            .menu("Move to Trash", Box::new(crate::workspace::TrashNote))
                    }),
            )
    }

    /// The status chip, the note's tags and "Add tag".
    fn render_meta(&self, note: &Note, cx: &mut Context<Self>) -> impl IntoElement {
        let library = self.library.read(cx);
        let tags: Vec<(String, String, u32)> = library
            .tags
            .iter()
            .map(|t| (t.id.clone(), t.name.clone(), t.color.rgb()))
            .collect();
        let status = note.status;
        let view = cx.entity();
        let colors = Colors::get(cx);
        let dark = cx.theme().is_dark();

        let status_chip = (status != NoteStatus::None).then(|| {
            let view = view.clone();
            Button::new("status")
                .ghost()
                .xsmall()
                .icon(status_icon(status).text_color(colors.text_secondary))
                .child(
                    div()
                        .text_size(px(12.5))
                        .text_color(colors.text_secondary)
                        .child(status.label()),
                )
                .bg(colors.fill)
                .tooltip("Status")
                .dropdown_menu(move |mut menu, _, _| {
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
                })
        });

        let chips = note
            .tags
            .iter()
            .filter_map(|id| {
                let (_, name, rgb) = tags.iter().find(|(tid, _, _)| tid == id)?;
                let library = self.library.clone();
                let view = view.clone();
                let filter = Filter::Tag(id.clone());
                let tag_id = id.clone();
                Some(
                    div()
                        .id(SharedString::from(format!("note-tag-{id}")))
                        .cursor_pointer()
                        .child(
                            Tag::custom(
                                colors.fill.into(),
                                colors.text_secondary.into(),
                                colors.fill.into(),
                            )
                            .text_size(px(12.5))
                            .px(px(8.))
                            .py(px(2.))
                            .child(
                                div()
                                    .text_color(rgb_color(muted_tag_color(*rgb, dark)))
                                    .child("#"),
                            )
                            .child(name.clone()),
                        )
                        .on_click(move |_, _, cx| {
                            library
                                .update(cx, |library, cx| library.set_filter(filter.clone(), cx));
                        })
                        .context_menu(move |menu, _, _| {
                            let view = view.clone();
                            let tag_id = tag_id.clone();
                            menu.item(PopupMenuItem::new("Remove Tag").on_click(move |_, _, cx| {
                                view.update(cx, |this, cx| this.toggle_tag(tag_id.clone(), cx));
                            }))
                        }),
                )
            })
            .collect::<Vec<_>>();

        let add_tag = Button::new("tags")
            .ghost()
            .xsmall()
            .icon(Icon::new(IconName::Plus).text_color(colors.text_muted))
            .child(
                div()
                    .text_size(px(12.5))
                    .text_color(colors.text_muted)
                    .child("Add tag"),
            )
            .debug_selector(|| "add-tag".into())
            // The palette's list: it takes a tag off or puts one on, finds
            // one as you type, and makes a new one from what was typed, even
            // in a library with no tags yet.
            .on_click(cx.listener(|_, _, _, cx| cx.emit(EditorEvent::Pick(Picker::Tag))));

        h_flex()
            .flex_none()
            .flex_wrap()
            .gap(px(6.))
            .px(px(TEXT_INSET))
            .pb(px(6.))
            .children(status_chip)
            .children(chips)
            .child(add_tag)
    }

    /// The formatting toolbar: headings, lists, code, links, images, tables
    /// and quotes, each a toggle or a template at the cursor.
    fn render_format_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let button = |id: &'static str, icon: Icon, tooltip: &'static str, format: Format| {
            Button::new(id)
                .ghost()
                .small()
                .icon(icon.text_color(colors.text_secondary))
                .tooltip(tooltip)
                .on_click(
                    cx.listener(move |this, _, window, cx| this.apply_format(format, window, cx)),
                )
        };
        let separator = || {
            div()
                .flex_none()
                .w(px(1.))
                .h(px(16.))
                .mx(px(6.))
                .bg(colors.divider)
        };
        let view = cx.entity();
        h_flex()
            .id("format-bar")
            .flex_none()
            .h(px(34.))
            .pl(px(TEXT_INSET - 8.))
            .gap(px(2.))
            .child(button(
                "h1",
                Lucide::Heading1.into(),
                "Heading 1",
                Format::Heading(1),
            ))
            .child(button(
                "h2",
                Lucide::Heading2.into(),
                "Heading 2",
                Format::Heading(2),
            ))
            .child(button(
                "h3",
                Lucide::Heading3.into(),
                "Heading 3",
                Format::Heading(3),
            ))
            .child(separator())
            .child(button(
                "bulleted",
                Lucide::List.into(),
                "Bulleted list",
                Format::Bulleted,
            ))
            .child(button(
                "numbered",
                Lucide::ListOrdered.into(),
                "Numbered list",
                Format::Numbered,
            ))
            .child(button(
                "task",
                Lucide::ListTodo.into(),
                "Task",
                Format::Task,
            ))
            .child(button("code", Lucide::Code.into(), "Code", Format::Code))
            .child(button("link", Lucide::Link.into(), "Link", Format::Link))
            .child(button(
                "image",
                Lucide::Image.into(),
                "Image",
                Format::Image,
            ))
            .child(button(
                "table",
                Lucide::Table.into(),
                "Table",
                Format::Table,
            ))
            .child(button(
                "quote",
                Lucide::TextQuote.into(),
                "Quote",
                Format::Quote,
            ))
            .child(separator())
            .child(
                Button::new("format-more")
                    .ghost()
                    .small()
                    .icon(Icon::new(IconName::Ellipsis).text_color(colors.text_secondary))
                    .tooltip("More formatting")
                    .dropdown_menu(move |mut menu, _, _| {
                        for (label, format) in [
                            ("Bold", Format::Bold),
                            ("Italic", Format::Italic),
                            ("Strikethrough", Format::Strikethrough),
                            ("Horizontal Rule", Format::Rule),
                        ] {
                            let view = view.clone();
                            menu = menu.item(PopupMenuItem::new(label).on_click(
                                move |_, window, cx| {
                                    view.update(cx, |this, cx| {
                                        this.apply_format(format, window, cx)
                                    });
                                },
                            ));
                        }
                        menu
                    }),
            )
    }

    /// A quiet strip above the text with a message and its choices.
    fn banner(&self, tint: Hsla, message: &'static str, cx: &mut Context<Self>) -> Div {
        let colors = Colors::get(cx);
        h_flex()
            .flex_none()
            .gap_2()
            .mx(px(TEXT_INSET - 12.))
            .my_1()
            .px(px(12.))
            .py(px(8.))
            .rounded(px(6.))
            .bg(tint)
            .text_size(px(13.))
            .text_color(colors.text)
            .child(div().flex_1().child(message))
    }

    fn render_incoming_banner(&self, deleted: bool, cx: &mut Context<Self>) -> impl IntoElement {
        let (message, keep, take) = if deleted {
            (
                "This note was deleted elsewhere while you were editing.",
                "Keep it",
                "Discard",
            )
        } else {
            (
                "This note was changed elsewhere while you were editing. The version you don't keep stays in its history.",
                "Keep mine",
                "Use theirs",
            )
        };
        let tint = Hsla::from(Colors::get(cx).warning).opacity(0.16);
        self.banner(tint, message, cx)
            .child(
                Button::new("keep-mine")
                    .small()
                    .primary()
                    .label(keep)
                    .on_click(cx.listener(|this, _, _, cx| this.keep_mine(cx))),
            )
            .child(
                Button::new("take-theirs")
                    .small()
                    .label(take)
                    .on_click(cx.listener(|this, _, window, cx| this.take_theirs(window, cx))),
            )
    }

    fn render_trash_banner(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let tint = Colors::get(cx).fill.into();
        self.banner(tint, "This note is in the trash.", cx).child(
            Button::new("restore-from-trash")
                .small()
                .label("Restore")
                .on_click(cx.listener(|this, _, _, cx| {
                    let book = this.library.read(cx).default_book();
                    if let Some(book) = book {
                        this.set_book(book, cx);
                    }
                })),
        )
    }
}

/// What the header says about saving the open note.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SaveState {
    /// Typing paused; the save is about to run.
    Saving,
    /// The editor holds text the note doesn't: a change made elsewhere is
    /// waiting for an answer, or a save failed.
    NotSaved,
    Saved,
}

/// What the header says, from whether a change made elsewhere is waiting,
/// whether a save is pending, and (asked only when neither) whether the
/// editor's text differs from the note's.
// no-prop-test: three flags; `save_states` checks all of them.
fn save_state(conflict: bool, pending: bool, unsaved: impl FnOnce() -> bool) -> SaveState {
    if conflict {
        SaveState::NotSaved
    } else if pending {
        SaveState::Saving
    } else if unsaved() {
        SaveState::NotSaved
    } else {
        SaveState::Saved
    }
}

fn status_icon(status: NoteStatus) -> Icon {
    match status {
        NoteStatus::Active => Lucide::CircleDot.into(),
        NoteStatus::OnHold => Lucide::CirclePause.into(),
        NoteStatus::Completed => IconName::CircleCheck.into(),
        NoteStatus::Dropped => IconName::CircleX.into(),
        NoteStatus::None => IconName::File.into(),
    }
}

pub fn rgb_color(rgb: u32) -> Hsla {
    gpui_kit::rgb(rgb).into()
}

impl Render for NoteEditor {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // The face follows the view mode and the font setting, however they
        // changed.
        let face = self
            .editor_font
            .face(self.view_mode)
            .with_line_numbers(self.allow_line_numbers);
        if face.line_numbers != self.line_numbers {
            self.line_numbers = face.line_numbers;
            self.body.update(cx, |body, cx| {
                body.set_line_number(face.line_numbers, window, cx);
                body.set_folding(face.line_numbers, window, cx);
            });
        }
        let colors = Colors::get(cx);
        let Some(note) = self.note.clone() else {
            let pane = v_flex()
                .flex_1()
                .min_h_0()
                .w_full()
                .items_center()
                .justify_center()
                .track_focus(&self.empty_focus);
            let content = match self.library.read(cx).empty_place() {
                // Centred a little above the middle, as the list's is: the
                // top band's room less the bottom band's.
                Some(title) => pane
                    .px(px(24.))
                    .pb(px(TOP_BAND_HEIGHT - BOTTOM_BAND_HEIGHT))
                    .child(EmptyNotebook::titled(title, self.empty_focus.clone())),
                None => pane
                    .gap_2()
                    .text_color(colors.text_muted)
                    .child("No note selected")
                    .child(div().text_xs().child("⌘N creates a note")),
            };
            // The bottom band runs on under the editor with no note too, so
            // the bands line up across the panes.
            return v_flex()
                .size_full()
                .bg(colors.editor)
                .child(content)
                .child(
                    AnyView::from(self.footer.clone()).cached(
                        StyleRefinement::default()
                            .w_full()
                            .flex_none()
                            .h(px(BOTTOM_BAND_HEIGHT)),
                    ),
                )
                .into_any_element();
        };
        let trashed = note.is_trashed();
        let mode = self.view_mode;
        let family = if face.mono {
            cx.theme().mono_font_family.clone()
        } else {
            cx.theme().font_family.clone()
        };
        let size = self.font_size;
        // Alone, the source keeps to a readable measure for its face and
        // size, starting under the title; beside the preview it takes its
        // half.
        let measure = (mode == ViewMode::Edit).then(|| {
            let numbers = if face.line_numbers {
                LINE_NUMBER_COLUMN
            } else {
                0.
            };
            px(readable_width(size, face.mono) + SOURCE_EDGES + numbers)
        });
        v_flex()
            .size_full()
            .bg(colors.editor)
            .child(self.render_band(&note, cx))
            .child(
                div()
                    .flex_none()
                    .pl(px(TEXT_INSET))
                    .pr_4()
                    .pb(px(8.))
                    .child(
                        Input::new(&self.title)
                            .appearance(false)
                            .text_size(px(28.))
                            .line_height(px(36.))
                            .h(px(40.))
                            .p_0()
                            .font_semibold(),
                    ),
            )
            .child(self.render_meta(&note, cx))
            .when(mode == ViewMode::Edit && !trashed, |el| {
                el.child(self.render_format_bar(cx))
            })
            .when(trashed, |el| el.child(self.render_trash_banner(cx)))
            .when_some(self.incoming.as_ref(), |el, incoming| {
                let deleted = matches!(incoming, Incoming::Deleted);
                el.child(self.render_incoming_banner(deleted, cx))
            })
            .child(
                h_flex()
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .when(mode != ViewMode::Preview, |el| {
                        el.child(
                            div()
                                .flex_1()
                                .h_full()
                                .min_w_0()
                                .pl(px(TEXT_INSET - SOURCE_TEXT_LEFT))
                                .pr_4()
                                .py_2()
                                // The margin beside a narrowed source scrolls
                                // it too; over the text, the editor takes the
                                // wheel first.
                                .when(measure.is_some(), |el| {
                                    el.on_scroll_wheel(cx.listener(
                                        |this, event: &ScrollWheelEvent, _, cx| {
                                            this.body.update(cx, |body, cx| {
                                                let line = body.line_height().unwrap_or(px(24.));
                                                let offset = body.scroll_offset()
                                                    + event.delta.pixel_delta(line);
                                                body.set_scroll_offset(offset, cx);
                                            });
                                        },
                                    ))
                                })
                                .child(
                                    div()
                                        .h_full()
                                        .w_full()
                                        .when_some(measure, |el, width| el.max_w(width))
                                        .child(
                                            Editor::new(&self.body)
                                                .bordered(false)
                                                .h_full()
                                                .font_family(family)
                                                .text_size(px(size))
                                                .line_height(relative(BODY_LINE_HEIGHT)),
                                        ),
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
            // Redrawn only when the cursor or the counts change.
            .child(
                AnyView::from(self.footer.clone()).cached(
                    StyleRefinement::default()
                        .w_full()
                        .flex_none()
                        .h(px(BOTTOM_BAND_HEIGHT)),
                ),
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

    /// How far the body is scrolled.
    pub fn body_scroll_for_test(&self, cx: &App) -> Point<Pixels> {
        self.body.read(cx).scroll_offset()
    }

    pub fn scroll_body_for_test(&self, offset: Point<Pixels>, cx: &mut App) {
        self.body
            .update(cx, |body, cx| body.set_scroll_offset(offset, cx));
    }

    pub fn has_incoming(&self) -> bool {
        self.incoming.is_some()
    }

    pub fn title_focused(&self, window: &Window, cx: &App) -> bool {
        self.title.read(cx).focus_handle(cx).is_focused(window)
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
    use super::{NoteEditor, SaveState, backlinks, save_state};
    use crate::config::Appearance;
    use crate::library::Library;
    use crate::markdown_edit::Format;
    use gpui_kit::{AppContext as _, TestAppContext};
    use kanso_core::DocType;
    use kanso_store::{Origin, Put, Store};
    use proptest::prelude::*;
    use serde_json::json;
    use std::sync::Arc;
    use std::time::Duration;

    /// The toolbar edits the body the way typing does: at the selection,
    /// saved after the pause, and counted in the footer once typing rests.
    // no-prop-test: glue between the toolbar, the text editor and autosave;
    // the edits themselves are property-tested in `markdown_edit`.
    #[gpui_kit::test]
    fn formatting_edits_the_body_and_saves(cx: &mut TestAppContext) {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(Store::open(dir.path().join("k.db")).unwrap());
        let book = store
            .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
            .unwrap()
            .id;
        let id = store
            .put(
                DocType::Note,
                json!({"title": "t", "body": "Goals\nnext", "bookId": book}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        let editor_store = store.clone();
        let (window, editor) = cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit::open_window(Default::default(), cx, |window, cx| {
                let library = cx.new(|cx| Library::new(editor_store.clone(), cx));
                let (links, _) = async_channel::unbounded();
                cx.new(|cx| {
                    NoteEditor::new(
                        editor_store,
                        library,
                        links,
                        &Appearance::default(),
                        window,
                        cx,
                    )
                })
            })
            .unwrap()
        });
        let edit = |cx: &mut TestAppContext,
                    f: &dyn Fn(
            &mut NoteEditor,
            &mut gpui_kit::Window,
            &mut gpui_kit::Context<NoteEditor>,
        )| {
            window
                .update(cx, |_, window, cx| {
                    editor.update(cx, |editor, cx| f(editor, window, cx))
                })
                .unwrap();
            cx.executor().advance_clock(Duration::from_secs(1));
            cx.run_until_parked();
        };
        edit(cx, &|editor, window, cx| editor.open(Some(&id), window, cx));
        edit(cx, &|editor, window, cx| {
            editor.apply_format(Format::Heading(2), window, cx)
        });
        assert_eq!(store.note(&id).unwrap().unwrap().body, "## Goals\nnext");
        edit(cx, &|editor, window, cx| {
            editor
                .body
                .update(cx, |body, cx| body.set_selected_range(9..13, cx));
            editor.apply_format(Format::Bold, window, cx);
        });
        assert_eq!(store.note(&id).unwrap().unwrap().body, "## Goals\n**next**");
        let label = cx.read(|cx| editor.read(cx).footer.read(cx).label());
        assert!(label.ends_with("2 words · 17 chars"), "{label}");
    }

    #[test]
    fn save_states() {
        for unsaved in [false, true] {
            for pending in [false, true] {
                assert_eq!(save_state(true, pending, || unsaved), SaveState::NotSaved);
            }
            assert_eq!(save_state(false, true, || unsaved), SaveState::Saving);
        }
        assert_eq!(save_state(false, false, || true), SaveState::NotSaved);
        assert_eq!(save_state(false, false, || false), SaveState::Saved);
    }

    /// While a change made elsewhere waits for an answer the header says
    /// the typing isn't saved, and no autosave pretends otherwise; keeping
    /// the editor's text saves it.
    #[gpui_kit::test]
    fn a_waiting_conflict_reads_not_saved(cx: &mut TestAppContext) {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(Store::open(dir.path().join("k.db")).unwrap());
        let book = store
            .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
            .unwrap()
            .id;
        let id = store
            .put(
                DocType::Note,
                json!({"title": "t", "body": "one", "bookId": book}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        let editor_store = store.clone();
        let (window, editor) = cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit::open_window(Default::default(), cx, |window, cx| {
                let library = cx.new(|cx| Library::new(editor_store.clone(), cx));
                let (links, _) = async_channel::unbounded();
                cx.new(|cx| {
                    NoteEditor::new(
                        editor_store,
                        library,
                        links,
                        &Appearance::default(),
                        window,
                        cx,
                    )
                })
            })
            .unwrap()
        });
        let state = |cx: &mut TestAppContext| cx.read(|cx| editor.read(cx).save_state(cx));
        window
            .update(cx, |_, window, cx| {
                editor.update(cx, |editor, cx| {
                    editor.open(Some(&id), window, cx);
                    editor.edit_for_test("one, typed", window, cx);
                })
            })
            .unwrap();
        assert_eq!(state(cx), SaveState::Saving);
        // Saved elsewhere before the pause ends.
        store
            .put(
                DocType::Note,
                json!({"_id": id, "body": "theirs"}),
                Put::Merge,
                Origin::Api,
            )
            .unwrap();
        window
            .update(cx, |_, window, cx| {
                let change = kanso_store::Change {
                    id: id.clone(),
                    rev: store.note(&id).unwrap().unwrap().rev.unwrap(),
                    seq: 0,
                    deleted: false,
                    origin: Origin::Api,
                };
                editor.update(cx, |editor, cx| editor.on_store_change(&change, window, cx));
            })
            .unwrap();
        assert_eq!(state(cx), SaveState::NotSaved);
        window
            .update(cx, |_, window, cx| {
                editor.update(cx, |editor, cx| {
                    editor.edit_for_test("one, typed more", window, cx)
                })
            })
            .unwrap();
        cx.executor().advance_clock(Duration::from_secs(2));
        cx.run_until_parked();
        assert_eq!(state(cx), SaveState::NotSaved);
        assert_eq!(store.note(&id).unwrap().unwrap().body, "theirs");
        editor.update(cx, |editor, cx| editor.keep_mine(cx));
        assert_eq!(state(cx), SaveState::Saved);
        assert_eq!(store.note(&id).unwrap().unwrap().body, "one, typed more");
    }

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
