//! The Tasks view, in the note list's place: every task-list item across
//! the notes, grouped Overdue, Today, Upcoming and No date (and Completed,
//! when shown), each with its note and notebook. A task is ticked off in
//! place (its note's text changes in one write, with ⌘Z to take it back)
//! and opens its note at its line.
//!
//! The grouping, the selection and the check-offs are the [`Board`], which
//! knows nothing of drawing, so a model-based test can drive it against
//! the store.

use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenu, PopupMenuItem};
use gpui_kit::component::{
    Icon, IconName, Selectable as _, Sizable as _, StyledExt as _, VirtualListScrollHandle, h_flex,
    v_flex, v_virtual_list,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_core::dates::DateTime;
use kanso_core::links::plain_line;
use kanso_core::tasks::{Due, TaskGroup, due_token, task_group};
use kanso_store::{CheckOff, Origin, Store, StoreError, TaskQuery, TaskRow};

use crate::clock::Clock;
use crate::due::{Tone, chip_colors, chip_label, tone};
use crate::editor::NoteEditor;
use crate::layout::{BOTTOM_BAND_HEIGHT, TOP_BAND_HEIGHT};
use crate::library::{Filter, Library, LibraryEvent};
use crate::note_list::quiet_empty;
use crate::theme::{Colors, section_label};

/// A task row: padding, its words, a gap, its note and notebook, padding.
const ROW_HEIGHT: f32 = 54.;
const HEADER_HEIGHT: f32 = 32.;
const ROW_INSET_X: f32 = 8.;
const ROW_PAD_X: f32 = 10.;
/// How long changes to notes settle before the tasks are read again.
const RELOAD_DELAY: Duration = Duration::from_millis(120);
/// How often the groups are worked out again (a task falls overdue as
/// time passes) and the overdue count read again.
const TICK: Duration = Duration::from_secs(60);

actions!(
    tasks_view,
    [
        SelectNextTask,
        SelectPreviousTask,
        ToggleTask,
        OpenTask,
        UndoCheckOff
    ]
);

pub fn bind_keys(cx: &mut App) {
    let context = Some("TasksView");
    cx.bind_keys([
        KeyBinding::new("down", SelectNextTask, context),
        KeyBinding::new("j", SelectNextTask, context),
        KeyBinding::new("up", SelectPreviousTask, context),
        KeyBinding::new("k", SelectPreviousTask, context),
        KeyBinding::new("space", ToggleTask, context),
        KeyBinding::new("x", ToggleTask, context),
        KeyBinding::new("enter", OpenTask, context),
        KeyBinding::new("secondary-z", UndoCheckOff, context),
    ]);
}

/// A task as the list knows it: its note, its line and its words.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TaskId {
    pub note_id: String,
    pub line: u32,
    pub text: String,
}

impl TaskId {
    fn of(row: &TaskRow) -> TaskId {
        TaskId {
            note_id: row.note_id.clone(),
            line: row.line,
            text: row.text.clone(),
        }
    }
}

/// A line of the list: a group's header or a task (an index into the
/// board's rows).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Entry {
    Header {
        group: TaskGroup,
        count: usize,
        collapsed: bool,
    },
    Task(usize),
}

/// A check-off that ⌘Z can take back: the task and the state it was given.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedOff {
    pub id: TaskId,
    pub checked: bool,
}

/// What a check-off did, for the view to say.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Toggled {
    Done(CheckedOff),
    /// Nothing to do: no task selected, or it already was that way.
    Nothing,
    /// The task's note changed since the list was read: it is read again.
    Moved,
    Failed(String),
}

/// The tasks, how they are grouped and filtered, the selection and the
/// last check-off. Pure apart from the store calls it is handed.
#[derive(Default)]
pub struct Board {
    pub rows: Vec<TaskRow>,
    pub loaded: bool,
    /// Only tasks in this notebook's notes.
    pub book: Option<String>,
    pub show_done: bool,
    pub collapsed: HashSet<TaskGroup>,
    pub selected: Option<TaskId>,
    /// Ticked off here since the view opened: they stay in their group,
    /// struck through, rather than vanish under the pointer.
    pub kept: HashSet<TaskId>,
    pub last: Option<CheckedOff>,
}

impl Board {
    /// The group a row shows in at local time `now`.
    pub fn group_of(&self, row: &TaskRow, now: DateTime) -> TaskGroup {
        let due = row.due.as_deref().and_then(Due::parse);
        let kept = !self.show_done && row.checked && self.kept.contains(&TaskId::of(row));
        task_group(due, row.checked && !kept, now)
    }

