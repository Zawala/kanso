use kanso_import::library::{Source, SourceKind, import_source};
use kanso_store::Store;

fn xml() -> &'static str {
    r#"<note><title>Note</title><text><note-content>Body with <bold>bold</bold> and <link:internal>Other</link:internal> and <link:url>file:///tmp/attachment.png</link:url></note-content></text><create-date>2024-01-01T00:00:00Z</create-date><last-change-date>2024-01-02T00:00:00Z</last-change-date><tags><tag>system:notebook:Work</tag></tags></note>"#
}

#[test]
fn tomboy_keeps_ids_dates_tags_links_and_repeat_stability() {
    let input = tempfile::tempdir().unwrap();
    std::fs::write(input.path().join("id-1.note"), xml()).unwrap();
    std::fs::write(
        input.path().join("id-2.note"),
        "<note><title>Other</title><text><note-content>Target</note-content></text></note>",
    )
    .unwrap();
    std::fs::write(input.path().join("attachment.png"), [1, 2]).unwrap();
    let mut source = kanso_import::library::tomboy::open(input.path()).unwrap();
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
    assert_eq!(a[0].key, "id-1");
    assert_eq!(a[0].tags, ["Work"]);
    assert!(a[0].body.contains("**bold**"));
    assert!(a[0].body.contains("kanso-import:note/id-2"));
    assert!(a[0].body.contains("kanso-import:path/attachment.png"));
    let tmp = tempfile::tempdir().unwrap();
    let store = Store::open(tmp.path().join("db")).unwrap();
    let first = import_source(SourceKind::Tomboy, input.path(), &store, |_| {}).unwrap();
    let second = import_source(SourceKind::Tomboy, input.path(), &store, |_| {}).unwrap();
    assert_eq!(first.found.notes, 2);
    assert_eq!(second.notes.unchanged, 2);
}

#[test]
fn tomboy_skips_malformed_xml() {
    let d = tempfile::tempdir().unwrap();
    std::fs::write(d.path().join("bad.note"), "<note>").unwrap();
    let mut s = kanso_import::library::tomboy::open(d.path()).unwrap();
    assert!(s.files().is_empty());
    assert!(
        s.skipped()
            .iter()
            .any(|x| x.reason.contains("invalid Tomboy XML"))
    );
}

#[test]
fn tomboy_decodes_xml_entities_in_text_and_captured_values() {
    let input = tempfile::tempdir().unwrap();
    std::fs::write(
        input.path().join("entities.note"),
        "<note><title>A &amp; B &#x21;</title><text><note-content>Body: A &amp; B &#33;</note-content></text><create-date>2024-01-01T00:00:00&#x5A;</create-date><tags><tag>Rock &amp; Roll</tag></tags></note>",
    )
    .unwrap();

    let mut source = kanso_import::library::tomboy::open(input.path()).unwrap();
    let mut notes = Vec::new();
    source
        .notes(false, &mut |note| {
            notes.push(note);
            Ok(())
        })
        .unwrap();

    assert_eq!(notes[0].title, "A & B !");
    assert_eq!(notes[0].body, "Body: A & B !");
    assert_eq!(notes[0].tags, ["Rock & Roll"]);
    assert!(notes[0].created.is_some());
}
