//! Apple Notes importer tests use only Rust-built synthetic Core Data stores.

#[cfg(target_os = "macos")]
#[path = "fixtures/apple_notes/mod.rs"]
mod fixture;

#[cfg(target_os = "macos")]
mod macos_tests {
    use std::collections::HashSet;
    use std::io;
    use std::os::unix::fs::symlink;
    use std::path::PathBuf;

    use kanso_core::DocType;
    use kanso_import::library::apple_notes::{AppleNotesSource, open};
    use kanso_import::library::{Source, SourceKind, import_source, stable_id};
    use kanso_store::{Doc, Store};

    use super::fixture;

    struct TestEnv {
        directory: tempfile::TempDir,
        store: Store,
    }

    impl TestEnv {
        fn new() -> Self {
            let directory = tempfile::tempdir().expect("create test directory");
            let store = Store::open(directory.path().join("kanso.db")).expect("open test store");
            Self { directory, store }
        }

        fn source_directory(&self) -> PathBuf {
            self.directory.path().join("group.com.apple.notes")
        }

        fn note(&self, key: &str) -> Doc {
            let id = stable_id(DocType::Note, SourceKind::AppleNotes, "Apple Notes", key);
            self.store
                .get(&id)
                .expect("read imported note")
                .expect("note exists")
        }
    }

    fn collect_notes(
        source: &mut AppleNotesSource,
        full: bool,
    ) -> Vec<kanso_import::library::SourceNote> {
        let mut notes = Vec::new();
        source
            .notes(full, &mut |note| {
                notes.push(note);
                Ok(())
            })
            .expect("enumerate Apple Notes");
        notes
    }

