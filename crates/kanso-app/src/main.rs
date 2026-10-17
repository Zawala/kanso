//! Kanso: a Markdown notes app with an Inkdrop-compatible local API.
//!
//! `kanso` opens the app. Other commands run without a window; see `--help`.

mod config;
mod dialogs;
mod editor;
mod library;
mod note_list;
mod paths;
mod preview;
mod services;
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
  (none)    Open the app
  help      Show this message

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
        Some("help") => {
            println!("{USAGE}");
            Ok(())
        }
        Some(other) => bail!("unknown command `{other}`\n\n{USAGE}"),
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
                })
                .ok();
        }
        Err(err) => eprintln!("kanso: couldn't open a window: {err:#}"),
    }
}

/// Saves open notes, then quits.
fn quit(cx: &mut App) {
    for window in cx.windows() {
        window
            .update(cx, |root, _, cx| {
                if let Ok(root) = root.downcast::<gpui_kit::base::Root>() {
                    let view = root.read(cx).view().clone();
                    if let Ok(workspace) = view.downcast::<Workspace>() {
                        let editor = workspace.read(cx).editor.clone();
                        editor.update(cx, |editor, cx| editor.flush(cx));
                    }
                }
            })
            .ok();
    }
    cx.quit();
}
