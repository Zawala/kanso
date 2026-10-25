//! The facade's data types, declared again for the generator.
//!
//! Each `_Name` is the shape of `kanso_mobile::Name`, field by field; the
//! generated code builds and takes apart the real type with these fields, so
//! a change to the facade that these don't follow doesn't compile. Nothing
//! else is in this file: what the fields mean is said where the facade
//! declares them (`crates/kanso-mobile/src/model.rs`, `errors.rs`).

use flutter_rust_bridge::frb;
// Public: the generated code names these types by this module's path.
pub use kanso_mobile::{
    ColorRow, DiagramKind, Hit, KansoError, LockEvent, LockReason, LockState, NewNote, NoteQuery,
    NoteRow, NoteView, NotebookRow, PaletteDto, Property, PropertyKind, Range16, Saved, ServerInfo,
    SortKey, SpanRow, SvgRow, SyncPhase, SyncProblemKind, SyncReportRow, SyncStatus,
    SyntaxStyleRow, Table, TableRow, TagRow, ThemeTokens,
};

#[frb(mirror(KansoError))]
pub enum _KansoError {
    NotFound,
    Locked,
    Conflict { current_rev: String },
    Invalid { message: String },
    Storage { message: String },
    Network { message: String },
    SignedOut,
    Auth { message: String },
    Internal { message: String },
}

#[frb(mirror(NoteRow))]
pub struct _NoteRow {
    pub id: String,
    pub rev: String,
    pub title: String,
    pub snippet: String,
    pub notebook_id: String,
    pub status: String,
    pub tags: Vec<String>,
    pub pinned: bool,
    pub created_ms: i64,
    pub updated_ms: i64,
    pub tasks_done: u32,
    pub tasks_total: u32,
    pub locked: bool,
}

#[frb(mirror(LockState))]
pub enum _LockState {
    Plain,
    Open,
    Closed { reason: Option<String> },
}

#[frb(mirror(NoteView))]
pub struct _NoteView {
    pub id: String,
    pub rev: String,
    pub title: String,
    pub body: Option<String>,
    pub lock: LockState,
    pub notebook_id: String,
    pub tags: Vec<String>,
    pub status: String,
    pub pinned: bool,
    pub created_ms: i64,
    pub updated_ms: i64,
}

#[frb(mirror(NoteQuery))]
pub struct _NoteQuery {
    pub notebook_id: Option<String>,
    pub tag_id: Option<String>,
    pub status: Option<String>,
    pub trash: bool,
    pub limit: u32,
    pub skip: u32,
    pub sort: SortKey,
    pub descending: bool,
}

#[frb(mirror(SortKey))]
pub enum _SortKey {
    Updated,
    Created,
    Title,
}

#[frb(mirror(NewNote))]
pub struct _NewNote {
    pub notebook_id: Option<String>,
    pub title: String,
    pub body: String,
}

#[frb(mirror(Saved))]
pub struct _Saved {
    pub id: String,
    pub rev: String,
}

#[frb(mirror(NotebookRow))]
pub struct _NotebookRow {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub count: u32,
    pub locked: bool,
    pub icon: Option<String>,
    pub color: Option<String>,
}

#[frb(mirror(TagRow))]
pub struct _TagRow {
    pub id: String,
    pub name: String,
    pub color: u32,
    pub count: u32,
}

#[frb(mirror(Hit))]
pub struct _Hit {
    pub row: NoteRow,
    pub title_ranges: Vec<Range16>,
    pub snippet_ranges: Vec<Range16>,
}

#[frb(mirror(Range16))]
pub struct _Range16 {
    pub start: u32,
    pub end: u32,
}

#[frb(mirror(PropertyKind))]
pub enum _PropertyKind {
    Text,
    Number,
    Bool,
    Date,
    List,
    Raw,
}

#[frb(mirror(Property))]
pub struct _Property {
    pub key: String,
    pub kind: PropertyKind,
    pub text: String,
    pub items: Vec<String>,
}

#[frb(mirror(Table))]
pub struct _Table {
    pub columns: Vec<String>,
    pub rows: Vec<TableRow>,
}

#[frb(mirror(TableRow))]
pub struct _TableRow {
    pub id: String,
    pub title: String,
    pub cells: Vec<Option<String>>,
}

#[frb(mirror(LockReason))]
pub enum _LockReason {
    Manual,
    Idle,
}

#[frb(mirror(LockEvent))]
pub enum _LockEvent {
    Unlocked,
    Locked { reason: LockReason },
}

#[frb(mirror(SyncPhase))]
pub enum _SyncPhase {
    SignedOut,
    Idle,
    Syncing,
    Synced,
    Failed,
}

#[frb(mirror(SyncProblemKind))]
pub enum _SyncProblemKind {
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

#[frb(mirror(SyncStatus))]
pub struct _SyncStatus {
    pub phase: SyncPhase,
    pub username: Option<String>,
    pub server: Option<String>,
    pub last_synced_ms: Option<i64>,
    pub conflicts: u32,
    pub problem: Option<SyncProblemKind>,
}

#[frb(mirror(SyncReportRow))]
pub struct _SyncReportRow {
    pub pulled: u32,
    pub pushed: u32,
    pub conflicts: u32,
    pub pending: u32,
    pub queued: bool,
}

#[frb(mirror(ServerInfo))]
pub struct _ServerInfo {
    pub signups_open: bool,
}

#[frb(mirror(SpanRow))]
pub struct _SpanRow {
    pub range: Range16,
    pub tokens: Vec<String>,
}

#[frb(mirror(PaletteDto))]
pub struct _PaletteDto {
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

#[frb(mirror(ColorRow))]
pub struct _ColorRow {
    pub name: String,
    pub rgb: u32,
}

#[frb(mirror(SyntaxStyleRow))]
pub struct _SyntaxStyleRow {
    pub token: String,
    pub color: Option<u32>,
    pub italic: Option<bool>,
    pub weight: Option<u16>,
}

#[frb(mirror(ThemeTokens))]
pub struct _ThemeTokens {
    pub name: String,
    pub dark: bool,
    pub ui: Vec<ColorRow>,
    pub caret: u32,
    pub selection: u32,
    pub selection_alpha: u8,
    pub syntax: Vec<SyntaxStyleRow>,
    pub palette: PaletteDto,
}

#[frb(mirror(DiagramKind))]
pub enum _DiagramKind {
    Mermaid,
    Excalidraw,
}

#[frb(mirror(SvgRow))]
pub struct _SvgRow {
    pub markup: String,
    pub width: f32,
    pub height: f32,
    pub baseline: Option<f32>,
}
