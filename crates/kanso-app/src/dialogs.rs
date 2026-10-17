//! Small modal prompts: ask for a name, or confirm something destructive.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gpui_kit::component::WindowExt as _;
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::*;

type TextHandler = Rc<dyn Fn(String, &mut Window, &mut App)>;
type ConfirmHandler = Rc<dyn Fn(&mut Window, &mut App)>;

/// Asks for a line of text. `on_ok` gets the trimmed, non-empty answer.
pub fn prompt_text(
    title: impl Into<SharedString>,
    initial: &str,
    ok_label: impl Into<SharedString>,
    window: &mut Window,
    cx: &mut App,
    on_ok: impl Fn(String, &mut Window, &mut App) + 'static,
) {
    let title = title.into();
    let ok_label = ok_label.into();
    let initial = initial.to_owned();
    let input = cx.new(|cx| InputState::new(window, cx).default_value(initial));
    let on_ok: TextHandler = Rc::new(on_ok);

    // Enter in the field submits, like the OK button.
    let handle = window.window_handle();
    let submit = on_ok.clone();
    let enter = cx.subscribe(&input, move |input, event: &InputEvent, cx| {
        if !matches!(event, InputEvent::PressEnter { .. }) {
            return;
        }
        let value = input.read(cx).value().trim().to_owned();
        if value.is_empty() {
            return;
        }
        let submit = submit.clone();
        handle
            .update(cx, move |_, window, cx| {
                window.close_dialog(cx);
                submit(value, window, cx);
            })
            .ok();
    });
    let enter = RefCell::new(Some(enter));

    let first_build = Cell::new(true);
    window.open_alert_dialog(cx, move |alert, window, cx| {
        // Keeps the Enter subscription alive exactly as long as the dialog.
        let _ = &enter;
        if first_build.replace(false) {
            input.update(cx, |input, cx| input.focus(window, cx));
        }
        let input_for_ok = input.clone();
        let on_ok = on_ok.clone();
        alert
            .title(title.clone())
            .child(Input::new(&input))
            .show_cancel(true)
            .ok_text(ok_label.clone())
            .on_ok(move |_, window, cx| {
                let value = input_for_ok.read(cx).value().trim().to_owned();
                if value.is_empty() {
                    return false;
                }
                on_ok(value, window, cx);
                true
            })
    });
}

/// Asks before doing something that can't be undone.
pub fn confirm(
    title: impl Into<SharedString>,
    description: impl Into<SharedString>,
    ok_label: impl Into<SharedString>,
    window: &mut Window,
    cx: &mut App,
    on_ok: impl Fn(&mut Window, &mut App) + 'static,
) {
    let title = title.into();
    let description = description.into();
    let ok_label = ok_label.into();
    let on_ok: ConfirmHandler = Rc::new(on_ok);
    window.open_alert_dialog(cx, move |alert, _, _| {
        let on_ok = on_ok.clone();
        alert
            .title(title.clone())
            .description(description.clone())
            .show_cancel(true)
            .ok_text(ok_label.clone())
            .on_ok(move |_, window, cx| {
                on_ok(window, cx);
                true
            })
    });
}
