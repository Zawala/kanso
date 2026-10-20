//! The 17 tools and the note resource, implemented on the store, and
//! Kanso's own `kanso-…` tools after them.
//!
//! Each tool does what `@inkdropapp/mcp-server` 1.5.0 does with the HTTP API,
//! through the same operations the REST handlers use, and formats its result
//! the same way: documents as `JSON.stringify(value, null, 2)`, failed
//! requests as `Fetch error [<status>] <reason>`.

use std::path::Path;

use base64::Engine as _;
use base64::engine::DecodePaddingMode;
use base64::engine::general_purpose::{GeneralPurpose, GeneralPurposeConfig, STANDARD};
use kanso_core::id::with_prefix;
use kanso_core::model::{MAX_ATTACHMENT_SIZE, SUPPORTED_IMAGE_TYPES};
use kanso_core::{DocType, now_ms};
use kanso_store::{Attachment, Store};
use serde_json::{Map, Value, json};

use super::RpcError;
use super::schema;
use crate::AppState;
use crate::api::{self, NoteQuery};
use crate::error::ApiError;

type Args = Map<String, Value>;

/// A tool failure, reported as an `isError` result with this text.
struct ToolError(String);

impl From<ApiError> for ToolError {
    fn from(err: ApiError) -> Self {
        ToolError(err.fetch_error_text())
    }
}

type ToolResult = Result<Value, ToolError>;

const SOURCE_TEMPLATE_DESCRIPTION: &str = "The template this note was created from. Read it to get the fill-out instructions and examples before completing this note.";

/// Runs `tools/call`.
pub(crate) fn call(state: &AppState, name: &str, arguments: Option<&Value>) -> Value {
    let Some(tool) = schema::tool(name) else {
        return error_result(format!("MCP error -32602: Tool {name} not found"));
    };
    let args = match tool.validate(arguments) {
        Ok(args) => args,
        Err(issues) => {
            return error_result(format!(
                "MCP error -32602: Input validation error: Invalid arguments for tool {name}: {issues}"
            ));
        }
    };
    let store = &*state.store;
    let result = match name {
        "read-note" => read_note(store, &args),
        "read-backlinks" => read_backlinks(store, &args),
        "search-notes" => search_notes(store, &args),
        "list-notes" => list_notes(store, &args),
        "create-note" => upsert(store, DocType::Note, args),
        "update-note" => update_note(store, args),
        "patch-note" => patch_note(store, &args),
        "list-tags" => list(store, DocType::Tag),
        "read-tag" => read(store, DocType::Tag, &args, "tagId"),
        "create-tag" => upsert(store, DocType::Tag, args),
        "update-tag" => merge(store, args, false),
        "list-notebooks" => list(store, DocType::Book),
        "read-book" => read(store, DocType::Book, &args, "bookId"),
        "create-book" => upsert(store, DocType::Book, args),
        "update-book" => merge(store, args, true),
        "create-file" => create_file(store, state.local_files, &args),
        "read-file" => read_file(store, state.local_files, &args),
        "kanso-create-daily-note" => create_daily_note(store, &args),
        "kanso-create-from-template" => create_from_template(store, &args),
        "kanso-export-html" => export_html(store, &args),
        "kanso-get-outline" => get_outline(store, &args),
        "kanso-list-tasks" => list_tasks(store, &args),
        "kanso-resolve-wikilink" => resolve_wikilink(store, &args),
        _ => Err(ToolError(format!(
            "MCP error -32602: Tool {name} not found"
        ))),
    };
    result.unwrap_or_else(|ToolError(message)| error_result(message))
}

/// Now on this computer's clock, in its time zone.
fn local_now() -> kanso_core::dates::DateTime {
    let local = chrono::Local::now()
        .naive_local()
        .and_utc()
        .timestamp_millis();
    kanso_core::dates::DateTime::from_local_ms(local)
}

/// What a note made or opened by a tool is, for its answer.
fn made_note(store: &Store, id: &str, created: Option<bool>) -> ToolResult {
    let note = api::get_doc(store, id, None, false)?;
    let mut answer = Map::new();
    answer.insert("noteId".into(), json!(id));
    answer.insert(
        "title".into(),
        note.get("title").cloned().unwrap_or_default(),
    );
    answer.insert(
        "bookId".into(),
        note.get("bookId").cloned().unwrap_or_default(),
    );
    answer.insert("uri".into(), json!(note_uri(id)));
    if let Some(created) = created {
        answer.insert("created".into(), json!(created));
    }
    result(vec![text(pretty(&Value::Object(answer)))])
}