    /// Whether a row is listed at all: in the notebook asked for, and open
    /// (or done, when done ones show, or ticked off here).
    pub fn listed(&self, row: &TaskRow) -> bool {
        self.book.as_ref().is_none_or(|book| row.book_id == *book)
            && (!row.checked || self.show_done || self.kept.contains(&TaskId::of(row)))
    }

    /// The list's lines at local time `now`: each group with tasks, in
    /// order, its header and (unless collapsed) its tasks. Pure.
    pub fn entries(&self, now: DateTime) -> Vec<Entry> {
        let mut by_group: HashMap<TaskGroup, Vec<usize>> = HashMap::new();
        for (ix, row) in self.rows.iter().enumerate() {
            if self.listed(row) {
                by_group
                    .entry(self.group_of(row, now))
                    .or_default()
                    .push(ix);
            }
        }
        let mut entries = Vec::new();
        for group in TaskGroup::ALL {
            let Some(tasks) = by_group.remove(&group) else {
                continue;
            };
            let collapsed = self.collapsed.contains(&group);
            entries.push(Entry::Header {
                group,
                count: tasks.len(),
                collapsed,
            });
            if !collapsed {
                entries.extend(tasks.into_iter().map(Entry::Task));
            }
        }
        entries
    }

    /// The selected task's place in `entries`, if it shows.
    pub fn selected_entry(&self, entries: &[Entry]) -> Option<usize> {
        let selected = self.selected.as_ref()?;
        entries.iter().position(|entry| match entry {
            Entry::Task(ix) => TaskId::of(&self.rows[*ix]) == *selected,
            Entry::Header { .. } => false,
        })
    }

    /// Moves the selection `delta` tasks down (or up), over headers; the
    /// first task when none was selected.
    pub fn step(&mut self, delta: isize, now: DateTime) {
        let entries = self.entries(now);
        let tasks: Vec<usize> = entries
            .iter()
            .filter_map(|e| match e {
                Entry::Task(ix) => Some(*ix),
                Entry::Header { .. } => None,
            })
            .collect();
        if tasks.is_empty() {
            self.selected = None;
            return;
        }
        let current = self.selected.as_ref().and_then(|id| {
            tasks
                .iter()
                .position(|&ix| TaskId::of(&self.rows[ix]) == *id)
        });
        let next = match current {
            Some(at) => at.saturating_add_signed(delta).min(tasks.len() - 1),
            None => 0,
        };
        self.selected = Some(TaskId::of(&self.rows[tasks[next]]));
    }

    /// The selected task's row.
    pub fn selected_row(&self) -> Option<&TaskRow> {
        let id = self.selected.as_ref()?;
        self.rows.iter().find(|row| TaskId::of(row) == *id)
    }

    /// Takes newly read rows, keeping the selection on its task.
    pub fn set_rows(&mut self, rows: Vec<TaskRow>) {
        self.rows = rows;
        self.loaded = true;
        if let Some(id) = &self.selected
            && !self.rows.iter().any(|row| TaskId::of(row) == *id)
        {
            // The task's line moved: follow it by its words in its note.
            let moved = self
                .rows
                .iter()
                .find(|row| row.note_id == id.note_id && row.text == id.text)
                .map(TaskId::of);
            self.selected = moved;
        }
    }

    /// Ticks the task (or clears it) in its note, in one write, and
    /// remembers it for [`Board::undo`].
    pub fn toggle(&mut self, id: &TaskId, store: &Store, now_ms: i64) -> Toggled {
        let Some(row) = self.rows.iter().find(|row| TaskId::of(row) == *id) else {
            return Toggled::Nothing;
        };
        let checked = !row.checked;
        match self.set(id, checked, store, now_ms) {
            Toggled::Done(done) => {
                self.last = Some(done.clone());
                Toggled::Done(done)
            }
            other => other,
        }
    }

    /// Takes the last check-off back.
    pub fn undo(&mut self, store: &Store, now_ms: i64) -> Toggled {
        let Some(last) = self.last.take() else {
            return Toggled::Nothing;
        };
        self.set(&last.id, !last.checked, store, now_ms)
    }

    fn set(&mut self, id: &TaskId, checked: bool, store: &Store, now_ms: i64) -> Toggled {
        match store.set_task_checked(&id.note_id, id.line, &id.text, checked, now_ms, Origin::Ui) {
            Ok(CheckOff::Changed { .. }) => {
                for row in self.rows.iter_mut().filter(|row| TaskId::of(row) == *id) {
                    row.checked = checked;
                }
                if checked {
                    self.kept.insert(id.clone());
                }
                Toggled::Done(CheckedOff {
                    id: id.clone(),
                    checked,
                })
            }
            Ok(CheckOff::Unchanged) => Toggled::Nothing,
            Ok(CheckOff::Missing) | Err(StoreError::NotFound) => Toggled::Moved,
            Err(err) => Toggled::Failed(err.to_string()),
        }
    }
}

