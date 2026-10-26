//! Sign-in, sign-up, provider and passkey flows, locked sessions, device approval as the new device, recovery and the binding question.

use std::sync::Arc;

use crate::library::Inner;

/// Sign-in, sign-up, provider and passkey flows, locked sessions, device approval as the new device, recovery and the binding question.
#[derive(Clone)]
pub struct AccountHandle {
    pub(crate) inner: Arc<Inner>,
}
