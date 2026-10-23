//! A note's properties: the YAML front matter at the top of its text.
//!
//! Only the YAML notes use is understood: `key: value` lines, quoted
//! strings, numbers, booleans, ISO dates, inline lists (`[a, b]`) and block
//! lists (`- a`). Anything else, such as a nested map or an anchor, is kept
//! as the text it was written as; parsing never fails.
//!
//! [`set_property`] changes one value and leaves every other byte alone,
//! adding front matter when the note has none.

use crate::dates::Date;

/// A property's value.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Text(String),
    Number(f64),
    Bool(bool),
    /// `YYYY-MM-DD`.
    Date(String),
    List(Vec<Value>),
}

impl Value {
    /// The value as a line of text: lists comma-separated, dates as written.
    pub fn text(&self) -> String {
        match self {
            Value::Text(text) => text.clone(),
            Value::Number(number) => number_text(*number),
            Value::Bool(on) => on.to_string(),
            Value::Date(date) => date.clone(),
            Value::List(items) => items
                .iter()
                .map(|item| item.text())
                .collect::<Vec<_>>()
                .join(", "),
        }
    }

    /// Whether the value says nothing: no text, no items.
    pub fn is_empty(&self) -> bool {
        match self {
            Value::Text(text) => text.is_empty(),
            Value::List(items) => items.is_empty(),
            _ => false,
        }
    }
}

/// A note's text split at its front matter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrontMatter<'a> {
    /// Between the opening and closing lines.
    pub front: &'a str,
    /// Where `front` starts in the note.
    pub front_start: usize,
    /// Where the closing `---` or `...` line starts.
    pub close_start: usize,
    /// Where the text after the closing line starts.
    pub body_start: usize,
}

/// The front matter of `body`, if it opens with `---` and closes with
/// `---` or `...`.
pub fn split(body: &str) -> Option<FrontMatter<'_>> {
    let opening = body
        .strip_prefix("---\n")
        .map(|_| 4)
        .or_else(|| body.strip_prefix("---\r\n").map(|_| 5))?;
    let mut offset = opening;
    for line in body[opening..].split_inclusive('\n') {
        let end = offset + line.len();
        let content = line.trim_end_matches(['\n', '\r']);
        if content == "---" || content == "..." {
            return Some(FrontMatter {
                front: &body[opening..offset],
                front_start: opening,
                close_start: offset,
                body_start: end,
            });
        }
        offset = end;
    }
    None
}

/// The properties in `body`'s front matter, in the order they are written.
pub fn parse(body: &str) -> Vec<(String, Value)> {
    let Some(front) = split(body) else {
        return Vec::new();
    };
    entries(front.front)
        .into_iter()
        .map(|entry| (entry.key, entry.value))
        .collect()
}

/// The value of property `key`, ignoring case; the last one written wins.
pub fn get<'a>(properties: &'a [(String, Value)], key: &str) -> Option<&'a Value> {
    let key = key.trim();
    properties
        .iter()
        .rev()
        .find(|(name, _)| name.eq_ignore_ascii_case(key))
        .map(|(_, value)| value)
}

/// `body` with property `key` set to `value`. The front matter is added if
/// the note has none; every other byte stays as it was.
pub fn set_property(body: &str, key: &str, value: &Value) -> String {
    let key = key.trim();
    if key.is_empty() {
        return body.to_owned();
    }
    let line = format!("{}: {}\n", quote_key(key), render(value));
    match split(body) {
        None => format!("---\n{line}---\n{body}"),
        Some(front) => {
            let found = entries(front.front)
                .into_iter()
                .find(|entry| entry.key.eq_ignore_ascii_case(key));
            match found {
                // The rest of the entry (a block list or a comment) goes:
                // the new value is the whole of it.
                Some(entry) => {
                    let start = front.front_start + entry.start;
                    let end = front.front_start + entry.end;
                    format!("{}{}{}", &body[..start], line, &body[end..])
                }
                None => {
                    let at = front.close_start;
                    format!("{}{}{}", &body[..at], line, &body[at..])
                }
            }
        }
    }
}

/// The value a typed field should set, when the property it edits held
/// `like`: a number stays a number when it parses, a date stays a date, a
/// list is written comma-separated, and a new property takes the type its
/// text looks like.
pub fn typed(text: &str, like: Option<&Value>) -> Value {
    let text = text.trim();
    match like {
        Some(Value::Number(_)) => text
            .parse::<f64>()
            .ok()
            .filter(|n| n.is_finite())
            .map(Value::Number)
            .unwrap_or_else(|| Value::Text(text.to_owned())),
        Some(Value::Date(_)) => Date::parse_iso(text)
            .map(|date| Value::Date(date.iso()))
            .unwrap_or_else(|| Value::Text(text.to_owned())),
        Some(Value::Bool(on)) => match text.to_ascii_lowercase().as_str() {
            "true" => Value::Bool(true),
            "false" => Value::Bool(false),
            _ => Value::Bool(*on),
        },
        Some(Value::List(_)) => Value::List(
            split_flow(text)
                .into_iter()
                .map(|item| scalar(&item))
                .collect(),
        ),
        Some(Value::Text(_)) => Value::Text(text.to_owned()),
        None => scalar(text),
    }
}

