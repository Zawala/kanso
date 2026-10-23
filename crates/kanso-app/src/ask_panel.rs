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
use kanso_ai::{ClientSettings, Excerpt, Provider};
use kanso_store::Store;

use crate::editor::NoteEditor;
use crate::library::Library;
use crate::services::Services;
use crate::settings_ai::{read_from, requests_line};
use crate::text_stats::plural;
use crate::theme::{Colors, section_label};
use crate::workspace::AppState;
use crate::zoom::px;

/// How many characters of earlier exchanges a follow-up may resend. The
/// oldest turns are dropped first.
const HISTORY_BUDGET: usize = 12_000;

/// One completed exchange: what was asked, what was shown, and the notes it
/// drew on, so a later question can drop it if one of them is no longer
/// eligible.
#[derive(Clone, Debug, PartialEq, Eq)]
struct AskedTurn {
    question: String,
    answer: String,
    notes: Vec<String>,
}

impl AskedTurn {
    fn chars(&self) -> usize {
        self.question.chars().count() + self.answer.chars().count()
    }
}

/// What a conversation was asked under. A different provider, URL or model
/// starts a new one.
#[derive(Clone, Debug, PartialEq, Eq)]
struct ConversationKey {
    provider: Provider,
    base_url: String,
    model: String,
}

impl ConversationKey {
    fn of(ai: &crate::config::AiSettings) -> Self {
        Self {
            provider: ai.provider,
            base_url: ai.base_url.clone(),
            model: ai.model.clone(),
        }
    }
}

/// What the background thread says as the answer is made.
enum Update {
    /// Notes the answer has drawn on: this turn's first excerpts, then any
    /// the model searched or read through a tool.
    Used(Vec<Excerpt>),
    Text(String),
    Done,
    Error(String),
    /// The stream stopped without its end marker.
    CutOff,
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
    /// The exchange in progress: its question, and the words that arrived.
    asking: Option<String>,
    answer: String,
    /// Earlier exchanges, kept in memory for the panel's lifetime only.
    turns: Vec<AskedTurn>,
    /// The notes this turn's answer has drawn on, saved with it.
    turn_notes: Vec<String>,
    /// Completed turns as already-sanitised Markdown, so a streaming chunk
    /// only re-parses the turn in progress.
    finished: String,
    /// What the conversation was asked under, if it has one.
    conversation: Option<ConversationKey>,
    used: Vec<Excerpt>,
    cut_off: bool,
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
            asking: None,
            answer: String::new(),
            turns: Vec::new(),
            turn_notes: Vec::new(),
            finished: String::new(),
            conversation: None,
            used: Vec::new(),
            cut_off: false,
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

