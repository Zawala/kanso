//! Links in a note's Markdown as other apps write them, found so they can be
//! pointed at what the import made: wiki-links and embeds (`[[Note]]`,
//! `![[picture.png|300]]`), inline links and images (`[text](Other.md)`,
//! `![](assets/pic.png)`), reference definitions (`[1]: pic.png`) and HTML
//! images (`<img src="pic.png">`). Code is never searched.

use std::ops::Range;

use kanso_core::links::{code_ranges, parse_target};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// `[[target]]`; the range covers the brackets.
    Wiki,
    /// `![[target]]`; the range covers the `!` and the brackets.
    Embed,
    /// `[text](target)`; the range covers the destination only.
    Link,
    /// `![alt](target)`; the range covers the destination only.
    Image,
    /// `[label]: target`; the range covers the destination only.
    Definition,
    /// `<img src="target">`; the range covers the attribute's value.
    HtmlImage,
}

/// A link found in a body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Found {
    pub kind: Kind,
    /// The bytes a rewrite replaces (see [`Kind`]).
    pub range: Range<usize>,
    /// What the link points at, as written: a wiki-link's title, or a
    /// destination without its `<…>`.
    pub target: String,
    /// A wiki-link's heading (after `#`).
    pub heading: Option<String>,
    /// A wiki-link's alias (after `|`), or a Markdown link's text.
    pub label: Option<String>,
}

/// The links in `body`, in order of their ranges.
pub fn scan(body: &str) -> Vec<Found> {
    if !body.contains('[') && !body.contains("<img") && !body.contains("<IMG") {
        return Vec::new();
    }
    let code = code_ranges(body);
    let in_code = |at: usize| {
        let ix = code.partition_point(|r| r.end <= at);
        code.get(ix).is_some_and(|r| r.start <= at)
    };
    let mut found = Vec::new();
    let bytes = body.as_bytes();
    let mut wiki_end = 0;
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'[' && !in_code(i) {
            if bytes.get(i + 1) == Some(&b'[') && bytes.get(i + 2) != Some(&b'[') {
                if let Some(link) = wiki_at(body, i) {
                    wiki_end = link.range.end;
                    i = link.range.end;
                    found.push(link);
                    continue;
                }
            } else if i >= wiki_end
                && let Some(inline) = inline_at(body, i)
            {
                // Markdown does not allow a link inside another link, but
                // images may sit inside a link's label. Scan the label for
                // images only, then skip the whole link so brackets in its
                // destination or text cannot produce overlapping candidates.
                let offset = inline.label.start;
                for mut nested in scan(&body[inline.label.clone()]) {
                    if matches!(nested.kind, Kind::Image | Kind::HtmlImage) {
                        nested.range = nested.range.start + offset..nested.range.end + offset;
                        found.push(nested);
                    }
                }
                found.push(inline.found);
                i = inline.end;
                continue;
            } else if i >= wiki_end
                && let Some((link, end)) = definition_at(body, i)
            {
                found.push(link);
                i = end;
                continue;
            }
        } else if b == b'<'
            && !in_code(i)
            && let Some((link, end)) = html_image_at(body, i)
        {
            found.push(link);
            i = end;
            continue;
        }
        i += 1;
    }
    found.sort_by_key(|f| f.range.start);
    found
}

fn wiki_at(body: &str, start: usize) -> Option<Found> {
    let rest = &body[start + 2..];
    let close = rest.find("]]")?;
    let inner = &rest[..close];
    let (title, heading, alias) = parse_target(inner)?;
    let embed = start > 0 && body.as_bytes()[start - 1] == b'!';
    let end = start + 2 + close + 2;
    Some(Found {
        kind: if embed { Kind::Embed } else { Kind::Wiki },
        range: if embed { start - 1..end } else { start..end },
        target: title,
        heading,
        label: alias,
    })
}

/// The end of the bracketed text starting at `open` (the index after its
/// `]`), allowing nested brackets and backslash escapes, on one paragraph.
fn close_bracket(body: &str, open: usize) -> Option<usize> {
    let bytes = body.as_bytes();
    let mut depth = 0usize;
    let mut i = open;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 1,
            b'[' => depth += 1,
            b']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i + 1);
                }
            }
            b'\n' if bytes.get(i + 1) == Some(&b'\n') => return None,
            _ => {}
        }
        i += 1;
    }
    None
}

struct Inline {
    found: Found,
    label: Range<usize>,
    end: usize,
}

fn inline_at(body: &str, open: usize) -> Option<Inline> {
    let bytes = body.as_bytes();
    let close = close_bracket(body, open)?;
    if bytes.get(close) != Some(&b'(') {
        return None;
    }
    let image = open > 0 && bytes[open - 1] == b'!';
    let text = &body[open + 1..close - 1];
    let (range, target, end) = destination(body, close + 1, b')')?;
    Some(Inline {
        found: Found {
            kind: if image { Kind::Image } else { Kind::Link },
            range,
            target,
            heading: None,
            label: Some(text.to_owned()),
        },
        label: open + 1..close - 1,
        end,
    })
}

