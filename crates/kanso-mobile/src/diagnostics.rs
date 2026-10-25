//! What the shell asks of the library about itself: its version, whether
//! certificates can be checked, and a panic on demand to prove panics come
//! back as errors on a device.

use crate::boundary::guard;
use crate::errors::KansoError;

/// The library's version, the workspace's.
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_owned()
}

/// Panics with `message` inside the boundary: the result is
/// [`KansoError::Internal`], and every other call works afterwards. It holds
/// no lock of the store's, so it can't poison one. It ships in the release
/// library so a device test can check that a panic is caught there too.
pub fn panic_probe(message: String) -> Result<(), KansoError> {
    guard(|| panic!("{message}"))
}

/// Whether the library can check a server's certificate now: always on iOS
/// and anywhere else, on Android once the app has handed over its `Context`.
pub fn tls_ready() -> bool {
    kanso_tls::ready().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    // no-prop-test: the probe's message is arbitrary text; boundary.rs's
    // property covers every message, this checks the public call.
    #[test]
    fn the_probe_returns_internal_and_the_next_call_still_works() {
        let failed = panic_probe("probe".into());
        assert!(matches!(failed, Err(KansoError::Internal { message }) if message == "probe"));
        assert_eq!(version(), env!("CARGO_PKG_VERSION"));
        assert_eq!(tls_ready(), !cfg!(target_os = "android"));
    }
}
