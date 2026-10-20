//! Example tests for importing folders of Markdown and Obsidian vaults,
//! against the fixtures in `tests/fixtures`.

use std::path::{Path, PathBuf};

use kanso_core::DocType;
use kanso_import::library::{
    IMPORT_FIELD, LibraryReport, SourceKind, import_markdown_file, import_source, preview,
};
use kanso_store::{Doc, Origin, Put, Store};
use serde_json::{Value, json};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

/// A copy of a fixture to change, and an empty store.
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

    /// Copies `fixtures/<name>` into the temporary folder and returns it.
    fn copy(&self, name: &str) -> PathBuf {
        let to = self.dir.path().join(name);
        copy_dir(&fixture(name), &to);
        to
    }

    fn import(&self, kind: SourceKind, path: &Path) -> LibraryReport {
        import_source(kind, path, &self.store, |_| {}).unwrap()
    }

    fn notes(&self) -> Vec<Doc> {
        let mut notes: Vec<Doc> = self
            .store
            .search(&kanso_store::SearchParams::default())
            .unwrap();
        notes.sort_by(|a, b| a.str_field("title").cmp(&b.str_field("title")));
        notes
    }

    fn note(&self, title: &str) -> Doc {
        self.notes()
            .into_iter()
            .find(|n| n.str_field("title") == Some(title))
            .unwrap_or_else(|| panic!("no note titled {title}"))
    }

    fn book_name(&self, id: &str) -> String {
        self.store
            .get(id)
            .unwrap()
            .unwrap()
            .str_field("name")
            .unwrap()
            .to_owned()
    }

    fn tag_names(&self, note: &Doc) -> Vec<String> {
        let mut names: Vec<String> = note.fields["tags"]
            .as_array()
            .unwrap()
            .iter()
            .map(|id| {
                let tag = self.store.get(id.as_str().unwrap()).unwrap().unwrap();
                tag.str_field("name").unwrap().to_owned()
            })
            .collect();
        names.sort();
        names
    }
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn file_suffix(body: &str, name: &str) -> String {
    let at = body
        .find(name)
        .unwrap_or_else(|| panic!("{name} not in {body}"));
    let rest = &body[at..];
    let start = rest.find("inkdrop://file:").unwrap() + "inkdrop://file:".len();
    rest[start..]
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .collect()
}

#[test]
fn an_obsidian_vault_comes_in_whole() {
    let env = Env::new();
    let vault = env.copy("obsidian");
    let report = env.import(SourceKind::Obsidian, &vault);
    assert!(report.is_clean(), "{report}");
    assert_eq!(report.found.notes, 3, "{report}");
    assert_eq!(report.notes.created, 3);
    // The vault, Projects and People.
    assert_eq!(report.notebooks.created, 3);
    assert_eq!(report.attachments.created, 2);
    assert_eq!(report.embeds_as_links, 1);
    assert_eq!(report.unresolved_links, 1, "[[Missing note]]");
    let mut skipped: Vec<(&str, &str)> = report
        .skipped
        .iter()
        .map(|s| (s.path.as_str(), s.reason.as_str()))
        .collect();
    skipped.sort();
    assert_eq!(
        skipped.iter().map(|(p, _)| *p).collect::<Vec<_>>(),
        [".obsidian/", ".trash/", "Board.canvas", "orphan.png"]
    );

    let welcome = env.note("Welcome");
    let body = welcome.str_field("body").unwrap();
    assert!(
        body.starts_with("---\naliases: [Home]\n---\n\n# Intro\n"),
        "{body}"
    );
    assert!(body.contains("[[Plan|the plan]]"), "{body}");
    assert!(body.contains("[[Ada Lovelace|Ada]]"), "{body}");
    assert!(
        body.contains("\n[[Plan]]\n"),
        "the embedded note is a link: {body}"
    );
    assert!(body.contains("![diagram|300](inkdrop://file:"), "{body}");
    assert!(body.contains("[spec](inkdrop://file:"), "{body}");
    assert!(body.contains("[[Missing note]]"), "{body}");
    assert!(body.contains("`[[not a link]]`"), "{body}");
    assert!(body.contains("> [!note] A callout\n> stays as Markdown."));
    assert_eq!(welcome.fields["createdAt"], json!(1_709_285_400_000i64));
    assert_eq!(env.tag_names(&welcome), ["inbox/today", "start"]);
    assert_eq!(
        env.book_name(welcome.str_field("bookId").unwrap()),
        "obsidian"
    );

    let plan = env.note("Plan");
    let plan_body = plan.str_field("body").unwrap();
    assert!(
        plan_body.starts_with("Back to [[Welcome#Intro]]."),
        "{plan_body}"
    );
    // `![[diagram.png]]` and `../Attachments/diagram.png` are one file.
    assert_eq!(file_suffix(body, "diagram"), file_suffix(plan_body, "![]("));
    assert_eq!(env.tag_names(&plan), ["project/acme"]);
    let book = env
        .store
        .get(plan.str_field("bookId").unwrap())
        .unwrap()
        .unwrap();
    assert_eq!(book.str_field("name"), Some("Projects"));
    assert_eq!(
        book.fields["parentBookId"].as_str(),
        welcome.str_field("bookId")
    );

    let diagram = format!("file:{}", file_suffix(plan_body, "![]("));
    let (content_type, attachment) = env.store.attachment(&diagram).unwrap().unwrap();
    assert_eq!(content_type, "image/png");
    assert_eq!(attachment.data, b"\x89PNG\r\n\x1a\nfixture");
}

