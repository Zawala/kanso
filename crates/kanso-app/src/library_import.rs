//! Import…: notes from a folder of Markdown or another app's export, with
//! a summary to look over before anything is written, progress while it
//! runs and a report after. File › Open… brings in one Markdown file.
//! `kanso import <kind> <path>` does the same from a terminal.
//!
//! The source is only ever read. Importing again updates what the last
//! import made and never undoes changes made in Kanso
//! ([`kanso_import::library`]).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context as _, bail};
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::notification::Notification;
use gpui_kit::component::{Disableable as _, WindowExt as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_import::library::{
    LibraryPhase, LibraryProgress, LibraryReport, SourceKind, import_markdown_file, import_source,
    preview,
};
use kanso_store::Store;

use crate::paths;
use crate::theme::{Colors, Rgb};
use crate::workspace::AppState;
use crate::zoom::px;

/// The width of the Import window.
pub(crate) const WIDTH: f32 = 500.;

/// Where notes can come from: the library kinds, then the PouchDB library
/// found on this computer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Choice {
    Library(SourceKind),
    PouchLibrary,
}

impl Choice {
    pub fn all() -> Vec<Choice> {
        let mut all: Vec<Choice> = SourceKind::ALL
            .into_iter()
            // Apple Notes can only be read from its macOS database. The CLI
            // still lists it everywhere and returns a precise platform error.
            .filter(|kind| cfg!(target_os = "macos") || *kind != SourceKind::AppleNotes)
            .map(Choice::Library)
            .collect();
        all.push(Choice::PouchLibrary);
        all
    }

    fn label(self) -> &'static str {
        match self {
            Choice::Library(kind) => kind.label(),
            Choice::PouchLibrary => "Inkdrop library",
        }
    }

    fn hint(self) -> &'static str {
        match self {
            Choice::Library(kind) => kind.hint(),
            Choice::PouchLibrary => {
                "Found on this computer and read in place; its app can keep running."
            }
        }
    }
}

/// What the picker should offer for `kind`: `(files, folders)`. Pure.
pub fn picks(kind: SourceKind) -> (bool, bool) {
    match kind {
        SourceKind::Markdown | SourceKind::Obsidian | SourceKind::Logseq | SourceKind::UpNote => {
            (false, true)
        }
        SourceKind::StandardNotes | SourceKind::Simplenote | SourceKind::Roam => (true, false),
        SourceKind::Notion
        | SourceKind::Bear
        | SourceKind::Evernote
        | SourceKind::Joplin
        | SourceKind::AppleNotes
        | SourceKind::AppleJournal
        | SourceKind::GoogleKeep
        | SourceKind::Html
        | SourceKind::TextBundle
        | SourceKind::Tomboy => (true, true),
    }
}

fn phase_label(phase: LibraryPhase) -> &'static str {
    match phase {
        LibraryPhase::Reading => "Reading notes",
        LibraryPhase::Notebooks => "Notebooks",
        LibraryPhase::Notes => "Notes and attachments",
    }
}