/// A task's words as the list shows them: plain, without its due date
/// (the chip beside it says that). Pure.
pub fn display_text(text: &str) -> String {
    let without = match due_token(text) {
        Some(range) => format!("{}{}", &text[..range.start], &text[range.end..]),
        None => text.to_owned(),
    };
    let plain = plain_line(&without);
    let plain = plain.split_whitespace().collect::<Vec<_>>().join(" ");
    if plain.is_empty() {
        text.trim().to_owned()
    } else {
        plain
    }
}

/// The workspace opens notes.
pub enum TasksEvent {
    /// Open the task's note at its line; `focus` puts the keyboard in it.
    Open {
        note_id: String,
        line: u32,
        focus: bool,
    },
    /// Something to tell the user.
    Notice(String),
}

pub struct TasksView {
    library: Entity<Library>,
    editor: Entity<NoteEditor>,
    store: Arc<Store>,
    pub(crate) board: Board,
    focus: FocusHandle,
    scroll: VirtualListScrollHandle,
    leading_inset: Pixels,
    /// The rows were read before the last change to the notes.
    stale: bool,
    reload_task: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
    _tick: Task<()>,
}

impl EventEmitter<TasksEvent> for TasksView {}

impl Focusable for TasksView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl TasksView {
    pub fn new(
        library: Entity<Library>,
        editor: Entity<NoteEditor>,
        cx: &mut Context<Self>,
    ) -> Self {
        let store = library.read(cx).store.clone();
        let subscriptions = vec![cx.subscribe(&library, |this, library, event, cx| {
            match event {
                LibraryEvent::Changed(change) if change.id.starts_with("note:") => {
                    this.stale = true;
                    if library.read(cx).filter == Filter::Tasks {
                        this.reload_soon(cx);
                    }
                }
                LibraryEvent::Shown if library.read(cx).filter == Filter::Tasks => {
                    // Opened afresh: what was ticked off last time goes.
                    this.board.kept.clear();
                    this.reload(cx);
                }
                _ => {}
            }
        })];
        let tick = cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(TICK).await;
                let alive = this.update(cx, |this, cx| {
                    this.library
                        .update(cx, |library, cx| library.refresh_overdue(cx));
                    cx.notify();
                });
                if alive.is_err() {
                    break;
                }
            }
        });
        let mut view = TasksView {
            library,
            editor,
            store,
            board: Board::default(),
            focus: cx.focus_handle(),
            scroll: VirtualListScrollHandle::new(),
            leading_inset: px(0.),
            stale: true,
            reload_task: None,
            _subscriptions: subscriptions,
            _tick: tick,
        };
        view.reload(cx);
        view
    }

    pub fn set_leading_inset(&mut self, inset: Pixels, cx: &mut Context<Self>) {
        if self.leading_inset != inset {
            self.leading_inset = inset;
            cx.notify();
        }
    }

    pub fn focus_list(&self, window: &mut Window, cx: &mut App) {
        window.focus(&self.focus, cx);
    }

    fn reload_soon(&mut self, cx: &mut Context<Self>) {
        self.reload_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(RELOAD_DELAY).await;
            this.update(cx, |this, cx| this.reload(cx)).ok();
        }));
    }

    /// Reads every task again, off the main thread.
    pub fn reload(&mut self, cx: &mut Context<Self>) {
        self.stale = false;
        let store = self.store.clone();
        let task = cx.background_executor().spawn(async move {
            store.tasks(&TaskQuery {
                include_done: true,
                ..Default::default()
            })
        });
        self.reload_task = Some(cx.spawn(async move |this, cx| {
            let rows = task.await;
            this.update(cx, |this, cx| {
                match rows {
                    Ok(rows) => this.board.set_rows(rows),
                    Err(err) => cx.emit(TasksEvent::Notice(format!(
                        "Couldn't read the tasks: {err}"
                    ))),
                }
                cx.notify();
            })
            .ok();
        }));
    }

    /// Reads the tasks now, on this thread: for tests and snapshots.
    #[cfg(test)]
    pub fn reload_now(&mut self, cx: &mut Context<Self>) {
        let rows = self
            .store
            .tasks(&TaskQuery {
                include_done: true,
                ..Default::default()
            })
            .unwrap_or_default();
        self.board.set_rows(rows);
        cx.notify();
    }

    fn now(cx: &App) -> DateTime {
        crate::journal::now(cx)
    }

    fn select_step(&mut self, delta: isize, cx: &mut Context<Self>) {
        self.board.step(delta, Self::now(cx));
        let entries = self.board.entries(Self::now(cx));
        if let Some(at) = self.board.selected_entry(&entries) {
            self.scroll.scroll_to_item(at, ScrollStrategy::Nearest);
        }
        cx.notify();
    }

    /// Ticks (or clears) task `id` in its note. The note's unsaved typing
    /// is saved first, so the write goes on top of it.
    pub fn toggle(&mut self, id: &TaskId, cx: &mut Context<Self>) {
        if writes_held(cx) {
            cx.emit(TasksEvent::Notice(
                "Tasks can't be changed while the library is being replaced.".into(),
            ));
            return;
        }
        self.editor.update(cx, |editor, cx| editor.flush(cx));
        let outcome = self.board.toggle(id, &self.store, Clock::now_ms(cx));
        self.after(outcome, cx);
    }

    fn undo(&mut self, cx: &mut Context<Self>) {
        if writes_held(cx) {
            return;
        }
        self.editor.update(cx, |editor, cx| editor.flush(cx));
        let outcome = self.board.undo(&self.store, Clock::now_ms(cx));
        self.after(outcome, cx);
    }

    fn after(&mut self, outcome: Toggled, cx: &mut Context<Self>) {
        match outcome {
            Toggled::Done(_) | Toggled::Nothing => {}
            Toggled::Moved => {
                cx.emit(TasksEvent::Notice(
                    "That task changed in its note. The list is up to date now.".into(),
                ));
                self.reload(cx);
            }
            Toggled::Failed(err) => cx.emit(TasksEvent::Notice(format!(
                "Couldn't change the task: {err}"
            ))),
        }
        cx.notify();
    }

    fn open(&mut self, id: &TaskId, focus: bool, cx: &mut Context<Self>) {
        self.board.selected = Some(id.clone());
        cx.emit(TasksEvent::Open {
            note_id: id.note_id.clone(),
            line: id.line,
            focus,
        });
        cx.notify();
    }

    fn toggle_group(&mut self, group: TaskGroup, cx: &mut Context<Self>) {
        if !self.board.collapsed.remove(&group) {
            self.board.collapsed.insert(group);
        }
        cx.notify();
    }

    fn set_book(&mut self, book: Option<String>, cx: &mut Context<Self>) {
        self.board.book = book;
        cx.notify();
    }

    fn set_show_done(&mut self, show: bool, cx: &mut Context<Self>) {
        self.board.show_done = show;
        cx.notify();
    }

    // MARK: Drawing

    fn render_top_band(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let open = self
            .board
            .rows
            .iter()
            .filter(|r| !r.checked && self.board.listed(r))
            .count();
        let library = self.library.read(cx);
        let book_name = self
            .board
            .book
            .as_ref()
            .and_then(|id| library.book(id))
            .map(|b| b.name.clone());
        let refined = self.board.book.is_some() || self.board.show_done;
        let view = cx.entity();
        let filter = Button::new("tasks-filter")
            .ghost()
            .small()
            .icon(
                Icon::new(Lucide::ListFilter)
                    .size(px(15.))
                    .text_color(colors.text_secondary),
            )
            .selected(refined)
            .tooltip("Filter tasks")
            .dropdown_menu_with_anchor(Anchor::TopRight, move |menu, _, cx| {
                filter_menu(menu, &view, cx)
            });
        h_flex()
            .h(px(TOP_BAND_HEIGHT))
            .flex_shrink_0()
            .pl(px(16.) + self.leading_inset)
            .pr(px(10.))
            .gap_1p5()
            .child(
                div()
                    .text_size(px(15.))
                    .font_semibold()
                    .text_color(colors.text)
                    .child("Tasks"),
            )
            .child(
                div()
                    .flex_shrink_0()
                    .text_size(px(12.))
                    .text_color(colors.text_muted)
                    .child(open.to_string()),
            )
            .when_some(book_name, |el, name| {
                el.child(
                    div()
                        .min_w_0()
                        .flex_shrink(1000.)
                        .truncate()
                        .text_size(px(12.))
                        .text_color(colors.text_muted)
                        .child(format!("· {name}")),
                )
            })
            .child(div().flex_1())
            .child(filter)
    }

    fn render_header(
        &self,
        group: TaskGroup,
        count: usize,
        collapsed: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let colors = Colors::get(cx);
        let tint = if group == TaskGroup::Overdue {
            colors.danger
        } else {
            colors.text_muted
        };
        h_flex()
            .id(SharedString::from(format!("tasks-group-{}", group.label())))
            .h(px(HEADER_HEIGHT))
            .w_full()
            .px(px(ROW_INSET_X + ROW_PAD_X - 4.))
            .pt(px(8.))
            .gap(px(4.))
            .cursor_pointer()
            .child(
                Icon::new(if collapsed {
                    IconName::ChevronRight
                } else {
                    IconName::ChevronDown
                })
                .size(px(13.))
                .text_color(colors.text_muted),
            )
            .child(
                section_label(div(), colors)
                    .text_color(tint)
                    .child(group.label()),
            )
            .child(
                div()
                    .text_size(px(11.5))
                    .text_color(colors.text_muted)
                    .child(count.to_string()),
            )
            .on_click(cx.listener(move |this, _, _, cx| this.toggle_group(group, cx)))
            .into_any_element()
    }

    fn render_task(
        &self,
        ix: usize,
        selected: bool,
        focused: bool,
        now: DateTime,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let colors = Colors::get(cx);
        let row = &self.board.rows[ix];
        let id = TaskId::of(row);
        let checked = row.checked;
        let due = row.due.as_deref().and_then(Due::parse);
        let library = self.library.read(cx);
        let book = library
            .book(&row.book_id)
            .map(|b| b.name.clone())
            .unwrap_or_default();
        let title = if row.note_title.trim().is_empty() {
            "Untitled".to_owned()
        } else {
            row.note_title.clone()
        };
        let text = display_text(&row.text);
        let toggle_id = id.clone();
        let open_id = id.clone();
        let check = Icon::new(if checked {
            Lucide::CircleCheck
        } else {
            Lucide::Circle
        })
        .size(px(16.))
        .text_color(if checked {
            colors.success
        } else {
            colors.text_muted
        });
        let chip = due.map(|due| {
            let tone = tone(due, checked, now);
            let (text, fill) = chip_colors(tone, colors);
            div()
                .flex_shrink_0()
                .px(px(6.))
                .h(px(18.))
                .flex()
                .items_center()
                .rounded(px(5.))
                .bg(fill)
                .text_size(px(11.))
                .font_medium()
                .text_color(text)
                .when(tone == Tone::Done, |el| el.line_through())
                .child(chip_label(due, checked, now))
        });
        div()
            .id(SharedString::from(format!(
                "task-{}-{}",
                row.note_id, row.line
            )))
            .h(px(ROW_HEIGHT))
            .w_full()
            .px(px(ROW_INSET_X))
            .py(px(1.))
            .child(
                h_flex()
                    .size_full()
                    .px(px(ROW_PAD_X))
                    .gap(px(10.))
                    .items_start()
                    .pt(px(9.))
                    .rounded(px(7.))
                    .when(selected, |el| el.bg(colors.row_active))
                    .when(!selected, |el| el.hover(|el| el.bg(colors.row_hover)))
                    .when(selected && focused, |el| {
                        el.border_1()
                            .border_color(Hsla::from(colors.accent).opacity(0.5))
                    })
                    .child(
                        div()
                            .id(SharedString::from(format!(
                                "task-check-{}-{}",
                                row.note_id, row.line
                            )))
                            .flex_none()
                            .pt(px(1.))
                            .cursor_pointer()
                            .child(check)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                cx.stop_propagation();
                                this.toggle(&toggle_id, cx);
                            })),
                    )
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .gap(px(3.))
                            .child(
                                h_flex()
                                    .gap(px(8.))
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_w_0()
                                            .truncate()
                                            .text_size(px(13.5))
                                            .line_height(px(18.))
                                            .text_color(if checked {
                                                colors.text_muted
                                            } else {
                                                colors.text
                                            })
                                            .when(checked, |el| el.line_through())
                                            .child(text),
                                    )
                                    .children(chip),
                            )
                            .child(
                                div()
                                    .truncate()
                                    .text_size(px(12.))
                                    .line_height(px(16.))
                                    .text_color(colors.quiet(selected))
                                    .child(if book.is_empty() {
                                        title
                                    } else {
                                        format!("{title} · {book}")
                                    }),
                            ),
                    ),
            )
            .on_click(cx.listener(move |this, _, window, cx| {
                window.focus(&this.focus, cx);
                this.open(&open_id, false, cx);
            }))
            .into_any_element()
    }

    fn render_rows(
        &mut self,
        range: std::ops::Range<usize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Vec<AnyElement> {
        let now = Self::now(cx);
        let entries = self.board.entries(now);
        let selected = self.board.selected_entry(&entries);
        let focused = self.focus.contains_focused(window, cx);
        range
            .filter_map(|at| {
                Some(match *entries.get(at)? {
                    Entry::Header {
                        group,
                        count,
                        collapsed,
                    } => self.render_header(group, count, collapsed, cx),
                    Entry::Task(ix) => self.render_task(ix, selected == Some(at), focused, now, cx),
                })
            })
            .collect()
    }

    fn render_empty(&self, cx: &App) -> AnyElement {
        let colors = Colors::get(cx);
        if !self.board.loaded {
            return div().into_any_element();
        }
        let any = self.board.rows.iter().any(|r| {
            self.board
                .book
                .as_ref()
                .is_none_or(|book| r.book_id == *book)
        });
        let (title, detail) = if !any && self.board.book.is_some() {
            let name = self
                .board
                .book
                .as_ref()
                .and_then(|id| self.library.read(cx).book(id))
                .map_or("this notebook".to_owned(), |b| b.name.clone());
            (format!("No tasks in {name}"), None)
        } else if !any {
            (
                "No tasks yet".to_owned(),
                Some(
                    "Write “- [ ] Call Sam @2026-10-25” in any note, and it shows here.".to_owned(),
                ),
            )
        } else {
            (
                "All done".to_owned(),
                Some("Every task is ticked off.".to_owned()),
            )
        };
        quiet_empty(Lucide::ListChecks, title, detail, colors).into_any_element()
    }

    fn render_bottom_band(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let last = self.board.last.clone();
        h_flex()
            .h(px(BOTTOM_BAND_HEIGHT))
            .flex_shrink_0()
            .px(px(12.))
            .gap_1()
            .border_t_1()
            .border_color(colors.divider)
            .text_size(px(12.))
            .text_color(colors.text_muted)
            .map(|el| match last {
                Some(last) => el
                    .child(div().flex_1().min_w_0().truncate().child(if last.checked {
                        format!("Ticked off “{}”", display_text(&last.id.text))
                    } else {
                        format!("Opened “{}” again", display_text(&last.id.text))
                    }))
                    .child(
                        Button::new("tasks-undo")
                            .ghost()
                            .xsmall()
                            .label("Undo")
                            .tooltip_with_action("Undo", &UndoCheckOff, Some("TasksView"))
                            .on_click(cx.listener(|this, _, _, cx| this.undo(cx))),
                    ),
                None => el.child(
                    div()
                        .flex_1()
                        .truncate()
                        .child("Space ticks off · Return opens the note"),
                ),
            })
    }
}

