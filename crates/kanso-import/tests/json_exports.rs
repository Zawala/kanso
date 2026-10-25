//! Standard Notes' decrypted backups and Simplenote's exports, imported end
//! to end against `tests/fixtures/standard-notes` and
//! `tests/fixtures/simplenote`, and property tests of Simplenote's title
//! split and Standard Notes' nested tags. CI pins the seed with
//! `PROPTEST_RNG_SEED`.

use std::collections::BTreeSet;
use std::io::Write as _;
use std::path::{Path, PathBuf};

use kanso_core::{DocType, TRASH_BOOK_ID};
use kanso_import::library::jsonexports::{parse_standard_notes, split_title};
use kanso_import::library::{
    LibraryReport, Source, SourceKind, SourceNote, dates, import_source, stable_id,
};
use kanso_store::{Doc, Store};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use serde_json::json;

/// The crate's folder: cargo's at run time (the phone runners set it to where
/// they copied the fixtures), else where it was built.
fn manifest_dir() -> std::path::PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(Into::into)
        .unwrap_or_else(|| env!("CARGO_MANIFEST_DIR").into())
}

fn fixture(name: &str) -> PathBuf {
    manifest_dir().join("tests/fixtures").join(name)
}

struct Env {
    dir: tempfile::TempDir,
    store: Store,
}

impl Env {
    fn new() -> Env {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("kanso.db")).unwrap();
        Env { dir, store }
    }

    fn import(&self, kind: SourceKind, path: &Path) -> LibraryReport {
        import_source(kind, path, &self.store, |_| {}).unwrap()
    }

    fn doc(&self, id: &str) -> Doc {
        self.store
            .get(id)
            .unwrap()
            .unwrap_or_else(|| panic!("no {id}"))
    }

    fn note(&self, kind: SourceKind, source: &str, key: &str) -> Doc {
        self.doc(&stable_id(DocType::Note, kind, source, key))
    }

    fn name_of(&self, id: &str) -> String {
        self.doc(id).str_field("name").unwrap().to_owned()
    }

    fn tag_names(&self, note: &Doc) -> Vec<String> {
        let mut names: Vec<String> = note.fields["tags"]
            .as_array()
            .unwrap()
            .iter()
            .map(|id| self.name_of(id.as_str().unwrap()))
            .collect();
        names.sort();
        names
    }

    /// `files` zipped into `name` in the temporary folder.
    fn zip(&self, name: &str, files: &[(&str, &[u8])]) -> PathBuf {
        let path = self.dir.path().join(name);
        let mut zip = zip::ZipWriter::new(std::fs::File::create(&path).unwrap());
        for (file, data) in files {
            zip.start_file(*file, zip::write::SimpleFileOptions::default())
                .unwrap();
            zip.write_all(data).unwrap();
        }
        zip.finish().unwrap();
        path
    }
}

fn ms(text: &str) -> i64 {
    dates::parse(text).unwrap()
}

// ---------------------------------------------------------------------------
// Standard Notes

const SN: SourceKind = SourceKind::StandardNotes;

fn check_standard_notes(env: &Env, report: &LibraryReport) {
    assert!(report.is_clean(), "{report}");
    assert_eq!(report.found.notes, 4, "{report}");
    assert_eq!(report.notes.created, 4, "{report}");
    // Standard Notes and Archived.
    assert_eq!(report.notebooks.created, 2, "{report}");
    assert_eq!(report.trashed, 1);
    let mut skipped: Vec<&str> = report.skipped.iter().map(|s| s.path.as_str()).collect();
    skipped.sort();
    assert_eq!(
        skipped,
        ["1 SN|Theme item(s)", "1 SN|UserPreferences item(s)"]
    );

    let plan = env.note(SN, "Standard Notes", "note-plan");
    assert_eq!(plan.str_field("title"), Some("Plan"));
    assert_eq!(plan.str_field("body"), Some("# Plan\n\n- ship it\n"));
    assert_eq!(env.tag_names(&plan), ["work", "work/acme"]);
    assert_eq!(plan.fields["pinned"], true);
    assert_eq!(plan.fields["createdAt"], ms("2024-03-01T09:30:00.000Z"));
    // The client's edit time, not the server's.
    assert_eq!(plan.fields["updatedAt"], ms("2024-03-05T09:30:00.000Z"));
    let top = plan.str_field("bookId").unwrap().to_owned();
    assert_eq!(env.name_of(&top), "Standard Notes");

    let kept = env.note(SN, "Standard Notes", "note-archive");
    let archived = env.doc(kept.str_field("bookId").unwrap());
    assert_eq!(archived.str_field("name"), Some("Archived"));
    assert_eq!(archived.str_field("parentBookId"), Some(top.as_str()));
    assert_eq!(kept.fields["pinned"], true, "pinned in appData");
    assert_eq!(env.tag_names(&kept), ["work/acme"]);

    let old = env.note(SN, "Standard Notes", "note-old");
    assert_eq!(old.str_field("bookId"), Some(TRASH_BOOK_ID));
    assert_eq!(old.fields["pinned"], false);

    let rich = env.note(SN, "Standard Notes", "note-super");
    assert_eq!(rich.str_field("body"), Some("*Rich*"));
}