/// Reads a destination starting at `from` (after the `(`), up to an
/// optional title and the `close` byte. Returns the destination's range
/// (inside any `<…>`), its text, and where scanning can resume after the
/// whole link or definition.
fn destination(body: &str, from: usize, close: u8) -> Option<(Range<usize>, String, usize)> {
    let bytes = body.as_bytes();
    let mut i = from;
    while bytes.get(i).is_some_and(|&b| b == b' ' || b == b'\t') {
        i += 1;
    }
    let (start, destination_end) = if bytes.get(i) == Some(&b'<') {
        let start = i + 1;
        let len = body[start..].find(['>', '\n'])?;
        if bytes[start + len] != b'>' {
            return None;
        }
        i = start + len + 1;
        (start, start + len)
    } else {
        let start = i;
        let mut depth = 0usize;
        while let Some(&b) = bytes.get(i) {
            match b {
                b'\\' => i += 1,
                b'(' => depth += 1,
                b')' if depth == 0 => break,
                b')' => depth -= 1,
                b'\n' => break,
                // Apps write spaces in paths (`[a](My Note.md)`); a space
                // ends the destination only before a title or the end.
                b' ' | b'\t' if close != b')' => break,
                b' ' | b'\t' => {
                    let next = body[i..].trim_start_matches([' ', '\t']);
                    if next.starts_with(['"', '\'', ')', '\n']) || next.is_empty() {
                        break;
                    }
                    if next.starts_with('(') && depth == 0 {
                        // `(title)`, unless the path goes on after it.
                        let after = next.find(')').map(|at| next[at + 1..].trim_start());
                        if after.is_some_and(|a| a.starts_with(')')) {
                            break;
                        }
                    }
                }
                _ => {}
            }
            i += 1;
        }
        (start, i.min(bytes.len()))
    };
    // An optional title, then the closing byte.
    while bytes.get(i).is_some_and(|&b| b == b' ' || b == b'\t') {
        i += 1;
    }
    if close == b')' {
        if let Some(&q) = bytes.get(i)
            && (q == b'"' || q == b'\'' || q == b'(')
        {
            let end_q = if q == b'(' { b')' } else { q };
            let len = body[i + 1..].find(end_q as char)?;
            i += 1 + len + 1;
            while bytes.get(i).is_some_and(|&b| b == b' ' || b == b'\t') {
                i += 1;
            }
        }
        if bytes.get(i) != Some(&b')') {
            return None;
        }
    }
    if start == destination_end {
        return None;
    }
    let resume = if close == b')' { i + 1 } else { i };
    Some((
        start..destination_end,
        body[start..destination_end].to_owned(),
        resume,
    ))
}

/// `[label]: destination` at the start of a line (up to three spaces in).
fn definition_at(body: &str, open: usize) -> Option<(Found, usize)> {
    let line_start = body[..open].rfind('\n').map_or(0, |at| at + 1);
    let indent = &body[line_start..open];
    if indent.len() > 3 || !indent.bytes().all(|b| b == b' ') {
        return None;
    }
    let close = close_bracket(body, open)?;
    if body.as_bytes().get(close) != Some(&b':') || close == open + 2 {
        return None;
    }
    let label = &body[open + 1..close - 1];
    if label.starts_with('^') {
        // A footnote, not a link.
        return None;
    }
    let (range, target, end) = destination(body, close + 1, b'\n')?;
    Some((
        Found {
            kind: Kind::Definition,
            range,
            target,
            heading: None,
            label: Some(label.to_owned()),
        },
        end,
    ))
}

/// `<img … src="…" …>`: the `src` value.
fn html_image_at(body: &str, open: usize) -> Option<(Found, usize)> {
    let rest = &body[open..];
    if !rest.get(..4)?.eq_ignore_ascii_case("<img") {
        return None;
    }
    let end = rest.find('>')?;
    let tag = &rest[..end];
    let lower = tag.to_ascii_lowercase();
    let at = lower.find("src=")? + 4;
    let quote = *tag.as_bytes().get(at)?;
    let (start, len) = if quote == b'"' || quote == b'\'' {
        let len = tag[at + 1..].find(quote as char)?;
        (at + 1, len)
    } else {
        let len = tag[at..].find([' ', '/', '>']).unwrap_or(tag.len() - at);
        (at, len)
    };
    if len == 0 {
        return None;
    }
    let range = open + start..open + start + len;
    Some((
        Found {
            kind: Kind::HtmlImage,
            target: body[range.clone()].to_owned(),
            range,
            heading: None,
            label: None,
        },
        open + end + 1,
    ))
}