    #[test]
    fn formatting_hierarchy_links_dates_and_pinned_state_import() {
        let env = TestEnv::new();
        let database = fixture::build_full_fixture(&env.source_directory());
        let mut source = open(&database).expect("open direct NoteStore.sqlite path");
        let notes = collect_notes(&mut source, true);
        let overview = notes
            .iter()
            .find(|note| note.key == "note-overview")
            .unwrap();
        assert_eq!(overview.tags, ["release"]);
        assert!(
            overview.body.starts_with("## Details\n"),
            "{}",
            overview.body
        );
        for expected in [
            "**bold**",
            "*italic*",
            "***both***",
            "<u>under</u>",
            "<sup>up</sup>",
            "<sub>down</sub>",
            "~~strike~~",
            "==🟠highlight==",
            "[web](https://example.test/path)",
            "#release",
            "#Release",
            "@Ada",
            "[Linked note](kanso-import:note/note-target)",
            "[**Example card**](https://example.test/card)",
            "\nCentered\n",
            "> Quoted",
            "- First\n\t- Child",
            "- [x] Done",
            "1. Number one\n2. Number two",
            "```\nlet x = 1;",
        ] {
            assert!(
                overview.body.contains(expected),
                "missing {expected:?} in {}",
                overview.body
            );
        }
        assert!(!overview.body.contains("<p"), "{}", overview.body);
        assert_eq!(overview.folder, ["iCloud", "Work", "Projects"]);
        assert!(
            notes
                .iter()
                .find(|note| note.key == "note-url-title")
                .unwrap()
                .body
                .contains("https://example.test/")
        );
        assert!(
            notes
                .iter()
                .find(|note| note.key == "note-nested-list")
                .unwrap()
                .body
                .starts_with("- Nested first\n\t- Deeper")
        );

        let embedded_keys: Vec<_> = notes
            .iter()
            .flat_map(|note| note.embedded.iter().map(|file| file.key.as_str()))
            .collect();
        assert_eq!(
            embedded_keys.len(),
            embedded_keys.iter().copied().collect::<HashSet<_>>().len()
        );
        let files_note = notes.iter().find(|note| note.key == "note-files").unwrap();
        assert!(
            files_note.body.contains("| Cell value |"),
            "{}",
            files_note.body
        );
        assert!(
            files_note
                .body
                .contains("![Scan Page.jpg](kanso-import:embedded/"),
            "{}",
            files_note.body
        );
        assert!(
            files_note
                .body
                .contains("![Drawing.png](kanso-import:embedded/"),
            "{}",
            files_note.body
        );
        assert!(
            files_note
                .body
                .contains("[document.pdf](kanso-import:embedded/"),
            "{}",
            files_note.body
        );
        assert!(
            !files_note.body.contains("![document.pdf]"),
            "non-image must remain a normal link"
        );
        assert!(
            files_note.body.contains("attachment unavailable"),
            "{}",
            files_note.body
        );
        assert_eq!(
            files_note.embedded.len(),
            6,
            "ordinary files, scan page and drawing are embedded"
        );
        let audio = files_note
            .embedded
            .iter()
            .find(|file| file.name == "voice.mp3")
            .unwrap();
        assert_eq!(audio.content_type.as_deref(), Some("audio/mpeg"));
        let pdf = files_note
            .embedded
            .iter()
            .find(|file| file.name == "document.pdf")
            .unwrap();
        assert_eq!(pdf.content_type.as_deref(), Some("application/pdf"));
        let drawing = files_note
            .embedded
            .iter()
            .find(|file| file.name == "Drawing.png")
            .unwrap();
        assert_eq!(drawing.content_type.as_deref(), Some("image/png"));
        assert!(
            notes
                .iter()
                .find(|note| note.key == "note-attachment-only")
                .unwrap()
                .body
                .contains("![only.jpg]")
        );

        let missing = source.skipped();
        assert!(
            missing
                .iter()
                .any(|skip| skip.reason.contains("password-protected"))
        );
        assert!(
            missing
                .iter()
                .any(|skip| skip.reason.contains("ZICNoteData row is missing"))
        );
        assert!(missing.iter().any(|skip| skip.reason.contains("MISSING-1")));
        assert!(
            missing
                .iter()
                .any(|skip| skip.reason.contains("MISSING-MEDIA-1 media row is missing"))
        );
        assert!(
            missing
                .iter()
                .any(|skip| skip.reason.contains("NOT-DOWNLOADED-1 is unavailable"))
        );
        assert!(source.files().is_empty());
        assert_eq!(
            source.read("anything", 1024).unwrap_err().kind(),
            io::ErrorKind::NotFound
        );

        let report = import_source(SourceKind::AppleNotes, &database, &env.store, |_| {})
            .expect("import source");
        assert_eq!(report.found.notes, 7);
        assert_eq!(report.trashed, 1);
        assert!(
            report
                .skipped
                .iter()
                .any(|skip| skip.reason.contains("password-protected"))
        );
        assert!(
            report
                .skipped
                .iter()
                .any(|skip| skip.reason.contains("MISSING-1"))
        );
        assert!(
            report
                .skipped
                .iter()
                .any(|skip| { skip.reason.contains("MISSING-MEDIA-1 media row is missing") })
        );

        let imported = env.note("note-overview");
        let imported_tag_ids = imported.fields["tags"].as_array().unwrap();
        assert_eq!(imported_tag_ids.len(), 1);
        let imported_tag = env
            .store
            .tags()
            .unwrap()
            .into_iter()
            .find(|tag| Some(tag.id.as_str()) == imported_tag_ids[0].as_str())
            .unwrap();
        assert_eq!(imported_tag.name, "release");
        assert_eq!(imported.fields["pinned"], serde_json::json!(true));
        assert_eq!(
            imported.fields["createdAt"],
            serde_json::json!(1_678_307_200_000_i64)
        );
        let notebook = env
            .store
            .get(imported.str_field("bookId").unwrap())
            .unwrap()
            .unwrap();
        assert_eq!(notebook.str_field("name"), Some("Projects"));
        let parent = env
            .store
            .get(notebook.str_field("parentBookId").unwrap())
            .unwrap()
            .unwrap();
        assert_eq!(parent.str_field("name"), Some("Work"));
        let trash = env.note("note-deleted");
        assert_eq!(trash.str_field("bookId"), Some(kanso_core::TRASH_BOOK_ID));

        let again = import_source(SourceKind::AppleNotes, &database, &env.store, |_| {})
            .expect("repeat import");
        assert_eq!(again.notes.unchanged, report.found.notes);
        assert_eq!(again.notes.written(), 0);
        assert_eq!(again.attachments.unchanged, report.found.attachments);
    }