#[test]
fn a_standard_notes_backup_comes_in_whole() {
    let env = Env::new();
    let backup = fixture("standard-notes/backup.txt");
    let report = env.import(SN, &backup);
    check_standard_notes(&env, &report);

    let again = env.import(SN, &backup);
    assert_eq!(again.notes.unchanged, 4, "{again}");
    assert_eq!(again.notes.written(), 0);
    assert_eq!(again.notebooks.unchanged, 2);
    assert_eq!(again.tags_created, 0);
}

#[test]
fn a_standard_notes_backup_in_a_zip() {
    let env = Env::new();
    let data = std::fs::read(fixture("standard-notes/backup.txt")).unwrap();
    let zip = env.zip(
        "backup.zip",
        &[
            ("Items/readme.txt", b"not the backup".as_slice()),
            ("Standard Notes Backup and Import File.txt", &data),
        ],
    );
    let report = env.import(SN, &zip);
    check_standard_notes(&env, &report);
}

#[test]
fn an_encrypted_backup_says_what_to_export() {
    let env = Env::new();
    let err = import_source(
        SN,
        &fixture("standard-notes/encrypted.txt"),
        &env.store,
        |_| {},
    )
    .expect_err("an encrypted backup is refused");
    assert_eq!(
        err.to_string(),
        "This backup is encrypted; in Standard Notes choose Export › Decrypted backup."
    );
}

#[test]
fn json_that_is_not_a_backup_is_refused() {
    assert!(parse_standard_notes("{\"notes\": []}").is_err());
    assert!(parse_standard_notes("not json").is_err());
}

// ---------------------------------------------------------------------------
// Simplenote

const SIMPLENOTE: SourceKind = SourceKind::Simplenote;

fn check_simplenote(env: &Env, report: &LibraryReport) {
    assert!(report.is_clean(), "{report}");
    assert_eq!(report.found.notes, 3, "{report}");
    assert_eq!(report.notes.created, 3, "{report}");
    assert_eq!(report.notebooks.created, 1, "{report}");
    assert_eq!(report.trashed, 1);

    let groceries = env.note(SIMPLENOTE, "Simplenote", "s-1");
    assert_eq!(groceries.str_field("title"), Some("Groceries"));
    assert_eq!(groceries.str_field("body"), Some("milk\neggs"));
    assert_eq!(env.tag_names(&groceries), ["home", "lists"]);
    assert_eq!(groceries.fields["pinned"], true);
    assert_eq!(
        groceries.fields["createdAt"],
        ms("2024-03-01T09:30:00.000Z")
    );
    assert_eq!(
        groceries.fields["updatedAt"],
        ms("2024-03-02T09:30:00.000Z")
    );
    assert_eq!(
        env.name_of(groceries.str_field("bookId").unwrap()),
        "Simplenote"
    );

    let phones = env.note(SIMPLENOTE, "Simplenote", "s-2");
    assert_eq!(phones.str_field("title"), Some("Phone numbers"));
    assert_eq!(phones.str_field("body"), Some("Ada 555"));
    assert_eq!(phones.fields["pinned"], true, "pinned by system tag");

    let draft = env.note(SIMPLENOTE, "Simplenote", "s-3");
    assert_eq!(draft.str_field("bookId"), Some(TRASH_BOOK_ID));
    assert_eq!(draft.str_field("body"), Some("not needed"));
}

#[test]
fn simplenote_notes_json() {
    let env = Env::new();
    let path = fixture("simplenote/notes.json");
    let report = env.import(SIMPLENOTE, &path);
    check_simplenote(&env, &report);
    let again = env.import(SIMPLENOTE, &path);
    assert_eq!(again.notes.unchanged, 3, "{again}");
    assert_eq!(again.notes.written(), 0);
}