fn writes_held(cx: &App) -> bool {
    cx.try_global::<crate::services::Services>()
        .is_some_and(|services| services.writes_held)
}

/// The filter menu: which notebook's tasks, and whether done ones show.
fn filter_menu(menu: PopupMenu, view: &Entity<TasksView>, cx: &App) -> PopupMenu {
    let board = &view.read(cx).board;
    let current = board.book.clone();
    let show_done = board.show_done;
    let library = view.read(cx).library.read(cx);
    let books: Vec<(usize, String, String)> = library
        .book_tree()
        .into_iter()
        .map(|(depth, b)| (depth, b.id.clone(), b.name.clone()))
        .collect();
    let all = view.clone();
    let mut menu = menu.item(
        PopupMenuItem::new("All notebooks")
            .checked(current.is_none())
            .on_click(move |_, _, cx| all.update(cx, |this, cx| this.set_book(None, cx))),
    );
    for (depth, id, name) in books {
        let view = view.clone();
        let label = format!("{}{name}", "   ".repeat(depth));
        let checked = current.as_deref() == Some(id.as_str());
        menu = menu.item(
            PopupMenuItem::new(label)
                .checked(checked)
                .on_click(move |_, _, cx| {
                    let id = id.clone();
                    view.update(cx, |this, cx| this.set_book(Some(id), cx))
                }),
        );
    }
    let done = view.clone();
    menu.separator().item(
        PopupMenuItem::new("Show Completed")
            .checked(show_done)
            .on_click(move |_, _, cx| {
                done.update(cx, |this, cx| this.set_show_done(!show_done, cx))
            }),
    )
}

