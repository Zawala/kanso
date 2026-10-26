//! Settings › Account: the signed-in account's devices and ways in, its password and recovery key, signing out and deleting it.

use std::sync::Arc;

use crate::library::Inner;

/// Settings › Account: the signed-in account's devices and ways in, its password and recovery key, signing out and deleting it.
#[derive(Clone)]
pub struct AccountSettings {
    pub(crate) inner: Arc<Inner>,
}
