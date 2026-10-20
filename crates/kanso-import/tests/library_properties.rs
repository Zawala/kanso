//! Property tests for the pure parts of the library importer: front matter,
//! link finding and rewriting, wiki-link resolution, Notion's names, paths
//! to notebooks, CSV and dates. CI pins the seed with `PROPTEST_RNG_SEED`.

use std::collections::BTreeSet;

use kanso_import::library::frontmatter::{self, Entry, Value};
use kanso_import::library::links::{self, Kind};
use kanso_import::library::markdown::{Flavor, parse_note};
use kanso_import::library::notion;
use kanso_import::library::resolve::{Index, IndexedNote, Target};
use kanso_import::library::tree::{self, TreeFile};
use kanso_import::library::{SourceKind, dates, stable_id};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;

fn config(cases: u32) -> ProptestConfig {
    ProptestConfig {
        cases,
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
        ..ProptestConfig::default()
    }
}

// ---------------------------------------------------------------------------
// Front matter

fn scalar() -> impl Strategy<Value = String> {
    // Words, punctuation that YAML cares about, and quotes.
    "[A-Za-z0-9][A-Za-z0-9 ,.:#'\"/\\-]{0,20}".prop_map(|s| s.trim().to_owned())
}

fn yaml_value() -> impl Strategy<Value = Value> {
    prop_oneof![
        scalar().prop_map(Value::Scalar),
        prop::collection::vec("[A-Za-z0-9][A-Za-z0-9 _/-]{0,10}", 1..4).prop_map(|items| {
            Value::List(items.into_iter().map(|i| i.trim().to_owned()).collect())
        }),
    ]
}

/// An entry as an app would write it: block lists, or a quoted scalar.
fn render_entry(key: &str, value: &Value, block: bool) -> Entry {
    let raw = match value {
        Value::Scalar(text) => format!("{key}: {}\n", frontmatter::quote(text)),
        Value::List(items) if block => {
            let mut raw = format!("{key}:\n");
            for item in items {
                raw.push_str(&format!("  - {}\n", frontmatter::quote(item)));
            }
            raw
        }
        Value::List(items) => {
            let quoted: Vec<String> = items.iter().map(|i| format!("\"{i}\"")).collect();
            format!("{key}: [{}]\n", quoted.join(", "))
        }
        Value::Other => format!("{key}:\n  nested: 1\n"),
    };
    Entry {
        key: key.to_owned(),
        value: value.clone(),
        raw,
    }
}

proptest! {
    #![proptest_config(config(256))]

    /// Round trip: entries written as YAML read back as the same keys and values.
    #[test]
    fn prop_front_matter_round_trips(
        entries in prop::collection::btree_map("[a-z][a-z_]{0,8}", (yaml_value(), any::<bool>()), 0..6)
    ) {
        let written: Vec<Entry> = entries
            .iter()
            .map(|(key, (value, block))| render_entry(key, value, *block))
            .collect();
        let text = frontmatter::render(&written);
        let parsed = frontmatter::parse(&text);
        let expected: Vec<(String, Value)> = written.iter().map(|e| (e.key.clone(), e.value.clone())).collect();
        let got: Vec<(String, Value)> = parsed.iter().map(|e| (e.key.clone(), e.value.clone())).collect();
        prop_assert_eq!(got, expected);
        // Every line belongs to exactly one entry.
        prop_assert_eq!(frontmatter::render(&parsed), text);
    }

    /// Invariant: what Kanso takes out of front matter and what stays in the
    /// note partition its entries; nothing is lost or duplicated.
    #[test]
    fn prop_meta_keeps_every_other_line(
        extra in prop::collection::btree_map("x[a-z]{1,6}", yaml_value(), 0..4),
        title in proptest::option::of(scalar().prop_filter("non-empty", |s| !s.is_empty())),
        tags in prop::collection::vec("[a-z][a-z0-9/]{0,8}", 0..4),
    ) {
        let mut entries = Vec::new();
        if let Some(title) = &title {
            entries.push(render_entry("title", &Value::Scalar(title.clone()), false));
        }
        if !tags.is_empty() {
            entries.push(render_entry("tags", &Value::List(tags.clone()), true));
        }
        let kept: Vec<Entry> = extra.iter().map(|(k, v)| render_entry(k, v, true)).collect();
        entries.extend(kept.iter().cloned());
        let meta = frontmatter::meta(&frontmatter::render(&entries));
        prop_assert_eq!(meta.title, title);
        let expected_tags: Vec<String> = tags
            .iter()
            .map(|t| t.trim_end_matches('/').to_owned())
            .filter(|t| !t.is_empty())
            .collect();
        prop_assert_eq!(meta.tags, expected_tags);
        prop_assert_eq!(meta.rest, frontmatter::render(&kept));
    }

    /// split never loses text: front matter and body are the input minus
    /// the two `---` lines.
    #[test]
    fn prop_split_front_matter(front in "([a-z]{1,5}: [a-z ]{0,8}\n){0,4}", body in "[^\u{0}]{0,60}") {
        let text = format!("---\n{front}---\n{body}");
        prop_assert_eq!(frontmatter::split(&text), Some((front.as_str(), body.as_str())));
    }
}