/// `n thing`/`n things`. Pure.
fn count(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// The summary lines of a report: what the source holds, then what the
/// import did (or would do), then anything left alone. Pure.
pub fn summary(report: &LibraryReport) -> Vec<String> {
    let found = &report.found;
    let mut lines = vec![format!(
        "{}, {}, {}, {}.",
        count(found.notes, "note", "notes"),
        count(found.notebooks, "notebook", "notebooks"),
        count(found.tags, "tag", "tags"),
        count(found.attachments, "attachment", "attachments"),
    )];
    let notes = &report.notes;
    let again = notes.updated + notes.unchanged + notes.kept() > 0;
    let verb = if report.dry_run { "will be" } else { "were" };
    if again {
        lines.push(format!(
            "{} new, {} {verb} updated, {} unchanged since the last import.",
            notes.created, notes.updated, notes.unchanged
        ));
    } else if notes.created > 0 {
        lines.push(format!(
            "{} {verb} added.",
            count(notes.created, "note", "notes")
        ));
    }
    if notes.kept_edited > 0 {
        lines.push(format!(
            "{} changed in Kanso since {} kept as they are.",
            count(notes.kept_edited, "note", "notes"),
            if report.dry_run { "will be" } else { "were" }
        ));
    }
    if notes.kept_deleted > 0 {
        lines.push(format!(
            "{} deleted in Kanso {} stay deleted.",
            count(notes.kept_deleted, "note", "notes"),
            if report.dry_run { "will" } else { "did" }
        ));
    }
    if report.tags_created > 0 {
        lines.push(format!(
            "{} {}.",
            count(report.tags_created, "new tag", "new tags"),
            if report.dry_run {
                "will be made"
            } else {
                "made"
            }
        ));
    }
    if report.unresolved_links > 0 {
        lines.push(format!(
            "{} to notes or files that aren't in it.",
            count(report.unresolved_links, "link points", "links point")
        ));
    }
    lines
}

/// The skipped files, a line per reason with a few examples. Pure.
pub fn skipped_lines(report: &LibraryReport) -> Vec<String> {
    report
        .skipped_by_reason()
        .into_iter()
        .map(|(reason, n, paths)| {
            let more = if n > paths.len() { ", …" } else { "" };
            format!("{n} skipped — {reason}: {}{more}", paths.join(", "))
        })
        .collect()
}

enum Stage {
    Choose,
    Reading {
        kind: SourceKind,
        path: PathBuf,
        progress: Option<LibraryProgress>,
    },
    Preview {
        kind: SourceKind,
        path: PathBuf,
        report: Box<LibraryReport>,
    },
    Importing {
        kind: SourceKind,
        progress: Option<LibraryProgress>,
    },
    Done(Box<LibraryReport>),
    Failed(String),
}

enum Message {
    Progress(LibraryProgress),
    Done(Result<Box<LibraryReport>, String>),
}

/// The Import window's body.
pub struct ImportView {
    stage: Stage,
    store: Arc<Store>,
    _task: Option<Task<()>>,
}

impl ImportView {
    fn new(store: Arc<Store>) -> Self {
        ImportView {
            stage: Stage::Choose,
            store,
            _task: None,
        }
    }

    fn busy(&self) -> bool {
        matches!(self.stage, Stage::Importing { .. })
    }

    fn title(&self) -> SharedString {
        match &self.stage {
            Stage::Choose => "Import".into(),
            Stage::Reading { kind, .. }
            | Stage::Preview { kind, .. }
            | Stage::Importing { kind, .. } => format!("Import {}", kind.label()).into(),
            Stage::Done(_) => "Imported".into(),
            Stage::Failed(_) => "Import".into(),
        }
    }

    fn choose(&mut self, choice: Choice, window: &mut Window, cx: &mut Context<Self>) {
        let kind = match choice {
            Choice::PouchLibrary => {
                window.close_dialog(cx);
                crate::pouch_import::start_import(window, cx);
                return;
            }
            Choice::Library(kind) => kind,
        };
        let (files, directories) = picks(kind);
        let picked = cx.prompt_for_paths(PathPromptOptions {
            files,
            directories,
            multiple: false,
            prompt: Some("Import".into()),
        });
        self._task = Some(cx.spawn(async move |this, cx| {
            let Ok(Ok(Some(paths))) = picked.await else {
                return;
            };
            let Some(path) = paths.into_iter().next() else {
                return;
            };
            this.update(cx, |this, cx| this.start_preview(kind, path, cx))
                .ok();
        }));
    }

    /// Runs `job` on a thread, showing its progress, then `done`.
    fn run(
        &mut self,
        job: impl FnOnce(&mut dyn FnMut(LibraryProgress)) -> anyhow::Result<LibraryReport>
        + Send
        + 'static,
        done: impl Fn(&mut Self, Box<LibraryReport>) + 'static,
        cx: &mut Context<Self>,
    ) {
        let (tx, rx) = async_channel::unbounded();
        std::thread::spawn(move || {
            let progress_tx = tx.clone();
            let result = job(&mut |progress| {
                progress_tx.try_send(Message::Progress(progress)).ok();
            });
            tx.try_send(Message::Done(
                result.map(Box::new).map_err(|err| format!("{err:#}")),
            ))
            .ok();
        });
        self._task = Some(cx.spawn(async move |this, cx| {
            while let Ok(message) = rx.recv().await {
                let alive = this.update(cx, |this, cx| {
                    match message {
                        Message::Progress(update) => match &mut this.stage {
                            Stage::Reading { progress, .. } | Stage::Importing { progress, .. } => {
                                *progress = Some(update)
                            }
                            _ => {}
                        },
                        Message::Done(Ok(report)) => done(this, report),
                        Message::Done(Err(err)) => this.stage = Stage::Failed(err),
                    }
                    cx.notify();
                });
                if alive.is_err() {
                    break;
                }
            }
        }));
    }

    fn start_preview(&mut self, kind: SourceKind, path: PathBuf, cx: &mut Context<Self>) {
        self.stage = Stage::Reading {
            kind,
            path: path.clone(),
            progress: None,
        };
        let store = self.store.clone();
        let source = path.clone();
        self.run(
            move |progress| preview(kind, &source, &store, progress),
            move |this, report| {
                this.stage = Stage::Preview {
                    kind,
                    path: path.clone(),
                    report,
                }
            },
            cx,
        );
        cx.notify();
    }

    fn start_import(&mut self, cx: &mut Context<Self>) {
        let Stage::Preview { kind, path, .. } = &self.stage else {
            return;
        };
        let (kind, path) = (*kind, path.clone());
        self.stage = Stage::Importing {
            kind,
            progress: None,
        };
        let store = self.store.clone();
        self.run(
            move |progress| import_source(kind, &path, &store, progress),
            |this, report| this.stage = Stage::Done(report),
            cx,
        );
        cx.notify();
    }

    fn back(&mut self, cx: &mut Context<Self>) {
        self.stage = Stage::Choose;
        self._task = None;
        cx.notify();
    }
}

fn progress_bar(progress: Option<LibraryProgress>, cx: &App) -> Div {
    let colors = Colors::get(cx);
    let (label, fraction) = match progress {
        None => ("Starting…".to_owned(), 0.0),
        Some(p) if p.total == 0 => (format!("{}… {}", phase_label(p.phase), p.done), 0.0),
        Some(p) => (
            format!("{}… {} of {}", phase_label(p.phase), p.done, p.total),
            p.done as f32 / p.total as f32,
        ),
    };
    v_flex()
        .gap_2()
        .child(
            div()
                .text_size(px(13.))
                .text_color(colors.text_secondary)
                .child(label),
        )
        .child(
            div()
                .h(px(6.))
                .w_full()
                .rounded_full()
                .bg(colors.fill)
                .child(
                    div()
                        .h_full()
                        .rounded_full()
                        .bg(colors.accent)
                        .w(relative(fraction.clamp(0.0, 1.0))),
                ),
        )
}

fn lines(lines: Vec<String>, color: Rgb, size: f32) -> Div {
    v_flex()
        .gap(px(4.))
        .children(lines.into_iter().map(move |line| {
            div()
                .text_size(px(size))
                .line_height(px(size + 5.))
                .text_color(color)
                .child(line)
        }))
}

fn path_line(path: &Path, cx: &App) -> Div {
    div()
        .text_size(px(12.))
        .text_color(Colors::get(cx).text_muted)
        .child(path.display().to_string())
}

// no-prop-test: drawing; `summary` and `skipped_lines` carry the logic.
impl Render for ImportView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        match &self.stage {
            Stage::Choose => {
                let cards = Choice::all().into_iter().enumerate().map(|(ix, choice)| {
                    v_flex()
                        .id(("import-choice", ix))
                        .debug_selector(move || format!("import-choice-{ix}"))
                        .w(px((WIDTH - 58.) / 2.))
                        .px(px(12.))
                        .py(px(9.))
                        .gap(px(3.))
                        .rounded(px(8.))
                        .border_1()
                        .border_color(colors.divider)
                        .cursor_pointer()
                        .hover(|el| el.bg(colors.row_hover))
                        .child(
                            div()
                                .text_size(px(13.5))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(colors.text)
                                .child(choice.label()),
                        )
                        .child(
                            div()
                                .text_size(px(11.5))
                                .line_height(px(15.))
                                .text_color(colors.text_secondary)
                                .child(choice.hint()),
                        )
                        .on_click(
                            cx.listener(move |this, _, window, cx| this.choose(choice, window, cx)),
                        )
                });
                v_flex()
                    .gap(px(12.))
                    .child(
                        div()
                            .text_size(px(13.))
                            .line_height(px(18.))
                            .text_color(colors.text_secondary)
                            .child("Choose where your notes are. Nothing is written until you've seen what will come in, and the source is only read."),
                    )
                    .child(
                        h_flex()
                            .id("import-sources")
                            .flex_wrap()
                            .items_stretch()
                            .gap(px(8.))
                            .max_h(px(440.))
                            .overflow_y_scroll()
                            .pr(px(4.))
                            .children(cards),
                    )
            }
            Stage::Reading { path, progress, .. } => v_flex()
                .gap(px(10.))
                .child(path_line(path, cx))
                .child(progress_bar(*progress, cx)),
            Stage::Preview { path, report, .. } => {
                let empty = report.found.notes == 0;
                v_flex()
                    .gap(px(10.))
                    .child(path_line(path, cx))
                    .when(empty, |el| {
                        el.child(
                            div()
                                .text_size(px(13.5))
                                .text_color(colors.danger)
                                .child("There are no notes here. Was it the right folder or file?"),
                        )
                    })
                    .child(lines(summary(report), colors.text, 13.5))
                    .child(lines(skipped_lines(report), colors.text_muted, 12.))
            }
            Stage::Importing { progress, .. } => v_flex()
                .gap(px(10.))
                .child(progress_bar(*progress, cx))
                .child(
                    div()
                        .text_size(px(12.))
                        .text_color(colors.text_muted)
                        .child("The source is only read."),
                ),
            Stage::Done(report) => v_flex()
                .gap(px(10.))
                .child(lines(summary(report), colors.text, 13.5))
                .child(lines(skipped_lines(report), colors.text_muted, 12.))
                .when(!report.rejected.is_empty(), |el| {
                    el.child(lines(
                        std::iter::once(format!(
                            "{} couldn't be imported:",
                            count(report.rejected.len(), "item", "items")
                        ))
                        .chain(report.rejected.iter().take(5).cloned())
                        .collect(),
                        colors.danger,
                        12.,
                    ))
                }),
            Stage::Failed(err) => v_flex().gap(px(6.)).child("The import didn't run.").child(
                div()
                    .text_size(px(13.))
                    .text_color(colors.danger)
                    .child(err.clone()),
            ),
        }
    }
}

