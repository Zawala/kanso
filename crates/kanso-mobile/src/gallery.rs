//! The theme gallery: the signed index, installing and removing themes.

use std::sync::Arc;

use crate::library::Inner;

/// The theme gallery: the signed index, installing and removing themes.
#[derive(Clone)]
pub struct Gallery {
    pub(crate) inner: Arc<Inner>,
}