/// `kanso-create-daily-note`: the day's note in the Journal, made from the
/// daily template when missing.
fn create_daily_note(store: &Store, args: &Args) -> ToolResult {
    let mut now = local_now();
    if let Some(date) = opt_str(args, "date") {
        now.date = kanso_core::dates::Date::parse_iso(date.trim())
            .ok_or_else(|| ToolError(format!("`{date}` is not a date: write it as YYYY-MM-DD.")))?;
    }
    let daily = store
        .open_daily(now, kanso_store::Origin::Api)
        .map_err(|err| ToolError(err.to_string()))?;
    made_note(store, &daily.id, Some(daily.created))
}

/// `kanso-create-from-template`: a note from a template, its variables
/// filled in.
fn create_from_template(store: &Store, args: &Args) -> ToolResult {
    let template = with_prefix(DocType::Note, str_arg(args, "templateId"));
    // The same failure as reading a note that isn't there.
    api::get_doc(store, &template, None, false)?;
    let book = opt_str(args, "bookId").map(|id| with_prefix(DocType::Book, id));
    let made = store
        .create_from_template(
            &template,
            Some(str_arg(args, "title")),
            book.as_deref(),
            local_now(),
            kanso_store::Origin::Api,
        )
        .map_err(|err| ToolError(err.to_string()))?;
    made_note(store, &made.id, None)
}

/// `kanso-export-html`: the note as a standalone page, in Kanso's light or
/// dark theme.
fn export_html(store: &Store, args: &Args) -> ToolResult {
    let id = with_prefix(DocType::Note, str_arg(args, "noteId"));
    let note = api::get_doc(store, &id, None, false)?;
    let palette = match opt_str(args, "theme") {
        Some("dark") => kanso_render::Palette::DARK,
        _ => kanso_render::Palette::LIGHT,
    };
    let attachments = |id: &str| {
        store
            .attachment(id)
            .ok()
            .flatten()
            .map(|(content_type, attachment)| (content_type, attachment.data))
    };
    let html = kanso_render::html::page(
        &js_string(note.get("title")),
        &js_string(note.get("body")),
        &palette,
        &attachments,
    );
    result(vec![text(html)])
}

/// Runs `resources/read` for `inkdrop://note/{noteId}`.
pub(crate) fn read_resource(state: &AppState, uri: &str) -> Result<Value, RpcError> {
    let note_id = uri
        .get(..10)
        .filter(|scheme| scheme.eq_ignore_ascii_case("inkdrop://"))
        .and_then(|_| uri[10..].strip_prefix("note/"))
        .filter(|id| !id.is_empty() && !id.contains(['/', ',']))
        .ok_or_else(|| {
            RpcError::invalid_params(format!("MCP error -32602: Resource {uri} not found"))
        })?;
    let id = with_prefix(DocType::Note, note_id);
    let note = api::get_doc(&state.store, &id, None, false)
        .map_err(|err| RpcError::internal(err.fetch_error_text()))?;
    Ok(json!({
        "contents": [{ "uri": uri, "text": pretty(&note), "mimeType": "application/json" }]
    }))
}

fn text(text: impl Into<String>) -> Value {
    json!({ "type": "text", "text": text.into() })
}

fn result(blocks: Vec<Value>) -> ToolResult {
    Ok(json!({ "content": blocks }))
}

fn error_result(message: impl Into<String>) -> Value {
    json!({ "content": [text(message)], "isError": true })
}

/// `JSON.stringify(value, null, 2)`.
fn pretty(value: &Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_default()
}

fn str_arg<'a>(args: &'a Args, key: &str) -> &'a str {
    opt_str(args, key).unwrap_or_default()
}

fn opt_str<'a>(args: &'a Args, key: &str) -> Option<&'a str> {
    args.get(key).and_then(Value::as_str)
}

fn bool_arg(args: &Args, key: &str) -> bool {
    args.get(key).and_then(Value::as_bool).unwrap_or(false)
}

/// `inkdrop://note/<id without the note: prefix>`.
fn note_uri(id: &str) -> String {
    format!("inkdrop://note/{}", id.strip_prefix("note:").unwrap_or(id))
}