/// The Import window's buttons, redrawn with its stage.
struct ImportFooter {
    view: Entity<ImportView>,
    _observing: Subscription,
}

// no-prop-test: drawing.
impl Render for ImportFooter {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = self.view.clone();
        let close = |label: &'static str| {
            Button::new("import-close")
                .label(label)
                .debug_selector(|| "import-close".into())
                .on_click(|_, window, cx| window.close_dialog(cx))
        };
        let row = h_flex().w_full().gap_2().justify_end();
        match &self.view.read(cx).stage {
            Stage::Choose | Stage::Reading { .. } => row.child(close("Cancel")),
            Stage::Preview { report, .. } => {
                let empty = report.found.notes == 0;
                let back = view.clone();
                row.child(
                    Button::new("import-back")
                        .label("Back")
                        .on_click(move |_, _, cx| back.update(cx, |v, cx| v.back(cx))),
                )
                .child(close("Cancel"))
                .child(
                    Button::new("import-run")
                        .primary()
                        .label("Import")
                        .disabled(empty)
                        .debug_selector(|| "import-run".into())
                        .on_click(move |_, _, cx| view.update(cx, |v, cx| v.start_import(cx))),
                )
            }
            Stage::Importing { .. } => row.child(
                Button::new("import-running")
                    .primary()
                    .label("Importing…")
                    .loading(true),
            ),
            Stage::Done(_) => row.child(close("Done")),
            Stage::Failed(_) => {
                let back = view.clone();
                row.child(
                    Button::new("import-back")
                        .label("Back")
                        .on_click(move |_, _, cx| back.update(cx, |v, cx| v.back(cx))),
                )
                .child(close("Close"))
            }
        }
    }
}

