//! The files of a source, wherever they are: a folder, a zip (Notion's and
//! Bear's exports, with Notion's zips of zips), or one file. Paths are
//! relative, with `/` between components, and every read is read-only.

use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read as _};
use std::path::{Path, PathBuf};

use anyhow::Context as _;

/// A file in a [`Tree`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeFile {
    /// Relative to the tree's root, `/`-separated.
    pub path: String,
    pub size: u64,
    /// Last modified, in milliseconds since the epoch.
    pub modified: Option<i64>,
    /// Created, when the file system or archive records it.
    pub created: Option<i64>,
}

pub enum Tree {
    /// A folder on disk. Symbolic links are not followed out of it.
    Dir(PathBuf),
    /// One file on disk, seen as a tree holding only it.
    Single(PathBuf),
    /// An archive and where each cleaned path is in it.
    Zip(Box<zip::ZipArchive<File>>, HashMap<String, usize>),
    /// Several trees seen as one (the parts of a split Notion export).
    /// The first holding a path wins.
    Many(Vec<Tree>, HashMap<String, usize>),
}

/// The longest walk into nested folders; deeper ones are skipped.
const MAX_DEPTH: usize = 64;

impl Tree {
    pub fn dir(path: &Path) -> anyhow::Result<Tree> {
        anyhow::ensure!(path.is_dir(), "{} is not a folder", path.display());
        Ok(Tree::Dir(path.to_owned()))
    }

    pub fn zip(path: &Path) -> anyhow::Result<Tree> {
        let file = File::open(path).with_context(|| format!("cannot open {}", path.display()))?;
        let archive = zip::ZipArchive::new(file)
            .with_context(|| format!("{} is not a zip archive", path.display()))?;
        let mut index = HashMap::new();
        for ix in 0..archive.len() {
            if let Some(path) = archive.name_for_index(ix).and_then(clean_zip_path) {
                index.entry(path).or_insert(ix);
            }
        }
        Ok(Tree::Zip(Box::new(archive), index))
    }

    /// Joins trees; `files` lists each path once.
    pub fn many(trees: Vec<Tree>) -> anyhow::Result<Tree> {
        let mut trees = trees;
        let mut owner = HashMap::new();
        for (ix, tree) in trees.iter_mut().enumerate() {
            for file in tree.files()? {
                owner.entry(file.path).or_insert(ix);
            }
        }
        Ok(Tree::Many(trees, owner))
    }

    /// Every file, sorted by path. Folders and files whose names start with
    /// `.` are left out, as are folders deeper than [`MAX_DEPTH`].
    pub fn files(&mut self) -> anyhow::Result<Vec<TreeFile>> {
        let mut files = match self {
            Tree::Dir(root) => {
                let mut files = Vec::new();
                walk(root, "", 0, &mut files)?;
                files
            }
            Tree::Single(path) => {
                let meta = std::fs::metadata(&*path)
                    .with_context(|| format!("cannot read {}", path.display()))?;
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .context("the file name is not valid Unicode")?;
                vec![tree_file(name.to_owned(), &meta)]
            }
            Tree::Zip(archive, _) => {
                let mut files = Vec::new();
                for ix in 0..archive.len() {
                    let entry = archive.by_index_raw(ix)?;
                    if entry.is_dir() {
                        continue;
                    }
                    let Some(path) = clean_zip_path(entry.name()) else {
                        continue;
                    };
                    files.push(TreeFile {
                        path,
                        size: entry.size(),
                        modified: entry.last_modified().and_then(zip_time),
                        created: None,
                    });
                }
                files
            }
            Tree::Many(trees, _) => {
                let mut seen = std::collections::HashSet::new();
                let mut files = Vec::new();
                for tree in trees.iter_mut() {
                    for file in tree.files()? {
                        if seen.insert(file.path.clone()) {
                            files.push(file);
                        }
                    }
                }
                files
            }
        };
        files.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(files)
    }

