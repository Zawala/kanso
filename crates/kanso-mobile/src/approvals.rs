//! Requests from other devices to join the account, which this phone can approve.

use std::sync::Arc;

use crate::library::Inner;

/// Requests from other devices to join the account, which this phone can approve.
#[derive(Clone)]
pub struct Approvals {
    pub(crate) inner: Arc<Inner>,
}