struct Entry {
    key: String,
    value: Value,
    /// Where the entry starts and ends in the front matter.
    start: usize,
    end: usize,
}

/// The top-level entries of front matter, with where each one is.
fn entries(front: &str) -> Vec<Entry> {
    struct Raw {
        key: String,
        start: usize,
        lines: Vec<(usize, String)>,
    }
    let mut raw: Vec<Raw> = Vec::new();
    let mut offset = 0;
    for line in front.split_inclusive('\n') {
        let content = line.trim_end_matches(['\n', '\r']);
        let key_line = !content.starts_with([' ', '\t', '-', '#'])
            && !content.is_empty()
            && split_key(content).is_some();
        if key_line {
            let key = split_key(content).map(|(key, _)| key).unwrap_or_default();
            raw.push(Raw {
                key,
                start: offset,
                lines: vec![(offset, line.to_owned())],
            });
        } else if let Some(entry) = raw.last_mut() {
            entry.lines.push((offset, line.to_owned()));
        }
        offset += line.len();
    }
    raw.into_iter()
        .map(|entry| {
            let (value, _) = value_of(&entry.lines);
            let last = entry
                .lines
                .last()
                .map(|(at, line)| at + line.len())
                .unwrap_or(entry.start);
            Entry {
                key: entry.key,
                value,
                start: entry.start,
                end: last,
            }
        })
        .collect()
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

/// The value of an entry, and its text as written after the key.
fn value_of(lines: &[(usize, String)]) -> (Value, String) {
    let first = lines[0].1.trim_end_matches(['\n', '\r']);
    let Some((_, rest)) = split_key(first) else {
        return (Value::Text(String::new()), String::new());
    };
    let trimmed = rest.trim();
    let more: Vec<&str> = lines[1..]
        .iter()
        .map(|(_, line)| line.trim_end_matches(['\n', '\r']))
        .filter(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'))
        .collect();
    if trimmed.starts_with('|') || trimmed.starts_with('>') {
        let folded = trimmed.starts_with('>');
        let text: Vec<&str> = lines[1..]
            .iter()
            .map(|(_, line)| line.trim_end_matches(['\n', '\r']).trim())
            .collect();
        let text = if folded {
            text.iter()
                .filter(|line| !line.is_empty())
                .copied()
                .collect::<Vec<_>>()
                .join(" ")
        } else {
            text.join("\n").trim_end().to_owned()
        };
        return (Value::Text(text), rest.to_owned());
    }
    if trimmed.is_empty() {
        if more.is_empty() {
            return (Value::Text(String::new()), rest.to_owned());
        }
        if more.iter().all(|line| line.trim_start().starts_with('-')) {
            let items = more
                .iter()
                .map(|line| {
                    let item = line.trim_start().strip_prefix('-').unwrap_or("");
                    scalar(item.trim())
                })
                .collect();
            return (Value::List(items), rest.to_owned());
        }
        // A nested map, or anything else not understood: its lines as
        // written.
        let mut raw = String::new();
        for (_, line) in &lines[1..] {
            raw.push_str(line);
        }
        return (Value::Text(raw), rest.to_owned());
    }
    if !more.is_empty() {
        // A plain scalar continued on the next lines, or something we
        // don't interpret, kept as written.
        if trimmed.starts_with(['"', '\'', '[', '{', '&', '*']) {
            let mut raw = rest.trim_start().to_owned();
            for (_, line) in &lines[1..] {
                raw.push_str(line);
            }
            return (Value::Text(raw), rest.to_owned());
        }
        let mut text = trimmed.to_owned();
        for line in more {
            text.push(' ');
            text.push_str(line.trim());
        }
        return (
            Value::Text(strip_comment(&text).to_owned()),
            rest.to_owned(),
        );
    }
    if let Some(inner) = trimmed.strip_prefix('[').and_then(|r| r.strip_suffix(']')) {
        let items = split_flow(inner)
            .into_iter()
            .map(|item| scalar(&item))
            .collect();
        return (Value::List(items), rest.to_owned());
    }
    if trimmed.starts_with('{') || trimmed.starts_with(['&', '*']) {
        return (Value::Text(rest.trim_start().to_owned()), rest.to_owned());
    }
    (scalar(trimmed), rest.to_owned())
}

/// A scalar as YAML reads it: a quoted or plain string, a number, a boolean
/// or an ISO date.
fn scalar(text: &str) -> Value {
    let text = text.trim();
    if let Some(quoted) = unquote_exact(text) {
        return Value::Text(quoted);
    }
    let text = strip_comment(text);
    if text.is_empty() {
        return Value::Text(String::new());
    }
    if text.eq_ignore_ascii_case("true") {
        return Value::Bool(true);
    }
    if text.eq_ignore_ascii_case("false") {
        return Value::Bool(false);
    }
    if let Some(date) = Date::parse_iso(text) {
        return Value::Date(date.iso());
    }
    if let Ok(number) = text.parse::<i64>() {
        return Value::Number(number as f64);
    }
    if let Ok(number) = text.parse::<f64>()
        && number.is_finite()
    {
        return Value::Number(number);
    }
    Value::Text(text.to_owned())
}

/// A quoted scalar `text` with nothing after it.
fn unquote_exact(text: &str) -> Option<String> {
    let mut chars = text.chars();
    let quote = chars.next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let mut out = String::new();
    while let Some(c) = chars.next() {
        match (quote, c) {
            ('"', '\\') => match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some(other) => out.push(other),
                None => out.push('\\'),
            },
            ('\'', '\'') if chars.as_str().starts_with('\'') => {
                chars.next();
                out.push('\'');
            }
            (_, c) if c == quote => {
                return chars.as_str().is_empty().then_some(out);
            }
            _ => out.push(c),
        }
    }
    None
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

/// A value as YAML that [`parse`] reads back as itself.
fn render(value: &Value) -> String {
    match value {
        Value::Text(text) => quote(text),
        Value::Number(number) => number_text(*number),
        Value::Bool(on) => on.to_string(),
        Value::Date(date) => date.clone(),
        Value::List(items) => format!(
            "[{}]",
            items.iter().map(render).collect::<Vec<_>>().join(", ")
        ),
    }
}

fn number_text(number: f64) -> String {
    if number.fract() == 0. && number.abs() < 9e15 {
        format!("{}", number as i64)
    } else {
        format!("{number}")
    }
}

/// `text` as a plain scalar when that reads back as itself, quoted when not.
fn quote(text: &str) -> String {
    let plain = !text.is_empty()
        && text.trim() == text
        && !text.starts_with([
            '"', '\'', '[', '{', '|', '>', '-', '#', '&', '*', '!', '%', '@', '`',
        ])
        && !text.contains(": ")
        && !text.contains(" #")
        && !text.contains('\n')
        && !text.ends_with(':')
        && matches!(scalar(text), Value::Text(ref kept) if kept == text);
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

fn quote_key(key: &str) -> String {
    let plain = !key.is_empty()
        && key.trim() == key
        && !key.starts_with(['"', '\'', '#', '&', '*', '!', '%', '@', '`'])
        && !key.contains(['\n', '\r', ':'])
        && !key.contains(" #");
    if plain {
        key.to_owned()
    } else {
        let escaped = key.replace('\\', "\\\\").replace('"', "\\\"");
        format!("\"{escaped}\"")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    #[test]
    fn reads_the_yaml_notes_use() {
        let body = "---
client: Acme
rating: 4
done: true
due: 2026-11-01
people: [Ana, Ben]
empty:
quoted: \"Plan: v2\"
tags:
  - work
  - '#project/acme'
nested:
  a: 1
---
# Note
";
        let properties = parse(body);
        assert_eq!(
            properties,
            vec![
                ("client".to_owned(), Value::Text("Acme".into())),
                ("rating".to_owned(), Value::Number(4.)),
                ("done".to_owned(), Value::Bool(true)),
                ("due".to_owned(), Value::Date("2026-11-01".into())),
                (
                    "people".to_owned(),
                    Value::List(vec![Value::Text("Ana".into()), Value::Text("Ben".into()),])
                ),
                ("empty".to_owned(), Value::Text(String::new())),
                ("quoted".to_owned(), Value::Text("Plan: v2".into())),
                (
                    "tags".to_owned(),
                    Value::List(vec![
                        Value::Text("work".into()),
                        Value::Text("#project/acme".into()),
                    ])
                ),
                ("nested".to_owned(), Value::Text("  a: 1\n".into())),
            ]
        );
    }

    #[test]
    fn closing_with_three_dots_and_crlf() {
        assert_eq!(
            parse("---\r\na: 1\r\n...\r\nBody"),
            vec![("a".to_owned(), Value::Number(1.))]
        );
        assert!(parse("---\nno close").is_empty());
        assert!(parse("no front matter").is_empty());
    }

    #[test]
    fn set_changes_one_value_and_nothing_else() {
        let body = "---\nclient: Acme  # the one\nrating: 4\n---\n# Note\n\nBody.\n";
        let changed = set_property(body, "client", &Value::Text("Globex".into()));
        assert_eq!(
            changed,
            "---\nclient: Globex\nrating: 4\n---\n# Note\n\nBody.\n"
        );
        let added = set_property(&changed, "due", &Value::Date("2026-11-01".into()));
        assert_eq!(
            added,
            "---\nclient: Globex\nrating: 4\ndue: 2026-11-01\n---\n# Note\n\nBody.\n"
        );
    }

    #[test]
    fn set_adds_front_matter_when_there_is_none() {
        assert_eq!(
            set_property("# Note\n", "done", &Value::Bool(true)),
            "---\ndone: true\n---\n# Note\n"
        );
        assert_eq!(
            set_property("---\nno close", "done", &Value::Bool(true)),
            "---\ndone: true\n---\n---\nno close"
        );
    }

    #[test]
    fn text_that_looks_like_another_type_is_quoted() {
        for (text, value) in [
            ("4", Value::Text("4".into())),
            ("true", Value::Text("true".into())),
            ("2026-11-01", Value::Text("2026-11-01".into())),
            ("[a]", Value::Text("[a]".into())),
            ("", Value::Text(String::new())),
        ] {
            let body = set_property("", "key", &value);
            assert_eq!(parse(&body), vec![("key".to_owned(), value)], "{text}");
        }
    }

    #[test]
    fn typed_keeps_a_property_its_type() {
        assert_eq!(
            typed("5", Some(&Value::Text("4".into()))),
            Value::Text("5".into())
        );
        assert_eq!(typed("5", Some(&Value::Number(4.))), Value::Number(5.));
        assert_eq!(
            typed("a, b", Some(&Value::List(Vec::new()))),
            Value::List(vec![Value::Text("a".into()), Value::Text("b".into())])
        );
        assert_eq!(typed("4", None), Value::Number(4.));
        assert_eq!(typed("2026-11-01", None), Value::Date("2026-11-01".into()));
    }

    fn property_text() -> impl Strategy<Value = String> {
        "[A-Za-z0-9 _:/.-]{0,20}"
    }

    fn property_value() -> impl Strategy<Value = Value> {
        prop_oneof![
            property_text().prop_map(Value::Text),
            (-10000i64..10000).prop_map(|n| Value::Number(n as f64)),
            any::<bool>().prop_map(Value::Bool),
            (2000i32..2100, 1u32..13, 1u32..29)
                .prop_map(|(y, m, d)| { Value::Date(format!("{y:04}-{m:02}-{d:02}")) }),
            prop::collection::vec(property_text().prop_map(Value::Text), 0..5)
                .prop_map(Value::List),
        ]
    }

    /// A front matter of keys unique ignoring case (a later `o` replaces an
    /// `O`), and a body after it.
    fn document() -> impl Strategy<Value = (Vec<(String, Value)>, String)> {
        (
            prop::collection::btree_map("[a-z][a-z0-9_ ]{0,12}", property_value(), 0..8),
            "[^\\u{0}]{0,60}",
        )
            .prop_map(|(entries, body)| (entries.into_iter().collect(), body))
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        })]

        /// Round trip: writing a property and reading it back gives the
        /// same value.
        #[test]
        fn prop_set_then_parse_round_trips(
            (entries, body) in document(),
            key in "[a-zA-Z][a-zA-Z0-9_ ]{0,12}",
            value in property_value(),
        ) {
            let mut text = body;
            for (name, value) in &entries {
                text = set_property(&text, name, value);
            }
            let before = parse(&text);
            for (name, value) in &entries {
                prop_assert_eq!(get(&before, name), Some(value), "{}", name);
            }
            let text = set_property(&text, &key, &value);
            let after = parse(&text);
            prop_assert_eq!(get(&after, &key), Some(&value));
            // Every other property is unchanged.
            for (name, kept) in &entries {
                if !name.eq_ignore_ascii_case(&key) {
                    prop_assert_eq!(get(&after, name), Some(kept), "{}", name);
                }
            }
        }

        /// Setting a property never changes the text after the front matter.
        #[test]
        fn prop_set_keeps_the_body(
            front in "([a-z][a-z0-9]{0,6}: [A-Za-z0-9 .]{0,10}\n){0,5}",
            body in "[^\\u{0}]{0,80}",
            key in "[a-z][a-z0-9]{0,8}",
            value in property_value(),
        ) {
            let text = format!("---\n{front}---\n{body}");
            let changed = set_property(&text, &key, &value);
            let before = split(&text).map(|f| &text[f.body_start..]).unwrap_or("");
            let after = split(&changed).map(|f| &changed[f.body_start..]).unwrap_or("");
            prop_assert_eq!(before, after);
        }

        /// Any text parses without panicking, and so does setting in it.
        #[test]
        fn prop_any_text_parses(text in "[\\s\\S]{0,200}") {
            let _ = parse(&text);
            let _ = set_property(&text, "key", &Value::Text("v".into()));
        }
    }
}