#[test]
fn simplenote_export_zip() {
    let env = Env::new();
    let data = std::fs::read(fixture("simplenote/notes.json")).unwrap();
    let zip = env.zip(
        "notes.zip",
        &[
            ("Groceries.txt", b"# Groceries\n\nmilk\neggs".as_slice()),
            ("source/notes.json", &data),
        ],
    );
    let report = env.import(SIMPLENOTE, &zip);
    check_simplenote(&env, &report);
}

// ---------------------------------------------------------------------------
// Properties

fn config(cases: u32) -> ProptestConfig {
    ProptestConfig {
        cases,
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
        ..ProptestConfig::default()
    }
}

/// What a title line becomes: trimmed, without leading `#`s.
fn clean(line: &str) -> String {
    line.trim().trim_start_matches('#').trim().to_owned()
}

/// The lines with more than whitespace, without trailing whitespace.
fn squash(text: &str) -> Vec<&str> {
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .map(str::trim_end)
        .collect()
}

/// A random parent/child tree of tags: each tag's parent is an earlier one
/// or none.
fn tag_tree() -> impl Strategy<Value = Vec<(String, Option<usize>)>> {
    prop::collection::vec(("[a-z]{1,6}", any::<Option<prop::sample::Index>>()), 1..12).prop_map(
        |tags| {
            tags.into_iter()
                .enumerate()
                .map(|(ix, (title, parent))| {
                    let parent = parent.filter(|_| ix > 0).map(|p| p.index(ix));
                    (title, parent)
                })
                .collect()
        },
    )
}

fn all_notes(source: &mut dyn Source) -> Vec<SourceNote> {
    let mut notes = Vec::new();
    source
        .notes(true, &mut |note| {
            notes.push(note);
            Ok(())
        })
        .unwrap();
    notes
}

proptest! {
    #![proptest_config(config(256))]

    #[test]
    fn simplenote_title_and_body_reassemble_the_content(
        lines in prop::collection::vec(
            prop_oneof![
                Just(String::new()),
                Just("  ".to_owned()),
                "#{0,3} ?[^\n\r]{0,12}",
            ],
            0..8,
        ),
    ) {
        let content = lines.join("\n");
        let (title, body) = split_title(&content);
        prop_assert!(content.ends_with(&body), "the body is the content's end");
        let head = &content[..content.len() - body.len()];
        prop_assert!(!head.trim().contains('\n'), "one line before the body: {:?}", head);
        prop_assert_eq!(clean(head), title.clone());
        prop_assert!(body.lines().next().is_none_or(|l| !l.trim().is_empty()));
        // Title, a line break and the body give the content back, but for
        // the blank lines and the title's `#`s and spaces.
        let again = format!("{title}\n{body}");
        let mut expected = squash(&content);
        if let Some(first) = expected.first_mut() {
            *first = title.as_str();
        }
        let expected: Vec<&str> = expected.into_iter().filter(|l| !l.is_empty()).collect();
        prop_assert_eq!(squash(&again), expected);
    }

    #[test]
    fn standard_notes_tags_nest_by_their_parents(
        tree in tag_tree(),
        on in prop::collection::vec(any::<bool>(), 12),
    ) {
        let mut items = vec![json!({
            "uuid": "note", "content_type": "Note",
            "created_at": "2024-03-01T09:30:00.000Z", "updated_at": "2024-03-01T09:30:00.000Z",
            "content": {"title": "Note", "text": "", "references": []},
        })];
        let mut expected = BTreeSet::new();
        for (ix, (title, parent)) in tree.iter().enumerate() {
            let mut references = Vec::new();
            if let Some(parent) = parent {
                references.push(json!({
                    "uuid": format!("tag-{parent}"), "content_type": "Tag",
                    "reference_type": "TagToParentTag",
                }));
            }
            if on[ix] {
                references.push(json!({"uuid": "note", "content_type": "Note"}));
                let mut names = vec![title.clone()];
                let mut at = *parent;
                while let Some(p) = at {
                    names.push(tree[p].0.clone());
                    at = tree[p].1;
                }
                names.reverse();
                expected.insert(names.join("/"));
            }
            items.push(json!({
                "uuid": format!("tag-{ix}"), "content_type": "Tag",
                "content": {"title": title, "references": references},
            }));
        }
        let backup = json!({"version": "004", "items": items}).to_string();
        let mut source = parse_standard_notes(&backup).unwrap();
        let notes = all_notes(&mut source);
        prop_assert_eq!(notes.len(), 1);
        let got: BTreeSet<String> = notes[0].tags.iter().cloned().collect();
        prop_assert_eq!(got.len(), notes[0].tags.len(), "no duplicates");
        prop_assert_eq!(got, expected);
    }
}
