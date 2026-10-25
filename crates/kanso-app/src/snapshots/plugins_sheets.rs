//! Snapshot states for the plugin sheets: installing from the directory,
//! an update that asks for more and has a new key, a developer folder, and
//! a batch waiting for review.

use std::collections::BTreeSet;

use gpui_kit::{AnyWindowHandle, Entity, HeadlessAppContext};
use kanso_core::DocType;
use kanso_plugins::batch::{Batch, LineKind, ReviewNote};
use kanso_plugins::consent::Signed;
use kanso_plugins::model::{
    Grant, Manifest, NotesRequest, Permissions, Publisher, ReadRequest, Source,
};
use kanso_plugins::registry_model::{Bundle, Entry, Offer};
use kanso_store::{Origin, Put, Store};

use crate::plugins_model::{InstallPlan, PendingReview};
use crate::workspace::{AppState, Workspace};

/// The shots: name (`dark-…` or `light-…`), how to set the window up,
/// and the width of a dialog it opens.
pub const SHOTS: &[(&str, super::Prepare, Option<f32>)] = &[
    ("dark-plugin-install", install, Some(WIDTH)),
    ("light-plugin-install", install, Some(WIDTH)),
    ("dark-plugin-update", update, Some(WIDTH)),
    ("light-plugin-update", update, Some(WIDTH)),
    ("dark-plugin-developer", developer, Some(WIDTH)),
    ("light-plugin-developer", developer, Some(WIDTH)),
    ("dark-plugin-review", review, Some(WIDTH)),
    ("light-plugin-review", review, Some(WIDTH)),
];

const WIDTH: f32 = crate::plugin_sheets::SHEET_WIDTH;

const KEY: &str = "ed25519:9f2a71c4d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68";

fn manifest(read: bool, network: bool) -> Manifest {
    Manifest {
        schema: 1,
        api_version: 1,
        id: "com.samrivera.wordcount".into(),
        name: "Word count".into(),
        version: "1.4.0".into(),
        entry: "main.js".into(),
        publisher: Publisher {
            name: "Sam Rivera".into(),
            key: KEY.into(),
        },
        min_app_version: "0.1.0".into(),
        icon: None,
        description: Some("Counts the words in your notes and keeps a tally.".into()),
        permissions: Permissions {
            commands: true,
            notes: NotesRequest {
                read: read.then(ReadRequest::default),
                ..NotesRequest::default()
            },
            network: if network {
                ["api.wordcount.example".to_owned()].into()
            } else {
                BTreeSet::new()
            },
            ..Permissions::default()
        },
    }
}

fn bundle(manifest: Manifest) -> Bundle {
    Bundle {
        manifest,
        files: Vec::new(),
        sha256: "0".repeat(64),
    }
}

/// Sets the plugin service up and adds the notebooks the sheets list.
fn setup(cx: &mut HeadlessAppContext, store: &Store) -> String {
    let put = |fields: serde_json::Value| {
        store
            .put(DocType::Book, fields, Put::Upsert, Origin::Ui)
            .unwrap()
            .id
    };
    let work = put(serde_json::json!({ "name": "Work" }));
    put(serde_json::json!({ "name": "Journal" }));
    put(serde_json::json!({ "name": "Personal vault", "kansoLock": { "salt": "abc" } }));
    let dir = std::env::temp_dir().join("kanso-plugin-sheets-snapshot");
    std::fs::create_dir_all(&dir).unwrap();
    cx.update(|cx| {
        let store = cx.global::<AppState>().store.clone();
        crate::plugins::init_for_test(store, &dir, Vec::new(), cx);
    });
    work
}

fn plan(manifest: Manifest, source: Source, signed: Signed, work: &str) -> InstallPlan {
    let mut grant: Grant = kanso_plugins::grant::initial(&manifest);
    if manifest.permissions.notes.read.is_some() {
        grant.notes.read_books.insert(work.to_owned());
    }
    InstallPlan {
        bundle: bundle(manifest),
        source,
        signed,
        lookalikes: Vec::new(),
        replacing: None,
        grant,
        offer: None,
    }
}

