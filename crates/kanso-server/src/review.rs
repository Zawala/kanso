//! The review queue: where an agent's suggested edits wait for the person.
//!
//! An agent whose scope may suggest edits never writes a note. Its
//! `patch-note`, `update-note` and `create-note` calls stage a
//! [`ProposedChange`] here (`mcp/suggest.rs`) and the store stays as it was.
//! The only way a suggestion reaches the store is [`ReviewQueue::apply`],
//! which the app calls when the person presses Apply. It writes at most once,
//! only while the note is as the agent read it, and only if the agent could
//! still see the note: locking it or its notebook, trashing it or tagging it
//! `#ai-exclude` since drops the suggestion.
//!
//! The queue lives in memory: a suggestion expires [`EXPIRY`] after it was
//! made and is gone when the app quits, so a crash loses suggestions, never
//! a note. Clearing the Ask conversation or ending an agent's token does not
//! drop them.

use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use kanso_core::{DocType, now_ms};
use kanso_store::{Put, Store, StoreError};
use serde_json::json;

use crate::proposal::{Applied, ApplyError, ChangeId, NewChange, ProposedChange};
use crate::scope::Visibility;
use crate::tokens::TokenId;

/// How long a suggestion waits before it expires.
pub const EXPIRY: Duration = Duration::from_secs(60 * 60);
/// The most suggestions waiting at once.
pub const MAX_PENDING: usize = 50;
/// The most text, in bytes, one suggestion may carry.
pub const MAX_TEXT_BYTES: usize = 1 << 20;

/// What subscribers are told.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReviewEvent {
    /// A suggestion was made.
    Arrived(ChangeId),
    /// One was applied, discarded, dropped or expired.
    Changed,
}

/// The queue is full.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QueueFull;

type Clock = Arc<dyn Fn() -> Instant + Send + Sync>;

struct Entry {
    change: ProposedChange,
    made: Instant,
}

struct Inner {
    entries: Vec<Entry>,
    next_id: u64,
    subscribers: Vec<async_channel::Sender<ReviewEvent>>,
}

impl Inner {
    fn tell(&mut self, event: ReviewEvent) {
        self.subscribers
            .retain(|subscriber| subscriber.try_send(event).is_ok());
    }

    /// Drops what has expired; tells subscribers if anything did.
    fn prune(&mut self, now: Instant) {
        let before = self.entries.len();
        self.entries
            .retain(|entry| now.saturating_duration_since(entry.made) < EXPIRY);
        if self.entries.len() != before {
            self.tell(ReviewEvent::Changed);
        }
    }
}

/// The suggestions waiting for review. Cloning shares the same queue: the app
/// keeps one (it rides with the agents' [`TokenStore`](crate::TokenStore))
/// and the server stages into it.
#[derive(Clone)]
pub struct ReviewQueue {
    inner: Arc<Mutex<Inner>>,
    clock: Clock,
}

impl Default for ReviewQueue {
    fn default() -> Self {
        ReviewQueue::new()
    }
}

impl fmt::Debug for ReviewQueue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReviewQueue")
            .field("pending", &self.len())
            .finish()
    }
}

impl ReviewQueue {
    pub fn new() -> ReviewQueue {
        ReviewQueue::with_clock(Arc::new(Instant::now))
    }

    /// A queue reading the time from `clock`, for tests.
    pub fn with_clock(clock: Arc<dyn Fn() -> Instant + Send + Sync>) -> ReviewQueue {
        ReviewQueue {
            inner: Arc::new(Mutex::new(Inner {
                entries: Vec::new(),
                next_id: 1,
                subscribers: Vec::new(),
            })),
            clock,
        }
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        inner.prune((self.clock)());
        inner
    }

    /// Tells the receiver of every change to the queue: a suggestion that
    /// arrived, and any that went. The channel is unbounded and a dropped
    /// receiver is forgotten.
    pub fn subscribe(&self) -> async_channel::Receiver<ReviewEvent> {
        let (sender, receiver) = async_channel::unbounded();
        self.lock().subscribers.push(sender);
        receiver
    }

