//! YAML front matter, as far as notes use it: `key: value` lines with plain
//! or quoted scalars, `[inline, lists]`, block lists (`- item`), and `|`/`>`
//! block scalars. Anything else (nested maps, anchors) is kept as written
//! and not interpreted.
//!
//! Kanso keeps a note's title, tags and dates in fields of their own, so
//! those keys leave the front matter on import; every other line stays in
//! the body exactly as it was.

/// A front-matter value as far as it is understood.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Scalar(String),
    List(Vec<String>),
    /// A nested map or anything else: not interpreted.
    Other,
}

/// One top-level key and the lines it spans.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub key: String,
    pub value: Value,
    /// The entry's lines as written, each ending in `\n`.
    pub raw: String,
}

/// A body split at its front matter: the text between the `---` lines,
/// and what follows the closing line. `None` without front matter.
pub fn split(text: &str) -> Option<(&str, &str)> {
    let rest = text
        .strip_prefix("---\n")
        .or_else(|| text.strip_prefix("---\r\n"))?;
    let mut offset = 0;
    for line in rest.split_inclusive('\n') {
        let content = line.trim_end_matches(['\n', '\r']);
        if content == "---" || content == "..." {
            let body = &rest[offset + line.len()..];
            return Some((&rest[..offset], body));
        }
        offset += line.len();
    }
    None
}

/// The entries of front matter (the text between the `---` lines). Lines
/// before the first key (comments, blank lines) belong to no entry and
/// are dropped from the result; lines after an entry belong to it.
pub fn parse(front: &str) -> Vec<Entry> {
    let mut entries: Vec<(String, Vec<&str>)> = Vec::new();
    let mut leading: Vec<&str> = Vec::new();
    for line in front.split_inclusive('\n') {
        let content = line.trim_end_matches(['\n', '\r']);
        let starts_key = !content.starts_with([' ', '\t', '-', '#'])
            && !content.is_empty()
            && key_of(content).is_some();
        if starts_key {
            let key = key_of(content).unwrap_or_default();
            entries.push((key, vec![line]));
        } else if let Some((_, lines)) = entries.last_mut() {
            lines.push(line);
        } else {
            leading.push(line);
        }
    }
    entries
        .into_iter()
        .map(|(key, lines)| {
            let mut raw: String = lines.concat();
            if !raw.ends_with('\n') {
                raw.push('\n');
            }
            Entry {
                value: value_of(&lines),
                key,
                raw,
            }
        })
        .collect()
}

/// The key of a `key: value` line, unquoted.
fn key_of(line: &str) -> Option<String> {
    split_key(line).map(|(key, _)| key)
}

/// `key: rest` → (key, rest). The colon must be followed by a space or the
/// end of the line; quoted keys are unquoted.
fn split_key(line: &str) -> Option<(String, &str)> {
    let (key, rest) = if let Some(quoted) = line.strip_prefix('"') {
        let end = quoted.find('"')?;
        let rest = quoted[end + 1..].strip_prefix(':')?;
        (quoted[..end].to_owned(), rest)
    } else if let Some(quoted) = line.strip_prefix('\'') {
        let end = quoted.find('\'')?;
        let rest = quoted[end + 1..].strip_prefix(':')?;
        (quoted[..end].to_owned(), rest)
    } else {
        let mut at = None;
        for (ix, c) in line.char_indices() {
            if c == ':' {
                let next = line[ix + 1..].chars().next();
                if next.is_none_or(|n| n == ' ' || n == '\t') {
                    at = Some(ix);
                    break;
                }
            }
        }
        let at = at?;
        (line[..at].trim_end().to_owned(), &line[at + 1..])
    };
    if key.is_empty() || !(rest.is_empty() || rest.starts_with([' ', '\t'])) {
        return None;
    }
    Some((key, rest))
}

