//! The plain data the facade returns and takes: rows, views, queries and
//! statuses. No logic lives here; the modules that build them say what each
//! field holds.
//!
//! Every type is `Clone + Debug + PartialEq` with public fields, so the
//! bridge can mirror them field by field. Counts are `u32`, times are
//! milliseconds since the Unix epoch (`i64`), colors are `0xRRGGBB` (`u32`),
//! and text offsets are UTF-16 code units, the unit Flutter counts in.

/// One note in a list.
#[derive(Clone, Debug, PartialEq)]
pub struct NoteRow {
    pub id: String,
    pub rev: String,
    /// A locked note's title shows as any note's does.
    pub title: String,
    /// A few lines of the text. Empty for a locked note, open or not.
    pub snippet: String,
    pub notebook_id: String,
    pub status: String,
    /// Tag IDs.
    pub tags: Vec<String>,
    pub pinned: bool,
    pub created_ms: i64,
    pub updated_ms: i64,
    /// Task-list items done, of how many. Both 0 for a locked note.
    pub tasks_done: u32,
    pub tasks_total: u32,
    /// Sealed under the lock password, or in a locked notebook.
    pub locked: bool,
}

/// What a note's lock allows.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum LockState {
    /// Not locked.
    #[default]
    Plain,
    /// Locked, and open with a key this session holds.
    Open,
    /// Locked and closed. `reason` is a sentence when a password won't help
    /// (the sealed text is damaged, or from a newer Kanso).
    Closed { reason: Option<String> },
}

impl LockState {
    /// Whether the note's text can't be read.
    pub fn is_closed(&self) -> bool {
        matches!(self, LockState::Closed { .. })
    }
}

/// One note, to read or edit.
#[derive(Clone, Debug, PartialEq)]
pub struct NoteView {
    pub id: String,
    pub rev: String,
    pub title: String,
    /// The text; `None` while the note is locked and closed.
    pub body: Option<String>,
    pub lock: LockState,
    pub notebook_id: String,
    pub tags: Vec<String>,
    pub status: String,
    pub pinned: bool,
    pub created_ms: i64,
    pub updated_ms: i64,
}

/// Which notes a list shows. The default is every note outside the trash,
/// newest edit first.
#[derive(Clone, Debug, PartialEq)]
pub struct NoteQuery {
    /// Only this notebook's notes.
    pub notebook_id: Option<String>,
    /// Only notes with this tag (its ID).
    pub tag_id: Option<String>,
    /// Only notes with this status (`active`, `onHold`, ...).
    pub status: Option<String>,
    /// The trash instead.
    pub trash: bool,
    /// Most rows; 0 means all.
    pub limit: u32,
    pub skip: u32,
    pub sort: SortKey,
    pub descending: bool,
}

impl Default for NoteQuery {
    fn default() -> Self {
        NoteQuery {
            notebook_id: None,
            tag_id: None,
            status: None,
            trash: false,
            limit: 0,
            skip: 0,
            sort: SortKey::Updated,
            descending: true,
        }
    }
}

/// What a list is ordered by.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SortKey {
    #[default]
    Updated,
    Created,
    Title,
}

/// A new note.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NewNote {
    /// The notebook it goes in; `None` is the first notebook (made if there
    /// is none). A locked notebook takes it sealed, and needs the lock open.
    pub notebook_id: Option<String>,
    pub title: String,
    pub body: String,
}

/// A write's result: the document and its new revision.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Saved {
    pub id: String,
    pub rev: String,
}

/// A notebook in the sidebar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NotebookRow {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    /// Notes in it.
    pub count: u32,
    /// The notebook has a lock of its own; its notes read as locked.
    pub locked: bool,
    /// The icon's Lucide name and the wash color, as the notebook syncs them.
    pub icon: Option<String>,
    pub color: Option<String>,
}

/// A tag in the sidebar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TagRow {
    pub id: String,
    pub name: String,
    /// `0xRRGGBB`.
    pub color: u32,
    pub count: u32,
}

/// A search result: the row, and where the words matched.
#[derive(Clone, Debug, PartialEq)]
pub struct Hit {
    pub row: NoteRow,
    /// `[start, end)` in UTF-16 units of `row.title`.
    pub title_ranges: Vec<Range16>,
    /// `[start, end)` in UTF-16 units of `row.snippet`.
    pub snippet_ranges: Vec<Range16>,
}

/// A half-open range of UTF-16 code units.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Range16 {
    pub start: u32,
    pub end: u32,
}

/// What a property's value is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PropertyKind {
    Text,
    Number,
    Bool,
    /// `YYYY-MM-DD`.
    Date,
    List,
    /// Kept as written: YAML the properties panel doesn't understand.
    Raw,
}

