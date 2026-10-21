//! Markdown export: one `.md` file per note in folders named after its
//! notebooks, attachments in `_attachments/`, and in-app links rewritten to
//! relative paths so the export reads well in any Markdown editor. Files
//! already in the chosen folder are never replaced; new ones get numbered
//! names instead.

use std::collections::{HashMap, HashSet};
use std::io::Write as _;
use std::path::{Component, Path, PathBuf};

use anyhow::Context as _;
use kanso_core::links::{FILE_LINK_PREFIX, LINK_SCHEME, NOTE_LINK_PREFIX};
use kanso_core::model::TRASH_BOOK_ID;
use kanso_core::{Book, Note, NoteStatus};
use kanso_store::{SearchParams, Store};

const MAX_NAME_CHARS: usize = 100;
const ATTACHMENTS_DIR: &str = "_attachments";

/// A file or folder name that is safe on macOS, Windows and Linux.
pub fn sanitize_file_name(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '-',
            c if c.is_control() => ' ',
            c => c,
        })
        .collect();
    let collapsed = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    let truncated: String = collapsed.chars().take(MAX_NAME_CHARS).collect();
    let trimmed = truncated.trim_matches(|c: char| c == '.' || c.is_whitespace());
    if trimmed.is_empty() {
        "Untitled".to_owned()
    } else {
        trimmed.to_owned()
    }
}

/// `base` + `extension`, numbered when the name is taken. Names compare
/// case-insensitively, as on macOS.
pub fn unique_name(base: &str, extension: &str, taken: &mut HashSet<String>) -> String {
    let mut n = 1;
    loop {
        let name = if n == 1 {
            format!("{base}{extension}")
        } else {
            format!("{base} {n}{extension}")
        };
        if taken.insert(name.to_lowercase()) {
            return name;
        }
        n += 1;
    }
}

/// The path from directory `from` to `to`, both relative to the export root,
/// with `/` separators.
pub fn relative_path(from: &Path, to: &Path) -> String {
    let from: Vec<Component> = from.components().collect();
    let to: Vec<Component> = to.components().collect();
    let common = from.iter().zip(&to).take_while(|(a, b)| a == b).count();
    let mut parts: Vec<String> = vec!["..".to_owned(); from.len() - common];
    parts.extend(
        to[common..]
            .iter()
            .map(|c| c.as_os_str().to_string_lossy().into_owned()),
    );
    parts.join("/")
}

/// Percent-encodes the characters that would end a Markdown link target.
pub fn encode_link(path: &str) -> String {
    let mut out = String::with_capacity(path.len());
    for c in path.chars() {
        match c {
            ' ' => out.push_str("%20"),
            '(' => out.push_str("%28"),
            ')' => out.push_str("%29"),
            '<' => out.push_str("%3C"),
            '>' => out.push_str("%3E"),
            '%' => out.push_str("%25"),
            c => out.push(c),
        }
    }
    out
}

/// Replaces note links ([`NOTE_LINK_PREFIX`]`<id>`) and attachment links
/// ([`FILE_LINK_PREFIX`]`<id>`) with what the
/// lookups return for the full IDs (`note:…`, `file:…`). Links the lookups
/// don't know are left as they are.
pub fn rewrite_links(
    body: &str,
    note_link: impl Fn(&str) -> Option<String>,
    file_link: impl Fn(&str) -> Option<String>,
) -> String {
    const NOTE: &str = NOTE_LINK_PREFIX;
    const FILE: &str = FILE_LINK_PREFIX;
    let mut out = String::with_capacity(body.len());
    let mut rest = body;
    while let Some(at) = rest.find(LINK_SCHEME) {
        out.push_str(&rest[..at]);
        let tail = &rest[at..];
        let (prefix, kind) = if tail.starts_with(NOTE) {
            (NOTE, "note:")
        } else if tail.starts_with(FILE) {
            (FILE, "file:")
        } else {
            out.push_str(LINK_SCHEME);
            rest = &tail[LINK_SCHEME.len()..];
            continue;
        };
        let suffix_len = tail[prefix.len()..]
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'))
            .unwrap_or(tail.len() - prefix.len());
        let suffix = &tail[prefix.len()..prefix.len() + suffix_len];
        let id = format!("{kind}{suffix}");
        let replacement = if suffix.is_empty() {
            None
        } else if kind == "note:" {
            note_link(&id)
        } else {
            file_link(&id)
        };
        match replacement {
            Some(link) => out.push_str(&link),
            None => out.push_str(&tail[..prefix.len() + suffix_len]),
        }
        rest = &tail[prefix.len() + suffix_len..];
    }
    out.push_str(rest);
    out
}