    /// The bytes of `path`, refusing anything over `limit` bytes.
    pub fn read(&mut self, path: &str, limit: u64) -> io::Result<Vec<u8>> {
        let too_big = || io::Error::other(format!("larger than {limit} bytes"));
        match self {
            Tree::Dir(root) => {
                let full = root.join(path);
                if std::fs::metadata(&full)?.len() > limit {
                    return Err(too_big());
                }
                read_limited(File::open(full)?, limit)
            }
            Tree::Single(file) => {
                if file.file_name().and_then(|n| n.to_str()) != Some(path) {
                    return Err(io::ErrorKind::NotFound.into());
                }
                if std::fs::metadata(&*file)?.len() > limit {
                    return Err(too_big());
                }
                read_limited(File::open(&*file)?, limit)
            }
            Tree::Zip(archive, index) => {
                let ix = *index.get(path).ok_or(io::ErrorKind::NotFound)?;
                let entry = archive.by_index(ix).map_err(io::Error::other)?;
                if entry.size() > limit {
                    return Err(too_big());
                }
                read_limited(entry, limit)
            }
            Tree::Many(trees, owner) => {
                let ix = *owner.get(path).ok_or(io::ErrorKind::NotFound)?;
                trees[ix].read(path, limit)
            }
        }
    }

    /// Copies `path` to the file `to`, streaming.
    pub fn copy_to(&mut self, path: &str, to: &Path) -> io::Result<()> {
        let mut out = File::create(to)?;
        match self {
            Tree::Zip(archive, index) => {
                let ix = *index.get(path).ok_or(io::ErrorKind::NotFound)?;
                let mut entry = archive.by_index(ix).map_err(io::Error::other)?;
                io::copy(&mut entry, &mut out)?;
            }
            Tree::Dir(root) => {
                io::copy(&mut File::open(root.join(path))?, &mut out)?;
            }
            Tree::Single(file) => {
                io::copy(&mut File::open(&*file)?, &mut out)?;
            }
            Tree::Many(trees, owner) => {
                let ix = *owner.get(path).ok_or(io::ErrorKind::NotFound)?;
                drop(out);
                return trees[ix].copy_to(path, to);
            }
        }
        Ok(())
    }

    /// The text of `path`: UTF-8 (invalid bytes replaced), without a byte
    /// order mark, with `\r\n` turned into `\n`.
    pub fn read_text(&mut self, path: &str, limit: u64) -> io::Result<String> {
        Ok(text_of(&self.read(path, limit)?))
    }
}

/// Bytes as text: UTF-8 (invalid bytes replaced), without a byte order
/// mark, with `\r\n` and lone `\r` turned into `\n`.
pub fn text_of(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    let text = text.strip_prefix('\u{feff}').unwrap_or(&text);
    if text.contains('\r') {
        text.replace("\r\n", "\n").replace('\r', "\n")
    } else {
        text.to_owned()
    }
}

fn read_limited(reader: impl io::Read, limit: u64) -> io::Result<Vec<u8>> {
    let mut data = Vec::new();
    reader
        .take(limit.saturating_add(1))
        .read_to_end(&mut data)?;
    if data.len() as u64 > limit {
        return Err(io::Error::other(format!("larger than {limit} bytes")));
    }
    Ok(data)
}

fn walk(dir: &Path, prefix: &str, depth: usize, out: &mut Vec<TreeFile>) -> anyhow::Result<()> {
    if depth > MAX_DEPTH {
        return Ok(());
    }
    let entries =
        std::fs::read_dir(dir).with_context(|| format!("cannot list {}", dir.display()))?;
    for entry in entries {
        let entry = entry?;
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        if name.starts_with('.') {
            continue;
        }
        // `file_type` doesn't follow symbolic links, so a link out of the
        // folder (or a loop) is never entered.
        let kind = entry.file_type()?;
        let path = if prefix.is_empty() {
            name
        } else {
            format!("{prefix}/{name}")
        };
        if kind.is_dir() {
            walk(&entry.path(), &path, depth + 1, out)?;
        } else if kind.is_file() {
            out.push(tree_file(path, &entry.metadata()?));
        }
    }
    Ok(())
}

fn tree_file(path: String, meta: &std::fs::Metadata) -> TreeFile {
    let ms = |time: io::Result<std::time::SystemTime>| {
        time.ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .and_then(|d| i64::try_from(d.as_millis()).ok())
    };
    TreeFile {
        path,
        size: meta.len(),
        modified: ms(meta.modified()),
        created: ms(meta.created()),
    }
}