/// `inkdrop://file:<id>`.
fn file_uri(id: &str) -> String {
    format!("inkdrop://{}", with_prefix(DocType::File, id))
}

/// JavaScript truthiness.
fn js_truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|x| x != 0.0),
        Value::String(s) => !s.is_empty(),
        Value::Array(_) | Value::Object(_) => true,
    }
}

/// JavaScript's `String(value)`, with `None` as `undefined`.
fn js_string(value: Option<&Value>) -> String {
    match value {
        None => "undefined".into(),
        Some(Value::Null) => "null".into(),
        Some(Value::String(s)) => s.clone(),
        Some(Value::Bool(b)) => b.to_string(),
        Some(Value::Number(n)) => n.to_string(),
        Some(Value::Array(items)) => items
            .iter()
            .map(|item| match item {
                Value::Null => String::new(),
                item => js_string(Some(item)),
            })
            .collect::<Vec<_>>()
            .join(","),
        Some(Value::Object(_)) => "[object Object]".into(),
    }
}

/// JavaScript's `s.substring(0, n)`, counting UTF-16 code units. A surrogate
/// pair cut in half is dropped rather than split.
fn js_prefix(s: &str, n: usize) -> &str {
    let mut units = 0;
    for (i, c) in s.char_indices() {
        units += c.len_utf16();
        if units > n {
            return &s[..i];
        }
    }
    s
}

/// JavaScript's `String.prototype.trim`.
fn js_trim(s: &str) -> &str {
    s.trim_matches(|c: char| c.is_whitespace() || c == '\u{feff}')
}

/// A note as `search-notes` and `list-notes` show it: body cut to 200 characters.
fn summary(mut note: Value) -> Value {
    if let Some(Value::String(body)) = note.get_mut("body") {
        let cut = js_prefix(body, 200).len();
        body.truncate(cut);
    }
    note
}

fn read_note(store: &Store, args: &Args) -> ToolResult {
    let id = with_prefix(DocType::Note, str_arg(args, "noteId"));
    let note = api::get_doc(store, &id, None, false)?;
    let source = note
        .get("sourceTemplateId")
        .filter(|v| js_truthy(v))
        .map(|v| js_string(Some(v)));
    let template = match &source {
        // A missing template (e.g. deleted) only skips the embed.
        Some(source) if bool_arg(args, "includeTemplate") => {
            api::get_doc(store, source, None, false).ok().map(|t| {
                format!(
                    "# Source template: {}\n\n{}",
                    js_string(t.get("title")),
                    js_string(t.get("body"))
                )
            })
        }
        _ => None,
    };
    let mut blocks = vec![text(pretty(&note))];
    if let Some(source) = &source {
        blocks.push(json!({
            "type": "resource_link",
            "uri": note_uri(source),
            "name": "Source template",
            "mimeType": "application/json",
            "description": SOURCE_TEMPLATE_DESCRIPTION,
        }));
    }
    if let Some(template) = template {
        blocks.push(text(template));
    }
    result(blocks)
}

fn read_backlinks(store: &Store, args: &Args) -> ToolResult {
    let link = note_uri(&with_prefix(DocType::Note, str_arg(args, "noteId")));
    let query = NoteQuery {
        keyword: link.clone(),
        limit: Some(100),
        ..NoteQuery::default()
    };
    let notes = api::list_notes(store, &query)?;
    if notes.is_empty() {
        return result(vec![text(format!("No backlinks found for {link}."))]);
    }
    result(
        notes
            .iter()
            .map(|note| {
                let name = note
                    .get("title")
                    .filter(|t| js_truthy(t))
                    .map_or_else(|| "Untitled note".to_owned(), |t| js_string(Some(t)));
                json!({
                    "type": "resource_link",
                    "uri": note_uri(&js_string(note.get("_id"))),
                    "name": name,
                    "mimeType": "application/json",
                })
            })
            .collect(),
    )
}

/// `kanso-get-outline`: the note's headings as JSON.
fn get_outline(store: &Store, args: &Args) -> ToolResult {
    let id = with_prefix(DocType::Note, str_arg(args, "noteId"));
    // The same failure as reading a note that isn't there.
    api::get_doc(store, &id, None, false)?;
    let headings = store
        .outline(&id)
        .map_err(|err| ToolError(err.to_string()))?;
    if headings.is_empty() {
        return result(vec![text(format!("No headings in {}.", note_uri(&id)))]);
    }
    let headings = serde_json::to_value(headings).unwrap_or_default();
    result(vec![text(pretty(&headings))])
}

