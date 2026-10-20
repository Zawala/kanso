//! Where a link in an imported note points: another imported note, or a
//! file in the source that becomes an attachment.
//!
//! Notes are found the way Obsidian finds them: by path (relative to the
//! linking note, then to the root, with or without `.md`), then by file
//! name anywhere (the nearest one when several share it), then by title or
//! alias. Files are found by path, then in the attachment folders, then by
//! name anywhere.

use std::collections::HashMap;

use kanso_core::links::normalize_title;

use crate::library::tree::{extension, file_name, join, parent_of, stem};

/// File extensions read as notes.
pub const NOTE_EXTENSIONS: &[&str] = &["md", "markdown", "mdown", "mkd", "mkdn", "txt"];

pub fn is_note_path(path: &str) -> bool {
    NOTE_EXTENSIONS.contains(&extension(path).as_str())
}

/// A note as links find it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IndexedNote {
    /// The note's file in the source tree, if it has one.
    pub path: Option<String>,
    pub title: String,
    pub aliases: Vec<String>,
}

/// What a link resolves to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    /// The note at this index of the [`Index`].
    Note(usize),
    /// The file at this index of the tree's file list.
    File(usize),
}

/// Every imported note and every file of the source, for finding links.
#[derive(Default)]
pub struct Index {
    notes: Vec<IndexedNote>,
    /// Lowercased note path, with and without its extension → note.
    note_paths: HashMap<String, usize>,
    /// The same with their case, which wins when two differ only by it.
    exact_paths: HashMap<String, usize>,
    /// Lowercased file stem → notes, in path order.
    note_stems: HashMap<String, Vec<usize>>,
    titles: HashMap<String, usize>,
    aliases: HashMap<String, usize>,
    files: Vec<String>,
    /// Lowercased path → file.
    file_paths: HashMap<String, usize>,
    /// Lowercased file name → files, in path order.
    file_names: HashMap<String, Vec<usize>>,
    /// Folders searched for attachments after the note's own, relative to
    /// the root (`""` is the root); `./x` is relative to the note's folder.
    attachment_dirs: Vec<String>,
}

impl Index {
    /// `files` are the source's non-note files, the candidates for
    /// attachments.
    pub fn new(notes: Vec<IndexedNote>, files: Vec<String>, attachment_dirs: Vec<String>) -> Self {
        let mut index = Index {
            attachment_dirs,
            ..Index::default()
        };
        for (ix, note) in notes.iter().enumerate() {
            if let Some(path) = &note.path {
                let lower = path.to_lowercase();
                index.exact_paths.entry(path.clone()).or_insert(ix);
                if is_note_path(&lower) {
                    index
                        .exact_paths
                        .entry(stem_path(path).to_owned())
                        .or_insert(ix);
                }
                index.note_paths.entry(lower.clone()).or_insert(ix);
                if is_note_path(&lower) {
                    index
                        .note_paths
                        .entry(stem_path(&lower).to_owned())
                        .or_insert(ix);
                }
                index
                    .note_stems
                    .entry(stem(file_name(&lower)).to_owned())
                    .or_default()
                    .push(ix);
            }
            index
                .titles
                .entry(normalize_title(&note.title))
                .or_insert(ix);
            for alias in &note.aliases {
                index.aliases.entry(normalize_title(alias)).or_insert(ix);
            }
        }
        for (ix, path) in files.iter().enumerate() {
            let lower = path.to_lowercase();
            index.file_paths.entry(lower.clone()).or_insert(ix);
            index
                .file_names
                .entry(file_name(&lower).to_owned())
                .or_default()
                .push(ix);
        }
        index.notes = notes;
        index.files = files;
        index
    }

    pub fn note(&self, ix: usize) -> &IndexedNote {
        &self.notes[ix]
    }

    pub fn file(&self, ix: usize) -> &str {
        &self.files[ix]
    }

    /// A wiki-link's target, from the note at `from` (its tree path).
    pub fn wiki(&self, from: Option<&str>, target: &str) -> Option<Target> {
        let target = target.trim().replace('\\', "/");
        if target.is_empty() {
            return None;
        }
        let dir = from.map_or("", parent_of);
        let lower = target.to_lowercase();
        let ext = extension(&lower);
        let looks_like_file = !ext.is_empty() && !is_note_path(&lower);
        if looks_like_file {
            return self.find_file(dir, &target).map(Target::File);
        }
        self.note_by_path(dir, &target)
            .or_else(|| self.note_by_stem(dir, &lower))
            .or_else(|| self.titles.get(&normalize_title(&target)).copied())
            .or_else(|| self.aliases.get(&normalize_title(&target)).copied())
            .map(Target::Note)
            // `[[Picture]]` for `Picture.png`, or a note named like `v1.2`.
            .or_else(|| self.find_file(dir, &target).map(Target::File))
    }

    /// A Markdown link's path (already decoded), from the note at `from`.
    pub fn path(&self, from: Option<&str>, path: &str) -> Option<Target> {
        let dir = from.map_or("", parent_of);
        let lower = path.to_lowercase();
        if is_note_path(&lower) || extension(&lower).is_empty() {
            if let Some(ix) = self.note_by_path(dir, path) {
                return Some(Target::Note(ix));
            }
        } else if let Some(ix) = [join(&dir.to_lowercase(), &lower), join("", &lower)]
            .into_iter()
            .flatten()
            .find_map(|path| self.note_paths.get(&path).copied())
        {
            // A note from a file that isn't Markdown (Notion's databases).
            return Some(Target::Note(ix));
        }
        self.find_file(dir, path).map(Target::File).or_else(|| {
            is_note_path(&lower)
                .then(|| self.note_by_stem(dir, &lower))
                .flatten()
                .map(Target::Note)
        })
    }

