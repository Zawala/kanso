//! Shared notes: sharing, invitations, links, joining, and the live room of an open shared note.

use std::sync::Arc;

use crate::library::Inner;

/// Shared notes: sharing, invitations, links, joining, and the live room of an open shared note.
#[derive(Clone)]
pub struct SharedNotes {
    pub(crate) inner: Arc<Inner>,
}