/// `kanso-list-tasks`: tasks across notes, with their notes.
fn list_tasks(store: &Store, args: &Args) -> ToolResult {
    let status = str_arg(args, "status");
    let limit = args.get("limit").and_then(Value::as_f64).unwrap_or(100.0);
    let query = kanso_store::TaskQuery {
        book_id: opt_str(args, "bookId").map(|id| with_prefix(DocType::Book, id)),
        include_done: status != "open",
        due_before: opt_str(args, "dueBefore").map(str::to_owned),
        due_after: opt_str(args, "dueAfter").map(str::to_owned),
        // Done ones are left out below, after the limit would cut.
        limit: (status != "done")
            .then(|| api::count_param("limit", limit))
            .transpose()?,
    };
    let limit = api::count_param("limit", limit)?;
    let tasks: Vec<Value> = store
        .tasks(&query)
        .map_err(|err| ToolError(err.to_string()))?
        .into_iter()
        .filter(|t| status != "done" || t.checked)
        .take(limit)
        .map(|t| {
            json!({
                "text": t.text,
                "done": t.checked,
                "due": t.due,
                "line": t.line,
                "noteId": t.note_id,
                "noteTitle": t.note_title,
                "bookId": t.book_id,
                "uri": note_uri(&t.note_id),
            })
        })
        .collect();
    if tasks.is_empty() {
        return result(vec![text("No tasks match.")]);
    }
    result(vec![text(pretty(&Value::Array(tasks)))])
}

/// `kanso-resolve-wikilink`: the note a wiki-link names.
fn resolve_wikilink(store: &Store, args: &Args) -> ToolResult {
    let link = str_arg(args, "link").trim();
    let inner = link
        .strip_prefix("[[")
        .and_then(|rest| rest.strip_suffix("]]"))
        .unwrap_or(link);
    let Some((title, heading, alias)) = kanso_core::links::parse_target(inner) else {
        return Err(ToolError(format!(
            "`{link}` is not a wiki-link: it needs a title, without brackets or line breaks inside."
        )));
    };
    let found = store
        .resolve_title(&title)
        .map_err(|err| ToolError(err.to_string()))?;
    let Some((id, note_title)) = found else {
        return Err(ToolError(format!("No note is titled \"{title}\".")));
    };
    let mut answer = Map::new();
    answer.insert("noteId".into(), json!(id));
    answer.insert("title".into(), json!(note_title));
    answer.insert("uri".into(), json!(note_uri(&id)));
    if let Some(alias) = alias {
        answer.insert("alias".into(), json!(alias));
    }
    if let Some(heading) = heading {
        let outline = store
            .outline(&id)
            .map_err(|err| ToolError(err.to_string()))?;
        let found = kanso_core::outline::find_heading(&outline, &heading);
        answer.insert(
            "heading".into(),
            match found {
                Some(h) => json!({ "text": h.text, "level": h.level, "line": h.line }),
                None => json!({ "text": heading, "found": false }),
            },
        );
    }
    result(vec![text(pretty(&Value::Object(answer)))])
}

fn summaries(notes: Vec<Value>) -> ToolResult {
    let notes = Value::Array(notes.into_iter().map(summary).collect());
    result(vec![text(pretty(&notes))])
}

fn search_notes(store: &Store, args: &Args) -> ToolResult {
    let query = NoteQuery {
        keyword: str_arg(args, "keyword").to_owned(),
        limit: Some(10),
        ..NoteQuery::default()
    };
    summaries(api::list_notes(store, &query)?)
}

/// The keyword `list-notes` searches for: `bookId:<id> tagId:<tag>… <keyword>`.
pub(crate) fn list_notes_keyword(
    book_id: Option<&str>,
    tag_ids: &[Value],
    keyword: Option<&str>,
) -> String {
    let book_filter = match book_id {
        Some(id) if !id.is_empty() => {
            format!("bookId:{}", id.split(':').nth(1).unwrap_or("undefined"))
        }
        _ => String::new(),
    };
    let tag_filter = tag_ids
        .iter()
        .map(|tag| format!("tagId:{}", js_string(Some(tag))))
        .collect::<Vec<_>>()
        .join(" ");
    js_trim(&format!(
        "{book_filter} {tag_filter} {}",
        keyword.unwrap_or_default()
    ))
    .to_owned()
}

