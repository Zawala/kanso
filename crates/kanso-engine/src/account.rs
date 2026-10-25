//! The account tasks, and what they say (docs/ACCOUNTS.md).

pub mod approval;
pub mod words;

/// Whether `server` is the compiled-in default, however it is written, or
/// the same server under its first name.
pub fn is_default_server(server: &str) -> bool {
    kanso_sync::check_endpoint(server).is_ok_and(|server| kanso_sync::is_owner_server(&server))
}