/// A zip entry's name as a tree path: `/`-separated, no `.`/`..`
/// components or leading `/`, and no hidden files (`__MACOSX/`, `.DS_Store`).
/// `None` for names that can't be one.
pub fn clean_zip_path(name: &str) -> Option<String> {
    let mut parts = Vec::new();
    for part in name.split(['/', '\\']) {
        match part {
            "" | "." => {}
            ".." => return None,
            _ if part.starts_with('.') || part == "__MACOSX" => return None,
            _ => parts.push(part),
        }
    }
    (!parts.is_empty()).then(|| parts.join("/"))
}

fn zip_time(time: zip::DateTime) -> Option<i64> {
    let date = chrono::NaiveDate::from_ymd_opt(
        i32::from(time.year()),
        u32::from(time.month()),
        u32::from(time.day()),
    )?;
    let at = date.and_hms_opt(
        u32::from(time.hour()),
        u32::from(time.minute()),
        u32::from(time.second()),
    )?;
    Some(at.and_utc().timestamp_millis())
}

/// The folder part of a tree path: `a/b/c.md` → `a/b`, `c.md` → ``.
pub fn parent_of(path: &str) -> &str {
    path.rsplit_once('/').map_or("", |(dir, _)| dir)
}

/// The last component of a tree path.
pub fn file_name(path: &str) -> &str {
    path.rsplit_once('/').map_or(path, |(_, name)| name)
}

/// A file name without its last extension: `Plan.md` → `Plan`,
/// `.hidden` → `.hidden`.
pub fn stem(name: &str) -> &str {
    match name.rsplit_once('.') {
        Some((stem, _)) if !stem.is_empty() => stem,
        _ => name,
    }
}

/// The lowercased last extension of a file name, without the dot.
pub fn extension(name: &str) -> String {
    match file_name(name).rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() => ext.to_ascii_lowercase(),
        _ => String::new(),
    }
}

/// `base` joined with a relative `link` (`..` and `.` resolved), or `None`
/// when the link climbs out of the tree.
pub fn join(base: &str, link: &str) -> Option<String> {
    let mut parts: Vec<&str> = if link.starts_with('/') {
        Vec::new()
    } else {
        base.split('/').filter(|p| !p.is_empty()).collect()
    };
    for part in link.split(['/', '\\']) {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            _ => parts.push(part),
        }
    }
    (!parts.is_empty()).then(|| parts.join("/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joins_relative_links() {
        assert_eq!(join("a/b", "c.png").as_deref(), Some("a/b/c.png"));
        assert_eq!(join("a/b", "../c.png").as_deref(), Some("a/c.png"));
        assert_eq!(join("a/b", "./x/../c.png").as_deref(), Some("a/b/c.png"));
        assert_eq!(join("a", "/top.png").as_deref(), Some("top.png"));
        assert_eq!(join("", "../escape.png"), None);
    }

    #[test]
    fn cleans_zip_paths() {
        assert_eq!(clean_zip_path("a/./b.md").as_deref(), Some("a/b.md"));
        assert_eq!(clean_zip_path("/a\\b.md").as_deref(), Some("a/b.md"));
        assert_eq!(clean_zip_path("../evil.md"), None);
        assert_eq!(clean_zip_path("__MACOSX/a.md"), None);
        assert_eq!(clean_zip_path("a/.DS_Store"), None);
    }

    #[test]
    fn names() {
        assert_eq!(stem("Plan.md"), "Plan");
        assert_eq!(stem("a.b.md"), "a.b");
        assert_eq!(stem(".hidden"), ".hidden");
        assert_eq!(extension("x/Pic.PNG"), "png");
        assert_eq!(extension("README"), "");
        assert_eq!(parent_of("a/b/c.md"), "a/b");
        assert_eq!(file_name("a/b/c.md"), "c.md");
    }

    #[test]
    fn text_drops_bom_and_carriage_returns() {
        assert_eq!(text_of(b"\xef\xbb\xbfa\r\nb\rc"), "a\nb\nc");
    }
}