// ---------------------------------------------------------------------------
// Links

/// A piece of a note: text without link syntax, or a link with its target.
#[derive(Clone, Debug)]
enum Piece {
    Text(String),
    Wiki(String),
    Embed(String),
    Link(String, String),
    Image(String),
    Code(String),
}

fn piece() -> impl Strategy<Value = Piece> {
    let name = "[A-Za-z0-9][A-Za-z0-9 _.-]{0,12}[A-Za-z0-9]";
    let path = "[A-Za-z0-9_-]{1,8}(/[A-Za-z0-9_-]{1,8}){0,2}\\.(md|png|pdf)";
    prop_oneof![
        3 => "[A-Za-z0-9 ,.!?\n-]{1,30}".prop_map(Piece::Text),
        1 => name.prop_map(Piece::Wiki),
        1 => name.prop_map(Piece::Embed),
        1 => ("[A-Za-z ]{1,10}", path).prop_map(|(t, p)| Piece::Link(t, p)),
        1 => path.prop_map(Piece::Image),
        1 => "[A-Za-z0-9 \\[\\]()!]{0,20}".prop_map(Piece::Code),
    ]
}

fn render(pieces: &[Piece]) -> (String, Vec<(Kind, String)>) {
    let mut body = String::new();
    let mut expected = Vec::new();
    for piece in pieces {
        // Keep pieces apart so one can't run into the next.
        body.push(' ');
        match piece {
            Piece::Text(text) => body.push_str(text),
            Piece::Wiki(name) => {
                body.push_str(&format!("[[{name}]]"));
                expected.push((Kind::Wiki, name.trim().to_owned()));
            }
            Piece::Embed(name) => {
                body.push_str(&format!("![[{name}]]"));
                expected.push((Kind::Embed, name.trim().to_owned()));
            }
            Piece::Link(text, path) => {
                body.push_str(&format!("[{text}]({path})"));
                expected.push((Kind::Link, path.clone()));
            }
            Piece::Image(path) => {
                body.push_str(&format!("![]({path})"));
                expected.push((Kind::Image, path.clone()));
            }
            Piece::Code(code) => body.push_str(&format!("`{}`", code.replace('`', ""))),
        }
        body.push(' ');
    }
    (body, expected)
}

proptest! {
    #![proptest_config(config(256))]

    /// Oracle: in a body built from known pieces, scan finds exactly the
    /// links that were put in, in order, and none inside code.
    #[test]
    fn prop_scan_finds_the_links_put_in(pieces in prop::collection::vec(piece(), 0..12)) {
        let (body, expected) = render(&pieces);
        let found: Vec<(Kind, String)> = links::scan(&body).into_iter().map(|f| (f.kind, f.target)).collect();
        prop_assert_eq!(found, expected);
    }

    /// Identity: replacing every link's range with its own text gives the
    /// body back; ranges are in order and don't overlap.
    #[test]
    fn prop_rewriting_with_the_same_text_is_identity(body in "[\\[\\]()!<>a-z `\n./|#]{0,80}") {
        let found = links::scan(&body);
        for pair in found.windows(2) {
            prop_assert!(pair[0].range.end <= pair[1].range.start);
        }
        let edits = found.iter().map(|f| (f.range.clone(), body[f.range.clone()].to_owned())).collect();
        prop_assert_eq!(links::apply(&body, edits), body);
    }

    /// Rewriting touches only the link ranges: the text around them stays.
    #[test]
    fn prop_rewriting_keeps_the_rest(pieces in prop::collection::vec(piece(), 0..10)) {
        let (body, _) = render(&pieces);
        let found = links::scan(&body);
        let edits = found.iter().map(|f| (f.range.clone(), "X".to_owned())).collect();
        let out = links::apply(&body, edits);
        let mut expected = String::new();
        let mut at = 0;
        for f in &found {
            expected.push_str(&body[at..f.range.start]);
            expected.push('X');
            at = f.range.end;
        }
        expected.push_str(&body[at..]);
        prop_assert_eq!(out, expected);
    }
}