/// Opens the Import window.
pub fn start(window: &mut Window, cx: &mut App) {
    open(Stage::Choose, window, cx);
}

/// The Import window showing a preview of `report`, for snapshots.
#[cfg(test)]
pub(crate) fn open_preview(
    kind: SourceKind,
    path: PathBuf,
    report: LibraryReport,
    window: &mut Window,
    cx: &mut App,
) {
    let report = Box::new(report);
    open(Stage::Preview { kind, path, report }, window, cx);
}

/// The Import window after an import, for snapshots.
#[cfg(test)]
pub(crate) fn open_done(report: LibraryReport, window: &mut Window, cx: &mut App) {
    open(Stage::Done(Box::new(report)), window, cx);
}

fn open(stage: Stage, window: &mut Window, cx: &mut App) {
    let store = cx.global::<AppState>().store.clone();
    let view = cx.new(|_| ImportView {
        stage,
        ..ImportView::new(store)
    });
    let footer = cx.new(|cx| ImportFooter {
        view: view.clone(),
        _observing: cx.observe(&view, |_, _, cx| cx.notify()),
    });
    window.open_alert_dialog(cx, move |alert, _, cx| {
        let busy = view.clone();
        alert
            .title(view.read(cx).title())
            .width(px(WIDTH))
            .child(view.clone())
            .footer(footer.clone())
            // Escape closes, but not while notes are being written.
            .on_cancel(move |_, _, cx| !busy.read(cx).busy())
    });
}

