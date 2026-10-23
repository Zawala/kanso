//! A note's properties at the top of the inspector: each key and value,
//! edited in place, and a row to add one.
//!
//! A value goes into the note's front matter through
//! [`kanso_core::properties::set_property`], and the editor takes the new
//! text as one undoable edit. A locked note that isn't open shows nothing.

use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::switch::Switch;
use gpui_kit::component::{Icon, Sizable as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_core::properties::{self, Value};

use crate::inspector::Inspector;
use crate::locked::LockView;
use crate::theme::{Colors, section_label};
use crate::zoom::px;

use super::PAD_X;

/// The value being edited in place.
pub(super) struct PropertyEdit {
    key: String,
    input: Entity<InputState>,
    _event: Subscription,
}

/// A property being added: its key and its value.
pub(super) struct PropertyAdd {
    key: Entity<InputState>,
    value: Entity<InputState>,
    _events: [Subscription; 2],
}

impl Inspector {
    /// The open note's text, if the section should show: a note is open,
    /// and a locked one is open with this session's key.
    fn property_source(&self, cx: &App) -> Option<SharedString> {
        let editor = self.editor.read(cx);
        if editor.note_id().is_none() || matches!(editor.lock_view(), LockView::Closed(_)) {
            return None;
        }
        Some(editor.body_text(cx))
    }

    /// The Properties section: key and value rows, the one being edited as
    /// a field, and Add property.
    pub(super) fn render_properties(&mut self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let body = self.property_source(cx)?;
        let colors = Colors::get(cx);
        let properties = properties::parse(&body);
        let section = section_label(div(), colors)
            .px(px(8.))
            .pt(px(10.))
            .pb(px(6.))
            .child("Properties");
        let mut list = v_flex()
            .id("properties")
            .debug_selector(|| "inspector-properties".into())
            .w_full()
            .flex_none()
            .px(px(PAD_X))
            .pt(px(2.))
            .pb(px(8.))
            .border_b_1()
            .border_color(colors.divider)
            .child(section);
        for (ix, (key, value)) in properties.iter().enumerate() {
            list = list.child(self.property_row(ix, key, value, colors, cx));
        }
        list = list.child(self.add_property_row(properties.len(), colors, cx));
        Some(list.into_any_element())
    }

    fn property_row(
        &mut self,
        ix: usize,
        key: &str,
        value: &Value,
        colors: &Colors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let editing = self
            .property_edit
            .as_ref()
            .is_some_and(|edit| edit.key.eq_ignore_ascii_case(key));
        let key_view = div()
            .flex_none()
            .w(relative(0.42))
            .min_w_0()
            .truncate()
            .text_size(px(12.5))
            .text_color(colors.text_secondary)
            .child(key.to_owned());
        let value_view = if editing {
            let input = self
                .property_edit
                .as_ref()
                .map(|edit| edit.input.clone())
                .expect("editing has an input");
            div()
                .flex_1()
                .min_w_0()
                .child(Input::new(&input).small())
                .into_any_element()
        } else if let Value::Bool(on) = value {
            let key = key.to_owned();
            let debug_key = key.clone();
            let on = *on;
            let switch = Switch::new(ElementId::NamedInteger("property-switch".into(), ix as u64))
                .checked(on)
                .on_click(cx.listener(move |this, on: &bool, window, cx| {
                    let body = this.editor.read(cx).body_text(cx).to_string();
                    let next = properties::set_property(&body, &key, &Value::Bool(*on));
                    this.editor
                        .update(cx, |editor, cx| editor.set_body(next, window, cx));
                    cx.notify();
                }));
            div()
                .id(ElementId::NamedInteger("property-bool".into(), ix as u64))
                .debug_selector(move || format!("property-{debug_key}"))
                .flex_1()
                .min_w_0()
                .child(switch)
                .into_any_element()
        } else {
            let key = key.to_owned();
            let current = value.clone();
            let shown = if value.is_empty() {
                "Empty".to_owned()
            } else {
                crate::query_view_ui::cell_text(value)
            };
            let muted = value.is_empty();
            div()
                .id(ElementId::NamedInteger("property-value".into(), ix as u64))
                .debug_selector({
                    let key = key.clone();
                    move || format!("property-{key}")
                })
                .flex_1()
                .min_w_0()
                .truncate()
                .text_size(px(12.5))
                .when(muted, |el| el.text_color(colors.text_muted))
                .when(!muted, |el| el.text_color(colors.text))
                .cursor_pointer()
                .hover(|el| el.text_color(colors.link))
                .child(shown)
                .on_click(cx.listener(move |this, _, window, cx| {
                    this.edit_property(&key, &current, window, cx);
                }))
                .into_any_element()
        };
        h_flex()
            .id(ElementId::NamedInteger("property-row".into(), ix as u64))
            .w_full()
            .min_w_0()
            .h(px(26.))
            .gap(px(8.))
            .rounded(px(5.))
            .hover(|el| el.bg(colors.row_hover))
            .child(key_view)
            .child(value_view)
            .into_any_element()
    }

    fn add_property_row(
        &mut self,
        ix: usize,
        colors: &Colors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if let Some(add) = &self.property_add {
            return h_flex()
                .id("add-property")
                .w_full()
                .min_w_0()
                .gap(px(6.))
                .py(px(2.))
                .child(
                    div()
                        .flex_none()
                        .w(relative(0.42))
                        .min_w_0()
                        .child(Input::new(&add.key).small()),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .child(Input::new(&add.value).small()),
                )
                .into_any_element();
        }
        h_flex()
            .id(ElementId::NamedInteger("add-property".into(), ix as u64))
            .debug_selector(|| "add-property".into())
            .w_full()
            .h(px(26.))
            .gap(px(6.))
            .pl(px(2.))
            .rounded(px(5.))
            .cursor_pointer()
            .text_size(px(12.5))
            .text_color(colors.text_muted)
            .hover(|el| el.bg(colors.row_hover).text_color(colors.text_secondary))
            .child(Icon::new(gpui_kit::assets::IconName::Plus).size(px(12.)))
            .child("Add property")
            .on_click(cx.listener(|this, _, window, cx| this.add_property(window, cx)))
            .into_any_element()
    }

    /// Starts editing `key`'s value, with `like` keeping its type.
    fn edit_property(
        &mut self,
        key: &str,
        like: &Value,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let input = cx.new(|cx| {
            let mut state = InputState::new(window, cx).placeholder("Value");
            state.set_value(like.text(), window, cx);
            state
        });
        input.update(cx, |state, cx| state.focus(window, cx));
        let key = key.to_owned();
        let watched = key.clone();
        let like = like.clone();
        let event = cx.subscribe_in(
            &input,
            window,
            move |this: &mut Inspector, input, event: &InputEvent, window, cx| {
                if matches!(event, InputEvent::PressEnter { .. }) {
                    let text = input.read(cx).value().to_string();
                    this.commit_property(&watched, &text, Some(&like), window, cx);
                }
            },
        );
        self.property_edit = Some(PropertyEdit {
            key,
            input,
            _event: event,
        });
        cx.notify();
    }

    /// Starts adding a property: a key and a value, Enter in either to save.
    fn add_property(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let key = cx.new(|cx| InputState::new(window, cx).placeholder("Property"));
        let value = cx.new(|cx| InputState::new(window, cx).placeholder("Value"));
        key.update(cx, |state, cx| state.focus(window, cx));
        let key_event = cx.subscribe_in(
            &key,
            window,
            |this: &mut Inspector, _, event: &InputEvent, window, cx| {
                if matches!(event, InputEvent::PressEnter { .. }) {
                    this.commit_add(window, cx);
                }
            },
        );
        let value_event = cx.subscribe_in(
            &value,
            window,
            |this: &mut Inspector, _, event: &InputEvent, window, cx| {
                if matches!(event, InputEvent::PressEnter { .. }) {
                    this.commit_add(window, cx);
                }
            },
        );
        self.property_add = Some(PropertyAdd {
            key,
            value,
            _events: [key_event, value_event],
        });
        cx.notify();
    }

    /// Saves `text` as `key`'s value, keeping the type of `like`, and closes
    /// the field.
    fn commit_property(
        &mut self,
        key: &str,
        text: &str,
        like: Option<&Value>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let body = self.editor.read(cx).body_text(cx).to_string();
        let value = properties::typed(text, like);
        let next = properties::set_property(&body, key, &value);
        self.property_edit = None;
        if next != body {
            self.editor
                .update(cx, |editor, cx| editor.set_body(next, window, cx));
        }
        cx.notify();
    }

    /// Saves the property being added.
    fn commit_add(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(add) = &self.property_add else {
            return;
        };
        let key = add.key.read(cx).value().trim().to_owned();
        let text = add.value.read(cx).value().to_string();
        self.property_add = None;
        if key.is_empty() {
            cx.notify();
            return;
        }
        let body = self.editor.read(cx).body_text(cx).to_string();
        let like = properties::parse(&body)
            .into_iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(&key))
            .map(|(_, value)| value);
        let value = properties::typed(&text, like.as_ref());
        let next = properties::set_property(&body, &key, &value);
        if next != body {
            self.editor
                .update(cx, |editor, cx| editor.set_body(next, window, cx));
        }
        cx.notify();
    }

    /// The properties as last drawn, for the tests.
    #[cfg(test)]
    pub(crate) fn properties_for_test(&self, cx: &App) -> Vec<(String, Value)> {
        self.property_source(cx)
            .map(|body| properties::parse(&body))
            .unwrap_or_default()
    }

    /// Edits a value as the section does, from the keyboard.
    #[cfg(test)]
    pub(crate) fn set_property_for_test(
        &mut self,
        key: &str,
        text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let like = properties::parse(&self.editor.read(cx).body_text(cx))
            .into_iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(key))
            .map(|(_, value)| value);
        self.commit_property(key, text, like.as_ref(), window, cx);
    }

    /// Adds a property as the section does, from the keyboard.
    #[cfg(test)]
    pub(crate) fn add_property_for_test(
        &mut self,
        key: &str,
        text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.add_property(window, cx);
        if let Some(add) = &self.property_add {
            add.key
                .update(cx, |state, cx| state.set_value(key, window, cx));
            add.value
                .update(cx, |state, cx| state.set_value(text, window, cx));
        }
        self.commit_add(window, cx);
    }
}