/// Each line's height for the virtual list.
fn item_sizes(entries: &[Entry]) -> Rc<Vec<Size<Pixels>>> {
    Rc::new(
        entries
            .iter()
            .map(|entry| {
                let height = match entry {
                    Entry::Header { .. } => HEADER_HEIGHT,
                    Entry::Task(_) => ROW_HEIGHT,
                };
                size(px(0.), px(height))
            })
            .collect(),
    )
}

impl Render for TasksView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.stale && self.reload_task.is_none() {
            self.reload(cx);
        }
        let colors = Colors::get(cx);
        let entries = self.board.entries(Self::now(cx));
        let body = if entries.is_empty() {
            self.render_empty(cx)
        } else {
            v_virtual_list(
                cx.entity(),
                "tasks",
                item_sizes(&entries),
                |this, range, window, cx| this.render_rows(range, window, cx),
            )
            .size_full()
            .track_scroll(&self.scroll)
            .into_any_element()
        };
        let _ = window;
        v_flex()
            .size_full()
            .bg(colors.list)
            .child(self.render_top_band(cx))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .key_context("TasksView")
                    .track_focus(&self.focus)
                    .on_action(
                        cx.listener(|this, _: &SelectNextTask, _, cx| this.select_step(1, cx)),
                    )
                    .on_action(
                        cx.listener(|this, _: &SelectPreviousTask, _, cx| this.select_step(-1, cx)),
                    )
                    .on_action(cx.listener(|this, _: &ToggleTask, _, cx| {
                        if let Some(row) = this.board.selected_row() {
                            let id = TaskId::of(row);
                            this.toggle(&id, cx);
                        }
                    }))
                    .on_action(cx.listener(|this, _: &OpenTask, _, cx| {
                        if let Some(row) = this.board.selected_row() {
                            let id = TaskId::of(row);
                            this.open(&id, true, cx);
                        }
                    }))
                    .on_action(cx.listener(|this, _: &UndoCheckOff, _, cx| this.undo(cx)))
                    .child(body),
            )
            .child(self.render_bottom_band(cx))
    }
}

