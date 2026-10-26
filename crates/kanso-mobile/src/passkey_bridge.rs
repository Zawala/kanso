//! Where a passkey ceremony's answer comes back: from the native plugin
//! straight to Rust (`kanso_passkey_answer` in `mobile/rust`), never through
//! Dart, so the PRF output never reaches the shell.

/// Hands the native plugin's `json` answer to the ceremony waiting on
/// `ticket`. False for a ticket nobody waits on.
pub fn answer(ticket: u64, json: &str) -> bool {
    let _ = (ticket, json);
    false
}