// ---------------------------------------------------------------------------
// Wiki-link resolution

fn vault() -> impl Strategy<Value = Vec<String>> {
    prop::collection::btree_set("(([a-c]{1,2})/){0,2}[A-Za-z][a-z]{0,4}\\.md", 1..12)
        .prop_map(|set| set.into_iter().collect())
}

fn index_of(paths: &[String]) -> Index {
    let notes = paths
        .iter()
        .map(|p| IndexedNote {
            path: Some(p.clone()),
            title: tree::stem(tree::file_name(p)).to_owned(),
            aliases: Vec::new(),
        })
        .collect();
    Index::new(notes, Vec::new(), Vec::new())
}

proptest! {
    #![proptest_config(config(256))]

    /// Every note is found by its full path without `.md`, from anywhere,
    /// unless the same path relative to the linking note's folder names
    /// another note (that one is nearer).
    #[test]
    fn prop_full_paths_resolve_exactly(paths in vault(), from in 0usize..12) {
        let index = index_of(&paths);
        let from = &paths[from % paths.len()];
        let dir = tree::parent_of(from);
        for (ix, path) in paths.iter().enumerate() {
            let target = path.trim_end_matches(".md");
            let nearer = tree::join(dir, target).is_some_and(|rel| {
                !dir.is_empty()
                    && paths.iter().any(|p| p.trim_end_matches(".md").eq_ignore_ascii_case(&rel))
            });
            if !nearer {
                prop_assert_eq!(index.wiki(Some(from), target), Some(Target::Note(ix)), "{}", target);
            }
        }
    }

    /// A bare name resolves to a note with that name (ignoring case), and
    /// to the note's own folder's one when it has one.
    #[test]
    fn prop_names_resolve_to_a_note_of_that_name(paths in vault(), from in 0usize..12) {
        let index = index_of(&paths);
        let from_path = &paths[from % paths.len()];
        let dir = tree::parent_of(from_path);
        for path in &paths {
            let name = tree::stem(tree::file_name(path));
            let Some(Target::Note(found)) = index.wiki(Some(from_path), name) else {
                return Err(TestCaseError::fail(format!("{name} not found")));
            };
            let found_path = &paths[found];
            prop_assert!(tree::stem(tree::file_name(found_path)).eq_ignore_ascii_case(name));
            let same_dir = paths.iter().any(|p| {
                tree::parent_of(p).eq_ignore_ascii_case(dir)
                    && tree::stem(tree::file_name(p)).eq_ignore_ascii_case(name)
            });
            if same_dir {
                prop_assert!(tree::parent_of(found_path).eq_ignore_ascii_case(dir));
            }
        }
    }

    /// `tree::join` never climbs out of the tree.
    #[test]
    fn prop_join_stays_inside(base in "([a-z]{1,3}/){0,3}", link in "((\\.\\.|\\.|[a-z]{1,3})/){0,5}[a-z]{1,3}") {
        if let Some(joined) = tree::join(base.trim_end_matches('/'), &link) {
            prop_assert!(!joined.split('/').any(|p| p == ".." || p == "." || p.is_empty()));
        }
    }
}

// ---------------------------------------------------------------------------
// Notion names and notebooks

fn notion_id() -> impl Strategy<Value = String> {
    "[0-9a-f]{32}"
}

