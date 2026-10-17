//! Kanso: a Markdown notes app with an Inkdrop-compatible local API.
//!
//! `kanso` opens the app. Other commands run without a window; see `--help`.

mod config;
mod dialogs;
mod editor;
mod export;
mod history;
mod inkdrop;
mod launch;
mod library;
mod note_list;
mod paths;
mod preview;
mod services;
mod settings;
mod sidebar;
mod workspace;

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context as _, Result, bail};
use gpui_kit::*;
use kanso_store::{BackupPolicy, Store, lock_data_dir};

use crate::config::AppConfig;
use crate::services::Services;
use crate::workspace::{AppState, Quit, Workspace};

const USAGE: &str = "\
Usage: kanso [command] [--data-dir <dir>]

Commands:
  (none)            Open the app
  serve             Run only the local API (Inkdrop-compatible REST and
                    /mcp), without a window
  import-inkdrop    Import notes from Inkdrop
                    [--inkdrop-dir <dir>] [--credentials]
                    --credentials also takes Inkdrop's API port, username
                    and password (macOS asks to allow Keychain access)
  help              Show this message

The data folder defaults to ~/Library/Application Support/Kanso
(or $KANSO_DATA_DIR).";

struct Args {
    command: Option<String>,
    data_dir: PathBuf,
    rest: Vec<String>,
}

fn parse_args() -> Result<Args> {
    let mut command = None;
    let mut data_dir = None;
    let mut rest = Vec::new();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--data-dir" => {
                data_dir = Some(PathBuf::from(
                    args.next().context("--data-dir needs a folder")?,
                ))
            }
            "-h" | "--help" => command = Some("help".to_owned()),
            // macOS adds this when launching from Finder.
            arg if arg.starts_with("-psn_") => {}
            _ if command.is_none() && !arg.starts_with('-') => command = Some(arg),
            _ => rest.push(arg),
        }
    }
    Ok(Args {
        command,
        data_dir: data_dir.unwrap_or_else(paths::default_data_dir),
        rest,
    })
}

fn main() -> Result<()> {
    let args = parse_args()?;
    match args.command.as_deref() {
        None => run_app(args.data_dir),
        Some("serve") => serve(args.data_dir),
        Some("import-inkdrop") => inkdrop::run_cli(&args.data_dir, &args.rest),
        Some("help") => {
            println!("{USAGE}");
            Ok(())
        }
        Some(other) => bail!("unknown command `{other}`\n\n{USAGE}"),
    }
}

/// Runs the local API until the process is stopped.
fn serve(data_dir: PathBuf) -> Result<()> {
    std::fs::create_dir_all(&data_dir)?;
    let Some(_lock) = lock_data_dir(&data_dir)? else {
        bail!(
            "Kanso is already running with {}; its API is already on",
            data_dir.display()
        );
    };
    let config = load_config(&data_dir)?;
    let server = services::server_config(&config.server).map_err(anyhow::Error::msg)?;
    let store = Arc::new(Store::open(paths::db_path(&data_dir))?);
    let handle = kanso_server::spawn(store, server);
    match handle.bind_result() {
        Ok(addr) => eprintln!("Kanso API on http://{addr} (MCP at /mcp). Ctrl-C stops it."),
        Err(err) => bail!(
            "couldn't listen on {}:{}: {err}",
            config.server.bind_address,
            config.server.port
        ),
    }
    loop {
        std::thread::park();
    }
}

/// Loads the settings, filling in API credentials on first run.
fn load_config(data_dir: &std::path::Path) -> Result<AppConfig> {
    let path = paths::config_path(data_dir);
    let mut config =
        AppConfig::load(&path).with_context(|| format!("reading {}", path.display()))?;
    if config.ensure_credentials() {
        config.save(&path)?;
    }
    Ok(config)
}

fn run_app(data_dir: PathBuf) -> Result<()> {
    std::fs::create_dir_all(&data_dir)
        .with_context(|| format!("creating {}", data_dir.display()))?;
    let Some(lock) = lock_data_dir(&data_dir)? else {
        bail!(
            "Kanso is already running with the data folder {}",
            data_dir.display()
        );
    };
    let config = load_config(&data_dir)?;
    let store = Arc::new(Store::open(paths::db_path(&data_dir))?);

    // A daily snapshot of the database, off the main thread.
    let backup_store = store.clone();
    let policy = BackupPolicy::daily(paths::backups_dir(&data_dir));
    std::thread::spawn(move || {
        if let Err(err) = backup_store.backup_if_due(&policy, kanso_core::now_ms()) {
            eprintln!("kanso: backup failed: {err}");
        }
    });

    let app = application();
    app.on_reopen(|cx| {
        if cx.windows().is_empty() {
            open_main_window(cx);
        }
    });
    app.run(move |cx| {
        gpui_kit::init(cx);
        cx.set_global(AppState {
            store,
            data_dir,
            config,
        });
        cx.set_global(Services::default());
        workspace::bind_keys(cx);
        note_list::bind_keys(cx);
        workspace::set_menus(cx);
        cx.on_action(|_: &Quit, cx| quit(cx));
        cx.on_action(|_: &workspace::OpenSettings, cx| settings::SettingsWindow::open(cx));
        services::apply_server_settings(cx);
        open_main_window(cx);
        cx.activate(true);
        // Held until the process exits so no second Kanso opens this folder.
        let _ = &lock;
        std::mem::forget(lock);
    });
    Ok(())
}

fn open_main_window(cx: &mut App) {
    let bounds = Bounds::centered(None, size(px(1360.), px(860.)), cx);
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: Some(TitlebarOptions {
            title: Some("Kanso".into()),
            ..Default::default()
        }),
        window_min_size: Some(size(px(720.), px(420.))),
        ..Default::default()
    };
    let theme = cx.global::<AppState>().config.appearance.theme;
    let opened = gpui_kit::open_window(options, cx, |window, cx| {
        workspace::apply_theme(theme, Some(window), cx);
        cx.new(|cx| Workspace::new(window, cx))
    });
    match opened {
        Ok((window, view)) => {
            window
                .update(cx, |_, window, cx| {
                    let editor = view.read(cx).editor.clone();
                    window.on_window_should_close(cx, move |_, cx| {
                        editor.update(cx, |editor, cx| editor.flush(cx));
                        true
                    });
                    offer_first_import(window, cx);
                })
                .ok();
        }
        Err(err) => eprintln!("kanso: couldn't open a window: {err:#}"),
    }
}

/// On a first run with an empty library, offers to bring notes over from
/// Inkdrop.
fn offer_first_import(window: &mut Window, cx: &mut App) {
    let state = cx.global::<AppState>();
    let empty = state
        .store
        .counts()
        .is_ok_and(|counts| counts.get(&kanso_core::DocType::Note).copied().unwrap_or(0) == 0);
    if !empty
        || state.store.meta("import_offered").ok().flatten().is_some()
        || inkdrop::find_inkdrop().is_none()
    {
        return;
    }
    state.store.set_meta("import_offered", "1").ok();
    dialogs::confirm(
        "Import your Inkdrop notes?",
        "Kanso found an Inkdrop library. Its notes, notebooks, tags and attachments can be copied in now; Inkdrop's files are only read.",
        "Import",
        window,
        cx,
        inkdrop::start_import,
    );
}

/// Saves open notes, then quits.
fn quit(cx: &mut App) {
    workspace::for_each_workspace(cx, |workspace, cx| {
        workspace.editor.update(cx, |editor, cx| editor.flush(cx));
    });
    services::stop_server(cx);
    cx.quit();
}