fn value_of(lines: &[&str]) -> Value {
    let first = lines[0].trim_end_matches(['\n', '\r']);
    let Some((_, rest)) = split_key(first) else {
        return Value::Other;
    };
    let rest = strip_comment(rest.trim());
    let more: Vec<&str> = lines[1..]
        .iter()
        .map(|l| l.trim_end_matches(['\n', '\r']))
        .filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'))
        .collect();
    if rest.starts_with('|') || rest.starts_with('>') {
        let folded = rest.starts_with('>');
        let text: Vec<&str> = lines[1..]
            .iter()
            .map(|l| l.trim_end_matches(['\n', '\r']).trim())
            .collect();
        let text = if folded {
            text.iter()
                .filter(|l| !l.is_empty())
                .copied()
                .collect::<Vec<_>>()
                .join(" ")
        } else {
            text.join("\n").trim_end().to_owned()
        };
        return Value::Scalar(text);
    }
    if rest.is_empty() {
        if more.is_empty() {
            return Value::Scalar(String::new());
        }
        if more.iter().all(|l| l.trim_start().starts_with('-')) {
            return Value::List(
                more.iter()
                    .map(|l| {
                        let item = l.trim_start().strip_prefix('-').unwrap_or("");
                        unquote(strip_comment(item.trim()))
                    })
                    .collect(),
            );
        }
        return Value::Other;
    }
    if !more.is_empty() {
        // A plain scalar continued on the next lines.
        if rest.starts_with(['"', '\'', '[', '{']) {
            return Value::Other;
        }
        let mut text = rest.to_owned();
        for line in more {
            text.push(' ');
            text.push_str(line.trim());
        }
        return Value::Scalar(text);
    }
    if let Some(inner) = rest.strip_prefix('[').and_then(|r| r.strip_suffix(']')) {
        return Value::List(split_flow(inner).into_iter().map(|s| unquote(&s)).collect());
    }
    if rest.starts_with('{') {
        return Value::Other;
    }
    Value::Scalar(unquote(rest))
}

/// A plain scalar without a trailing ` # comment`. Quoted text keeps its `#`.
fn strip_comment(text: &str) -> &str {
    if text.starts_with(['"', '\'']) {
        return text;
    }
    if text.starts_with('#') {
        return "";
    }
    match text.find(" #").or_else(|| text.find("\t#")) {
        Some(at) => text[..at].trim_end(),
        None => text,
    }
}

/// Splits `a, "b, c", d` at the commas outside quotes.
fn split_flow(inner: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    let mut chars = inner.chars().peekable();
    while let Some(c) = chars.next() {
        match quote {
            Some(q) if c == q => {
                if q == '\'' && chars.peek() == Some(&'\'') {
                    current.push('\'');
                    current.push(chars.next().unwrap_or('\''));
                    continue;
                }
                quote = None;
                current.push(c);
            }
            Some('"') if c == '\\' => {
                current.push(c);
                if let Some(next) = chars.next() {
                    current.push(next);
                }
            }
            Some(_) => current.push(c),
            None if c == '"' || c == '\'' => {
                quote = Some(c);
                current.push(c);
            }
            None if c == ',' => {
                items.push(current.trim().to_owned());
                current.clear();
            }
            None => current.push(c),
        }
    }
    if !current.trim().is_empty() || !items.is_empty() {
        items.push(current.trim().to_owned());
    }
    items.retain(|item| !item.is_empty());
    items
}

/// A scalar without its quotes: `"a \"b\""` → `a "b"`, `'it''s'` → `it's`.
pub fn unquote(text: &str) -> String {
    let text = text.trim();
    if text.len() >= 2 && text.starts_with('"') && text.ends_with('"') {
        let inner = &text[1..text.len() - 1];
        let mut out = String::with_capacity(inner.len());
        let mut chars = inner.chars();
        while let Some(c) = chars.next() {
            if c != '\\' {
                out.push(c);
                continue;
            }
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some(other) => out.push(other),
                None => out.push('\\'),
            }
        }
        out
    } else if text.len() >= 2 && text.starts_with('\'') && text.ends_with('\'') {
        text[1..text.len() - 1].replace("''", "'")
    } else {
        text.to_owned()
    }
}

/// Front-matter text for `entries` as [`parse`] reads them back. Used by
/// tests and to rebuild what remains of a note's front matter.
pub fn render(entries: &[Entry]) -> String {
    entries.iter().map(|e| e.raw.as_str()).collect()
}

/// Quotes `text` for a scalar when it would not read back as itself.
pub fn quote(text: &str) -> String {
    let plain = !text.is_empty()
        && text.trim() == text
        && !text.starts_with([
            '"', '\'', '[', '{', '|', '>', '-', '#', '&', '*', '!', '%', '@',
        ])
        && !text.contains(": ")
        && !text.contains(" #")
        && !text.contains('\n')
        && !text.ends_with(':');
    if plain {
        text.to_owned()
    } else {
        let escaped = text
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n");
        format!("\"{escaped}\"")
    }
}

/// What Kanso takes from a note's front matter.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Meta {
    pub title: Option<String>,
    pub tags: Vec<String>,
    pub aliases: Vec<String>,
    pub created: Option<String>,
    pub updated: Option<String>,
    /// The front matter that remains: every entry Kanso has no field for,
    /// as written. Empty when nothing remains.
    pub rest: String,
}

const TITLE_KEYS: &[&str] = &["title"];
const TAG_KEYS: &[&str] = &["tags", "tag"];
const ALIAS_KEYS: &[&str] = &["aliases", "alias"];
const CREATED_KEYS: &[&str] = &[
    "created",
    "date",
    "created_at",
    "createdat",
    "creation date",
    "date created",
    "creation_date",
];
const UPDATED_KEYS: &[&str] = &[
    "updated",
    "modified",
    "updated_at",
    "updatedat",
    "last_modified",
    "last modified",
    "lastmod",
    "date modified",
    "modification date",
];

