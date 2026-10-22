use kanso_import::library::{Source, SourceKind, import_source};
use kanso_store::Store;

#[test]
fn roam_keeps_uids_nesting_refs_tags_and_todos() {
    let input = tempfile::tempdir().unwrap();
    std::fs::write(input.path().join("graph.json"), r#"[{"title":"Page A","uid":"uid-a","children":[{"uid":"block-a","string":"{{[[TODO]]}} See [[Page B]] #[[work]]","children":[{"uid":"nested","string":"Child"}]},{"string":"{{[[DONE]]}} Finished"}]},{"title":"Page B","uid":"uid-b","children":[{"string":"Target"}]}]"#).unwrap();
    std::fs::write(input.path().join("reading.pdf"), [1, 2, 3]).unwrap();
    let graph_path = input.path().join("graph.json");
    let graph = std::fs::read_to_string(&graph_path)
        .unwrap()
        .replace("#[[work]]", "#[[work]] and [[reading.pdf]]");
    std::fs::write(graph_path, graph).unwrap();
    let mut source = kanso_import::library::roam::open(input.path()).unwrap();
    let mut one = Vec::new();
    let mut two = Vec::new();
    source
        .notes(false, &mut |n| {
            one.push((n.key, n.title, n.body, n.tags));
            Ok(())
        })
        .unwrap();
    source
        .notes(true, &mut |n| {
            two.push((n.key, n.title, n.body, n.tags));
            Ok(())
        })
        .unwrap();
    assert_eq!(one, two);
    assert_eq!(one[0].0, "uid-a");
    assert!(
        one[0]
            .2
            .contains("- [ ] See [Page B](kanso-import:note/uid-b)")
    );
    assert!(one[0].2.contains("  - Child"));
    assert!(one[0].2.contains("kanso-import:path/reading.pdf"));
    assert_eq!(one[0].3, ["work"]);
    let tmp = tempfile::tempdir().unwrap();
    let store = Store::open(tmp.path().join("db")).unwrap();
    let report = import_source(SourceKind::Roam, input.path(), &store, |_| {}).unwrap();
    assert_eq!(report.found.notes, 2);
}

#[test]
fn roam_rejects_malformed_graph() {
    let d = tempfile::tempdir().unwrap();
    std::fs::write(d.path().join("graph.json"), "{").unwrap();
    assert!(kanso_import::library::roam::open(d.path()).is_err());
}

#[test]
fn roam_converts_adjacent_references_independently() {
    let input = tempfile::tempdir().unwrap();
    std::fs::write(
        input.path().join("graph.json"),
        r#"[{"title":"Source","uid":"source","children":[{"string":"[[A]], then [[B]] rest"}]},{"title":"A","uid":"a"},{"title":"B","uid":"b"}]"#,
    )
    .unwrap();

    let mut source = kanso_import::library::roam::open(input.path()).unwrap();
    let mut notes = Vec::new();
    source
        .notes(false, &mut |note| {
            notes.push(note);
            Ok(())
        })
        .unwrap();

    assert!(
        notes[0]
            .body
            .contains("[A](kanso-import:note/a), then [B](kanso-import:note/b) rest")
    );
}