/// `body` with each `(range, text)` replacing those bytes. Edits must not
/// overlap; overlapping ones after the first are dropped.
pub fn apply(body: &str, mut edits: Vec<(Range<usize>, String)>) -> String {
    edits.sort_by_key(|(range, _)| range.start);
    let mut out = String::with_capacity(body.len());
    let mut at = 0;
    for (range, text) in edits {
        if range.start < at || range.end > body.len() {
            continue;
        }
        out.push_str(&body[at..range.start]);
        out.push_str(&text);
        at = range.end;
    }
    out.push_str(&body[at..]);
    out
}

/// A link destination as a path: `%20` decoded, without `#fragment` or
/// `?query`, and `None` for URLs with a scheme (`https:`, `mailto:`) or
/// links within the page (`#heading`).
pub fn as_relative_path(target: &str) -> Option<String> {
    let target = target.trim();
    if target.is_empty() || target.starts_with('#') || target.starts_with("//") {
        return None;
    }
    if let Some(colon) = target.find(':') {
        let scheme = &target[..colon];
        // A Windows drive letter isn't a scheme, but it isn't relative either.
        if !scheme.is_empty()
            && scheme
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'+' | b'-' | b'.'))
        {
            return None;
        }
    }
    let path = target.split(['#', '?']).next().unwrap_or("");
    if path.is_empty() {
        return None;
    }
    let decoded = percent_encoding::percent_decode_str(path)
        .decode_utf8()
        .map(|s| s.into_owned())
        .unwrap_or_else(|_| path.to_owned());
    Some(decoded.replace('\\', "/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn targets(body: &str) -> Vec<(Kind, String)> {
        scan(body).into_iter().map(|f| (f.kind, f.target)).collect()
    }

    #[test]
    fn finds_each_kind() {
        let body = "See [[Plan#Goals|the plan]] and ![[pic.png|300]].\n\
                    A [link](Other%20Note.md) and ![alt](<assets/a b.png> \"title\").\n\
                    [ref]: docs/spec.pdf\n\
                    <img src=\"x/y.jpg\" width=200>\n";
        assert_eq!(
            targets(body),
            [
                (Kind::Wiki, "Plan".to_owned()),
                (Kind::Embed, "pic.png".to_owned()),
                (Kind::Link, "Other%20Note.md".to_owned()),
                (Kind::Image, "assets/a b.png".to_owned()),
                (Kind::Definition, "docs/spec.pdf".to_owned()),
                (Kind::HtmlImage, "x/y.jpg".to_owned()),
            ]
        );
        let wiki = &scan(body)[0];
        assert_eq!(wiki.heading.as_deref(), Some("Goals"));
        assert_eq!(wiki.label.as_deref(), Some("the plan"));
        assert_eq!(&body[scan(body)[1].range.clone()], "![[pic.png|300]]");
    }

    #[test]
    fn skips_code() {
        assert!(scan("`[[A]]` and\n```\n[x](y.md)\n```\n").is_empty());
        // Two backticks open a span that only two more close.
        assert!(scan(" ``  [[Aa]]  `` ").is_empty());
    }

    #[test]
    fn nested_image_in_link() {
        assert_eq!(
            targets("[![badge](b.svg)](https://x.com)"),
            [
                (Kind::Image, "b.svg".to_owned()),
                (Kind::Link, "https://x.com".to_owned()),
            ]
        );
    }

    #[test]
    fn malformed_nested_links_do_not_overlap() {
        let body = "[a[](](> )";
        let found = scan(body);
        for pair in found.windows(2) {
            assert!(pair[0].range.end <= pair[1].range.start);
        }
        let edits = found
            .iter()
            .map(|f| (f.range.clone(), body[f.range.clone()].to_owned()))
            .collect();
        assert_eq!(apply(body, edits), body);
    }

    #[test]
    fn parentheses_in_destinations() {
        assert_eq!(
            targets("[a](Note (draft).md)"),
            [(Kind::Link, "Note (draft).md".to_owned())]
        );
    }

    #[test]
    fn footnotes_are_not_definitions() {
        assert!(scan("[^1]: a footnote").is_empty());
    }

    #[test]
    fn relative_paths() {
        assert_eq!(as_relative_path("a%20b.md#h").as_deref(), Some("a b.md"));
        assert_eq!(as_relative_path("https://x.com/a.md"), None);
        assert_eq!(as_relative_path("mailto:me@x.com"), None);
        assert_eq!(as_relative_path("#top"), None);
        assert_eq!(as_relative_path("../up.png").as_deref(), Some("../up.png"));
    }

    #[test]
    fn applies_edits() {
        let body = "[a](x.md) [b](y.md)";
        let found = scan(body);
        let edits = found
            .iter()
            .map(|f| (f.range.clone(), format!("<{}>", f.target)))
            .collect();
        assert_eq!(apply(body, edits), "[a](<x.md>) [b](<y.md>)");
    }
}