/// Reads the keys Kanso uses out of front matter.
pub fn meta(front: &str) -> Meta {
    let mut meta = Meta::default();
    let mut rest = Vec::new();
    for entry in parse(front) {
        let key = entry.key.to_lowercase();
        let key = key.as_str();
        let used = if TITLE_KEYS.contains(&key) {
            match &entry.value {
                Value::Scalar(title) if !title.trim().is_empty() => {
                    meta.title = Some(title.trim().to_owned());
                    true
                }
                _ => false,
            }
        } else if TAG_KEYS.contains(&key) {
            match &entry.value {
                Value::Scalar(text) => {
                    meta.tags.extend(split_tags(text));
                    true
                }
                Value::List(items) => {
                    meta.tags.extend(items.iter().flat_map(|i| split_tags(i)));
                    true
                }
                Value::Other => false,
            }
        } else if ALIAS_KEYS.contains(&key) {
            match &entry.value {
                Value::Scalar(text) if !text.trim().is_empty() => {
                    meta.aliases.extend(
                        text.split(',')
                            .map(str::trim)
                            .filter(|a| !a.is_empty())
                            .map(str::to_owned),
                    );
                }
                Value::List(items) => meta.aliases.extend(
                    items
                        .iter()
                        .map(|a| a.trim().to_owned())
                        .filter(|a| !a.is_empty()),
                ),
                _ => {}
            }
            // Kanso has no aliases of its own: they stay in the front matter.
            false
        } else if CREATED_KEYS.contains(&key) || UPDATED_KEYS.contains(&key) {
            let slot = if CREATED_KEYS.contains(&key) {
                &mut meta.created
            } else {
                &mut meta.updated
            };
            match &entry.value {
                Value::Scalar(text)
                    if !text.trim().is_empty() && crate::library::dates::parse(text).is_some() =>
                {
                    if slot.is_none() {
                        *slot = Some(text.trim().to_owned());
                    }
                    true
                }
                _ => false,
            }
        } else {
            false
        };
        if !used {
            rest.push(entry);
        }
    }
    meta.rest = render(&rest);
    meta
}

/// Tag names in a scalar: separated by commas or whitespace, without a
/// leading `#`.
pub fn split_tags(text: &str) -> Vec<String> {
    text.split(|c: char| c == ',' || c.is_whitespace())
        .map(|t| t.trim().trim_start_matches('#').trim_end_matches('/'))
        .filter(|t| !t.is_empty())
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_front_matter() {
        assert_eq!(
            split("---\ntitle: A\n---\nBody"),
            Some(("title: A\n", "Body"))
        );
        assert_eq!(split("---\n---\n"), Some(("", "")));
        assert_eq!(split("No front matter"), None);
        assert_eq!(split("---\nunterminated"), None);
    }

    #[test]
    fn reads_obsidian_front_matter() {
        let front = "title: \"Plan: v2\"\ntags:\n  - work\n  - '#project/acme'\naliases: [Roadmap, \"The plan\"]\ncreated: 2024-03-01\ncssclass: wide\nnested:\n  a: 1\n";
        let meta = meta(front);
        assert_eq!(meta.title.as_deref(), Some("Plan: v2"));
        assert_eq!(meta.tags, ["work", "project/acme"]);
        assert_eq!(meta.aliases, ["Roadmap", "The plan"]);
        assert_eq!(meta.created.as_deref(), Some("2024-03-01"));
        assert_eq!(
            meta.rest,
            "aliases: [Roadmap, \"The plan\"]\ncssclass: wide\nnested:\n  a: 1\n"
        );
    }

    #[test]
    fn tags_as_a_scalar() {
        assert_eq!(meta("tags: a, b c #d\n").tags, ["a", "b", "c"]);
        assert_eq!(meta("tags: \"#x, y\"\n").tags, ["x", "y"]);
    }

    #[test]
    fn block_scalars_and_comments() {
        assert_eq!(
            parse("title: >\n  A long\n  title\n")[0].value,
            Value::Scalar("A long title".into())
        );
        assert_eq!(
            parse("note: |\n  one\n  two\n")[0].value,
            Value::Scalar("one\ntwo".into())
        );
        assert_eq!(
            parse("a: b # comment\n")[0].value,
            Value::Scalar("b".into())
        );
        assert_eq!(
            parse("url: http://x.com/a#b\n")[0].value,
            Value::Scalar("http://x.com/a#b".into())
        );
    }

    #[test]
    fn an_unparseable_date_stays() {
        let meta = meta("date: someday\n");
        assert_eq!(meta.created, None);
        assert_eq!(meta.rest, "date: someday\n");
    }
}
