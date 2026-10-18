//! Kanso: a Markdown notes app with an Inkdrop-compatible local API.
//!
//! `kanso` opens the app. Other commands run without a window; see `--help`.

mod account_flow;
mod account_page;
mod account_words;
mod assets;
mod clipboard;
mod clock;
mod commands;
mod config;
mod dialogs;
mod editor;
mod editor_footer;
mod empty_state;
mod export;
mod fuzzy;
mod grouping;
mod history;
mod inkdrop;
mod keys;
mod launch;
mod layout;
mod library;
mod markdown_edit;
mod markdown_source;
mod note_list;
mod palette;
mod paths;
mod preview;
mod services;
mod settings;
#[cfg(test)]
mod settings_window_tests;
mod sidebar;
#[cfg(test)]
mod snapshots;
mod split;
mod text_match;
mod text_stats;
mod theme;
#[cfg(test)]
mod ui_tests;
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
    parse_args_from(std::env::args().skip(1))
}

/// Reads the command line after the program name.
fn parse_args_from(args: impl IntoIterator<Item = String>) -> Result<Args> {
    let mut command = None;
    let mut data_dir = None;
    let mut rest = Vec::new();
    let mut args = args.into_iter();
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

/// Loads the settings, bringing an older file up to date and filling in API
/// credentials on first run.
fn load_config(data_dir: &std::path::Path) -> Result<AppConfig> {
    let path = paths::config_path(data_dir);
    let mut config =
        AppConfig::load(&path).with_context(|| format!("reading {}", path.display()))?;
    let migrated = config.migrate();
    if config.ensure_credentials() || migrated {
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

    let app = application().with_assets(assets::KansoAssets);
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
        // The real servers and the Keychain; tests and snapshots leave both
        // out.
        let mut services = Services::default();
        services.backend = Some(Arc::new(kanso_sync::HttpBackend::new()));
        services.secrets = Some(Arc::new(kanso_sync::Keychain));
        cx.set_global(services);
        workspace::bind_keys(cx);
        note_list::bind_keys(cx);
        workspace::set_menus(cx);
        cx.on_action(|_: &Quit, cx| quit(cx));
        watch_for_quit(cx);
        settings::register_app_actions(cx, open_main_window);
        services::apply_server_settings(cx);
        // Signed in again once the config, the binding and the Keychain
        // agree; sync starts then.
        account_flow::restore(cx);
        open_main_window(cx);
        cx.activate(true);
        // Held until the process exits so no second Kanso opens this folder.
        let _ = &lock;
        std::mem::forget(lock);
    });
    Ok(())
}

fn open_main_window(cx: &mut App) {
    let bounds = Bounds::centered(None, size(px(1440.), px(900.)), cx);
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        // The panes run to the top edge; the traffic lights sit in the
        // leftmost pane's top band, and the app moves the window from the
        // bands itself (see `layout::BandDrag`).
        titlebar: Some(TitlebarOptions {
            title: Some("Kanso".into()),
            appears_transparent: true,
            traffic_light_position: Some(point(
                px(layout::TRAFFIC_LIGHTS.0),
                px(layout::TRAFFIC_LIGHTS.1),
            )),
        }),
        app_owns_titlebar_drag: true,
        window_min_size: Some(size(px(1100.), px(720.))),
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

/// Cleans up on any quit, including ones without the Quit action (the
/// Dock, logging out). Open notes save themselves.
fn watch_for_quit(cx: &mut App) {
    cx.on_app_quit(|cx| {
        services::stop_server(cx);
        AppState::save_pending_config(cx);
        async {}
    })
    .detach();
}

/// Saves open notes, then quits. The Quit action arrives while its window
/// is being updated, and a window can't be reached from inside its own
/// update, so the work waits until that update ends.
fn quit(cx: &mut App) {
    cx.defer(|cx| {
        workspace::for_each_workspace(cx, |workspace, cx| {
            workspace.editor.update(cx, |editor, cx| editor.flush(cx));
        });
        services::stop_server(cx);
        cx.quit();
    });
}
