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
