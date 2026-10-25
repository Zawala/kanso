//! What an agent suggested: one edit waiting for the person.
//!
//! Plain data, held in the [`ReviewQueue`](crate::review::ReviewQueue). A
//! suggestion is never a write: nothing here touches the store, and the only
//! way it becomes one is the person's Apply
//! ([`ReviewQueue::apply`](crate::review::ReviewQueue::apply)).

use std::sync::Arc;

use crate::scope::AgentScope;
use crate::tokens::TokenId;

/// Which suggestion, for as long as the app runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ChangeId(pub(crate) u64);

impl ChangeId {
    /// The number behind the ID, for element IDs.
    pub fn get(self) -> u64 {
        self.0
    }
}

/// An edit an agent suggested to the text or title of a note, or a new note.
#[derive(Clone, Debug)]
pub struct ProposedChange {
    pub id: ChangeId,
    /// The note it changes; `None` for a new note.
    pub note_id: Option<String>,
    /// The revision of the note the agent read; `None` for a new note.
    pub base_revision: Option<String>,
    /// The notebook a new note goes in; `None` for a change to a note.
    pub book_id: Option<String>,
    /// The note's title and text when the agent read it; empty for a new
    /// note.
    pub old_title: String,
    pub old_body: String,
    /// The title and text if the change is applied.
    pub new_title: String,
    pub new_body: String,
    /// What the person calls the agent, as the queue and the note's history
    /// name it.
    pub agent: String,
    /// The agent session's token. The token may be gone by Apply; the
    /// suggestion outlives it.
    pub token: TokenId,
    /// When it was suggested, in milliseconds since the epoch.
    pub created_at: i64,
    /// The scope the agent had, to decide again at Apply whether the note is
    /// still one it may change.
    pub(crate) scope: Arc<AgentScope>,
}

impl ProposedChange {
    /// Whether this suggests a new note.
    pub fn is_new_note(&self) -> bool {
        self.note_id.is_none()
    }
}

/// A suggestion before the queue gives it an ID and a time.
#[derive(Clone, Debug)]
pub struct NewChange {
    pub note_id: Option<String>,
    pub base_revision: Option<String>,
    pub book_id: Option<String>,
    pub old_title: String,
    pub old_body: String,
    pub new_title: String,
    pub new_body: String,
    pub agent: String,
    pub token: TokenId,
    pub scope: Arc<AgentScope>,
}

/// What Apply did, or why it did not.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ApplyError {
    /// No such suggestion: it was applied, discarded or has expired.
    Gone,
    /// The note, or the notebook for a new one, is locked, in the trash,
    /// tagged `#ai-exclude` or outside the agent's notebooks now. The
    /// suggestion is dropped.
    NotAllowed,
    /// The note changed since the agent read it. Nothing was written and the
    /// suggestion stays, so the person can compare it with the note as it is.
    Conflict {
        current_title: String,
        current_body: String,
    },
    /// The store refused or failed. Nothing was written and the suggestion
    /// stays.
    Failed(String),
}

impl ApplyError {
    /// What the person is told.
    pub fn message(&self) -> &str {
        match self {
            ApplyError::Gone => {
                "This suggestion is no longer waiting. It was applied, discarded or has expired."
            }
            ApplyError::NotAllowed => {
                "This note can no longer be changed by an agent. The suggestion was dropped."
            }
            ApplyError::Conflict { .. } => "The note changed since the agent read it.",
            ApplyError::Failed(message) => message,
        }
    }
}

/// A suggestion that was written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Applied {
    pub note_id: String,
    pub title: String,
}
