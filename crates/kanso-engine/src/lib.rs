//! Kanso's engine without a window: the parts of the app that decide what
//! happens, with nothing drawn (issue #48).
//!
//! The desktop app and the mobile app both call it. [`lock`] holds the keys
//! of unlocked notes and seals and opens notes with them, [`account`] runs
//! the account tasks (sign up, sign in, recover, unlock a device, restore at
//! launch) and says their results in plain words, and [`sync`] decides when
//! a sync starts, queues, backs off or stops.
//!
//! What the note list shows is here: [`list`] has the places, the search each
//! one is, the sort and narrowing, the order of the notebook tree and the
//! date headers a list sits under.
//!
//! What a note takes from outside is here as well: [`attach`] stores images
//! and PDFs as attachments and gives the Markdown that shows them, and
//! [`capture`] turns typed text, and what other apps share, into notes in
//! the Inbox.
//!
//! What happens in a note is here too. [`links`] decides how wiki-links
//! resolve, what `[[` and `#` autocomplete offer, which link a note copies
//! and when a rename offers to update links; [`fuzzy`] ranks what was typed
//! against candidates; [`daily`] makes daily notes and notes from templates;
//! [`tasks`] reads due dates and decides reminders; [`edit`] turns the
//! formatting toolbar, auto-pairing and Tab and Enter in lists and tables
//! into one change to the text; [`saving`] decides when typing is written
//! and when another writer's version is no conflict; [`text`] converts
//! offsets between UTF-8 and UTF-16; [`text_stats`] counts words and finds
//! the caret's line and column; [`word_goal`] keeps a note's word goal;
//! [`editor_settings`] has the editor's settings as the config file names
//! them; [`icons`] has the icon theme file format and what a theme draws for
//! an icon or a library slot; [`theme`] has the palettes, the theme file
//! format and how three themes are fitted together for drawing; and `highlight` (the `highlight`
//! feature, with the `code-grammars` feature for the code in fenced blocks)
//! says which parts of a note are headings, markers, links, keywords and the
//! like, as spans of tokens the theme colors. It is a tree-sitter
//! highlighter on the runtime and grammar crates the desktop editor's
//! component library compiles in, with the same queries, and a test in
//! `kanso-app` holds the two to the same colors on a corpus of notes.
//!
//! Nothing here opens a window, reads a clock of its own or touches the
//! system's secret store: the caller passes the time, and secrets go through
//! [`Secrets`].
//!
//! The system's Keychain is `kanso_sync::Keychain`, built by the desktop app
//! alone; tests use [`MemorySecrets`].

pub mod account;
pub mod attach;
pub mod batch;
pub mod capture;
pub mod daily;
pub mod edit;
pub mod editor_settings;
pub mod fuzzy;
#[cfg(feature = "highlight")]
pub mod highlight;
pub mod icons;
pub mod links;
pub mod list;
pub mod lock;
pub mod saving;
#[cfg(any(test, feature = "test-support"))]
pub mod strategies;
pub mod sync;
pub mod tasks;
pub mod text;
pub mod text_stats;
pub mod theme;
pub mod word_goal;

/// Where secrets live: the seam the Keychain, the Credential Manager and the
/// mobile keystores stand behind.
pub use kanso_sync::SecretStore as Secrets;
pub use kanso_sync::{MemorySecrets, SecretError};