#[cfg(test)]
mod tests {
    // Not `super::*`: GPUI's own `test` attribute would shadow the built-in
    // one inside `proptest!`.
    use super::{Board, Entry, TaskId, Toggled, display_text};
    use kanso_core::DocType;
    use kanso_core::dates::{Date, DateTime};
    use kanso_core::tasks::TaskGroup;
    use kanso_store::{Origin, Put, Store, TaskQuery};
    use proptest::prelude::*;
    use serde_json::json;

    fn now() -> DateTime {
        DateTime {
            date: Date::new(2026, 10, 20).unwrap(),
            hour: 10,
            minute: 0,
            second: 0,
        }
    }

    fn store_with(bodies: &[&str]) -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("kanso.db")).unwrap();
        store
            .put(
                DocType::Book,
                json!({"_id": "book:a", "name": "A"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
        for (ix, body) in bodies.iter().enumerate() {
            store
                .put(
                    DocType::Note,
                    json!({"_id": format!("note:n{ix}"), "title": format!("N{ix}"), "body": body, "bookId": "book:a"}),
                    Put::Upsert,
                    Origin::Ui,
                )
                .unwrap();
        }
        (dir, store)
    }

    fn read(store: &Store) -> Vec<kanso_store::TaskRow> {
        store
            .tasks(&TaskQuery {
                include_done: true,
                ..Default::default()
            })
            .unwrap()
    }

    #[test]
    fn display_text_drops_the_date() {
        assert_eq!(
            display_text("Pay **rent** @2026-10-25 soon"),
            "Pay rent soon"
        );
        assert_eq!(display_text("@2026-10-25"), "@2026-10-25");
        assert_eq!(display_text("See [doc](https://x.y)"), "See doc");
    }

    #[test]
    fn groups_selection_and_undo() {
        let (_dir, store) = store_with(&[
            "- [ ] Late @2026-10-19\n- [ ] Now @2026-10-20\n- [ ] Next @2026-10-22\n- [ ] Plain\n- [x] Old\n",
        ]);
        let mut board = Board::default();
        board.set_rows(read(&store));
        let groups: Vec<TaskGroup> = board
            .entries(now())
            .iter()
            .filter_map(|e| match e {
                Entry::Header { group, .. } => Some(*group),
                Entry::Task(_) => None,
            })
            .collect();
        assert_eq!(
            groups,
            [
                TaskGroup::Overdue,
                TaskGroup::Today,
                TaskGroup::Upcoming,
                TaskGroup::NoDate
            ]
        );
        board.show_done = true;
        assert_eq!(board.entries(now()).len(), 10);
        board.show_done = false;
        board.collapsed.insert(TaskGroup::Upcoming);
        assert_eq!(board.entries(now()).len(), 7);
        // Down from nothing selects the first task, then steps over headers.
        board.step(1, now());
        assert_eq!(board.selected_row().unwrap().text, "Late @2026-10-19");
        board.step(1, now());
        assert_eq!(board.selected_row().unwrap().text, "Now @2026-10-20");
        board.step(5, now());
        assert_eq!(board.selected_row().unwrap().text, "Plain");
        // Ticking off keeps the task where it was, struck through.
        let id = board.selected.clone().unwrap();
        assert!(matches!(board.toggle(&id, &store, 1), Toggled::Done(_)));
        board.set_rows(read(&store));
        let still: Vec<usize> = board
            .entries(now())
            .iter()
            .filter_map(|e| match e {
                Entry::Task(ix) => Some(*ix),
                Entry::Header { .. } => None,
            })
            .collect();
        assert!(still.iter().any(|&ix| board.rows[ix].text == "Plain"));
        assert!(matches!(board.undo(&store, 2), Toggled::Done(_)));
        assert!(read(&store).iter().any(|r| r.text == "Plain" && !r.checked));
        assert_eq!(board.undo(&store, 3), Toggled::Nothing);
        // A task whose note changed underneath is reported, not guessed at.
        let gone = TaskId {
            note_id: "note:n0".into(),
            line: 0,
            text: "Late @2026-10-19".into(),
        };
        store
            .put(
                DocType::Note,
                json!({"_id": "note:n0", "body": "- [ ] Renamed\n"}),
                Put::Merge,
                Origin::Api,
            )
            .unwrap();
        assert_eq!(board.toggle(&gone, &store, 4), Toggled::Moved);
    }

    #[derive(Clone, Debug)]
    enum Op {
        Step(isize),
        Toggle,
        Undo,
        ShowDone,
        Collapse(usize),
        Reload,
    }

    fn op() -> impl Strategy<Value = Op> {
        prop_oneof![
            (-3isize..=3).prop_map(Op::Step),
            Just(Op::Toggle),
            Just(Op::Toggle),
            Just(Op::Undo),
            Just(Op::ShowDone),
            (0usize..5).prop_map(Op::Collapse),
            Just(Op::Reload),
        ]
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        /// Model-based: check-offs, undo, moving the selection, collapsing
        /// groups and showing done tasks, in any order, against a model
        /// that is just a flag per task and the last check-off. The store
        /// always agrees with the model, every listed task shows in exactly
        /// one group, and a task ticked off here stays listed.
        #[test]
        fn prop_check_offs_follow_the_model(
            dates in prop::collection::vec(prop::option::of(15u32..26), 1..7),
            ops in prop::collection::vec(op(), 0..30),
        ) {
            let body: String = dates.iter().enumerate().map(|(i, d)| match d {
                Some(day) => format!("- [ ] task {i} @2026-10-{day:02}\n"),
                None => format!("- [ ] task {i}\n"),
            }).collect();
            let (_dir, store) = store_with(&[&body]);
            let mut board = Board::default();
            board.set_rows(read(&store));
            let mut model = vec![false; dates.len()];
            let mut last: Option<(usize, bool)> = None;
            let index = |text: &str| -> usize {
                text.split(' ').nth(1).unwrap().parse().unwrap()
            };
            for op in ops {
                match op {
                    Op::Step(d) => board.step(d, now()),
                    Op::Toggle => {
                        if let Some(row) = board.selected_row() {
                            let i = index(&row.text);
                            let id = board.selected.clone().unwrap();
                            let result = board.toggle(&id, &store, 10);
                            prop_assert!(matches!(result, Toggled::Done(_)), "{:?}", result);
                            model[i] = !model[i];
                            last = Some((i, model[i]));
                        }
                    }
                    Op::Undo => {
                        let result = board.undo(&store, 11);
                        match last.take() {
                            Some((i, set)) => {
                                prop_assert!(matches!(result, Toggled::Done(_)));
                                model[i] = !set;
                            }
                            None => prop_assert_eq!(result, Toggled::Nothing),
                        }
                    }
                    Op::ShowDone => board.show_done = !board.show_done,
                    Op::Collapse(g) => {
                        let group = TaskGroup::ALL[g];
                        if !board.collapsed.remove(&group) {
                            board.collapsed.insert(group);
                        }
                    }
                    Op::Reload => board.set_rows(read(&store)),
                }
                let rows = read(&store);
                for row in &rows {
                    prop_assert_eq!(row.checked, model[index(&row.text)]);
                }
                // Every listed task in one group; collapsed groups hide theirs.
                let entries = board.entries(now());
                let mut seen = std::collections::HashSet::new();
                for entry in &entries {
                    if let Entry::Task(ix) = entry {
                        prop_assert!(seen.insert(*ix), "listed twice");
                        let group = board.group_of(&board.rows[*ix], now());
                        prop_assert!(!board.collapsed.contains(&group));
                    }
                }
                let headed: usize = entries.iter().map(|e| match e {
                    Entry::Header { count, .. } => *count,
                    Entry::Task(_) => 0,
                }).sum();
                prop_assert_eq!(headed, board.rows.iter().filter(|r| board.listed(r)).count());
                // Ticked off here and not shown as done: still listed.
                for (ix, row) in board.rows.iter().enumerate() {
                    if row.checked && board.kept.contains(&TaskId { note_id: row.note_id.clone(), line: row.line, text: row.text.clone() }) {
                        prop_assert!(board.listed(&board.rows[ix]));
                    }
                }
            }
        }
    }
}