fn list_notes(store: &Store, args: &Args) -> ToolResult {
    let tag_ids = args
        .get("tagIds")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    let limit = args.get("limit").and_then(Value::as_f64).unwrap_or(100.0);
    let query = NoteQuery {
        keyword: list_notes_keyword(opt_str(args, "bookId"), tag_ids, opt_str(args, "keyword")),
        limit: Some(api::count_param("limit", limit)?),
        skip: 0,
        sort: api::parse_sort(str_arg(args, "sort")),
        descending: args
            .get("descending")
            .and_then(Value::as_bool)
            .unwrap_or(true),
    };
    summaries(api::list_notes(store, &query)?)
}

fn upsert(store: &Store, doc_type: DocType, args: Args) -> ToolResult {
    let res = api::upsert(store, doc_type, Value::Object(args))?;
    result(vec![text(pretty(&res))])
}

/// `update-tag` and `update-book`: post the given fields to `/<_id>`.
fn merge(store: &Store, mut args: Args, stamp: bool) -> ToolResult {
    let id = js_string(args.remove("_id").as_ref());
    if stamp {
        args.insert("updatedAt".into(), now_ms().into());
    }
    let res = api::merge(store, &id, Value::Object(args))?;
    result(vec![text(pretty(&res))])
}

fn update_note(store: &Store, args: Args) -> ToolResult {
    let existing = api::get_doc(
        store,
        str_arg(&args, "_id"),
        Some(str_arg(&args, "_rev")),
        false,
    )?;
    let mut note = match existing {
        Value::Object(note) => note,
        _ => Map::new(),
    };
    note.extend(args);
    note.insert("updatedAt".into(), now_ms().into());
    upsert_note(store, note)
}

fn upsert_note(store: &Store, note: Args) -> ToolResult {
    let res = api::upsert(store, DocType::Note, Value::Object(note))?;
    result(vec![text(pretty(&res))])
}

/// Where `old` occurs in `body`, when it occurs exactly once. Overlapping
/// occurrences count, as with JavaScript's `indexOf(old, first + 1)`.
pub(crate) enum Occurrence {
    None,
    Once(usize),
    Many,
}

pub(crate) fn find_once(body: &str, old: &str) -> Occurrence {
    let Some(first) = body.find(old) else {
        return Occurrence::None;
    };
    let next = first + body[first..].chars().next().map_or(1, char::len_utf8);
    if body[next..].contains(old) {
        Occurrence::Many
    } else {
        Occurrence::Once(first)
    }
}

fn patch_note(store: &Store, args: &Args) -> ToolResult {
    let existing = api::get_doc(
        store,
        str_arg(args, "_id"),
        Some(str_arg(args, "_rev")),
        false,
    )?;
    let mut note = match existing {
        Value::Object(note) => note,
        _ => Map::new(),
    };
    let body = note.get("body").and_then(Value::as_str).unwrap_or_default();
    let old = str_arg(args, "old_string");
    let new = str_arg(args, "new_string");
    let patched = match find_once(body, old) {
        Occurrence::None => {
            return Err(ToolError(
                "Failed to patch: `old_string` was not found in the note body.".into(),
            ));
        }
        Occurrence::Many => {
            return Err(ToolError(
                "Failed to patch: `old_string` matches multiple locations in the note body. Provide more surrounding context to make it unique.".into(),
            ));
        }
        Occurrence::Once(at) => format!("{}{new}{}", &body[..at], &body[at + old.len()..]),
    };
    note.insert("body".into(), patched.into());
    note.insert("updatedAt".into(), now_ms().into());
    upsert_note(store, note)
}

fn list(store: &Store, doc_type: DocType) -> ToolResult {
    let docs = Value::Array(api::list_docs(store, doc_type, None, 0)?);
    result(vec![text(pretty(&docs))])
}

fn read(store: &Store, doc_type: DocType, args: &Args, key: &str) -> ToolResult {
    let id = with_prefix(doc_type, str_arg(args, key));
    let doc = api::get_doc(store, &id, None, false)?;
    result(vec![text(pretty(&doc))])
}

