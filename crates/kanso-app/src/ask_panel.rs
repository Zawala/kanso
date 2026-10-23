//! Ask Your Notes: a question box, the answer as it streams in from the
//! provider the person chose, and the notes it used, each a link.
//!
//! Search, the excerpts and the prompt are made on this device
//! (`ask.rs`); only the excerpts chosen for the question leave it. The
//! answer is not saved, and no note is changed.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use gpui_kit::base::text::TextView;
use gpui_kit::component::Disableable as _;
use gpui_kit::component::button::Button;
use gpui_kit::component::input::{InputEvent, Textarea, TextareaState};
use gpui_kit::component::{Sizable as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_ai::{ClientSettings, Excerpt};
use kanso_store::Store;

use crate::editor::NoteEditor;
use crate::library::Library;
use crate::settings_ai::{read_key, requests_line};
use crate::text_stats::plural;
use crate::theme::{Colors, section_label};
use crate::workspace::AppState;
use crate::zoom::px;

/// What the background thread says as the answer is made.
enum Update {
    /// The excerpts chosen for the question.
    Used(Vec<Excerpt>),
    Text(String),
    Done,
    Error(String),
    /// Nothing in the notes matched, so no model was asked.
    Empty,
}

pub struct AskPanel {
    store: Arc<Store>,
    library: Entity<Library>,
    editor: Entity<NoteEditor>,
    question: Entity<TextareaState>,
    /// The question asked last, so the panel can tell "before the first".
    asked: bool,
    answer: String,
    used: Vec<Excerpt>,
    searching: bool,
    streaming: bool,
    empty: bool,
    error: Option<String>,
    cancel: Option<Arc<AtomicBool>>,
    task: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
}

impl AskPanel {
    pub fn new(
        store: Arc<Store>,
        library: Entity<Library>,
        editor: Entity<NoteEditor>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let question = cx.new(|cx| {
            TextareaState::new(window, cx)
                .placeholder("Ask about your notes…")
                .submit_on_enter(true)
        });
        let subscriptions = vec![cx.subscribe_in(
            &question,
            window,
            |this: &mut AskPanel, _, event: &InputEvent, window, cx| match event {
                InputEvent::PressEnter { shift: false, .. } => this.ask(window, cx),
                InputEvent::Change => cx.notify(),
                _ => {}
            },
        )];
        AskPanel {
            store,
            library,
            editor,
            question,
            asked: false,
            answer: String::new(),
            used: Vec::new(),
            searching: false,
            streaming: false,
            empty: false,
            error: None,
            cancel: None,
            task: None,
            _subscriptions: subscriptions,
        }
    }

    /// Puts the keyboard in the question box.
    pub fn focus_question(&self, window: &mut Window, cx: &mut App) {
        let focus = self.question.read(cx).focus_handle(cx);
        window.focus(&focus, cx);
    }

    /// Asks the question in the box.
    pub fn ask(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        let question = self.question.read(cx).value().trim().to_owned();
        if question.is_empty() {
            return;
        }
        let ai = cx.global::<AppState>().config.ai.clone();
        if !ai.enabled {
            return;
        }
        let saved = read_key(cx, ai.provider);
        let settings = ClientSettings {
            provider: ai.provider,
            base_url: ai.base_url.clone(),
            model: ai.model.clone(),
            api_key: saved,
        };
        let provider = match kanso_ai::client(&settings) {
            Ok(provider) => provider,
            Err(err) => {
                self.error = Some(err.to_string());
                self.asked = true;
                self.searching = false;
                self.streaming = false;
                cx.notify();
                return;
            }
        };
        self.stop(cx);
        self.asked = true;
        self.answer.clear();
        self.used.clear();
        self.empty = false;
        self.error = None;
        self.searching = true;
        let cancel = Arc::new(AtomicBool::new(false));
        self.cancel = Some(cancel.clone());
        cx.notify();

        let store = self.store.clone();
        let asked = question.clone();
        let (tx, rx) = async_channel::unbounded::<Update>();
        let spawned = std::thread::Builder::new()
            .name("kanso-ask".into())
            .spawn(move || {
                let excerpts = match crate::ask::retrieve(&store, &asked) {
                    Ok(excerpts) => excerpts,
                    Err(err) => {
                        tx.try_send(Update::Error(format!("Couldn't search your notes: {err}")))
                            .ok();
                        return;
                    }
                };
                if excerpts.is_empty() {
                    tx.try_send(Update::Empty).ok();
                    return;
                }
                tx.try_send(Update::Used(excerpts.clone())).ok();
                let prompt = kanso_ai::build_prompt(&asked, &excerpts, crate::ask::BUDGET);
                crate::ask::stream_answer(provider.as_ref(), &prompt, &cancel, |event| {
                    if cancel.load(Ordering::Relaxed) {
                        return;
                    }
                    let update = match event {
                        kanso_ai::Event::Text(text) => Update::Text(text),
                        kanso_ai::Event::Done => Update::Done,
                        kanso_ai::Event::Error(message) => Update::Error(message),
                    };
                    tx.try_send(update).ok();
                });
            });
        if spawned.is_err() {
            self.cancel = None;
            self.searching = false;
            self.error = Some("Couldn't start the question.".to_owned());
            cx.notify();
            return;
        }
        self.task = Some(cx.spawn(async move |this, cx| {
            while let Ok(update) = rx.recv().await {
                let finished = this
                    .update(cx, |this, cx| {
                        match update {
                            Update::Used(used) => {
                                this.used = used;
                                this.searching = false;
                                this.streaming = true;
                            }
                            Update::Text(text) => this.answer.push_str(&text),
                            Update::Done => {
                                this.streaming = false;
                                this.searching = false;
                                this.cancel = None;
                            }
                            Update::Error(message) => {
                                this.streaming = false;
                                this.searching = false;
                                this.error = Some(message);
                                this.cancel = None;
                            }
                            Update::Empty => {
                                this.searching = false;
                                this.empty = true;
                                this.cancel = None;
                            }
                        }
                        cx.notify();
                        !this.searching && !this.streaming
                    })
                    .unwrap_or(true);
                if finished {
                    break;
                }
            }
        }));
    }

    /// Stops the answer being read. What arrived stays.
    pub fn stop(&mut self, cx: &mut Context<Self>) {
        if let Some(cancel) = self.cancel.take() {
            cancel.store(true, Ordering::Relaxed);
        }
        self.task = None;
        self.searching = false;
        self.streaming = false;
        cx.notify();
    }

    /// Opens note `id` in the editor.
    fn open(&self, id: String, cx: &mut App) {
        self.editor
            .update(cx, |editor, _| editor.reveal_after_open(id.clone(), 0));
        self.library
            .update(cx, |library, cx| library.select(Some(id), cx));
    }

    fn status(&self, cx: &App) -> Option<AnyElement> {
        let colors = Colors::get(cx);
        if let Some(error) = &self.error {
            return Some(
                div()
                    .text_size(px(12.5))
                    .line_height(px(18.))
                    .text_color(colors.danger)
                    .child(error.clone())
                    .into_any_element(),
            );
        }
        if self.searching {
            return Some(
                div()
                    .text_size(px(12.5))
                    .text_color(colors.text_secondary)
                    .child("Searching your notes…")
                    .into_any_element(),
            );
        }
        if self.empty {
            return Some(
                div()
                    .text_size(px(12.5))
                    .line_height(px(18.))
                    .text_color(colors.text_secondary)
                    .child("Nothing in your notes matches this question.")
                    .into_any_element(),
            );
        }
        if self.answer.is_empty() && self.streaming {
            return Some(
                div()
                    .text_size(px(12.5))
                    .text_color(colors.text_secondary)
                    .child("Reading your notes…")
                    .into_any_element(),
            );
        }
        None
    }

    fn render_used(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        if self.used.is_empty() {
            return None;
        }
        let colors = Colors::get(cx);
        let view = cx.entity();
        Some(
            v_flex()
                .gap(px(2.))
                .child(
                    section_label(div(), colors)
                        .text_size(px(11.5))
                        .text_color(colors.text_muted)
                        .child(format!("Used {}", plural(self.used.len(), "note", "notes"))),
                )
                .children(self.used.iter().enumerate().map(|(ix, excerpt)| {
                    let view = view.clone();
                    let id = excerpt.id.clone();
                    let title = if excerpt.title.trim().is_empty() {
                        "Untitled".to_owned()
                    } else {
                        excerpt.title.clone()
                    };
                    h_flex()
                        .id(ElementId::NamedInteger("ask-used".into(), ix as u64))
                        .w_full()
                        .px(px(6.))
                        .py(px(4.))
                        .rounded(px(5.))
                        .cursor_pointer()
                        .hover(|el| el.bg(colors.row_hover))
                        .text_size(px(12.5))
                        .text_color(colors.link)
                        .child(div().truncate().child(title))
                        .on_click(move |_, _, cx| {
                            view.update(cx, |this, cx| this.open(id.clone(), cx));
                        })
                }))
                .into_any_element(),
        )
    }
}

impl Render for AskPanel {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let ai = cx.global::<AppState>().config.ai.clone();
        let running = self.searching || self.streaming;
        let answer = crate::ask::linkify(&self.answer, &self.used);
        let used = self.render_used(cx);
        let view = cx.entity();
        v_flex()
            .id("ask")
            .size_full()
            .px(px(10.))
            .pt(px(10.))
            .pb(px(10.))
            .gap(px(8.))
            .when(!self.asked && ai.provider.needs_key(), |el| {
                el.child(
                    div()
                        .text_size(px(11.5))
                        .line_height(px(16.))
                        .text_color(colors.text_muted)
                        .child(requests_line(ai.provider, &ai.base_url)),
                )
            })
            .child(
                div()
                    .debug_selector(|| "ask-question".into())
                    .child(Textarea::new(&self.question).h(px(78.))),
            )
            .child(
                h_flex()
                    .gap(px(8.))
                    .child(
                        Button::new("ask-send")
                            .outline()
                            .small()
                            .label("Ask")
                            .disabled(running)
                            .on_click({
                                let view = view.clone();
                                move |_, window, cx| {
                                    view.update(cx, |this, cx| this.ask(window, cx));
                                }
                            }),
                    )
                    .when(running, |el| {
                        let view = view.clone();
                        el.child(
                            Button::new("ask-stop")
                                .outline()
                                .small()
                                .label("Stop")
                                .on_click(move |_, _, cx| {
                                    view.update(cx, |this, cx| this.stop(cx));
                                }),
                        )
                    }),
            )
            .when(!self.asked, |el| {
                el.child(
                    div()
                        .text_size(px(12.5))
                        .line_height(px(18.))
                        .text_color(colors.text_secondary)
                        .child("A question is answered from your own notes, with links to the notes it used. The answer isn't saved and no note is changed."),
                )
            })
            .children(self.status(cx))
            .when(!answer.is_empty(), |el| {
                let open = view.clone();
                el.child(
                    div()
                        .id("ask-answer")
                        .flex_1()
                        .min_h_0()
                        .overflow_y_scroll()
                        .child(
                            TextView::markdown("ask-answer-text", answer)
                                .selectable(true)
                                .on_link_click(move |url, _, _, cx| {
                                    let Some(id) = url
                                        .strip_prefix(kanso_core::links::NOTE_LINK_PREFIX)
                                    else {
                                        return;
                                    };
                                    let id = format!("note:{id}");
                                    open.update(cx, |this, cx| this.open(id, cx));
                                }),
                        ),
                )
            })
            .when_some(used, |el, used| el.child(used))
    }
}
