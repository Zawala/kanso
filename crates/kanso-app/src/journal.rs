//! Daily notes and templates in the main window: ⌘D opens today's note in
//! the Journal (made from the daily template when there is one), the
//! previous and next day step through the days that have notes, and "New
//! from template…" makes a note from one in the Templates notebook. The
//! store does the making (`kanso_store::roles`); this is the window's side.

use gpui_kit::component::WindowExt as _;
use gpui_kit::component::notification::Notification;
use gpui_kit::*;
use kanso_core::dates::{Date, DateTime};
use kanso_core::roles::{Role, daily_date, daily_id};
use kanso_store::{Origin, Store};

use crate::clock::Clock;
use crate::commands::Picker;
use crate::config::ViewMode;
use crate::editor::NoteEditor;
use crate::library::Library;
use crate::palette::Mode;
use crate::workspace::Workspace;

actions!(
    kanso,
    [
        OpenToday,
        PreviousDay,
        NextDay,
        NewFromTemplate,
        NewTemplate,
        QuickCapture
    ]
);

pub fn bind_keys(cx: &mut App) {
    cx.bind_keys([KeyBinding::new("secondary-d", OpenToday, None)]);
}

/// The local date and time now, as the app's clock has it.
pub fn now(cx: &App) -> DateTime {
    DateTime::from_local_ms(Clock::local_ms(cx, Clock::now_ms(cx)))
}

/// The day before or after `current` that has a daily note, or today
/// (which ⌘D makes if needed), whichever is nearer. `None` when there is
/// no such day. Pure.
pub fn step_day(dates: &[Date], current: Date, today: Date, forward: bool) -> Option<Date> {
    let candidates = dates.iter().copied().chain([today]);
    if forward {
        candidates.filter(|&d| d > current).min()
    } else {
        candidates.filter(|&d| d < current).max()
    }
}

/// Where a note made from a template goes: the notebook the list shows,
/// unless that is the Templates notebook, where the note would itself be a
/// template; then the Inbox (`None`).
fn template_target(store: &Store, book: Option<String>) -> Option<String> {
    let templates = store.find_role_book(Role::Templates).ok().flatten();
    book.filter(|book| Some(book) != templates.as_ref())
}

pub fn on_actions(el: Div, cx: &mut Context<Workspace>) -> Div {
    el.on_action(cx.listener(|this, _: &OpenToday, window, cx| this.open_today(window, cx)))
        .on_action(
            cx.listener(|this, _: &PreviousDay, window, cx| this.step_day(false, window, cx)),
        )
        .on_action(cx.listener(|this, _: &NextDay, window, cx| this.step_day(true, window, cx)))
        .on_action(cx.listener(|this, _: &NewFromTemplate, window, cx| {
            this.palette.update(cx, |palette, cx| {
                palette.open(Mode::Pick(Picker::Template), window, cx)
            })
        }))
        .on_action(cx.listener(|this, _: &NewTemplate, window, cx| {
            new_template(&this.library, &this.editor, window, cx)
        }))
}

/// Opens note `id` with the caret at byte `caret` of its text when given,
/// and the keyboard in the text, or in the title with `title_first`.
fn open_made(
    library: &Entity<Library>,
    editor: &Entity<NoteEditor>,
    id: String,
    caret: Option<usize>,
    title_first: bool,
    window: &mut Window,
    cx: &mut App,
) {
    if let Some(caret) = caret {
        editor.update(cx, |editor, _| editor.caret_after_open(id.clone(), caret));
    }
    library.update(cx, |library, cx| library.select(Some(id), cx));
    if editor.read(cx).view_mode == ViewMode::Preview {
        return;
    }
    editor.update(cx, |editor, cx| {
        if title_first {
            editor.focus_title(window, cx)
        } else {
            editor.focus_body(window, cx)
        }
    });
}

fn report(what: &str, err: impl std::fmt::Display, window: &mut Window, cx: &mut App) {
    window.push_notification(Notification::error(format!("{what}: {err}")), cx);
}

/// A note from template `id` in the notebook the list shows, opened.
pub(crate) fn new_from_template(
    library: &Entity<Library>,
    editor: &Entity<NoteEditor>,
    id: &str,
    window: &mut Window,
    cx: &mut App,
) {
    editor.update(cx, |editor, cx| editor.flush(cx));
    let (store, book) = {
        let library = library.read(cx);
        (library.store.clone(), library.default_book())
    };
    let book = template_target(&store, book);
    match store.create_from_template(id, None, book.as_deref(), now(cx), Origin::Ui) {
        Ok(made) => {
            let untitled = made.title.trim().is_empty();
            open_made(library, editor, made.id, made.cursor, untitled, window, cx)
        }
        Err(err) => report("Couldn't make the note", err, window, cx),
    }
}