/// YAML front matter with the note's metadata.
pub fn front_matter(note: &Note, tag_names: &[String]) -> String {
    let quote = |s: &str| serde_json::to_string(s).unwrap_or_default();
    let mut out = String::from("---\n");
    out.push_str(&format!("title: {}\n", quote(&note.title)));
    if !tag_names.is_empty() {
        let tags: Vec<String> = tag_names.iter().map(|t| quote(t)).collect();
        out.push_str(&format!("tags: [{}]\n", tags.join(", ")));
    }
    if note.status != NoteStatus::None {
        out.push_str(&format!("status: {}\n", note.status.as_str()));
    }
    if note.pinned {
        out.push_str("pinned: true\n");
    }
    out.push_str(&format!("created: {}\n", iso_time(note.created_at)));
    out.push_str(&format!("updated: {}\n", iso_time(note.updated_at)));
    out.push_str("---\n\n");
    out
}

fn iso_time(ms: i64) -> String {
    let date = crate::note_list::format_date(ms);
    let secs = ms.div_euclid(1000).rem_euclid(86_400);
    format!(
        "{date}T{:02}:{:02}:{:02}Z",
        secs / 3600,
        secs / 60 % 60,
        secs % 60
    )
}

/// Folder path of each notebook, relative to the export root.
fn book_dirs(books: &[Book]) -> HashMap<String, PathBuf> {
    let mut dirs: HashMap<String, PathBuf> = HashMap::new();
    let mut taken: HashMap<Option<String>, HashSet<String>> = HashMap::new();
    // A top-level notebook must not share the attachments folder.
    taken
        .entry(None)
        .or_default()
        .insert(ATTACHMENTS_DIR.to_lowercase());
    for (_, book) in crate::library::book_tree(books) {
        let parent = book.parent_book_id.clone().filter(|p| dirs.contains_key(p));
        let parent_dir = parent.as_ref().map(|p| dirs[p].clone()).unwrap_or_default();
        let name = unique_name(
            &sanitize_file_name(&book.name),
            "",
            taken.entry(parent).or_default(),
        );
        dirs.insert(book.id.clone(), parent_dir.join(name));
    }
    dirs
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct ExportReport {
    pub notes: usize,
    pub attachments: usize,
    /// Locked notes left out: their text is only sealed, and an export is
    /// plain text.
    pub locked: usize,
}

/// Writes notes into `root`. With `only`, just those notes (and their
/// attachments); otherwise every note outside the trash.
pub fn export(store: &Store, root: &Path, only: Option<&[String]>) -> anyhow::Result<ExportReport> {
    let books = store.books()?;
    let tags: HashMap<String, String> = store.tags()?.into_iter().map(|t| (t.id, t.name)).collect();
    let dirs = book_dirs(&books);
    let notes: Vec<Note> = match only {
        Some(ids) => ids
            .iter()
            .filter_map(|id| store.note(id).ok().flatten())
            .collect(),
        None => store
            .search(&SearchParams::default())?
            .into_iter()
            .filter_map(|doc| serde_json::from_value(doc.to_api(Default::default())).ok())
            .collect(),
    };
    let notes: Vec<Note> = notes
        .into_iter()
        .filter(|n| n.book_id != TRASH_BOOK_ID)
        .collect();
    // Locked notes, and every note in a locked notebook, stay out.
    let locked_books = store.locked_books()?;
    let (locked, notes): (Vec<Note>, Vec<Note>) = notes
        .into_iter()
        .partition(|note| note.is_locked() || locked_books.contains(&note.book_id));

    // Decide every path first so links between notes can point at them.
    // Names in use, on disk or by notebook folders, are taken.
    let mut taken: HashMap<PathBuf, HashSet<String>> = HashMap::new();
    for dir in dirs.values() {
        if let (Some(parent), Some(name)) = (dir.parent(), dir.file_name()) {
            taken
                .entry(parent.to_path_buf())
                .or_insert_with(|| existing_names(&root.join(parent)))
                .insert(name.to_string_lossy().to_lowercase());
        }
    }
    let mut note_paths: HashMap<String, PathBuf> = HashMap::new();
    for note in &notes {
        let dir = dirs.get(&note.book_id).cloned().unwrap_or_default();
        let name = unique_name(
            &sanitize_file_name(&note.title),
            ".md",
            taken
                .entry(dir.clone())
                .or_insert_with(|| existing_names(&root.join(&dir))),
        );
        note_paths.insert(note.id.clone(), dir.join(name));
    }
    let mut file_paths: HashMap<String, PathBuf> = HashMap::new();
    let mut attachment_names = existing_names(&root.join(ATTACHMENTS_DIR));
    for note in &notes {
        for file_id in kanso_core::tasks::linked_file_ids(&note.body) {
            if file_paths.contains_key(&file_id) {
                continue;
            }
            let Some(meta) = store.file_meta(&file_id)? else {
                continue;
            };
            let (stem, extension) = split_extension(&meta.name);
            let name = unique_name(&sanitize_file_name(stem), &extension, &mut attachment_names);
            file_paths.insert(file_id, Path::new(ATTACHMENTS_DIR).join(name));
        }
    }

    let mut report = ExportReport {
        locked: locked.len(),
        ..ExportReport::default()
    };
    for (file_id, path) in &file_paths {
        if let Some((_, attachment)) = store.attachment(file_id)? {
            write_new(&root.join(path), &attachment.data)?;
            report.attachments += 1;
        }
    }
    for note in &notes {
        let path = &note_paths[&note.id];
        let dir = path.parent().unwrap_or(Path::new(""));
        let body = rewrite_links(
            &note.body,
            |id| {
                note_paths
                    .get(id)
                    .map(|to| encode_link(&relative_path(dir, to)))
            },
            |id| {
                file_paths
                    .get(id)
                    .map(|to| encode_link(&relative_path(dir, to)))
            },
        );
        let tag_names: Vec<String> = note
            .tags
            .iter()
            .filter_map(|t| tags.get(t).cloned())
            .collect();
        let text = format!("{}{}", front_matter(note, &tag_names), body);
        write_new(&root.join(path), text.as_bytes())?;
        report.notes += 1;
    }
    Ok(report)
}

fn split_extension(name: &str) -> (&str, String) {
    match name.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() && !ext.is_empty() && ext.len() <= 8 => {
            (stem, format!(".{}", sanitize_file_name(ext)))
        }
        _ => (name, String::new()),
    }
}