#[test]
fn importing_again_changes_only_what_changed_in_the_source() {
    let env = Env::new();
    let vault = env.copy("obsidian");
    env.import(SourceKind::Obsidian, &vault);
    let before = env.notes();

    let again = env.import(SourceKind::Obsidian, &vault);
    assert_eq!(again.notes.unchanged, 3, "{again}");
    assert_eq!(again.notes.written(), 0);
    assert_eq!(again.notebooks.unchanged, 3);
    assert_eq!(again.attachments.unchanged, 2);
    assert_eq!(again.tags_created, 0);
    assert_eq!(env.notes(), before, "nothing was rewritten");

    std::fs::write(vault.join("Projects/Plan.md"), "# Plan\n\nRevised.\n").unwrap();
    std::fs::write(vault.join("New.md"), "A new note.\n").unwrap();
    let third = env.import(SourceKind::Obsidian, &vault);
    assert_eq!(
        (
            third.notes.created,
            third.notes.updated,
            third.notes.unchanged
        ),
        (1, 1, 2),
        "{third}"
    );
    assert_eq!(env.note("Plan").str_field("body"), Some("Revised.\n"));
    assert_eq!(env.notes().len(), 4, "no duplicates");
}

#[test]
fn notes_changed_or_deleted_in_kanso_are_never_reverted() {
    let env = Env::new();
    let vault = env.copy("obsidian");
    env.import(SourceKind::Obsidian, &vault);
    let plan = env.note("Plan");
    env.store
        .put(
            DocType::Note,
            json!({"_id": plan.id, "body": "Edited in Kanso."}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    let ada = env.note("Ada Lovelace");
    env.store.delete(&ada.id, Origin::Ui).unwrap();

    std::fs::write(vault.join("Projects/Plan.md"), "Changed in Obsidian too.\n").unwrap();
    std::fs::write(vault.join("People/Ada Lovelace.md"), "Changed.\n").unwrap();
    let report = env.import(SourceKind::Obsidian, &vault);
    assert_eq!(report.notes.kept_edited, 1, "{report}");
    assert_eq!(report.notes.kept_deleted, 1, "{report}");
    assert_eq!(env.note("Plan").str_field("body"), Some("Edited in Kanso."));
    assert!(env.store.get(&ada.id).unwrap().is_none());
}

#[test]
fn a_preview_writes_nothing() {
    let env = Env::new();
    let vault = fixture("obsidian");
    let report = preview(SourceKind::Obsidian, &vault, &env.store, |_| {}).unwrap();
    assert!(report.dry_run);
    assert_eq!(report.notes.created, 3, "{report}");
    assert_eq!(report.notebooks.created, 3);
    assert_eq!(report.attachments.created, 2);
    assert_eq!(report.tags_created, 3);
    assert_eq!(report.found.tags, 3);
    assert!(env.notes().is_empty());
    assert!(env.store.tags().unwrap().is_empty());
    // And the real import agrees with it.
    let real = env.import(SourceKind::Obsidian, &vault);
    assert_eq!(
        (real.found.clone(), real.notes.clone(), real.tags_created),
        (report.found, report.notes, report.tags_created)
    );
}

#[test]
fn a_markdown_folder() {
    let env = Env::new();
    let report = env.import(SourceKind::Markdown, &fixture("markdown"));
    assert!(report.is_clean(), "{report}");
    assert_eq!(report.found.notes, 3, "{report}");
    let ideas = env.note("Big ideas");
    let other = env.note("Another note");
    let body = ideas.str_field("body").unwrap();
    assert!(
        body.contains(&format!(
            "[the other one](inkdrop://note/{})",
            other.id.trim_start_matches("note:")
        )),
        "{body}"
    );
    assert!(body.contains("![a picture](inkdrop://file:"), "{body}");
    assert!(body.contains("(https://example.com)"));
    assert_eq!(env.tag_names(&other), ["draft", "todo"]);
    assert_eq!(other.str_field("body"), Some("Text with a #todo tag.\n"));
    assert!(other.fields["updatedAt"].as_i64().is_some());
    assert_eq!(env.book_name(other.str_field("bookId").unwrap()), "sub");
    let readme = env.note("readme");
    assert_eq!(
        readme.str_field("body"),
        Some("Plain text notes work too.\n")
    );
    let marked = &ideas.fields[IMPORT_FIELD];
    assert_eq!(marked["source"], "markdown");
    assert_eq!(marked["key"], "Ideas.md");
}

#[test]
fn opening_a_markdown_file_imports_it_into_the_inbox() {
    let env = Env::new();
    let folder = env.copy("markdown");
    let file = folder.join("Ideas.md");
    let report = import_markdown_file(&file, &env.store).unwrap();
    assert_eq!(report.notes.created, 1, "{report}");
    assert_eq!(report.attachments.created, 1, "the picture beside it");
    let id = report.first_note.clone().unwrap();
    let note = env.store.get(&id).unwrap().unwrap();
    let book = env
        .store
        .get(note.str_field("bookId").unwrap())
        .unwrap()
        .unwrap();
    assert_eq!(book.fields.get("kansoRole"), Some(&Value::from("inbox")));

    std::fs::write(&file, "# Big ideas\n\nShorter now.\n").unwrap();
    let again = import_markdown_file(&file, &env.store).unwrap();
    assert_eq!(again.first_note.as_deref(), Some(id.as_str()));
    assert_eq!(again.notes.updated, 1, "{again}");
    assert_eq!(
        env.store.get(&id).unwrap().unwrap().str_field("body"),
        Some("Shorter now.\n")
    );
}
