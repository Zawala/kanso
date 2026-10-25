//! Daily notes and templates in the main window: ⌘D opens today's note in
//! the Journal (made from the daily template when there is one), the
//! previous and next day step through the days that have notes, and "New
//! from template…" makes a note from one in the Templates notebook. What to
//! make and open is decided in `kanso_engine::daily`; this is the window's
//! side.

use gpui_kit::component::WindowExt as _;
use gpui_kit::component::notification::Notification;
use gpui_kit::*;
use kanso_core::dates::DateTime;
use kanso_core::roles::daily_id;
use kanso_engine::daily;

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
pub(crate) fn open_made(
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
// no-prop-test: delegates to kanso_engine::daily::from_template, tested there.
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
    match daily::from_template(&store, id, book, now(cx)) {
        Ok(made) => open_made(
            library,
            editor,
            made.id,
            made.cursor,
            made.title_first,
            window,
            cx,
        ),
        Err(err) => report("Couldn't make the note", err, window, cx),
    }
}

/// An empty note in the Templates notebook (made if needed), opened to
/// write a template in.
// no-prop-test: delegates to kanso_engine::daily::new_template, tested there.
pub(crate) fn new_template(
    library: &Entity<Library>,
    editor: &Entity<NoteEditor>,
    window: &mut Window,
    cx: &mut App,
) {
    editor.update(cx, |editor, cx| editor.flush(cx));
    let store = library.read(cx).store.clone();
    match daily::new_template(&store) {
        Ok(id) => open_made(library, editor, id, None, true, window, cx),
        Err(err) => report("Couldn't make the template", err, window, cx),
    }
}

// no-prop-test: delegates to kanso_engine::daily::{today, step}, tested there.
impl Workspace {
    /// ⌘D: today's note, made if needed.
    pub(crate) fn open_today(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editor.update(cx, |editor, cx| editor.flush(cx));
        let store = self.library.read(cx).store.clone();
        match daily::today(&store, now(cx)) {
            Ok(made) => open_made(
                &self.library,
                &self.editor,
                made.id,
                made.cursor,
                made.title_first,
                window,
                cx,
            ),
            Err(err) => report("Couldn't open today's note", err, window, cx),
        }
    }

    /// The previous or next day's note, from the open daily note or today.
    pub(crate) fn step_day(&mut self, forward: bool, window: &mut Window, cx: &mut Context<Self>) {
        let today = now(cx).date;
        let store = self.library.read(cx).store.clone();
        let open = self.editor.read(cx).note_id().map(str::to_owned);
        let Some(day) = daily::step(&store, open.as_deref(), today, forward) else {
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