/// Names already in `dir`, lowercased.
fn existing_names(dir: &Path) -> HashSet<String> {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .map(|entry| entry.file_name().to_string_lossy().to_lowercase())
                .collect()
        })
        .unwrap_or_default()
}

/// Writes a file that must not exist yet.
fn write_new(path: &Path, data: &[u8]) -> anyhow::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    }
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .with_context(|| format!("creating {}", path.display()))?;
    file.write_all(data)
        .with_context(|| format!("writing {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::{
        encode_link, export, relative_path, rewrite_links, sanitize_file_name, unique_name,
    };
    use kanso_core::DocType;
    use kanso_store::{Origin, Put, Store};
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;
    use serde_json::json;
    use std::collections::{BTreeMap, HashSet};
    use std::path::{Path, PathBuf};

    /// Every file under `root`, relative path to contents.
    fn read_tree(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
        let mut out = BTreeMap::new();
        let mut stack = vec![root.to_path_buf()];
        while let Some(dir) = stack.pop() {
            for entry in std::fs::read_dir(&dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    stack.push(path);
                } else {
                    let data = std::fs::read(&path).unwrap();
                    out.insert(path.strip_prefix(root).unwrap().to_path_buf(), data);
                }
            }
        }
        out
    }

    /// A store with notebooks named `books` and notes `(book index, title)`.
    fn library(books: &[&str], notes: &[(usize, &str)]) -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("kanso.db")).unwrap();
        let ids: Vec<String> = books
            .iter()
            .map(|name| {
                store
                    .put(
                        DocType::Book,
                        json!({ "name": name }),
                        Put::Upsert,
                        Origin::Ui,
                    )
                    .unwrap()
                    .id
            })
            .collect();
        for (book, title) in notes {
            store
                .put(
                    DocType::Note,
                    json!({"title": title, "body": format!("body of {title}"), "bookId": ids[*book]}),
                    Put::Upsert,
                    Origin::Ui,
                )
                .unwrap();
        }
        (dir, store)
    }

    #[test]
    fn exporting_twice_keeps_the_first_export() {
        let (_dir, store) = library(&["Work"], &[(0, "Plan"), (0, "plan")]);
        let out = tempfile::tempdir().unwrap();
        assert_eq!(export(&store, out.path(), None).unwrap().notes, 2);
        let first = read_tree(out.path());
        assert_eq!(export(&store, out.path(), None).unwrap().notes, 2);
        let second = read_tree(out.path());
        assert_eq!(second.len(), 4);
        for (path, data) in &first {
            assert_eq!(second.get(path), Some(data), "{} changed", path.display());
        }
    }

    #[test]
    fn a_notebook_named_like_the_attachments_folder_gets_its_own() {
        let (_dir, store) = library(&["_Attachments"], &[(0, "image.png")]);
        let encoded = "aGVsbG8=";
        let file = store
            .put(
                DocType::File,
                json!({
                    "name": "image.png",
                    "contentType": "image/png",
                    "publicIn": [],
                    "_attachments": {"index": {"content_type": "image/png", "data": encoded}},
                }),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        let suffix = file.strip_prefix("file:").unwrap();
        let book = store.books().unwrap()[0].id.clone();
        store
            .put(
                DocType::Note,
                json!({"title": "pic", "body": format!("![](inkdrop://file:{suffix})"), "bookId": book}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
        let out = tempfile::tempdir().unwrap();
        let report = export(&store, out.path(), None).unwrap();
        assert_eq!((report.notes, report.attachments), (2, 1));
        let tree = read_tree(out.path());
        assert_eq!(tree[Path::new("_attachments/image.png")], b"hello");
        assert!(tree.contains_key(Path::new("_Attachments 2/image.png.md")));
        assert!(tree.contains_key(Path::new("_Attachments 2/pic.md")));
    }

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    fn gen_dir() -> impl Strategy<Value = PathBuf> {
        prop::collection::vec("[a-c]{1,2}", 0..4).prop_map(|parts| parts.iter().collect())
    }

    /// Resolves `rel` (with `..`) against `dir`, the way a Markdown viewer does.
    fn resolve(dir: &Path, rel: &str) -> PathBuf {
        let mut parts: Vec<String> = dir
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect();
        for part in rel.split('/') {
            if part == ".." {
                parts.pop();
            } else {
                parts.push(part.to_owned());
            }
        }
        parts.iter().collect()
    }

    proptest! {
        #![proptest_config(config())]

        /// Refinement: names are non-empty, short, free of path and reserved
        /// characters, and don't start or end with a dot or space.
        #[test]
        fn prop_sanitized_names_are_safe(name in "\\PC{0,200}") {
            let safe = sanitize_file_name(&name);
            prop_assert!(!safe.is_empty());
            prop_assert!(safe.chars().count() <= 100);
            prop_assert!(!safe.contains(['/', '\\', ':', '*', '?', '"', '<', '>', '|']));
            prop_assert!(!safe.chars().any(char::is_control));
            prop_assert!(!safe.starts_with(['.', ' ']) && !safe.ends_with(['.', ' ']));
        }

        /// Idempotence: sanitizing a sanitized name changes nothing.
        #[test]
        fn prop_sanitize_is_idempotent(name in "\\PC{0,200}") {
            let once = sanitize_file_name(&name);
            prop_assert_eq!(sanitize_file_name(&once), once);
        }

        /// Invariant: names handed out are distinct, ignoring case.
        #[test]
        fn prop_unique_names_never_collide(bases in prop::collection::vec("[aA]{1,2}", 1..20)) {
            let mut taken = HashSet::new();
            let names: Vec<String> = bases.iter().map(|b| unique_name(b, ".md", &mut taken)).collect();
            let lower: HashSet<String> = names.iter().map(|n| n.to_lowercase()).collect();
            prop_assert_eq!(lower.len(), names.len());
        }

        /// Oracle: following the relative path from `from` lands on `to`.
        #[test]
        fn prop_relative_path_resolves_to_target(from in gen_dir(), to_dir in gen_dir(), file in "[a-z]{1,4}\\.md") {
            let to = to_dir.join(file);
            prop_assert_eq!(resolve(&from, &relative_path(&from, &to)), to);
        }

        /// Metamorphic: rewriting with lookups that know nothing leaves the
        /// body as it was.
        #[test]
        fn prop_unknown_links_are_left_alone(body in "(\\PC|inkdrop://note/[a-z0-9]{0,5}|inkdrop://file:[a-z0-9]{0,5}|inkdrop://){0,30}") {
            prop_assert_eq!(rewrite_links(&body, |_| None, |_| None), body);
        }

        /// Round-trip: rewriting links to placeholders and back restores the body.
        #[test]
        fn prop_rewrite_round_trips(parts in prop::collection::vec((any::<bool>(), "[a-zA-Z0-9_-]{1,8}", "[ .)]{1,6}"), 0..10)) {
            let body: String = parts.iter().map(|(note, id, tail)| {
                format!("{}{id}{tail}", if *note { "inkdrop://note/" } else { "inkdrop://file:" })
            }).collect();
            let placeholder = rewrite_links(&body, |id| Some(format!("<<{id}>>")), |id| Some(format!("<<{id}>>")));
            prop_assert!(!placeholder.contains("inkdrop://"));
            let mut restored = placeholder.clone();
            for (note, id, _) in &parts {
                let (full, uri) = if *note {
                    (format!("<<note:{id}>>"), format!("inkdrop://note/{id}"))
                } else {
                    (format!("<<file:{id}>>"), format!("inkdrop://file:{id}"))
                };
                restored = restored.replacen(&full, &uri, 1);
            }
            prop_assert_eq!(restored, body);
        }

        /// Invariant: exporting into a folder that already has files keeps
        /// them byte for byte and adds one file per note.
        #[test]
        fn prop_export_never_replaces_files(
            books in prop::collection::vec("[aA_]{1,3}", 1..4),
            notes in prop::collection::vec((0usize..4, "[aA. ]{0,3}"), 0..8),
            existing in prop::collection::vec(("[aA_]{1,3}", "[aA. ]{0,3}"), 0..6),
        ) {
            let books: Vec<&str> = books.iter().map(String::as_str).collect();
            let notes: Vec<(usize, &str)> = notes.iter().map(|(b, t)| (b % books.len(), t.as_str())).collect();
            let (_dir, store) = library(&books, &notes);
            let out = tempfile::tempdir().unwrap();
            for (folder, file) in &existing {
                let dir = out.path().join(folder);
                std::fs::create_dir_all(&dir).unwrap();
                std::fs::write(dir.join(format!("{}.md", sanitize_file_name(file))), b"mine").unwrap();
            }
            let before = read_tree(out.path());
            let report = export(&store, out.path(), None).unwrap();
            let after = read_tree(out.path());
            for (path, data) in &before {
                prop_assert_eq!(after.get(path), Some(data));
            }
            prop_assert_eq!(report.notes, notes.len());
            prop_assert_eq!(after.len(), before.len() + notes.len());
        }

        /// Invariant: an encoded link has no character that ends a link target.
        #[test]
        fn prop_encoded_links_have_no_breaking_characters(path in "[ -~]{0,40}") {
            let encoded = encode_link(&path);
            prop_assert!(!encoded.contains([' ', '(', ')', '<', '>']));
        }
    }

    #[test]
    fn examples() {
        assert_eq!(sanitize_file_name("  a/b: c?  "), "a-b- c-");
        assert_eq!(sanitize_file_name("..."), "Untitled");
        assert_eq!(
            relative_path(Path::new("Work/Projects"), Path::new("Archive/x.md")),
            "../../Archive/x.md"
        );
        assert_eq!(
            rewrite_links(
                "see [it](inkdrop://note/abc) and ![](inkdrop://file:xyz)",
                |id| Some(format!("{id}.md")),
                |_| None
            ),
            "see [it](note:abc.md) and ![](inkdrop://file:xyz)"
        );
    }
}
