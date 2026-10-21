//! What related notes' embeddings say about the open note, for the views
//! around it: the tags its nearest notes suggest, for the tag list, `#`
//! autocomplete and the inspector.
//!
//! Read from the stored vectors in the background whenever another note
//! opens or new embeddings are kept, so nothing waits on it; nothing is
//! there while related notes are off or the note isn't embedded yet.

use std::sync::Arc;

use gpui_kit::*;
use kanso_store::{Store, TagSuggestion};

use crate::related::{MODEL_ID, Related};

/// How many tags are suggested at once.
pub const SUGGESTED_TAGS: usize = 5;

pub struct NoteMeaning {
    store: Arc<Store>,
    /// The open note, and the embeddings (by `Related::generation`) what is
    /// kept was read from.
    note: Option<String>,
    read_for: Option<(String, u64)>,
    suggested: Vec<TagSuggestion>,
    task: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
}

impl NoteMeaning {
    pub fn new(store: Arc<Store>, cx: &mut Context<Self>) -> Self {
        // New embeddings, or related notes turned on or off: read again.
        let subscriptions = vec![cx.observe_global::<Related>(|this, cx| this.reload(cx))];
        NoteMeaning {
            store,
            note: None,
            read_for: None,
            suggested: Vec::new(),
            task: None,
            _subscriptions: subscriptions,
        }
    }

    /// The note open now (`None`: none).
    pub fn follow(&mut self, note: Option<&str>, cx: &mut Context<Self>) {
        if self.note.as_deref() == note {
            return;
        }
        self.note = note.map(str::to_owned);
        self.suggested.clear();
        self.reload(cx);
        cx.notify();
    }

    fn reload(&mut self, cx: &mut Context<Self>) {
        let generation = cx.try_global::<Related>().map_or(0, |r| r.generation);
        let Some(id) = self.note.clone() else {
            self.read_for = None;
            self.task = None;
            return;
        };
        let wanted = Some((id.clone(), generation));
        if self.read_for == wanted {
            return;
        }
        self.read_for = wanted;
        let store = self.store.clone();
        let read = cx.background_executor().spawn(async move {
            store
                .tag_suggestions(&id, MODEL_ID, SUGGESTED_TAGS)
                .ok()
                .flatten()
                .unwrap_or_default()
        });
        self.task = Some(cx.spawn(async move |this, cx| {
            let suggested = read.await;
            this.update(cx, |this, cx| {
                this.task = None;
                if this.suggested != suggested {
                    this.suggested = suggested;
                    cx.notify();
                }
            })
            .ok();
        }));
    }

    /// Tags suggested for the open note that `own` (its tags now) hasn't
    /// got, heaviest first.
    pub fn suggested_tags(&self, own: &[String]) -> Vec<TagSuggestion> {
        self.suggested
            .iter()
            .filter(|s| !own.contains(&s.tag_id))
            .cloned()
            .collect()
    }

    /// Suggestions as the snapshots draw them.
    #[cfg(test)]
    pub fn set_suggested_for_test(&mut self, suggested: Vec<TagSuggestion>) {
        self.suggested = suggested;
    }
}