    /// Asks the question in the box as the next turn of this conversation.
    pub fn ask(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        let question = self.question.read(cx).value().trim().to_owned();
        if question.is_empty() {
            return;
        }
        let ai = cx.global::<AppState>().config.ai.clone();
        if !ai.enabled {
            return;
        }
        // Whatever is answering stops first, even if the new request cannot
        // be made: a failure to build the client must not leave an answer
        // running.
        self.stop(cx);
        // A locked note shown beside the panel ends the conversation.
        if self.editor.read(cx).lock_view() != crate::locked::LockView::Plain {
            self.clear_conversation();
        }
        // A different provider, URL or model is a different conversation: the
        // earlier answers were asked under conditions the person has changed.
        let conversation = ConversationKey::of(&ai);
        if self
            .conversation
            .as_ref()
            .is_some_and(|last| last != &conversation)
        {
            self.clear_conversation();
        }
        self.conversation = Some(conversation);
        // Drop any exchange whose notes can no longer be sent, and cap what
        // is resent by characters. `clear_conversation` above may have
        // emptied it already.
        self.prune_history();

        self.asked = true;
        self.answer.clear();
        self.empty = false;
        self.error = None;
        self.cut_off = false;
        self.searching = true;
        self.asking = Some(question.clone());
        let cancel = Arc::new(AtomicBool::new(false));
        self.cancel = Some(cancel.clone());
        cx.notify();

        let store = self.store.clone();
        let asked = question;
        let history: Vec<kanso_ai::Turn> = self
            .turns
            .iter()
            .map(|turn| kanso_ai::Turn {
                question: turn.question.clone(),
                answer: turn.answer.clone(),
            })
            .collect();
        let secrets = cx
            .try_global::<Services>()
            .and_then(|services| services.secrets.clone());
        let worker_ai = ai.clone();
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
                // The Keychain read and the client build happen here, never
                // on the UI thread: the macOS prompt can block, and building
                // a reqwest client is not work for a frame.
                let saved = secrets
                    .as_ref()
                    .and_then(|secrets| read_from(secrets.as_ref(), worker_ai.provider));
                let settings = ClientSettings {
                    provider: worker_ai.provider,
                    base_url: worker_ai.base_url.clone(),
                    model: worker_ai.model.clone(),
                    api_key: saved,
                    region: worker_ai.provider.default_region().to_owned(),
                };
                let provider = match kanso_ai::cached_client(&settings) {
                    Ok(provider) => provider,
                    Err(err) => {
                        tx.try_send(Update::Error(err.to_string())).ok();
                        return;
                    }
                };
                let provider = provider
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                // Every note this question's tools search or read is kept, so
                // the panel can list all of them.
                let used = Arc::new(std::sync::Mutex::new(excerpts));
                let tools = crate::ask::tool_registry(store.clone(), used.clone());
                let on_tool = {
                    let tx = tx.clone();
                    let used = used.clone();
                    Box::new(move || {
                        if let Ok(list) = used.lock() {
                            tx.try_send(Update::Used(list.clone())).ok();
                        }
                    })
                };
                crate::ask::stream_answer(
                    provider.as_ref(),
                    &prompt,
                    &history,
                    tools,
                    on_tool,
                    &cancel,
                    |event| {
                        if cancel.load(Ordering::Relaxed) {
                            return;
                        }
                        let update = match event {
                            kanso_ai::Event::Text(text) => Update::Text(text),
                            kanso_ai::Event::Done => Update::Done,
                            kanso_ai::Event::Error(message) if message == kanso_ai::CUT_OFF => {
                                Update::CutOff
                            }
                            kanso_ai::Event::Error(message) => Update::Error(message),
                        };
                        tx.try_send(update).ok();
                    },
                );
            });
        if spawned.is_err() {
            self.cancel = None;
            self.asking = None;
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
                                record_used(&mut this.turn_notes, &mut this.used, used);
                                this.searching = false;
                                this.streaming = true;
                            }
                            Update::Text(text) => this.answer.push_str(&text),
                            Update::Done => {
                                if let Some(question) = this.asking.take() {
                                    let answer = std::mem::take(&mut this.answer);
                                    if !answer.trim().is_empty() {
                                        let notes = std::mem::take(&mut this.turn_notes);
                                        this.turns.push(AskedTurn {
                                            question,
                                            answer,
                                            notes,
                                        });
                                    }
                                }
                                this.turn_notes.clear();
                                this.refresh_finished();
                                this.streaming = false;
                                this.searching = false;
                                this.cancel = None;
                            }
                            Update::CutOff => {
                                this.cut_off = true;
                                this.asking = None;
                                this.turn_notes.clear();
                                this.streaming = false;
                                this.searching = false;
                                this.cancel = None;
                            }
                            Update::Error(message) => {
                                this.streaming = false;
                                this.searching = false;
                                this.error = Some(message);
                                this.asking = None;
                                this.turn_notes.clear();
                                this.cancel = None;
                            }
                            Update::Empty => {
                                this.searching = false;
                                this.empty = true;
                                this.asking = None;
                                this.turn_notes.clear();
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
            // The worker or runtime ended without Done, Error or CutOff. Keep
            // Ask usable by settling the panel instead of leaving it running
            // forever.
            this.update(cx, |this, cx| {
                if this.settle_after_closed_channel() {
                    cx.notify();
                }
            })
            .ok();
        }));
    }

    /// Settles a request whose event channel closed without Done or Error.
    /// Returns whether it was still running (and so was settled).
    fn settle_after_closed_channel(&mut self) -> bool {
        if !self.searching && !self.streaming {
            return false;
        }
        self.cut_off = true;
        self.asking = None;
        self.turn_notes.clear();
        self.searching = false;
        self.streaming = false;
        self.cancel = None;
        true
    }

    #[cfg(test)]
    pub(crate) fn mark_running_for_test(&mut self) {
        self.searching = true;
        self.streaming = true;
        self.asking = Some("a question".to_owned());
        self.cut_off = false;
    }

    #[cfg(test)]
    pub(crate) fn settle_closed_for_test(&mut self) -> (bool, bool) {
        let settled = self.settle_after_closed_channel();
        (settled, self.cut_off)
    }

    /// Stops the answer being read. What arrived stays.
    pub fn stop(&mut self, cx: &mut Context<Self>) {
        if let Some(cancel) = self.cancel.take() {
            cancel.store(true, Ordering::Relaxed);
        }
        self.task = None;
        self.asking = None;
        self.searching = false;
        self.streaming = false;
        cx.notify();
    }

    /// Ends the conversation: cancels any answer and forgets every exchange.
    /// Called when the panel is cleared, closed, or AI is turned off.
    pub fn end_conversation(&mut self, cx: &mut Context<Self>) {
        self.stop(cx);
        self.clear_conversation();
        cx.notify();
    }

    fn clear_conversation(&mut self) {
        self.turns.clear();
        self.turn_notes.clear();
        self.finished.clear();
        self.conversation = None;
        self.used.clear();
        self.answer.clear();
        self.asking = None;
        self.asked = false;
        self.empty = false;
        self.error = None;
        self.cut_off = false;
    }

    /// Drops exchanges whose notes may no longer be sent, caps what is left
    /// by characters, and rebuilds the finished Markdown once.
    fn prune_history(&mut self) {
        let store = self.store.clone();
        self.turns = eligible_turns(std::mem::take(&mut self.turns), &store);
        self.turns = cap_history(std::mem::take(&mut self.turns), HISTORY_BUDGET);
        self.used
            .retain(|excerpt| crate::ask::note_is_eligible(&store, &excerpt.id));
        self.refresh_finished();
    }

    /// Rebuilds the already-rendered completed turns. Called when a turn ends
    /// or the conversation changes, never per streamed chunk.
    fn refresh_finished(&mut self) {
        let transcript = self.finished_transcript();
        self.finished = crate::ask::sanitize_answer(&transcript, &self.used);
    }

    fn finished_transcript(&self) -> String {
        let mut text = String::new();
        for turn in &self.turns {
            text.push_str("**You asked:** ");
            text.push_str(turn.question.trim());
            text.push_str("\n\n");
            text.push_str(&turn.answer);
            text.push_str("\n\n");
        }
        text
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
        if self.cut_off {
            return Some(
                div()
                    .text_size(px(12.5))
                    .line_height(px(18.))
                    .text_color(colors.text_secondary)
                    .child(kanso_ai::CUT_OFF)
                    .into_any_element(),
            );
        }
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
                    .child("Thinking…")
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

/// Drops every exchange that cited a note which is no longer eligible. Any
/// error in the eligibility check drops the turn too (fail closed).
/// Notes the answer being written read: every one goes on this turn's list,
/// even a note an earlier turn read too, so the turn is dropped if any of
/// them stops being eligible. `used` keeps one excerpt per note for links.
fn record_used(turn_notes: &mut Vec<String>, used: &mut Vec<Excerpt>, new: Vec<Excerpt>) {
    for excerpt in new {
        if !turn_notes.contains(&excerpt.id) {
            turn_notes.push(excerpt.id.clone());
        }
        if !used.iter().any(|seen| seen.id == excerpt.id) {
            used.push(excerpt);
        }
    }
}

fn eligible_turns(turns: Vec<AskedTurn>, store: &Store) -> Vec<AskedTurn> {
    turns
        .into_iter()
        .filter(|turn| {
            turn.notes
                .iter()
                .all(|id| crate::ask::note_is_eligible(store, id))
        })
        .collect()
}

/// Keeps the newest turns that fit `budget` characters, dropping the oldest.
/// The newest turn is always kept, even if it alone is over budget, so a
/// follow-up still has the exchange it follows.
fn cap_history(mut turns: Vec<AskedTurn>, budget: usize) -> Vec<AskedTurn> {
    let mut kept = Vec::new();
    let mut used = 0;
    while let Some(turn) = turns.pop() {
        let size = turn.chars();
        if !kept.is_empty() && used + size > budget {
            break;
        }
        used += size;
        kept.push(turn);
    }
    kept.reverse();
    kept
}

impl Render for AskPanel {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let ai = cx.global::<AppState>().config.ai.clone();
        let running = self.searching || self.streaming;
        // Only the turn in progress is sanitised again per chunk; finished
        // turns were rendered once (`finished`, refreshed at Done).
        let streaming = if self.answer.is_empty() {
            String::new()
        } else {
            crate::ask::sanitize_answer(&self.answer, &self.used)
        };
        let finished = self.finished.clone();
        let used = self.render_used(cx);
        let view = cx.entity();
        v_flex()
            .id("ask")
            .size_full()
            .px(px(10.))
            .pt(px(10.))
            .pb(px(10.))
            .gap(px(8.))
            .when(!self.asked && ai.provider.is_remote(&ai.base_url), |el| {
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
                    })
                    .when(
                        !running
                            && (self.asked || !self.turns.is_empty() || !self.used.is_empty()),
                        |el| {
                            let view = view.clone();
                            el.child(
                                Button::new("ask-clear")
                                    .outline()
                                    .small()
                                    .label("Clear")
                                    .on_click(move |_, _, cx| {
                                        view.update(cx, |this, cx| this.end_conversation(cx));
                                    }),
                            )
                        },
                    ),
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
            .when(!finished.is_empty() || !streaming.is_empty(), |el| {
                let mut answers = v_flex().gap(px(8.));
                if !finished.is_empty() {
                    let open = view.clone();
                    answers = answers.child(
                        TextView::markdown("ask-finished-text", finished)
                            .selectable(true)
                            .on_link_click(move |url, _, _, cx| {
                                let Some(id) =
                                    url.strip_prefix(kanso_core::links::NOTE_LINK_PREFIX)
                                else {
                                    return;
                                };
                                let id = format!("note:{id}");
                                open.update(cx, |this, cx| this.open(id, cx));
                            }),
                    );
                }
                if !streaming.is_empty() {
                    let open = view.clone();
                    answers = answers.child(
                        TextView::markdown("ask-answer-text", streaming)
                            .selectable(true)
                            .on_link_click(move |url, _, _, cx| {
                                let Some(id) =
                                    url.strip_prefix(kanso_core::links::NOTE_LINK_PREFIX)
                                else {
                                    return;
                                };
                                let id = format!("note:{id}");
                                open.update(cx, |this, cx| this.open(id, cx));
                            }),
                    );
                }
                el.child(
                    div()
                        .id("ask-answer")
                        .flex_1()
                        .min_h_0()
                        .overflow_y_scroll()
                        .child(answers),
                )
            })
            .when_some(used, |el, used| el.child(used))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AskedTurn, ConversationKey, Excerpt, HISTORY_BUDGET, cap_history, eligible_turns,
        record_used,
    };
    use crate::config::AiSettings;
    use kanso_ai::Provider;
    use kanso_core::DocType;
    use kanso_store::{Origin, Put, Store};
    use serde_json::json;

    fn turn(question: &str, answer: &str, notes: Vec<String>) -> AskedTurn {
        AskedTurn {
            question: question.to_owned(),
            answer: answer.to_owned(),
            notes,
        }
    }

    fn ai(provider: Provider, base_url: &str, model: &str) -> AiSettings {
        AiSettings {
            provider,
            base_url: base_url.to_owned(),
            model: model.to_owned(),
            ..AiSettings::default()
        }
    }

    #[test]
    fn a_different_provider_url_or_model_is_a_different_conversation() {
        let start = ConversationKey::of(&ai(
            Provider::LocalModel,
            "http://localhost:11434/v1",
            "llama3.2",
        ));
        let same = ConversationKey::of(&ai(
            Provider::LocalModel,
            "http://localhost:11434/v1",
            "llama3.2",
        ));
        let provider = ConversationKey::of(&ai(
            Provider::OpenAi,
            "http://localhost:11434/v1",
            "llama3.2",
        ));
        let url = ConversationKey::of(&ai(
            Provider::LocalModel,
            "https://example.com/v1",
            "llama3.2",
        ));
        let model = ConversationKey::of(&ai(
            Provider::LocalModel,
            "http://localhost:11434/v1",
            "gemma",
        ));
        assert_eq!(start, same);
        assert_ne!(start, provider);
        assert_ne!(start, url);
        assert_ne!(start, model);
    }

    #[test]
    fn history_is_capped_by_dropping_the_oldest_turns() {
        let turns = vec![
            turn("old", &"a".repeat(60), vec![]),
            turn("middle", &"b".repeat(60), vec![]),
            turn("new", &"c".repeat(60), vec![]),
        ];
        let kept = cap_history(turns, 140);
        let questions: Vec<&str> = kept.iter().map(|turn| turn.question.as_str()).collect();
        assert_eq!(questions, ["middle", "new"]);
        assert!(kept.iter().map(AskedTurn::chars).sum::<usize>() <= 140);
        assert!(cap_history(vec![], 10).is_empty());
    }

    #[test]
    fn a_turn_whose_note_is_locked_is_dropped() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("kanso.db")).unwrap();
        let book = store
            .put(
                DocType::Book,
                json!({"name": "Notes"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        let note = store
            .put(
                DocType::Note,
                json!({"title": "Pricing", "body": "The plan costs 40.", "bookId": book}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        let turns = vec![turn("what?", "the plan", vec![note.clone()])];
        assert_eq!(eligible_turns(turns.clone(), &store).len(), 1);
        store
            .put(
                DocType::Note,
                json!({"_id": note, "kansoLock": {"salt": "abc"}}),
                Put::Merge,
                Origin::Sync,
            )
            .unwrap();
        assert!(eligible_turns(turns, &store).is_empty());
    }

    #[test]
    fn a_note_read_again_is_on_every_turn_that_read_it() {
        let excerpt = |id: &str| Excerpt {
            id: id.to_owned(),
            title: String::new(),
            text: String::new(),
        };
        let mut used = Vec::new();
        let mut first = Vec::new();
        record_used(
            &mut first,
            &mut used,
            vec![excerpt("note:a"), excerpt("note:b")],
        );
        let mut second = Vec::new();
        record_used(
            &mut second,
            &mut used,
            vec![excerpt("note:a"), excerpt("note:a")],
        );
        assert_eq!(first, ["note:a", "note:b"]);
        assert_eq!(second, ["note:a"], "the second turn read note:a too");
        assert_eq!(used.len(), 2, "one excerpt per note for links");
    }

    #[test]
    fn an_eligibility_error_drops_the_turn() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("kanso.db")).unwrap();
        let book = store
            .put(
                DocType::Book,
                json!({"name": "Notes"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        let note = store
            .put(
                DocType::Note,
                json!({"title": "Pricing", "body": "The plan costs 40.", "bookId": book}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        {
            let conn = rusqlite::Connection::open(store.path()).unwrap();
            conn.execute("DROP VIEW locked_books", []).unwrap();
        }
        let turns = vec![turn("what?", "the plan", vec![note])];
        assert!(eligible_turns(turns, &store).is_empty());
    }

    #[test]
    fn the_history_budget_is_documented() {
        assert_eq!(HISTORY_BUDGET, 12_000);
    }
}