proptest! {
    #![proptest_config(config(256))]

    /// Notion's IDs come off names, extensions stay, and cleaning is idempotent.
    #[test]
    fn prop_notion_ids_are_stripped(name in "[A-Za-z0-9][A-Za-z0-9 ()&,-]{0,20}[A-Za-z0-9)]", id in notion_id(), ext in "(md|csv|png)?") {
        let file = if ext.is_empty() { format!("{name} {id}") } else { format!("{name} {id}.{ext}") };
        let expected = if ext.is_empty() { name.clone() } else { format!("{name}.{ext}") };
        let cleaned = notion::clean_name(&file);
        prop_assert_eq!(&cleaned, &expected);
        prop_assert_eq!(notion::clean_name(&cleaned), cleaned);
    }

    /// A page's notebooks are its folders, in order, without IDs.
    #[test]
    fn prop_notion_folders(folders in prop::collection::vec(("[A-Z][a-z]{0,6}", notion_id()), 0..4), id in notion_id()) {
        let mut path: String = folders.iter().map(|(n, id)| format!("{n} {id}/")).collect();
        path.push_str(&format!("Page {id}.md"));
        let expected: Vec<String> = folders.iter().map(|(n, _)| n.clone()).collect();
        prop_assert_eq!(notion::folder_of(&path), expected);
    }

    /// A Markdown file's notebooks are its folders.
    #[test]
    fn prop_markdown_folders(folders in prop::collection::vec("[A-Za-z][A-Za-z ]{0,6}[a-z]", 0..4), name in "[a-z]{1,8}") {
        let mut path = folders.join("/");
        if !path.is_empty() {
            path.push('/');
        }
        path.push_str(&format!("{name}.md"));
        let file = TreeFile { path, size: 0, modified: None, created: None };
        let note = parse_note(&file, "text", Flavor::Obsidian);
        prop_assert_eq!(note.folder, folders);
        prop_assert_eq!(note.title, name);
    }

    /// Round trip: records written as RFC 4180 CSV read back the same.
    #[test]
    fn prop_csv_round_trips(records in prop::collection::vec(prop::collection::vec("[a-z ,\"\n|]{0,8}", 1..4), 1..6)) {
        let width = records[0].len();
        let records: Vec<Vec<String>> = records
            .into_iter()
            .map(|mut r| { r.resize(width, String::new()); r })
            .collect();
        let csv: String = records
            .iter()
            .map(|r| {
                r.iter()
                    .map(|c| format!("\"{}\"", c.replace('"', "\"\"")))
                    .collect::<Vec<_>>()
                    .join(",")
                    + "\r\n"
            })
            .collect();
        prop_assert_eq!(notion::parse_csv(&csv), records);
    }

    /// A table has a header, a rule and one line per row, all as wide.
    #[test]
    fn prop_tables_are_rectangular(records in prop::collection::vec(prop::collection::vec("[a-z |\n]{0,8}", 1..4), 1..6)) {
        let table = notion::markdown_table(&records, |_| false);
        let lines: Vec<&str> = table.lines().collect();
        prop_assert_eq!(lines.len(), records.len() + 1);
        let width = records.iter().map(Vec::len).max().unwrap_or(1);
        for line in lines {
            let bars = line.matches('|').count() - line.matches("\\|").count();
            prop_assert_eq!(bars, width + 1, "{}", line);
        }
    }
}

// ---------------------------------------------------------------------------
// Dates and IDs

proptest! {
    #![proptest_config(config(256))]

    /// Round trip: an instant written as RFC 3339 or as Evernote does reads back.
    #[test]
    fn prop_dates_round_trip(seconds in 0i64..4_102_444_800) {
        let at = chrono::DateTime::from_timestamp(seconds, 0).unwrap();
        prop_assert_eq!(dates::parse(&at.to_rfc3339()), Some(seconds * 1000));
        prop_assert_eq!(dates::parse(&at.format("%Y%m%dT%H%M%SZ").to_string()), Some(seconds * 1000));
        prop_assert_eq!(dates::parse(&(seconds * 1000).to_string()).filter(|_| seconds >= 100_000_000), (seconds >= 100_000_000).then_some(seconds * 1000));
    }

    /// IDs are valid, stable, and differ between keys and sources.
    #[test]
    fn prop_stable_ids(keys in prop::collection::btree_set("[ -~]{1,30}", 1..20)) {
        let mut ids = BTreeSet::new();
        for key in &keys {
            let id = stable_id(kanso_core::DocType::Note, SourceKind::Obsidian, "vault", key);
            prop_assert!(kanso_core::id::is_valid_id(kanso_core::DocType::Note, &id));
            prop_assert_eq!(&id, &stable_id(kanso_core::DocType::Note, SourceKind::Obsidian, "vault", key));
            prop_assert_ne!(&id, &stable_id(kanso_core::DocType::Note, SourceKind::Markdown, "vault", key));
            ids.insert(id);
        }
        prop_assert_eq!(ids.len(), keys.len());
    }
}