    /// How many suggestions are waiting.
    pub fn len(&self) -> usize {
        self.lock().entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The suggestions waiting, oldest first.
    pub fn list(&self) -> Vec<ProposedChange> {
        self.lock()
            .entries
            .iter()
            .map(|entry| entry.change.clone())
            .collect()
    }

    /// Suggestion `id`, while it waits.
    pub fn get(&self, id: ChangeId) -> Option<ProposedChange> {
        self.lock()
            .entries
            .iter()
            .find(|entry| entry.change.id == id)
            .map(|entry| entry.change.clone())
    }

    /// The suggestion `token` already has waiting for note `note_id`, when
    /// the agent read the note at `base_revision`: a second edit to the same
    /// note builds on it rather than beside it.
    pub(crate) fn pending_for(
        &self,
        token: TokenId,
        note_id: &str,
        base_revision: &str,
    ) -> Option<ProposedChange> {
        self.lock()
            .entries
            .iter()
            .map(|entry| &entry.change)
            .find(|change| {
                change.token == token
                    && change.note_id.as_deref() == Some(note_id)
                    && change.base_revision.as_deref() == Some(base_revision)
            })
            .cloned()
    }

    /// Queues `new`. A suggestion that builds on one already waiting
    /// (`pending_for`) replaces it, in place.
    pub(crate) fn stage(&self, new: NewChange) -> Result<ChangeId, QueueFull> {
        let made = (self.clock)();
        let mut inner = self.lock();
        let same = new
            .note_id
            .as_deref()
            .zip(new.base_revision.as_deref())
            .and_then(|(note, rev)| {
                inner.entries.iter().position(|entry| {
                    entry.change.token == new.token
                        && entry.change.note_id.as_deref() == Some(note)
                        && entry.change.base_revision.as_deref() == Some(rev)
                })
            });
        let id = match same {
            Some(at) => {
                let entry = &mut inner.entries[at];
                entry.change.new_title = new.new_title;
                entry.change.new_body = new.new_body;
                entry.made = made;
                entry.change.created_at = now_ms();
                entry.change.id
            }
            None => {
                if inner.entries.len() >= MAX_PENDING {
                    return Err(QueueFull);
                }
                let id = ChangeId(inner.next_id);
                inner.next_id += 1;
                inner.entries.push(Entry {
                    change: ProposedChange {
                        id,
                        note_id: new.note_id,
                        base_revision: new.base_revision,
                        book_id: new.book_id,
                        old_title: new.old_title,
                        old_body: new.old_body,
                        new_title: new.new_title,
                        new_body: new.new_body,
                        agent: new.agent,
                        token: new.token,
                        created_at: now_ms(),
                        scope: new.scope,
                    },
                    made,
                });
                id
            }
        };
        inner.tell(ReviewEvent::Arrived(id));
        Ok(id)
    }

    /// Throws suggestion `id` away. `false` when it isn't waiting.
    pub fn discard(&self, id: ChangeId) -> bool {
        let mut inner = self.lock();
        let before = inner.entries.len();
        inner.entries.retain(|entry| entry.change.id != id);
        let removed = inner.entries.len() != before;
        if removed {
            inner.tell(ReviewEvent::Changed);
        }
        removed
    }

    /// Drops every suggestion that can no longer be applied, because its
    /// note (or notebook) is locked, in the trash, tagged `#ai-exclude` or
    /// outside the agent's notebooks, and says how many went.
    pub fn drop_unappliable(&self, store: &Store) -> usize {
        let mut inner = self.lock();
        let before = inner.entries.len();
        inner
            .entries
            .retain(|entry| Self::check(store, &entry.change) != Err(ApplyError::NotAllowed));
        let dropped = before - inner.entries.len();
        if dropped > 0 {
            inner.tell(ReviewEvent::Changed);
        }
        dropped
    }

    /// Whether `change` may be applied to `store` now: the agent could still
    /// see its note, or the notebook for a new one.
    fn check(store: &Store, change: &ProposedChange) -> Result<(), ApplyError> {
        let Some(visible) = Visibility::new(store, &change.scope) else {
            return Err(ApplyError::Failed("Couldn't read the notes.".into()));
        };
        let allowed = match (&change.note_id, &change.book_id) {
            (Some(note), _) => visible.allows(note),
            (None, Some(book)) => visible.book_visible(book),
            (None, None) => false,
        };
        if allowed {
            Ok(())
        } else {
            Err(ApplyError::NotAllowed)
        }
    }

    /// Writes suggestion `id` to `store` as the agent's edit, approved by the
    /// person who called this, and takes it off the queue: once only, even if
    /// called twice at once (the queue is locked throughout).
    ///
    /// Refused, with nothing written, when the suggestion is gone
    /// ([`ApplyError::Gone`]); when the agent could not see the note now
    /// ([`ApplyError::NotAllowed`], and it is dropped); when the note is no
    /// longer at the revision the agent read ([`ApplyError::Conflict`]); or
    /// when the store fails. In the last two it stays queued.
    pub fn apply(&self, store: &Store, id: ChangeId) -> Result<Applied, ApplyError> {
        let mut inner = self.lock();
        let Some(at) = inner
            .entries
            .iter()
            .position(|entry| entry.change.id == id)
        else {
            return Err(ApplyError::Gone);
        };
        let change = inner.entries[at].change.clone();
        let applied = match Self::check(store, &change) {
            Err(ApplyError::NotAllowed) => {
                inner.entries.remove(at);
                inner.tell(ReviewEvent::Changed);
                return Err(ApplyError::NotAllowed);
            }
            Err(other) => return Err(other),
            Ok(()) => Self::write(store, &change)?,
        };
        inner.entries.remove(at);
        inner.tell(ReviewEvent::Changed);
        Ok(applied)
    }

    fn write(store: &Store, change: &ProposedChange) -> Result<Applied, ApplyError> {
        let failed = |err: StoreError| ApplyError::Failed(err.to_string());
        let Some(note_id) = &change.note_id else {
            let body = json!({
                "title": change.new_title,
                "body": change.new_body,
                "bookId": change.book_id,
            });
            let written = store
                .put_by_agent(DocType::Note, body, Put::Upsert, &change.agent)
                .map_err(failed)?;
            return Ok(Applied {
                note_id: written.id,
                title: change.new_title.clone(),
            });
        };
        let current = |store: &Store| -> Result<(String, String, String), ApplyError> {
            let note = store
                .note(note_id)
                .map_err(failed)?
                .ok_or(ApplyError::NotAllowed)?;
            Ok((note.rev.unwrap_or_default(), note.title, note.body))
        };
        let (rev, title, body) = current(store)?;
        let conflict = |title, body| ApplyError::Conflict {
            current_title: title,
            current_body: body,
        };
        if Some(rev.as_str()) != change.base_revision.as_deref() {
            return Err(conflict(title, body));
        }
        let patch = json!({
            "_id": note_id,
            "_rev": rev,
            "title": change.new_title,
            "body": change.new_body,
            "updatedAt": now_ms(),
        });
        match store.put_by_agent(DocType::Note, patch, Put::Merge, &change.agent) {
            Ok(written) => Ok(Applied {
                note_id: written.id,
                title: change.new_title.clone(),
            }),
            Err(StoreError::Conflict | StoreError::NotFound) => {
                let (_, title, body) = current(store)?;
                Err(conflict(title, body))
            }
            Err(err) => Err(failed(err)),
        }
    }
}