/// File › Open…: asks for Markdown files and imports each into the Inbox,
/// then opens the last. Opening a file again updates its note.
pub fn open_files(
    window: &mut Window,
    cx: &mut App,
    on_opened: impl Fn(String, &mut Window, &mut App) + 'static,
) {
    let store = cx.global::<AppState>().store.clone();
    let picked = cx.prompt_for_paths(PathPromptOptions {
        files: true,
        directories: false,
        multiple: true,
        prompt: Some("Open".into()),
    });
    window
        .spawn(cx, async move |cx| {
            let Ok(Ok(Some(paths))) = picked.await else {
                return;
            };
            let result = cx
                .background_executor()
                .spawn(async move { open_paths(&paths, &store) })
                .await;
            cx.update(|window, cx| match result {
                Ok((Some(id), problems)) => {
                    on_opened(id, window, cx);
                    for problem in problems {
                        window.push_notification(Notification::warning(problem), cx);
                    }
                }
                Ok((None, problems)) => {
                    for problem in problems {
                        window.push_notification(Notification::error(problem), cx);
                    }
                }
                Err(err) => window.push_notification(
                    Notification::error(format!("Couldn't open it: {err:#}")),
                    cx,
                ),
            })
            .ok();
        })
        .detach();
}

/// Imports each Markdown file; returns the last note and what went wrong
/// with the others.
fn open_paths(paths: &[PathBuf], store: &Store) -> anyhow::Result<(Option<String>, Vec<String>)> {
    let mut last = None;
    let mut problems = Vec::new();
    for path in paths {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if !kanso_import::library::resolve::is_note_path(&name) {
            problems.push(format!(
                "{name} isn't a Markdown or text file. To bring in another app's export, use Import…"
            ));
            continue;
        }
        match import_markdown_file(path, store) {
            Ok(report) if report.is_clean() => last = report.first_note.or(last),
            Ok(report) => problems.push(format!("{name}: {}", report.rejected.join("; "))),
            Err(err) => problems.push(format!("{name}: {err:#}")),
        }
    }
    Ok((last, problems))
}

