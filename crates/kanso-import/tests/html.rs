use kanso_import::library::{Source, SourceKind, import_source};
use kanso_store::Store;

#[test]
fn html_folders_keep_hierarchy_links_assets_and_repeat_stability() {
    let input = tempfile::tempdir().unwrap();
    let sub = input.path().join("docs");
    std::fs::create_dir(&sub).unwrap();
    std::fs::write(sub.join("start.html"), "<title>Start</title><h1>Hello</h1><a href='next.html'>Next</a><img src='../pic.png'><script>secret()</script>").unwrap();
    std::fs::write(sub.join("next.html"), "<h1>Next</h1>").unwrap();
    std::fs::write(input.path().join("pic.png"), [1, 2]).unwrap();
    let mut source = kanso_import::library::html::open(input.path()).unwrap();
    assert_eq!(source.files().len(), 1);
    let mut a = Vec::new();
    let mut b = Vec::new();
    source
        .notes(false, &mut |n| {
            a.push(n);
            Ok(())
        })
        .unwrap();
    source
        .notes(true, &mut |n| {
            b.push(n);
            Ok(())
        })
        .unwrap();
    assert_eq!(a, b);
    let start = a.iter().find(|note| note.title == "Start").unwrap();
    assert!(start.body.contains("kanso-import:note/docs/next.html"));
    assert!(start.body.contains("pic.png"));
    assert!(!start.body.contains("secret"));
    let tmp = tempfile::tempdir().unwrap();
    let store = Store::open(tmp.path().join("db")).unwrap();
    let first = import_source(SourceKind::Html, input.path(), &store, |_| {}).unwrap();
    let second = import_source(SourceKind::Html, input.path(), &store, |_| {}).unwrap();
    assert_eq!(first.found.notes, 2);
    assert_eq!(second.notes.unchanged, 2);
}

#[test]
fn html_rejects_broken_zip() {
    let p = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(p.path(), b"not zip").unwrap();
    assert!(kanso_import::library::html::open(p.path()).is_err());
}

#[test]
fn html_reads_zip_documents_and_local_images() {
    use std::io::Write as _;
    let archive = tempfile::NamedTempFile::new().unwrap();
    let mut zip = zip::ZipWriter::new(std::fs::File::create(archive.path()).unwrap());
    let options = zip::write::SimpleFileOptions::default();
    zip.start_file("pages/a.html", options).unwrap();
    zip.write_all(b"<h1>A</h1><img src='../pic.png'>").unwrap();
    zip.start_file("pic.png", options).unwrap();
    zip.write_all(&[7, 8]).unwrap();
    zip.finish().unwrap();
    let mut source = kanso_import::library::html::open(archive.path()).unwrap();
    let mut notes = Vec::new();
    source
        .notes(true, &mut |n| {
            notes.push(n);
            Ok(())
        })
        .unwrap();
    assert_eq!(notes[0].folder, ["pages"]);
    assert_eq!(
        source
            .files()
            .iter()
            .map(|f| f.path.as_str())
            .collect::<Vec<_>>(),
        ["pic.png"]
    );
}

#[test]
fn html_keeps_links_that_escape_the_archive_root() {
    let input = tempfile::tempdir().unwrap();
    std::fs::create_dir(input.path().join("docs")).unwrap();
    std::fs::write(
        input.path().join("docs/start.html"),
        "<h1>Start</h1><a href='../../root.html'>Root</a>",
    )
    .unwrap();
    std::fs::write(input.path().join("root.html"), "<h1>Root</h1>").unwrap();

    let mut source = kanso_import::library::html::open(input.path()).unwrap();
    let mut notes = Vec::new();
    source
        .notes(false, &mut |note| {
            notes.push(note);
            Ok(())
        })
        .unwrap();

    let start = notes.iter().find(|note| note.title == "Start").unwrap();
    assert!(!start.body.contains("kanso-import:note/root.html"));
    assert!(start.body.contains("../../root.html"));
}

#[test]
fn html_links_pages_and_files_with_spaces_and_parentheses() {
    let input = tempfile::tempdir().unwrap();
    std::fs::write(
        input.path().join("start.html"),
        "<h1>Start</h1><a href='My Page (draft).html'>Draft</a>\
         <img src='Photo%20(1).png'><a href='doc (2).pdf'>Doc</a>",
    )
    .unwrap();
    std::fs::write(input.path().join("My Page (draft).html"), "<h1>Draft</h1>").unwrap();
    std::fs::write(input.path().join("Photo (1).png"), [1, 2]).unwrap();
    std::fs::write(input.path().join("doc (2).pdf"), [3]).unwrap();
    // Too deep for the converter's recursion: skipped, not a crash.
    let deep = format!("<h1>Deep</h1>{}", "<div>".repeat(5_000));
    std::fs::write(input.path().join("deep.html"), deep).unwrap();

    let tmp = tempfile::tempdir().unwrap();
    let store = Store::open(tmp.path().join("db")).unwrap();
    let report = import_source(SourceKind::Html, input.path(), &store, |_| {}).unwrap();
    assert_eq!(report.unresolved_links, 0);
    assert_eq!(report.notes.created, 2);
    assert_eq!(report.attachments.created, 2);
    let skipped: Vec<_> = report.skipped.iter().map(|s| s.path.as_str()).collect();
    assert_eq!(skipped, ["deep.html"]);
}
