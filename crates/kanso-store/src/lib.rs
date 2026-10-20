//! Kanso's document store.
//!
//! Every document is kept as its Inkdrop-shaped JSON plus indexed columns
//! derived from it. Writes are serialized through one connection inside
//! `BEGIN IMMEDIATE`; reads use a small pool of connections. Each committed
//! write is broadcast to subscribers as a [`Change`].

mod backup;
mod derived;
mod doc;
mod history;
mod read;
mod replace;
mod roles;
mod schema;
mod sync;
mod tasks;
mod validate;
mod write;

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use rusqlite::Connection;

pub use backup::BackupPolicy;
pub use derived::{Backlink, DERIVED_VERSION, DERIVED_VERSION_KEY, HeadingHit};
pub use doc::{Attachment, Doc};
pub use read::{NoteSummary, Revision, SearchParams, Sort, StatusCounts};
pub use replace::{
    NoteMatches, PREVIEW_MATCHES_PER_NOTE, Planned, ReplaceReport, Skipped, UndoReport,
};
pub use roles::{DAILY_TEMPLATE_FIELD, DailyNote, FromTemplate, NewNote};
pub use sync::{ApplyOutcome, SyncDoc, SyncGuard, SyncState};
pub use tasks::{CheckOff, ReminderTask, TaskQuery, TaskRow};
pub use write::{Put, WriteResult};

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("not found")]
    NotFound,
    #[error("Document update conflict")]
    Conflict,
    #[error("{0}")]
    Invalid(String),
    /// A guarded sync write found the library bound to another account (or
    /// none): its [`SyncGuard`] didn't match `meta`. Nothing was written.
    /// See docs/ACCOUNTS.md › Sync with encryption › The guard.
    #[error("the library is no longer bound to the account this sync started with")]
    GuardFailed,
    #[error("database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("invalid JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T, E = StoreError> = std::result::Result<T, E>;

/// Who made a write. Decides revision snapshots and sync bookkeeping.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    /// The desktop UI. Snapshots are throttled so autosave doesn't flood history.
    Ui,
    /// The local HTTP API (MCP clients, scripts). Every write is snapshotted.
    Api,
    /// The Cloudflare sync client applying a remote change.
    Sync,
    /// The Inkdrop importer.
    Import,
}

impl Origin {
    pub fn as_str(self) -> &'static str {
        match self {
            Origin::Ui => "ui",
            Origin::Api => "api",
            Origin::Sync => "sync",
            Origin::Import => "import",
        }
    }
}

/// A committed write, broadcast to subscribers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Change {
    pub id: String,
    pub rev: String,
    pub seq: i64,
    pub deleted: bool,
    pub origin: Origin,
}

const READER_POOL: usize = 4;

pub struct Store {
    path: PathBuf,
    writer: Mutex<Connection>,
    readers: Mutex<Vec<Connection>>,
    subscribers: Mutex<Vec<async_channel::Sender<Change>>>,
}

impl Store {
    /// Opens (creating if needed) the database at `path` and migrates it.
    pub fn open(path: impl AsRef<Path>) -> Result<Store> {
        let path = path.as_ref().to_path_buf();
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let mut writer = schema::connect(&path)?;
        schema::migrate(&writer)?;
        derived::rebuild_if_stale(&mut writer)?;
        Ok(Store {
            path,
            writer: Mutex::new(writer),
            readers: Mutex::new(Vec::new()),
            subscribers: Mutex::new(Vec::new()),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// A channel that receives every committed change from now on.
    pub fn subscribe(&self) -> async_channel::Receiver<Change> {
        let (tx, rx) = async_channel::unbounded();
        self.subscribers
            .lock()
            .expect("subscribers poisoned")
            .push(tx);
        rx
    }

    fn broadcast(&self, changes: &[Change]) {
        if changes.is_empty() {
            return;
        }
        let mut subscribers = self.subscribers.lock().expect("subscribers poisoned");
        subscribers.retain(|tx| changes.iter().all(|c| tx.try_send(c.clone()).is_ok()));
    }

    /// Runs `f` on a pooled read connection.
    fn read<T>(&self, f: impl FnOnce(&Connection) -> Result<T>) -> Result<T> {
        let conn = self.readers.lock().expect("readers poisoned").pop();
        let conn = match conn {
            Some(conn) => conn,
            None => schema::connect(&self.path)?,
        };
        let result = f(&conn);
        let mut readers = self.readers.lock().expect("readers poisoned");
        if readers.len() < READER_POOL {
            readers.push(conn);
        }
        result
    }

    /// Runs `f` inside one `BEGIN IMMEDIATE` transaction on the writer and
    /// broadcasts the changes it reports once the transaction commits.
    fn write<T>(
        &self,
        f: impl FnOnce(&rusqlite::Transaction, &mut Vec<Change>) -> Result<T>,
    ) -> Result<T> {
        let mut conn = self.writer.lock().expect("writer poisoned");
        let tx = rusqlite::Transaction::new(&mut conn, rusqlite::TransactionBehavior::Immediate)?;
        let mut changes = Vec::new();
        let value = f(&tx, &mut changes)?;
        tx.commit()?;
        drop(conn);
        self.broadcast(&changes);
        Ok(value)
    }

    /// Reads a value from the `meta` table.
    pub fn meta(&self, key: &str) -> Result<Option<String>> {
        self.read(|conn| schema::get_meta(conn, key))
    }

    /// Writes a value to the `meta` table.
    pub fn set_meta(&self, key: &str, value: &str) -> Result<()> {
        self.write(|tx, _| schema::set_meta(tx, key, value))
    }
}

/// An exclusive lock on a data directory, held for the process lifetime.
/// The holder owns the database writer and the API port.
pub struct DataDirLock {
    _file: std::fs::File,
}

/// Takes the data-directory lock, or returns `None` when another process holds it.
pub fn lock_data_dir(dir: &Path) -> Result<Option<DataDirLock>> {
    std::fs::create_dir_all(dir)?;
    let file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(dir.join(".lock"))?;
    match file.try_lock() {
        Ok(()) => Ok(Some(DataDirLock { _file: file })),
        Err(std::fs::TryLockError::WouldBlock) => Ok(None),
        Err(std::fs::TryLockError::Error(err)) => Err(err.into()),
    }
}