// MARK: Command line

pub const USAGE: &str = "\
Usage: kanso import <kind> <path> [--dry-run]

Imports notes from a folder or an export file. The source is only read;
importing it again updates what the last import made, and never undoes
changes made in Kanso. --dry-run shows what would happen and writes
nothing.

Kinds:
  markdown        A folder of .md, .markdown or .txt files, or one file
  obsidian        An Obsidian vault's folder
  logseq          A Logseq graph's folder
  notion          Notion's Markdown & CSV export (.zip or its folder)
  evernote        An .enex file, or a folder of them
  bear            A .bear2bk backup, or a TextBundle/Markdown export folder
  joplin          A .jex file, or a RAW export folder
  standard-notes  A decrypted Standard Notes backup (.txt, .json or .zip)
  simplenote      Simplenote's export (.zip or notes.json)
  upnote          An UpNote Markdown export folder
  apple-notes     Apple Notes' data folder or NoteStore.sqlite (macOS)
  apple-journal   Apple Journal entry HTML files, folder or .zip
  google-keep     Google Keep JSON, Takeout .zip or folder
  html            HTML file, folder or .zip
  roam            A Roam JSON graph export
  textbundle      A TextBundle folder, TextPack, or a .zip of bundles
  tomboy          Tomboy/Gnote .note files or folder
  pouchdb         The PouchDB notes library found on this computer, read in
                  place [--library-dir DIR] [--credentials]; --credentials
                  also copies its local API port, username and password";

/// The `kanso import` kind for the PouchDB notes library found on this
/// computer.
const POUCH_KIND: &str = "pouchdb";

/// `kanso import` with its arguments read.
#[derive(Debug, PartialEq, Eq)]
pub enum Cli {
    Help,
    PouchLibrary(Vec<String>),
    Library {
        kind: SourceKind,
        path: PathBuf,
        dry_run: bool,
    },
}

/// Reads `kanso import`'s arguments. Pure.
pub fn parse_cli(args: &[String]) -> anyhow::Result<Cli> {
    let mut dry_run = false;
    let mut positional = Vec::new();
    for (ix, arg) in args.iter().enumerate() {
        match arg.as_str() {
            "-h" | "--help" | "help" if positional.is_empty() => return Ok(Cli::Help),
            "--dry-run" | "-n" => dry_run = true,
            _ if positional.first().map(String::as_str) == Some(POUCH_KIND) => {
                return Ok(Cli::PouchLibrary(args[ix..].to_vec()));
            }
            other if other.starts_with("--") => bail!("unknown option `{other}`\n\n{USAGE}"),
            other => positional.push(other.to_owned()),
        }
    }
    match positional.as_slice() {
        [] => Ok(Cli::Help),
        [kind] if kind == POUCH_KIND => Ok(Cli::PouchLibrary(Vec::new())),
        [kind, path] => {
            let kind = SourceKind::parse(kind)
                .with_context(|| format!("unknown kind `{kind}`\n\n{USAGE}"))?;
            Ok(Cli::Library {
                kind,
                path: PathBuf::from(path),
                dry_run,
            })
        }
        [kind] => bail!("`kanso import {kind}` needs a folder or file to import\n\n{USAGE}"),
        _ => bail!("too many arguments\n\n{USAGE}"),
    }
}