    #[test]
    fn folder_and_direct_database_paths_produce_the_same_order_and_keys() {
        let env = TestEnv::new();
        let folder = env.source_directory();
        let database = fixture::build_full_fixture(&folder);
        let mut direct = open(&database).expect("open database path");
        let mut directory = open(&folder).expect("open group container path");
        let direct_notes = collect_notes(&mut direct, false);
        let folder_notes = collect_notes(&mut directory, false);
        let direct_identity: Vec<_> = direct_notes
            .iter()
            .map(|note| (note.key.clone(), note.title.clone(), note.folder.clone()))
            .collect();
        let folder_identity: Vec<_> = folder_notes
            .iter()
            .map(|note| (note.key.clone(), note.title.clone(), note.folder.clone()))
            .collect();
        assert_eq!(direct_identity, folder_identity);
    }

    #[test]
    fn malformed_and_missing_note_payloads_are_skipped_with_reasons() {
        let env = TestEnv::new();
        let folder = env.source_directory();
        let database = fixture::build_malformed_fixture(&folder);
        let mut source = open(&database).expect("open synthetic database");
        assert!(collect_notes(&mut source, false).is_empty());
        assert!(collect_notes(&mut source, true).is_empty());
        let skipped = source.skipped();
        assert!(
            skipped
                .iter()
                .any(|skip| skip.reason.contains("ZICNoteData row is missing"))
        );
        assert!(
            skipped
                .iter()
                .any(|skip| skip.reason.contains("invalid gzip stream"))
        );
        assert!(
            skipped
                .iter()
                .any(|skip| skip.reason.contains("invalid protobuf wire data"))
        );
        assert!(
            skipped
                .iter()
                .any(|skip| skip.reason.contains("password-protected"))
        );
    }