/// Node's `Buffer.from(data, 'base64')`: accepts both alphabets, padding or
/// not, and skips characters outside them.
fn decode_base64_leniently(data: &str) -> Vec<u8> {
    const LENIENT: GeneralPurpose = GeneralPurpose::new(
        &base64::alphabet::STANDARD,
        GeneralPurposeConfig::new()
            .with_decode_padding_mode(DecodePaddingMode::Indifferent)
            .with_decode_allow_trailing_bits(true),
    );
    let mut clean: String = data
        .split('=')
        .next()
        .unwrap_or_default()
        .chars()
        .filter_map(|c| match c {
            '-' => Some('+'),
            '_' => Some('/'),
            c if c.is_ascii_alphanumeric() || c == '+' || c == '/' => Some(c),
            _ => None,
        })
        .collect();
    if clean.len() % 4 == 1 {
        clean.pop();
    }
    LENIENT.decode(clean).unwrap_or_default()
}

/// The image type for a path's extension, as `inferImageContentType` does.
fn infer_image_type(path: &str) -> Option<&'static str> {
    let ext = Path::new(path).extension()?.to_str()?.to_lowercase();
    match ext.as_str() {
        "png" => Some("image/png"),
        "jpeg" | "jpg" => Some("image/jpeg"),
        "svg" | "svg+xml" => Some("image/svg+xml"),
        "gif" => Some("image/gif"),
        "heic" => Some("image/heic"),
        "heif" => Some("image/heif"),
        _ => None,
    }
}

const NO_LOCAL_FILES: &str =
    "local file access is only available when the server is bound to a loopback address";

fn create_file(store: &Store, local_files: bool, args: &Args) -> ToolResult {
    let file_path = opt_str(args, "filePath");
    let data = opt_str(args, "data");
    if file_path.is_some() == data.is_some() {
        return Err(ToolError(
            "Failed to create the attachment: provide exactly one of `filePath` or `data`.".into(),
        ));
    }
    let mut name = opt_str(args, "name").map(str::to_owned);
    let mut content_type = opt_str(args, "contentType").map(str::to_owned);
    let bytes = match (file_path, data) {
        (Some(path), _) => {
            if !local_files {
                return Err(ToolError(format!(
                    "Failed to read the file at {path}: {NO_LOCAL_FILES}"
                )));
            }
            let bytes = std::fs::read(path)
                .map_err(|err| ToolError(format!("Failed to read the file at {path}: {err}")))?;
            content_type = content_type.or_else(|| infer_image_type(path).map(str::to_owned));
            name = name.or_else(|| {
                Path::new(path)
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
            });
            bytes
        }
        (None, Some(data)) => decode_base64_leniently(data),
        (None, None) => unreachable!("checked above"),
    };
    let Some(content_type) = content_type else {
        let from = file_path.map(|p| format!(" from {p}")).unwrap_or_default();
        return Err(ToolError(format!(
            "Failed to create the attachment: could not infer the content type{from}. Pass `contentType` explicitly. Supported types: {}.",
            SUPPORTED_IMAGE_TYPES.join(", ")
        )));
    };
    let encoded = STANDARD.encode(&bytes);
    let content_length = encoded.len() * 3 / 4;
    if content_length > MAX_ATTACHMENT_SIZE {
        return Err(ToolError(format!(
            "Failed to create the attachment: the file is {content_length} bytes, over the {MAX_ATTACHMENT_SIZE} byte limit."
        )));
    }
    let name = name.unwrap_or_else(|| {
        format!(
            "image.{}",
            content_type.strip_prefix("image/").unwrap_or(&content_type)
        )
    });
    let body = json!({
        "name": name,
        "contentType": content_type,
        "contentLength": content_length,
        "publicIn": [],
        "md5digest": Attachment::new(bytes).md5_hex(),
        "_attachments": { "index": { "content_type": content_type, "data": encoded } },
    });
    let res = api::upsert(store, DocType::File, body)?;
    let embed = format!(
        "Markdown to embed this attachment in a note body:\n\n![{name}]({})",
        file_uri(&js_string(res.get("id")))
    );
    result(vec![text(pretty(&res)), text(embed)])
}

/// Image types an MCP `image` block can carry.
fn is_viewable_image(content_type: &str) -> bool {
    matches!(
        content_type,
        "image/png" | "image/jpeg" | "image/jpg" | "image/gif"
    )
}

