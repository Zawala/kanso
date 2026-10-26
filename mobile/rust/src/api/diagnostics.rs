//! The library's version, TLS check and panic probe.

use kanso_mobile::KansoError;

/// The library's version.
pub fn version() -> String {
    kanso_mobile::diagnostics::version()
}

/// Panics inside the library's boundary: comes back as `KansoError::Internal`.
pub fn panic_probe(message: String) -> Result<(), KansoError> {
    kanso_mobile::diagnostics::panic_probe(message)
}

/// Whether the library can check a server's certificate now.
pub fn tls_ready() -> bool {
    kanso_mobile::diagnostics::tls_ready()
}

/// Writes a value to the phone's secret store, reads it back and removes it.
pub fn secrets_probe() -> Result<(), KansoError> {
    kanso_mobile::diagnostics::secrets_probe()
}

/// Whether the library folder at `folder` is kept out of the device's
/// backups: `Some(true)` on iOS once a library is open in it, `None` on
/// Android (its manifest does that) and for a folder that isn't there.
pub fn backup_excluded(folder: String) -> Option<bool> {
    kanso_mobile::diagnostics::backup_excluded(std::path::Path::new(&folder))
}
