use std::{path::Path, path::PathBuf};

use kanso_import::library::{Source, SourceKind, import_source};
use kanso_store::Store;

fn entry() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("index.html"), "ignored").unwrap();
    std::fs::write(dir.path().join("entry.html"), r#"<h2 class="pageHeader">Monday, 1 January 2024</h2><div class="reflectionPrompt">Reflect</div><p class="p2">A thought.</p><p class="p3">Another thought.</p><div class="assetGrid">Morning Walk</div>"#).unwrap();
    dir
}

#[test]
fn apple_journal_two_pass_and_engine_import() {
    let source_dir = entry();
    let mut source = kanso_import::library::apple_journal::open(source_dir.path()).unwrap();
    let files = source.files().to_vec();
    let mut keys = Vec::new();
    for full in [false, true] {
        let mut pass = Vec::new();
        source
            .notes(full, &mut |note| {
                pass.push((note.key, note.title, note.body));
                Ok(())
            })
            .unwrap();
        keys.push(pass);
    }
    assert_eq!(keys[0], keys[1]);
    assert_eq!(files.len(), 0);
    let report = import_source(
        SourceKind::AppleJournal,
        source_dir.path(),
        &Store::open(tempfile::tempdir().unwrap().path().join("db")).unwrap(),
        |_| {},
    )
    .unwrap();
    assert_eq!(report.found.notes, 1);
    assert!(report.skipped.iter().any(|s| s.path == "index.html"));
}

#[test]
fn apple_journal_rejects_malformed_archive() {
    let path = PathBuf::from("/not/read");
    assert!(kanso_import::library::apple_journal::open(Path::new(&path)).is_err());
}

#[test]
fn apple_journal_reads_html_inside_zip() {
    use std::io::Write as _;
    let archive = tempfile::NamedTempFile::new().unwrap();
    let mut zip = zip::ZipWriter::new(std::fs::File::create(archive.path()).unwrap());
    let options = zip::write::SimpleFileOptions::default();
    zip.start_file("Journal/entry.htm", options).unwrap();
    zip.write_all(b"<p class='p2'>From archive</p>").unwrap();
    zip.start_file("Journal/index.html", options).unwrap();
    zip.write_all(b"index").unwrap();
    zip.finish().unwrap();
    let mut source = kanso_import::library::apple_journal::open(archive.path()).unwrap();
    let mut notes = Vec::new();
    source
        .notes(true, &mut |n| {
            notes.push(n);
            Ok(())
        })
        .unwrap();
    assert_eq!(notes.len(), 1);
    assert!(notes[0].body.contains("From archive"));
    assert_eq!(source.skipped()[0].path, "Journal/index.html");
}