    #[test]
    fn independent_golden_wire_payload_uses_apple_field_tags() {
        let env = TestEnv::new();
        let folder = env.source_directory();
        let database = fixture::build_golden_wire_fixture(&folder);
        let mut source = open(&database).expect("open golden-wire database");
        let notes = collect_notes(&mut source, false);
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].title, "Hi");
        assert_eq!(fixture::GOLDEN_WIRE_HEX, "120c10011a08120248692a020802");
    }

    #[test]
    fn synthetic_gzip_payloads_are_deterministic() {
        let first = tempfile::tempdir().expect("first fixture directory");
        let second = tempfile::tempdir().expect("second fixture directory");
        let first_db = fixture::build_full_fixture(&first.path().join("group.com.apple.notes"));
        let second_db = fixture::build_full_fixture(&second.path().join("group.com.apple.notes"));
        let first_connection = rusqlite::Connection::open(first_db).expect("open first fixture");
        let second_connection = rusqlite::Connection::open(second_db).expect("open second fixture");
        let first_payload: Vec<u8> = first_connection
            .query_row(
                "SELECT ZDATA FROM ZICNoteData WHERE ZNOTE = 100",
                [],
                |row| row.get(0),
            )
            .expect("read first deterministic payload");
        let second_payload: Vec<u8> = second_connection
            .query_row(
                "SELECT ZDATA FROM ZICNoteData WHERE ZNOTE = 100",
                [],
                |row| row.get(0),
            )
            .expect("read second deterministic payload");
        assert_eq!(first_payload, second_payload);
    }

    #[test]
    fn embedded_attachment_identity_is_note_scoped_and_repeatable() {
        let env = TestEnv::new();
        let database = fixture::build_full_fixture(&env.source_directory());
        let mut first = open(&database).expect("open first source");
        let first_notes = collect_notes(&mut first, true);
        let mut second = open(&database).expect("open second source");
        let second_notes = collect_notes(&mut second, true);
        let first_keys: Vec<_> = first_notes
            .iter()
            .flat_map(|note| note.embedded.iter().map(|file| file.key.clone()))
            .collect();
        let second_keys: Vec<_> = second_notes
            .iter()
            .flat_map(|note| note.embedded.iter().map(|file| file.key.clone()))
            .collect();
        assert_eq!(first_keys, second_keys);
        assert!(first_keys.iter().all(|key| key.starts_with("apple-notes-")));
    }

    #[test]
    fn embedded_symlink_outside_notes_root_is_never_read() {
        let env = TestEnv::new();
        let folder = env.source_directory();
        let database = fixture::build_full_fixture(&folder);
        let outside_bytes = b"synthetic bytes outside the selected Notes root";
        let outside_path = env.directory.path().join("outside-attachment.bin");
        std::fs::write(&outside_path, outside_bytes).expect("write synthetic outside file");
        let image_path = folder.join("Accounts/ACC-A/Media/MEDIA-IMAGE/photo.png");
        std::fs::remove_file(&image_path).expect("remove fixture image");
        symlink(&outside_path, &image_path).expect("link fixture image outside Notes root");

        let mut source = open(&database).expect("open synthetic Apple Notes database");
        let notes = collect_notes(&mut source, true);
        assert!(
            !notes
                .iter()
                .flat_map(|note| &note.embedded)
                .any(|file| file.data == outside_bytes)
        );
        let files_note = notes.iter().find(|note| note.key == "note-files").unwrap();
        assert!(
            files_note
                .embedded
                .iter()
                .all(|file| file.name != "photo.png")
        );
        let skipped = source.skipped();
        assert!(
            skipped.iter().any(|item| {
                item.reason.contains("IMAGE-1") && item.reason.contains("symbolic link")
            }),
            "{skipped:#?}"
        );
    }

    #[test]
    fn rows_marked_for_deletion_are_left_out_rather_than_trashed() {
        let env = TestEnv::new();
        let database = fixture::build_deletion_and_inline_fixture(&env.source_directory());
        let mut source = open(&database).expect("open synthetic Apple Notes database");
        let notes = collect_notes(&mut source, false);
        let keys: Vec<_> = notes.iter().map(|note| note.key.as_str()).collect();
        assert_eq!(keys, ["note-live", "note-in-purged-folder"]);
        assert!(notes.iter().all(|note| !note.trashed));
        let orphan = notes
            .iter()
            .find(|note| note.key == "note-in-purged-folder")
            .unwrap();
        assert!(orphan.folder.is_empty(), "{:?}", orphan.folder);
    }

    #[test]
    fn other_inline_attachments_render_their_alt_text() {
        let env = TestEnv::new();
        let database = fixture::build_deletion_and_inline_fixture(&env.source_directory());
        let mut source = open(&database).expect("open synthetic Apple Notes database");
        let notes = collect_notes(&mut source, true);
        let live = notes.iter().find(|note| note.key == "note-live").unwrap();
        assert_eq!(
            live.body,
            "Total = 42 and **(attachment unavailable: inline text)**"
        );
        assert!(live.embedded.is_empty());
        let skipped = source.skipped();
        assert!(
            skipped
                .iter()
                .any(|skip| skip.reason == "inline attachment CALC-EMPTY has no text"),
            "{skipped:#?}"
        );
    }

    #[test]
    fn source_name_is_fixed_for_folder_and_file_inputs() {
        let env = TestEnv::new();
        let folder = env.source_directory();
        let database = fixture::build_full_fixture(&folder);
        let file_source = open(&database).expect("open file");
        let folder_source = open(&folder).expect("open folder");
        assert_eq!(file_source.name(), "Apple Notes");
        assert_eq!(folder_source.name(), "Apple Notes");
    }
}

#[cfg(not(target_os = "macos"))]
#[test]
fn opening_apple_notes_on_unsupported_platform_has_a_clear_error() {
    let Err(error) = kanso_import::library::apple_notes::open(std::path::Path::new("missing"))
    else {
        panic!("Apple Notes opening is macOS-only");
    };
    assert!(error.to_string().contains("only on macOS"), "{error:#}");
}