/// `kanso import <kind> <path> [--dry-run]`
pub fn run_cli(data_dir: &Path, args: &[String]) -> anyhow::Result<()> {
    let (kind, path, dry_run) = match parse_cli(args)? {
        Cli::Help => {
            println!("{USAGE}");
            return Ok(());
        }
        Cli::PouchLibrary(rest) => return crate::pouch_import::run_cli(data_dir, &rest),
        Cli::Library {
            kind,
            path,
            dry_run,
        } => (kind, path, dry_run),
    };
    std::fs::create_dir_all(data_dir)?;
    let Some(_lock) = kanso_store::lock_data_dir(data_dir)? else {
        bail!(
            "Kanso is running with {}; quit it first, or use File › Import… in the app",
            data_dir.display()
        );
    };
    let store = Store::open(paths::db_path(data_dir))?;
    eprintln!("Reading {}", path.display());
    let report = if dry_run {
        preview(kind, &path, &store, |_| {})?
    } else {
        import_source(kind, &path, &store, |_| {})?
    };
    println!("{report}");
    if !report.is_clean() {
        bail!("some notes couldn't be imported; see above");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{Cli, LibraryReport, SourceKind, parse_cli, summary};
    use proptest::prelude::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn cli_examples() {
        assert_eq!(parse_cli(&args(&[])).unwrap(), Cli::Help);
        assert_eq!(
            parse_cli(&args(&["obsidian", "/v", "--dry-run"])).unwrap(),
            Cli::Library {
                kind: SourceKind::Obsidian,
                path: "/v".into(),
                dry_run: true
            }
        );
        assert_eq!(
            parse_cli(&args(&["pouchdb", "--credentials"])).unwrap(),
            Cli::PouchLibrary(args(&["--credentials"]))
        );
        assert!(parse_cli(&args(&["word", "/x"])).is_err());
        assert!(parse_cli(&args(&["notion"])).is_err());
        assert!(parse_cli(&args(&["notion", "/x", "--force"])).is_err());
    }

    #[test]
    fn a_fresh_import_says_how_many_are_added() {
        let mut report = LibraryReport {
            dry_run: true,
            ..LibraryReport::default()
        };
        report.found.notes = 2;
        report.notes.created = 2;
        assert_eq!(
            summary(&report),
            [
                "2 notes, 0 notebooks, 0 tags, 0 attachments.",
                "2 notes will be added."
            ]
        );
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(128))]

        /// Every kind's name reads back as that kind, wherever --dry-run is.
        #[test]
        fn prop_cli_reads_every_kind(ix in 0usize..SourceKind::ALL.len(), path in "[a-z/]{1,12}", at in 0usize..3) {
            let kind = SourceKind::ALL[ix];
            let mut list = vec![kind.as_str().to_owned(), path.clone()];
            list.insert(at.min(2), "--dry-run".to_owned());
            prop_assert_eq!(
                parse_cli(&list).unwrap(),
                Cli::Library { kind, path: PathBuf::from(path), dry_run: true }
            );
        }

        /// The summary's first line always counts what was found, and a
        /// preview never says anything was done.
        #[test]
        fn prop_summary_tense(created in 0usize..5, updated in 0usize..5, unchanged in 0usize..5, dry in any::<bool>()) {
            let mut report = LibraryReport { dry_run: dry, ..LibraryReport::default() };
            report.found.notes = created + updated + unchanged;
            report.notes.created = created;
            report.notes.updated = updated;
            report.notes.unchanged = unchanged;
            let lines = summary(&report);
            let notes_phrase = if report.found.notes == 1 { "1 note," } else { "notes," };
            prop_assert!(lines[0].contains(notes_phrase));
            let past = lines.iter().any(|l| l.contains(" were "));
            prop_assert_eq!(past, !dry && report.found.notes > 0);
        }
    }
}
