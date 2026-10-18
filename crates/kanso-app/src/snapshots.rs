//! Renders the real views to PNG files with GPUI's Metal renderer, so the
//! UI can be reviewed without capturing the screen. Run with
//!
//! ```text
//! KANSO_SNAPSHOT_DIR=/tmp/shots [KANSO_SNAPSHOT_DB=copy-of-kanso.db] \
//!     cargo test -p kanso-app snapshots -- --ignored --nocapture
//! ```
//!
//! Without `KANSO_SNAPSHOT_DB` a small demo library is used. The database is
//! copied first, so the original is never written.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use gpui_kit::assets::Assets;
use gpui_kit::{
    AnyWindowHandle, AppContext as _, Bounds, HeadlessAppContext, WindowBounds, WindowOptions, px,
    size,
};
use kanso_core::DocType;
use kanso_store::{Origin, Put, Store};
use serde_json::json;

use crate::config::{AppConfig, ThemePreference, ViewMode};
use crate::services::Services;
use crate::workspace::{AppState, Workspace, apply_theme};

/// A demo library for when no real database is given.
fn demo_store(path: &Path) -> Store {
    let store = Store::open(path).unwrap();
    let book = |name: &str, parent: Option<&str>| {
        store
            .put(
                DocType::Book,
                json!({"name": name, "parentBookId": parent}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id
    };
    let work = book("Work", None);
    let projects = book("Projects", Some(&work));
    let personal = book("Personal", None);
    book("Research", None);
    let tag = |name: &str, color: &str| {
        store
            .put(
                DocType::Tag,
                json!({"name": name, "color": color}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id
    };
    let rust = tag("rust", "orange");
    let idea = tag("idea", "blue");
    let notes = [
        (
            &projects,
            "Kanso roadmap",
            "# Kanso roadmap\n\nA calm, local-first notes app.\n\n## Next\n\n- [x] Import from Inkdrop\n- [x] Local API on port 19840\n- [ ] Command palette\n- [ ] Outline view\n\n```rust\nfn main() {\n    println!(\"hello\");\n}\n```\n\n> Simplicity is the ultimate sophistication.\n",
            vec![rust.clone()],
            "active",
            true,
        ),
        (
            &work,
            "Weekly sync notes",
            "# Weekly sync\n\n| Topic | Owner |\n|---|---|\n| Release | Ngoni |\n| Docs | Team |\n\nAction items are tracked in the tracker.",
            vec![],
            "onHold",
            false,
        ),
        (
            &personal,
            "Reading list",
            "# Reading list\n\n1. *The Design of Everyday Things*\n2. **Shape Up**\n3. Crafting Interpreters\n",
            vec![idea.clone()],
            "none",
            false,
        ),
        (
            &projects,
            "Sync design",
            "# Sync design\n\nDocuments live in D1, attachments in R2. Last writer wins by `updatedAt`; the loser stays in history.",
            vec![rust, idea],
            "completed",
            false,
        ),
    ];
    for (book, title, body, tags, status, pinned) in notes {
        store
            .put(
                DocType::Note,
                json!({"title": title, "body": body, "bookId": book, "tags": tags, "status": status, "pinned": pinned}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
    }
    store
}

fn context() -> HeadlessAppContext {
    let mut cx = HeadlessAppContext::with_platform(
        Arc::new(gpui_apple::AppleTextSystem::new()),
        Arc::new(Assets),
        gpui_kit::platform::current_headless_renderer,
    );
    cx.update(gpui_kit::init);
    cx
}

/// What to show before drawing.
#[derive(Clone, Copy)]
enum Scene {
    /// The most recently edited note.
    Note,
    /// An empty notebook.
    EmptyNotebook,
    /// Search results for a word.
    Search(&'static str),
}

struct Shot {
    name: &'static str,
    theme: ThemePreference,
    view_mode: ViewMode,
    scene: Scene,
}

fn open(
    cx: &mut HeadlessAppContext,
    store: Arc<Store>,
    data_dir: PathBuf,
    shot: &Shot,
) -> (AnyWindowHandle, gpui_kit::Entity<Workspace>) {
    let mut config = AppConfig::default();
    config.appearance.theme = shot.theme;
    config.appearance.view_mode = shot.view_mode;
    // Open the most recently edited note, as after a restart.
    config.last_note = store
        .search_summaries(&Default::default())
        .ok()
        .and_then(|notes| notes.into_iter().next())
        .map(|n| n.id);
    let theme = shot.theme;
    cx.update(|cx| {
        cx.set_global(AppState {
            store,
            data_dir,
            config,
        });
        cx.set_global(Services::default());
        apply_theme(theme, None, cx);
    });
    let (window, workspace) = cx
        .update(|cx| {
            gpui_kit::open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds {
                        origin: Default::default(),
                        size: size(px(1440.), px(900.)),
                    })),
                    focus: false,
                    show: false,
                    ..Default::default()
                },
                cx,
                |window, cx| {
                    apply_theme(theme, Some(window), cx);
                    cx.new(|cx| Workspace::new(window, cx))
                },
            )
        })
        .unwrap();
    (window, workspace)
}

/// Puts the workspace in the shot's scene.
fn set_scene(
    cx: &mut HeadlessAppContext,
    workspace: &gpui_kit::Entity<Workspace>,
    store: &Store,
    scene: Scene,
) {
    let library = cx.update(|cx| workspace.read(cx).library.clone());
    match scene {
        Scene::Note => {}
        Scene::EmptyNotebook => {
            let empty = store
                .books()
                .unwrap()
                .into_iter()
                .find(|b| b.count == 0 && b.name == "Research")
                .or_else(|| store.books().unwrap().into_iter().find(|b| b.count == 0));
            if let Some(book) = empty {
                cx.update(|cx| {
                    library.update(cx, |library, cx| {
                        library.set_filter(crate::library::Filter::Book(book.id), cx);
                        library.select(None, cx);
                    })
                });
            }
        }
        Scene::Search(text) => {
            cx.update(|cx| {
                library.update(cx, |library, cx| library.set_search(text.to_owned(), cx))
            });
        }
    }
}

#[test]
#[ignore = "writes PNGs; run on demand with KANSO_SNAPSHOT_DIR set"]
fn snapshots() {
    let out = PathBuf::from(std::env::var("KANSO_SNAPSHOT_DIR").expect("set KANSO_SNAPSHOT_DIR"));
    std::fs::create_dir_all(&out).unwrap();
    use ThemePreference::{Dark, Light};
    use ViewMode::{Edit, Preview, Split};
    let shots = [
        Shot {
            name: "dark-split",
            theme: Dark,
            view_mode: Split,
            scene: Scene::Note,
        },
        Shot {
            name: "light-split",
            theme: Light,
            view_mode: Split,
            scene: Scene::Note,
        },
        Shot {
            name: "dark-edit",
            theme: Dark,
            view_mode: Edit,
            scene: Scene::Note,
        },
        Shot {
            name: "light-preview",
            theme: Light,
            view_mode: Preview,
            scene: Scene::Note,
        },
        Shot {
            name: "dark-empty",
            theme: Dark,
            view_mode: Split,
            scene: Scene::EmptyNotebook,
        },
        Shot {
            name: "dark-search",
            theme: Dark,
            view_mode: Edit,
            scene: Scene::Search("sync"),
        },
    ];
    let only = std::env::var("KANSO_SNAPSHOT_ONLY").ok();
    for shot in shots.iter().filter(|s| {
        only.as_deref()
            .is_none_or(|o| o.split(',').any(|n| n == s.name))
    }) {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("kanso.db");
        let store = match std::env::var_os("KANSO_SNAPSHOT_DB") {
            Some(source) => {
                std::fs::copy(&source, &db).unwrap();
                Store::open(&db).unwrap()
            }
            None => demo_store(&db),
        };
        let store = Arc::new(store);
        let mut cx = context();
        let (window, workspace) = open(&mut cx, store.clone(), dir.path().to_path_buf(), shot);
        cx.run_until_parked();
        set_scene(&mut cx, &workspace, &store, shot.scene);
        // Search waits for typing to pause.
        cx.advance_clock(std::time::Duration::from_secs(1));
        // Let background work (backlinks, image decoding) settle, then draw.
        for _ in 0..3 {
            cx.run_until_parked();
            cx.update_window(window, |_, window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
            })
            .unwrap();
        }
        let image = cx.capture_screenshot(window).expect("Metal rendering");
        let path = out.join(format!("{}.png", shot.name));
        image.save(&path).unwrap();
        println!(
            "wrote {} ({}x{})",
            path.display(),
            image.width(),
            image.height()
        );
    }
}
