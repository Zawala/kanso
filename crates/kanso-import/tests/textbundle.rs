use kanso_import::library::{Source, SourceKind, import_source};
use kanso_store::Store;

#[test]
fn textbundle_imports_body_assets_metadata_and_is_repeatable() {
    let input = tempfile::tempdir().unwrap();
    let bundle = input.path().join("Plan.textbundle");
    std::fs::create_dir(&bundle).unwrap();
    std::fs::create_dir(bundle.join("assets")).unwrap();
    std::fs::write(
        bundle.join("text.md"),
        "# Plan\n\nSee ![image](assets/pic.png)",
    )
    .unwrap();
    std::fs::write(bundle.join("assets/pic.png"), [3, 4]).unwrap();
    std::fs::write(bundle.join("info.json"),r#"{"net.shinyfrog.bear":{"uniqueIdentifier":"stable-id","creationDate":"2024-01-01T00:00:00Z","modificationDate":"2024-01-02T00:00:00Z"}}"#).unwrap();
    let mut source = kanso_import::library::textbundle::open(input.path()).unwrap();
    let mut first = Vec::new();
    let mut second = Vec::new();
    source
        .notes(false, &mut |n| {
            first.push(n);
            Ok(())
        })
        .unwrap();
    source
        .notes(true, &mut |n| {
            second.push(n);
            Ok(())
        })
        .unwrap();
    assert_eq!(first, second);
    assert_eq!(source.files().len(), 1);
    let tmp = tempfile::tempdir().unwrap();
    let store = Store::open(tmp.path().join("db")).unwrap();
    let a = import_source(SourceKind::TextBundle, input.path(), &store, |_| {}).unwrap();
    let b = import_source(SourceKind::TextBundle, input.path(), &store, |_| {}).unwrap();
    assert_eq!(a.found.notes, 1);
    assert_eq!(b.notes.unchanged, 1);
}

#[test]
fn textbundle_skips_non_markdown_bundles() {
    let d = tempfile::tempdir().unwrap();
    let b = d.path().join("Empty.textbundle");
    std::fs::create_dir(&b).unwrap();
    std::fs::write(b.join("info.json"), "{}").unwrap();
    let mut s = kanso_import::library::textbundle::open(d.path()).unwrap();
    let mut notes = Vec::new();
    s.notes(true, &mut |n| {
        notes.push(n);
        Ok(())
    })
    .unwrap();
    assert!(notes.is_empty());
    assert!(s.skipped().iter().any(|x| x.reason.contains("no Markdown")));
}

#[test]
fn textpack_zip_keeps_bundle_identity_and_assets() {
    use std::io::Write as _;
    let pack = tempfile::NamedTempFile::new().unwrap();
    let mut zip = zip::ZipWriter::new(std::fs::File::create(pack.path()).unwrap());
    let options = zip::write::SimpleFileOptions::default();
    zip.start_file("Notes/Idea.textbundle/text.md", options)
        .unwrap();
    zip.write_all(b"A bundle note").unwrap();
    zip.start_file("Notes/Idea.textbundle/assets/pic.png", options)
        .unwrap();
    zip.write_all(&[1, 2]).unwrap();
    zip.finish().unwrap();
    let mut source = kanso_import::library::textbundle::open(pack.path()).unwrap();
    let mut notes = Vec::new();
    source
        .notes(true, &mut |n| {
            notes.push(n);
            Ok(())
        })
        .unwrap();
    assert_eq!(notes[0].key, "Notes/Idea.textbundle");
    assert_eq!(source.files().len(), 1);
}
