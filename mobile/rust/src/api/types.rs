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
    BoardCard, BoardColumn, BoardKind, BoardLane, BoardView, BuiltinTheme, CalendarDay, CardKey,
    CardMoved, ColorRow, DateField, DayDate, DiagramKind, DueChipRow, DueTone, Hit, KansoError,
    LibraryOverview, ListPlace, ListQuery, ListShape, LocalTime, LockEvent, LockReason, LockState,
    MadeNote, MonthGrid, NewNote, NoteChange, NoteList, NoteRow, NoteView, NotebookRole,
    NotebookRow, PaletteDto, Property, PropertyKind, Range16, ReminderAnswer, ReminderNote,
    ReminderPlan, Saved, SearchScope, ServerInfo, SortKey, SpanRow, StyledRun, SvgRow, SyncPhase,
    SyncProblemKind, SyncReportRow, SyncStatus, SyntaxStyleRow, Table, TableRow, TagRow, TaskPlace,
    TemplateRow, ThemeTokens,
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

#[frb(mirror(ListPlace))]
pub enum _ListPlace {
    All,
    Pinned,
    Recent,
    Notebook { id: String },
    Tag { id: String },
    Trash,
}

#[frb(mirror(ListQuery))]
pub struct _ListQuery {
    pub place: ListPlace,
    pub status_only: Option<String>,
    pub pinned_only: bool,
    pub sort: SortKey,
    pub descending: bool,
}

#[frb(mirror(DateField))]
pub enum _DateField {
    Updated,
    Created,
}

#[frb(mirror(ListShape))]
pub struct _ListShape {
    pub sort: SortKey,
    pub by_date: bool,
    pub date: DateField,
    pub group_pinned: bool,
}

#[frb(mirror(NoteList))]
pub struct _NoteList {
    pub rows: Vec<NoteRow>,
    pub shape: ListShape,
}

#[frb(mirror(LibraryOverview))]
pub struct _LibraryOverview {
    pub all: u32,
    pub pinned: u32,
    pub recent: u32,
    pub trash: u32,
}

#[frb(mirror(NoteChange))]
pub enum _NoteChange {
    TogglePin,
    Move { notebook_id: String },
    Trash,
}

#[frb(mirror(SearchScope))]
pub enum _SearchScope {
    All,
    Titles,
    Tags,
    Notebook { id: String },
}

#[frb(mirror(NotebookRole))]
pub enum _NotebookRole {
    Inbox,
    Journal,
    Templates,
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
    pub role: Option<NotebookRole>,
    pub depth: u32,
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

#[frb(mirror(BuiltinTheme))]
pub struct _BuiltinTheme {
    pub id: String,
    pub name: String,
    pub author: String,
    pub license: String,
    pub source: Option<String>,
    pub dark: bool,
    pub tokens: ThemeTokens,
}

#[frb(mirror(StyledRun))]
pub struct _StyledRun {
    pub range: Range16,
    pub color: Option<u32>,
    pub italic: Option<bool>,
    pub weight: Option<u16>,
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

#[frb(mirror(LocalTime))]
pub struct _LocalTime {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
}

#[frb(mirror(DayDate))]
pub struct _DayDate {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

#[frb(mirror(MadeNote))]
pub struct _MadeNote {
    pub id: String,
    pub cursor16: Option<u32>,
    pub title_first: bool,
}

#[frb(mirror(CalendarDay))]
pub struct _CalendarDay {
    pub date: DayDate,
    pub in_month: bool,
    pub has_note: bool,
    pub is_today: bool,
}

#[frb(mirror(MonthGrid))]
pub struct _MonthGrid {
    pub year: i32,
    pub month: u32,
    pub title: String,
    pub weekday_labels: Vec<String>,
    pub weeks: Vec<Vec<CalendarDay>>,
}

#[frb(mirror(TemplateRow))]
pub struct _TemplateRow {
    pub id: String,
    pub title: String,
}

#[frb(mirror(BoardKind))]
pub enum _BoardKind {
    Notes,
    Tasks,
}

#[frb(mirror(BoardLane))]
pub enum _BoardLane {
    Status { status: String },
    Todo,
    Done,
}

#[frb(mirror(CardKey))]
pub enum _CardKey {
    Note {
        id: String,
    },
    Task {
        note_id: String,
        line: u32,
        text: String,
    },
}

#[frb(mirror(BoardCard))]
pub struct _BoardCard {
    pub key: CardKey,
    pub title: String,
    pub note_title: String,
    pub snippet: String,
    pub due: Option<String>,
    pub checked: bool,
    pub locked: bool,
}

#[frb(mirror(BoardColumn))]
pub struct _BoardColumn {
    pub lane: BoardLane,
    pub label: String,
    pub cards: Vec<BoardCard>,
}

#[frb(mirror(BoardView))]
pub struct _BoardView {
    pub kind: BoardKind,
    pub columns: Vec<BoardColumn>,
}

#[frb(mirror(CardMoved))]
pub enum _CardMoved {
    Changed,
    Unchanged,
    Missing,
}

#[frb(mirror(DueTone))]
pub enum _DueTone {
    Plain,
    Today,
    Overdue,
    Done,
}

#[frb(mirror(DueChipRow))]
pub struct _DueChipRow {
    pub label: String,
    pub tone: DueTone,
    pub tooltip: String,
}

#[frb(mirror(ReminderNote))]
pub struct _ReminderNote {
    pub key: String,
    pub note_id: String,
    pub line: u32,
    pub title: String,
    pub body: String,
    pub fire_at: LocalTime,
}

#[frb(mirror(ReminderPlan))]
pub struct _ReminderPlan {
    pub show_now: Vec<ReminderNote>,
    pub schedule: Vec<ReminderNote>,
}

#[frb(mirror(ReminderAnswer))]
pub enum _ReminderAnswer {
    Open,
    MarkDone,
    Dismiss,
}

#[frb(mirror(TaskPlace))]
pub struct _TaskPlace {
    pub note_id: String,
    pub line: u32,
}
