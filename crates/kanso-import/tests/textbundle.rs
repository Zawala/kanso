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

fn notes_of(source: &mut impl Source) -> Vec<kanso_import::library::SourceNote> {
    let mut notes = Vec::new();
    source
        .notes(true, &mut |n| {
            notes.push(n);
            Ok(())
        })
        .unwrap();
    notes
}

#[test]
fn a_textbundle_opened_itself_is_one_note_with_its_assets() {
    let input = tempfile::tempdir().unwrap();
    let bundle = input.path().join("Plan.textbundle");
    std::fs::create_dir_all(bundle.join("assets")).unwrap();
    std::fs::write(
        bundle.join("text.markdown"),
        "# Plan\n\n![image](assets/pic.png)",
    )
    .unwrap();
    std::fs::write(bundle.join("assets/pic.png"), [3, 4]).unwrap();
    std::fs::write(bundle.join("info.json"), "{}").unwrap();
    let mut source = kanso_import::library::textbundle::open(&bundle).unwrap();
    let notes = notes_of(&mut source);
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].title, "Plan");
    assert_eq!(notes[0].key, "Plan.textbundle");
    assert!(notes[0].folder.is_empty());
    assert_eq!(source.files()[0].path, "assets/pic.png");
    assert!(source.skipped().is_empty());
    let tmp = tempfile::tempdir().unwrap();
    let store = Store::open(tmp.path().join("db")).unwrap();
    let report = import_source(SourceKind::TextBundle, &bundle, &store, |_| {}).unwrap();
    assert_eq!(report.found.notes, 1);
    assert_eq!(report.found.attachments, 1);
    assert_eq!(report.unresolved_links, 0);
}

#[test]
fn a_textpack_with_the_bundle_at_its_root_is_one_note() {
    use std::io::Write as _;
    let dir = tempfile::tempdir().unwrap();
    let pack = dir.path().join("Idea.textpack");
    let mut zip = zip::ZipWriter::new(std::fs::File::create(&pack).unwrap());
    let options = zip::write::SimpleFileOptions::default();
    zip.start_file("text.txt", options).unwrap();
    zip.write_all(b"Plain ![](assets/pic.png)").unwrap();
    zip.start_file("assets/pic.png", options).unwrap();
    zip.write_all(&[1, 2]).unwrap();
    zip.start_file("info.json", options).unwrap();
    zip.write_all(br#"{"version":2}"#).unwrap();
    zip.finish().unwrap();
    let mut source = kanso_import::library::textbundle::open(&pack).unwrap();
    let notes = notes_of(&mut source);
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].title, "Idea");
    assert_eq!(notes[0].body, "Plain ![](assets/pic.png)");
    assert_eq!(source.files().len(), 1);
    let tmp = tempfile::tempdir().unwrap();
    let store = Store::open(tmp.path().join("db")).unwrap();
    let report = import_source(SourceKind::TextBundle, &pack, &store, |_| {}).unwrap();
    assert_eq!(report.found.attachments, 1);
}

#[test]
fn any_text_file_is_a_bundles_body() {
    let input = tempfile::tempdir().unwrap();
    for (bundle, body) in [
        ("A.textbundle", "text.markdown"),
        ("B.textbundle", "text.txt"),
        ("C.textbundle", "text.md"),
    ] {
        let dir = input.path().join(bundle);
        std::fs::create_dir(&dir).unwrap();
        std::fs::write(dir.join(body), format!("from {body}")).unwrap();
    }
    let mut source = kanso_import::library::textbundle::open(input.path()).unwrap();
    let bodies: Vec<String> = notes_of(&mut source).into_iter().map(|n| n.body).collect();
    assert_eq!(
        bodies,
        ["from text.markdown", "from text.txt", "from text.md"]
    );
    assert!(source.skipped().is_empty());
}
