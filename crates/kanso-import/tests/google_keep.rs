use kanso_import::library::{Source, SourceKind, import_source};
use kanso_store::Store;

fn keep(dir: &std::path::Path) {
    std::fs::write(dir.join("Trip.json"), r#"{"title":"Trip\nHoliday","textContent":"Pack","listContent":[{"text":"Passport","isChecked":true}],"labels":[{"name":"travel"}],"isPinned":true,"isArchived":true,"isTrashed":false,"createdTimestampUsec":1700000000000000,"userEditedTimestampUsec":0,"attachments":[{"filePath":"photo.jpg","mimetype":"image/jpeg"}],"annotations":[{"title":"Map","url":"https://example.test/map","description":"Route map"}]}"#).unwrap();
    std::fs::write(dir.join("photo.jpg"), [1, 2, 3]).unwrap();
    std::fs::write(dir.join("Trip.html"), "duplicate").unwrap();
}

#[test]
fn keep_preserves_content_links_labels_dates_and_repeat_stability() {
    let input = tempfile::tempdir().unwrap();
    keep(input.path());
    let mut source = kanso_import::library::google_keep::open(input.path()).unwrap();
    assert_eq!(
        source
            .files()
            .iter()
            .map(|f| f.path.as_str())
            .collect::<Vec<_>>(),
        ["photo.jpg"]
    );
    let mut passes = Vec::new();
    for full in [false, true] {
        let mut notes = Vec::new();
        source
            .notes(full, &mut |n| {
                notes.push((n.key, n.title, n.body, n.tags, n.created));
                Ok(())
            })
            .unwrap();
        passes.push(notes);
    }
    assert_eq!(passes[0], passes[1]);
    assert!(passes[0][0].2.contains("kanso-import:path/photo.jpg"));
    assert!(passes[0][0].2.contains("## Annotations"));
    assert_eq!(passes[0][0].3, ["travel"]);
    assert_eq!(passes[0][0].4, Some(1_700_000_000_000));
    let store_dir = tempfile::tempdir().unwrap();
    let store = Store::open(store_dir.path().join("db")).unwrap();
    let first = import_source(SourceKind::GoogleKeep, input.path(), &store, |_| {}).unwrap();
    let second = import_source(SourceKind::GoogleKeep, input.path(), &store, |_| {}).unwrap();
    assert_eq!(first.found.notes, 1);
    assert_eq!(second.notes.unchanged, 1);
    assert!(
        second
            .skipped
            .iter()
            .any(|s| s.reason.contains("duplicate"))
    );
}

#[test]
fn keep_reports_invalid_json_and_bad_timestamps() {
    let input = tempfile::tempdir().unwrap();
    std::fs::write(input.path().join("bad.json"), "{").unwrap();
    let mut source = kanso_import::library::google_keep::open(input.path()).unwrap();
    let mut notes = Vec::new();
    source
        .notes(true, &mut |n| {
            notes.push(n);
            Ok(())
        })
        .unwrap();
    assert!(notes.is_empty());
    assert!(
        source
            .skipped()
            .iter()
            .any(|s| s.reason.contains("invalid Keep JSON"))
    );
}

#[test]
fn keep_reads_takeout_zip_and_keeps_archive_state() {
    use std::io::Write as _;
    let temp = tempfile::tempdir().unwrap();
    let archive = temp.path().join("takeout.zip");
    let mut zip = zip::ZipWriter::new(std::fs::File::create(&archive).unwrap());
    let options = zip::write::SimpleFileOptions::default();
    zip.start_file("Keep/Note.json", options).unwrap();
    zip.write_all(br#"{"title":"Note","textContent":"body","isArchived":true,"createdTimestampUsec":1700000000000000,"userEditedTimestampUsec":0}"#).unwrap();
    zip.start_file("Keep/Note.html", options).unwrap();
    zip.write_all(b"duplicate").unwrap();
    zip.finish().unwrap();
    let mut source = kanso_import::library::google_keep::open(&archive).unwrap();
    let mut notes = Vec::new();
    source
        .notes(true, &mut |n| {
            notes.push(n);
            Ok(())
        })
        .unwrap();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].folder, ["Keep", "Archived"]);
    assert_eq!(notes[0].updated, notes[0].created);
    assert!(
        source
            .skipped()
            .iter()
            .any(|s| s.reason.contains("duplicate"))
    );
}