/// One property from a note's front matter.
#[derive(Clone, Debug, PartialEq)]
pub struct Property {
    pub key: String,
    pub kind: PropertyKind,
    /// The value as a line of text (a list comma-separated).
    pub text: String,
    /// A list's items, as text; empty for any other kind.
    pub items: Vec<String>,
}

/// A query view's table.
#[derive(Clone, Debug, PartialEq)]
pub struct Table {
    /// The column labels; the first is the title.
    pub columns: Vec<String>,
    pub rows: Vec<TableRow>,
}

/// A row of a query view: the note it links to and a cell for each column
/// after the title (`None` where the note has no value).
#[derive(Clone, Debug, PartialEq)]
pub struct TableRow {
    pub id: String,
    pub title: String,
    pub cells: Vec<Option<String>>,
}

/// Why a lock event happened.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LockReason {
    /// The person locked the notes.
    Manual,
    /// The auto-lock time passed.
    Idle,
}

/// The lock session changed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LockEvent {
    /// A lock password opened the locked notes.
    Unlocked,
    /// The locked notes closed.
    Locked { reason: LockReason },
}

/// Where sync stands.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SyncPhase {
    /// No account is signed in.
    #[default]
    SignedOut,
    /// Signed in, with no sync run yet or none since.
    Idle,
    Syncing,
    /// The last sync finished (perhaps with a `problem`).
    Synced,
    /// The last sync failed.
    Failed,
}

/// What went wrong with a sync, in the words of the sync problems table
/// (docs/ACCOUNTS.md › Problems).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyncProblemKind {
    SignedOut,
    AccountDisabled,
    Paused,
    ServerBusy,
    BackOff,
    AccountFull,
    ServerFull,
    SomeItemsSkipped,
    UpdateKanso,
    KeysDontMatch,
    ServerNeedsUpdate,
    Offline,
    Other,
}

/// The sync status the sidebar shows.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SyncStatus {
    pub phase: SyncPhase,
    /// The account's name, when signed in.
    pub username: Option<String>,
    pub server: Option<String>,
    /// When the last sync finished.
    pub last_synced_ms: Option<i64>,
    /// Conflicts in the last sync; the losing versions are in history.
    pub conflicts: u32,
    pub problem: Option<SyncProblemKind>,
}

/// What `sync_now` did.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SyncReportRow {
    /// Remote changes written here.
    pub pulled: u32,
    /// Local changes the server took.
    pub pushed: u32,
    pub conflicts: u32,
    /// Local changes still not on the server.
    pub pending: u32,
    /// A sync was already running, and this one runs after it: nothing else
    /// here is filled in.
    pub queued: bool,
}

/// What a server said about itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServerInfo {
    /// Whether the server takes new accounts.
    pub signups_open: bool,
}

/// A highlighted run of a note's source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpanRow {
    /// `[start, end)` in UTF-16 units of the text.
    pub range: Range16,
    /// The theme `syntax` keys that style it, outermost first.
    pub tokens: Vec<String>,
}

/// The colors a drawing or a preview is made in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PaletteDto {
    pub dark: bool,
    pub background: u32,
    pub text: u32,
    pub muted: u32,
    pub accent: u32,
    pub border: u32,
    pub surface: u32,
    pub heading: Option<u32>,
    pub quote: u32,
    pub task_done: u32,
    pub code_text: Option<u32>,
    pub inline_code_background: u32,
    pub inline_code_text: Option<u32>,
    pub table_header_background: Option<u32>,
}

/// A named color.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ColorRow {
    pub name: String,
    pub rgb: u32,
}

/// How a syntax token is drawn.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyntaxStyleRow {
    /// The token's theme key (`heading`, `keyword`, ...).
    pub token: String,
    pub color: Option<u32>,
    pub italic: Option<bool>,
    /// 100 to 900.
    pub weight: Option<u16>,
}

/// A theme fitted for drawing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ThemeTokens {
    pub name: String,
    pub dark: bool,
    /// The interface colors, by name.
    pub ui: Vec<ColorRow>,
    pub caret: u32,
    /// Selected text: `0xRRGGBB` and its alpha out of 255.
    pub selection: u32,
    pub selection_alpha: u8,
    pub syntax: Vec<SyntaxStyleRow>,
    /// The colors the preview and drawings are made in.
    pub palette: PaletteDto,
}

/// Which diagram language a source is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiagramKind {
    Mermaid,
    Excalidraw,
}

/// A finished drawing.
#[derive(Clone, Debug, PartialEq)]
pub struct SvgRow {
    /// A standalone SVG document, its size set in points.
    pub markup: String,
    pub width: f32,
    pub height: f32,
    /// For inline math: how far below the top the text's baseline falls.
    pub baseline: Option<f32>,
}