fn read_file(store: &Store, local_files: bool, args: &Args) -> ToolResult {
    let id = with_prefix(DocType::File, str_arg(args, "fileId"));
    let output_path = opt_str(args, "outputPath");
    let include_image = args
        .get("includeImage")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let needs_data = include_image || output_path.is_some();
    let file = api::get_doc(store, &id, None, needs_data)?;
    // `const { _attachments, ...metadata } = file`, keeping the key order.
    let metadata: Map<String, Value> = file
        .as_object()
        .into_iter()
        .flatten()
        .filter(|(key, _)| *key != "_attachments")
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect();
    let metadata = Value::Object(metadata);
    let data = file
        .get("_attachments")
        .and_then(|a| a.get("index"))
        .and_then(|index| index.get("data"))
        .and_then(Value::as_str);
    let mut blocks = vec![text(pretty(&metadata))];
    if needs_data {
        let content_type = js_string(file.get("contentType"));
        blocks.push(match (data, output_path) {
            (None, _) => text("The attachment data could not be read from the document."),
            (Some(data), Some(path)) => {
                if !local_files {
                    return Err(ToolError(format!(
                        "Failed to write the attachment to {path}: {NO_LOCAL_FILES}"
                    )));
                }
                std::fs::write(path, decode_base64_leniently(data)).map_err(|err| {
                    ToolError(format!("Failed to write the attachment to {path}: {err}"))
                })?;
                text(format!("Attachment written to {path}"))
            }
            (Some(_), None) if !is_viewable_image(&content_type) => text(format!(
                "The attachment is a {content_type} file, which cannot be displayed inline. Pass `outputPath` to write it to disk."
            )),
            (Some(data), None) => json!({
                "type": "image",
                "data": data,
                "mimeType": if content_type == "image/jpg" { "image/jpeg" } else { content_type.as_str() },
            }),
        });
    }
    result(blocks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefixes_count_utf16_units() {
        assert_eq!(js_prefix("abc", 2), "ab");
        assert_eq!(js_prefix("ab", 5), "ab");
        // 😀 is two UTF-16 units; a cut through it drops it.
        assert_eq!(js_prefix("a😀b", 2), "a");
        assert_eq!(js_prefix("a😀b", 3), "a😀");
        assert_eq!(js_prefix("ééé", 2), "éé");
    }

    #[test]
    fn occurrences_overlap_like_index_of() {
        assert!(matches!(find_once("aaa", "aa"), Occurrence::Many));
        assert!(matches!(find_once("abc", "b"), Occurrence::Once(1)));
        assert!(matches!(find_once("abc", "x"), Occurrence::None));
        assert!(matches!(find_once("é😀é", "é"), Occurrence::Many));
        assert!(matches!(find_once("😀ab", "ab"), Occurrence::Once(4)));
    }

    #[test]
    fn list_notes_keywords_match_the_node_server() {
        let tags = [json!("tag:a"), json!("tag:b")];
        assert_eq!(
            list_notes_keyword(Some("book:xyz"), &tags, Some("hello")),
            "bookId:xyz tagId:tag:a tagId:tag:b hello"
        );
        assert_eq!(list_notes_keyword(None, &[], None), "");
        assert_eq!(list_notes_keyword(Some(""), &[], Some("x")), "x");
        assert_eq!(
            list_notes_keyword(Some("trash"), &[], None),
            "bookId:undefined"
        );
        assert_eq!(
            list_notes_keyword(None, &tags, None),
            "tagId:tag:a tagId:tag:b"
        );
    }

    #[test]
    fn base64_decoding_is_as_lenient_as_node() {
        assert_eq!(decode_base64_leniently("aGVsbG8="), b"hello");
        assert_eq!(decode_base64_leniently("aGVsbG8"), b"hello");
        assert_eq!(decode_base64_leniently("aGVs\nbG8="), b"hello");
        assert_eq!(decode_base64_leniently("-_8="), [0xfb, 0xff]);
    }

    #[test]
    fn image_types_are_inferred_from_extensions() {
        assert_eq!(infer_image_type("/a/b.PNG"), Some("image/png"));
        assert_eq!(infer_image_type("x.jpg"), Some("image/jpeg"));
        assert_eq!(infer_image_type("x.svg"), Some("image/svg+xml"));
        assert_eq!(infer_image_type(".png"), None);
        assert_eq!(infer_image_type("x.bmp"), None);
    }
}