fn open(cx: &mut HeadlessAppContext, window: AnyWindowHandle, plan: InstallPlan) {
    cx.update_window(window, |_, window, cx| {
        crate::plugin_sheets::open_install(plan, window, cx);
    })
    .unwrap();
}

/// From the directory, asking for one notebook and the network.
fn install(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    _: &Entity<Workspace>,
    store: &Store,
) {
    let work = setup(cx, store);
    let plan = plan(
        manifest(true, true),
        Source::Registry {
            sha256: "0".repeat(64),
        },
        Signed::Directory,
        &work,
    );
    open(cx, window, plan);
}

/// An update that asks for more, under a new key.
fn update(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    _: &Entity<Workspace>,
    store: &Store,
) {
    let work = setup(cx, store);
    let mut plan = plan(
        manifest(true, true),
        Source::Registry {
            sha256: "0".repeat(64),
        },
        Signed::Directory,
        &work,
    );
    let entry = Entry {
        id: plan.id().to_owned(),
        name: "Word count".into(),
        publisher: "Sam Rivera".into(),
        description: String::new(),
        version: "1.4.0".into(),
        sha256: "0".repeat(64),
        size: 2048,
        signer: KEY.into(),
        author_signature: String::new(),
        min_app_version: "0.1.0".into(),
        permissions: plan.bundle.manifest.permissions.clone(),
        published_at: "2026-10-01T00:00:00Z".into(),
        revoked: false,
    };
    plan.offer = Some(Offer {
        entry,
        added: vec![
            "read notes in notebooks you choose".into(),
            "connect to api.wordcount.example".into(),
        ],
        removed: vec!["show a panel in the sidebar".into()],
        key_changed: true,
    });
    open(cx, window, plan);
}

/// A folder of code on this Mac, in developer mode.
fn developer(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    _: &Entity<Workspace>,
    store: &Store,
) {
    let work = setup(cx, store);
    let plan = plan(
        manifest(false, false),
        Source::Developer {
            path: "/Users/sam/Developer/wordcount".into(),
        },
        Signed::Developer,
        &work,
    );
    open(cx, window, plan);
}

fn lines(kinds: &[(LineKind, &str)]) -> Vec<(LineKind, String)> {
    kinds.iter().map(|(k, t)| (*k, (*t).to_owned())).collect()
}

/// A batch of three notes waiting, the first expanded.
fn review(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    _: &Entity<Workspace>,
    store: &Store,
) {
    setup(cx, store);
    let note = |title: &str, changes: Vec<&'static str>, body: Vec<(LineKind, String)>| {
        let added = body.iter().filter(|l| l.0 == LineKind::Added).count();
        let removed = body.len() - added;
        ReviewNote {
            id: Some(format!("note:{title}")),
            title: title.to_owned(),
            changes,
            added,
            removed,
            lines: body,
        }
    };
    let notes = vec![
        note(
            "Trip to Lisbon",
            vec!["Text", "Tags"],
            lines(&[
                (LineKind::Removed, "Flight lands at 9:40 on Friday."),
                (
                    LineKind::Added,
                    "Flight lands at 9:40 on Friday (word count: 7).",
                ),
                (LineKind::Added, "Hotel is two streets from the river."),
            ]),
        ),
        note(
            "Weekly review",
            vec!["Text"],
            lines(&[(LineKind::Added, "Words this week: 4,310")]),
        ),
        note("Reading list", vec!["Title"], Vec::new()),
    ];
    let mut batch = Batch::new();
    for note in &notes {
        batch
            .stage_edit(kanso_store::PluginEdit {
                id: note.id.clone().unwrap(),
                base_rev: "1-a".into(),
                ..Default::default()
            })
            .unwrap();
    }
    cx.update(|cx| {
        cx.global_mut::<crate::plugins::Plugins>().review = Some(PendingReview {
            plugin_id: "com.samrivera.wordcount".into(),
            plugin_name: "Word count".into(),
            batch,
            notes,
        });
    });
    cx.update_window(window, |_, window, cx| {
        crate::plugin_sheets::open_review_with(&BTreeSet::from([0]), window, cx);
    })
    .unwrap();
}
