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

fn roam_notes(graph: &str) -> Vec<kanso_import::library::SourceNote> {
    let input = tempfile::tempdir().unwrap();
    std::fs::write(input.path().join("graph.json"), graph).unwrap();
    let mut source = kanso_import::library::roam::open(input.path()).unwrap();
    let mut notes = Vec::new();
    source
        .notes(false, &mut |note| {
            notes.push(note);
            Ok(())
        })
        .unwrap();
    notes
}

#[test]
fn roam_pages_keep_their_times_else_their_blocks_times() {
    let notes = roam_notes(
        r#"[
        {"title":"Own","uid":"own","create-time":1600000000000,"edit-time":1700000000000,
         "children":[{"string":"x","create-time":1500000000000,"edit-time":1650000000000}]},
        {"title":"Blocks","uid":"blocks","children":[
          {"string":"a","create-time":1610000000000,"edit-time":1620000000000,
           "children":[{"string":"b","create-time":1605000000000,"edit-time":1630000000000}]}]},
        {"title":"Later block","uid":"later","create-time":1600000000000,"edit-time":1600000000000,
         "children":[{"string":"c","edit-time":1690000000000}]},
        {"title":"None","uid":"none","children":[{"string":"d"}]}
    ]"#,
    );
    let times: Vec<_> = notes.iter().map(|n| (n.created, n.updated)).collect();
    assert_eq!(
        times,
        [
            (Some(1_600_000_000_000), Some(1_700_000_000_000)),
            (Some(1_605_000_000_000), Some(1_630_000_000_000)),
            (Some(1_600_000_000_000), Some(1_690_000_000_000)),
            (None, None),
        ]
    );
}

#[test]
fn roam_block_references_become_the_blocks_text_one_level_deep() {
    let notes = roam_notes(
        r#"[
        {"title":"Source","uid":"source","children":[
          {"uid":"quote","string":"{{[[TODO]]}} Quoted on [[Target]]"},
          {"string":"See ((quote)) and ((missing))"},
          {"uid":"loop-a","string":"A says ((loop-b))"},
          {"uid":"loop-b","string":"B says ((loop-a))"},
          {"string":"Go [there](((quote)))"}]},
        {"title":"Target","uid":"target"}
    ]"#,
    );
    let body = &notes[0].body;
    assert!(
        body.contains("- See Quoted on [Target](kanso-import:note/target) and ((missing))"),
        "{body}"
    );
    assert!(body.contains("- A says B says ((loop-a))"), "{body}");
    assert!(body.contains("- B says A says ((loop-b))"), "{body}");
    assert!(
        body.contains("- Go [there](kanso-import:note/source)"),
        "{body}"
    );
}

#[test]
fn roam_multi_line_blocks_stay_in_their_list_item() {
    let notes = roam_notes(
        r#"[{"title":"Lines","uid":"lines","children":[
        {"string":"Parent first\nparent second","children":[
          {"string":"Child first\n\nchild second"}]},
        {"string":"Next"}]}]"#,
    );
    assert_eq!(
        notes[0].body,
        "- Parent first\n  parent second\n  - Child first\n\n    child second\n- Next"
    );
}

proptest::proptest! {
    #![proptest_config(proptest::prelude::ProptestConfig::with_cases(64))]

    /// However many lines a block has, every one of them sits inside its
    /// own list item: deeper than its marker, and before the next marker.
    #[test]
    fn prop_block_lines_stay_under_their_marker(
        blocks in proptest::collection::vec(
            ("[a-z ]{0,6}(\n[a-z ]{0,6}){0,3}", "[a-z ]{0,6}(\n[a-z ]{0,6}){0,3}"),
            1..4,
        )
    ) {
        let children: Vec<_> = blocks
            .iter()
            .map(|(parent, child)| serde_json::json!({
                "string": parent,
                "children": [{ "string": child }],
            }))
            .collect();
        let graph = serde_json::json!([{ "title": "P", "uid": "p", "children": children }]);
        let notes = roam_notes(&graph.to_string());
        let mut marker: Option<usize> = None;
        for line in notes[0].body.lines().filter(|l| !l.trim().is_empty()) {
            let indent = line.len() - line.trim_start().len();
            if line.trim_start().starts_with('-') && (indent == 0 || indent == 2) {
                proptest::prop_assert!(indent == 0 || marker.is_some(), "{line:?}");
                marker = Some(indent);
            } else {
                let at = marker.expect("text before any list item");
                proptest::prop_assert!(indent >= at + 2, "{line:?} in {:?}", notes[0].body);
            }
        }
    }
}

#[test]
fn roam_aliases_link_to_their_page() {
    let notes = roam_notes(
        r#"[{"title":"Source","uid":"source","children":[
        {"string":"Read [the target]([[Target]]) and [gone]([[Nowhere]]) now"}]},
        {"title":"Target","uid":"target"}]"#,
    );
    assert_eq!(
        notes[0].body,
        "- Read [the target](kanso-import:note/target) and gone now"
    );
}