    /// A note by its path, relative to `dir` and then to the root, with or
    /// without its extension: as written first, then ignoring case.
    fn note_by_path(&self, dir: &str, target: &str) -> Option<usize> {
        let lower = target.to_lowercase();
        [dir, ""].into_iter().find_map(|base| {
            let exact = join(base, target).and_then(|path| {
                self.exact_paths
                    .get(&path)
                    .or_else(|| self.exact_paths.get(stem_path(&path)))
                    .copied()
            });
            exact.or_else(|| {
                join(&base.to_lowercase(), &lower).and_then(|path| {
                    self.note_paths
                        .get(&path)
                        .or_else(|| self.note_paths.get(stem_path(&path)))
                        .copied()
                })
            })
        })
    }

    fn note_by_stem(&self, dir: &str, lower: &str) -> Option<usize> {
        let name = file_name(lower);
        let name = if is_note_path(name) { stem(name) } else { name };
        let found = self.note_stems.get(name)?;
        nearest(found, dir, |ix| {
            self.notes[ix].path.as_deref().unwrap_or("")
        })
    }

    fn find_file(&self, dir: &str, target: &str) -> Option<usize> {
        let lower = target.to_lowercase();
        let lower_dir = dir.to_lowercase();
        let mut candidates = vec![join(&lower_dir, &lower), join("", &lower)];
        for folder in &self.attachment_dirs {
            let folder = folder.to_lowercase();
            let base = match folder.strip_prefix("./") {
                Some(rest) => join(&lower_dir, rest).unwrap_or_default(),
                None if folder == "." => lower_dir.clone(),
                None => folder.trim_matches('/').to_owned(),
            };
            candidates.push(join(&base, &lower));
        }
        if let Some(ix) = candidates
            .into_iter()
            .flatten()
            .find_map(|path| self.file_paths.get(&path).copied())
        {
            return Some(ix);
        }
        let found = self.file_names.get(file_name(&lower))?;
        nearest(found, dir, |ix| &self.files[ix])
    }
}

/// A path without its last extension.
fn stem_path(path: &str) -> &str {
    match path.rsplit_once('.') {
        Some((head, _)) if !head.ends_with('/') && !head.is_empty() => head,
        _ => path,
    }
}

/// Of several matches, the one whose folder shares the longest prefix with
/// `dir`, then the shortest path, then the first.
fn nearest<'a>(found: &[usize], dir: &str, path_of: impl Fn(usize) -> &'a str) -> Option<usize> {
    let dir: Vec<&str> = dir.split('/').filter(|p| !p.is_empty()).collect();
    found.iter().copied().max_by(|&a, &b| {
        let score = |ix: usize| {
            let path = path_of(ix);
            let parts: Vec<&str> = parent_of(path)
                .split('/')
                .filter(|p| !p.is_empty())
                .collect();
            let shared = dir
                .iter()
                .zip(&parts)
                .take_while(|(x, y)| x.eq_ignore_ascii_case(y))
                .count();
            (shared, std::cmp::Reverse(path.len()), std::cmp::Reverse(ix))
        };
        score(a).cmp(&score(b))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn note(path: &str, title: &str) -> IndexedNote {
        IndexedNote {
            path: Some(path.to_owned()),
            title: title.to_owned(),
            aliases: Vec::new(),
        }
    }

    fn index() -> Index {
        Index::new(
            vec![
                note("Plan.md", "Plan"),
                note("Projects/Plan.md", "Project plan"),
                note("Projects/Acme/Notes.md", "Acme notes"),
                IndexedNote {
                    path: Some("People/Ada.md".into()),
                    title: "Ada Lovelace".into(),
                    aliases: vec!["Ada".into()],
                },
            ],
            vec![
                "Attachments/pic.png".into(),
                "Projects/Acme/diagram.png".into(),
                "Projects/pic.png".into(),
            ],
            vec!["Attachments".into()],
        )
    }

    #[test]
    fn wiki_links_find_notes() {
        let index = index();
        assert_eq!(index.wiki(Some("Daily.md"), "Plan"), Some(Target::Note(0)));
        assert_eq!(
            index.wiki(Some("Projects/x.md"), "Plan"),
            Some(Target::Note(1))
        );
        assert_eq!(
            index.wiki(Some("Daily.md"), "Projects/Plan"),
            Some(Target::Note(1))
        );
        assert_eq!(index.wiki(None, "acme notes"), Some(Target::Note(2)));
        assert_eq!(index.wiki(None, "Ada"), Some(Target::Note(3)));
        assert_eq!(index.wiki(None, "Nobody"), None);
    }

    #[test]
    fn wiki_embeds_find_files() {
        let index = index();
        assert_eq!(
            index.wiki(Some("Daily.md"), "pic.png"),
            Some(Target::File(0))
        );
        assert_eq!(
            index.wiki(Some("Projects/Plan.md"), "pic.png"),
            Some(Target::File(2))
        );
        assert_eq!(
            index.wiki(Some("Daily.md"), "diagram.png"),
            Some(Target::File(1))
        );
    }

    #[test]
    fn markdown_paths() {
        let index = index();
        assert_eq!(
            index.path(Some("Projects/Plan.md"), "Acme/Notes.md"),
            Some(Target::Note(2))
        );
        assert_eq!(
            index.path(Some("Projects/Acme/Notes.md"), "../Plan.md"),
            Some(Target::Note(1))
        );
        assert_eq!(
            index.path(Some("Projects/Acme/Notes.md"), "diagram.png"),
            Some(Target::File(1))
        );
        assert_eq!(index.path(Some("x.md"), "missing.png"), None);
    }
}