/// An empty note in the Templates notebook (made if needed), opened to
/// write a template in.
pub(crate) fn new_template(
    library: &Entity<Library>,
    editor: &Entity<NoteEditor>,
    window: &mut Window,
    cx: &mut App,
) {
    editor.update(cx, |editor, cx| editor.flush(cx));
    let store = library.read(cx).store.clone();
    let made = store
        .role_book(Role::Templates, Origin::Ui)
        .and_then(|book| new_note_in(&store, &book));
    match made {
        Ok(id) => open_made(library, editor, id, None, true, window, cx),
        Err(err) => report("Couldn't make the template", err, window, cx),
    }
}

impl Workspace {
    /// ⌘D: today's note, made if needed.
    pub(crate) fn open_today(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editor.update(cx, |editor, cx| editor.flush(cx));
        let store = self.library.read(cx).store.clone();
        match store.open_daily(now(cx), Origin::Ui) {
            Ok(daily) => open_made(
                &self.library,
                &self.editor,
                daily.id,
                daily.cursor,
                false,
                window,
                cx,
            ),
            Err(err) => report("Couldn't open today's note", err, window, cx),
        }
    }

    /// The previous or next day's note, from the open daily note or today.
    pub(crate) fn step_day(&mut self, forward: bool, window: &mut Window, cx: &mut Context<Self>) {
        let today = now(cx).date;
        let current = self
            .editor
            .read(cx)
            .note_id()
            .and_then(daily_date)
            .unwrap_or(today);
        let store = self.library.read(cx).store.clone();
        let dates = store.daily_dates().unwrap_or_default();
        let Some(day) = step_day(&dates, current, today, forward) else {
            return;
        };
        if day == today {
            return self.open_today(window, cx);
        }
        self.editor.update(cx, |editor, cx| editor.flush(cx));
        open_made(
            &self.library,
            &self.editor,
            daily_id(day),
            None,
            false,
            window,
            cx,
        );
    }
}

fn new_note_in(store: &Store, book: &str) -> kanso_store::Result<String> {
    let fields = serde_json::json!({"title": "", "body": "", "bookId": book});
    store
        .put(
            kanso_core::DocType::Note,
            fields,
            kanso_store::Put::Upsert,
            Origin::Ui,
        )
        .map(|made| made.id)
}

#[cfg(test)]
mod tests {
    use super::{Date, step_day};
    use proptest::prelude::*;

    fn d(day: u32) -> Date {
        Date::new(2026, 5, day).unwrap()
    }

    #[test]
    fn steps_between_days_with_notes_and_today() {
        let dates = [d(3), d(10), d(12)];
        let today = d(20);
        assert_eq!(step_day(&dates, d(12), today, false), Some(d(10)));
        assert_eq!(step_day(&dates, d(10), today, true), Some(d(12)));
        // Today, though it has no note yet.
        assert_eq!(step_day(&dates, d(12), today, true), Some(today));
        assert_eq!(step_day(&dates, today, today, true), None);
        assert_eq!(step_day(&dates, d(3), today, false), None);
        // From a day without a note (today's, not made yet).
        assert_eq!(step_day(&dates, today, today, false), Some(d(12)));
        assert_eq!(step_day(&[], today, today, false), None);
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Stepping lands on the nearest candidate in that direction, and
        /// stepping back again returns, unless a candidate lies between.
        #[test]
        fn prop_step_is_nearest(
            days in prop::collection::btree_set(1u32..=28, 0..10),
            today in 1u32..=28,
            current in 1u32..=28,
            forward in any::<bool>(),
        ) {
            let dates: Vec<Date> = days.iter().map(|&n| d(n)).collect();
            let mut all: Vec<Date> = dates.clone();
            all.push(d(today));
            match step_day(&dates, d(current), d(today), forward) {
                Some(next) => {
                    prop_assert!(all.contains(&next));
                    let past = if forward { next > d(current) } else { next < d(current) };
                    prop_assert!(past);
                    let between = all.iter().any(|&x| {
                        if forward { x > d(current) && x < next } else { x < d(current) && x > next }
                    });
                    prop_assert!(!between);
                }
                None => {
                    let none = all.iter().all(|&x| if forward { x <= d(current) } else { x >= d(current) });
                    prop_assert!(none);
                }
            }
        }
    }
}
