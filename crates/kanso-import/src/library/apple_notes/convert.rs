//! Converts Apple's styled text and CRDT tables to Markdown.
//!
//! The conversion rules are adapted from Obsidian Importer's Apple Notes
//! converter (Copyright (c) 2023 Obsidian; MIT; see `LICENSE.apple-notes`).
//! The supported subset and Kanso-specific attachment links are documented
//! here rather than relying on generated protobuf descriptors or runtime code.

use std::collections::{HashMap, HashSet};

use kanso_core::model::MAX_TITLE_LEN;

use super::proto::{AttributeRun, MergeableDataProto, Note, OrderedSetOrdering, ParagraphStyle};
use super::{AttachmentResult, proto};

const STYLE_TITLE: i32 = 0;
const STYLE_HEADING: i32 = 1;
const STYLE_SUBHEADING: i32 = 2;
const STYLE_MONOSPACED: i32 = 4;
const STYLE_DOTTED_LIST: i32 = 100;
const STYLE_DASHED_LIST: i32 = 101;
const STYLE_NUMBERED_LIST: i32 = 102;
const STYLE_CHECKBOX: i32 = 103;
const SOFT_RETURN: char = '\u{2028}';
const OBJECT_REPLACEMENT: char = '\u{fffc}';

pub(super) struct RenderedNote {
    pub body: String,
    pub embedded: Vec<super::Embedded>,
    pub tags: Vec<String>,
}

#[derive(Clone)]
struct Piece {
    text: String,
    attributes: AttributeRun,
}

#[derive(Default)]
struct TagCollector {
    tags: Vec<String>,
    keys: HashSet<String>,
}

impl TagCollector {
    fn push(&mut self, tag: &str) {
        let normalized = tag.trim().trim_start_matches('#').trim().to_owned();
        if !normalized.is_empty() && self.keys.insert(normalized.to_lowercase()) {
            self.tags.push(normalized);
        }
    }
}

/// Derive Apple's displayed title from the first nonempty text line.
pub(super) fn note_title(note_text: &str, stored: &str) -> String {
    let first = note_text
        .split('\n')
        .map(|line| {
            line.chars()
                .filter(|character| *character != OBJECT_REPLACEMENT)
                .map(|character| {
                    if matches!(character, SOFT_RETURN | '\u{2029}') {
                        ' '
                    } else {
                        character
                    }
                })
                .collect::<String>()
                .trim()
                .to_owned()
        })
        .find(|line| !line.is_empty());
    let Some(line) = first else {
        return stored.trim().to_owned();
    };
    let trimmed = line.trim_start();
    let title = strip_list_marker(trimmed).trim_start();
    let title = if title.is_empty() {
        line.as_str()
    } else {
        title
    };
    title
        .chars()
        .take(200)
        .collect::<String>()
        .trim_end()
        .to_owned()
}

pub(super) fn render_note(
    note: &Note,
    title: &str,
    _note_key: &str,
    note_keys: &HashMap<String, String>,
    omit_first_line: bool,
    resolve: &mut dyn FnMut(&str, &str) -> AttachmentResult,
) -> RenderedNote {
    let mut paragraphs = paragraphs(note);
    let skip_first = omit_first_line
        && note.note_text.contains('\n')
        && !is_url_only(note)
        && !first_line_has_attachment(note)
        && first_line_fits_title(note)
        && !starts_nested_list(&paragraphs);
    if skip_first && !paragraphs.is_empty() {
        paragraphs.remove(0);
    }

    let mut output = String::new();
    let mut embedded = Vec::new();
    let mut embedded_keys = HashSet::new();
    let mut tags = TagCollector::default();
    let mut numbering: HashMap<i32, usize> = HashMap::new();
    let mut previous_list = None;
    let mut in_code = false;
    let mut first_output = true;

    for paragraph in paragraphs {
        let mut closed_code = false;
        let mut opened_code = false;
        let style = paragraph_style(&paragraph).cloned();
        let style_type = style
            .as_ref()
            .and_then(|value| value.style_type)
            .unwrap_or(-1);
        let is_code = style_type == STYLE_MONOSPACED;
        if in_code && !is_code {
            if !output.ends_with('\n') {
                output.push('\n');
            }
            output.push_str("```\n");
            in_code = false;
            closed_code = true;
        }
        if is_code && !in_code {
            if !output.is_empty() && !output.ends_with("\n\n") {
                output.push('\n');
            }
            output.push_str("```\n");
            in_code = true;
            opened_code = true;
        }

        let content = paragraph
            .iter()
            .map(|piece| piece.text.as_str())
            .collect::<String>();
        let has_content = content
            .chars()
            .any(|character| !character.is_whitespace() && character != SOFT_RETURN);
        let indent = style
            .as_ref()
            .and_then(|value| value.indent_amount)
            .unwrap_or(0)
            .clamp(0, 32);
        let is_list = matches!(
            style_type,
            STYLE_DOTTED_LIST | STYLE_DASHED_LIST | STYLE_NUMBERED_LIST | STYLE_CHECKBOX
        );
        if is_list {
            let current_list = (style_type, indent);
            if style_type == STYLE_NUMBERED_LIST && previous_list != Some(current_list) {
                numbering.insert(indent, 0);
            }
            previous_list = Some(current_list);
        } else {
            numbering.clear();
            previous_list = None;
        }

        if !first_output && !opened_code && !closed_code {
            output.push('\n');
        }
        first_output = false;

        if !is_code
            && has_content
            && is_list
            && indent > 0
            && numbering.is_empty()
            && output.trim().is_empty()
        {
            output.push_str("- &nbsp;\n");
        }
        let mut prefix = String::new();
        if style
            .as_ref()
            .is_some_and(|value| value.blockquote.unwrap_or(0) != 0)
        {
            prefix.push_str("> ");
        }
        if !is_code {
            match style_type {
                STYLE_TITLE => prefix.push_str("# "),
                STYLE_HEADING => prefix.push_str("## "),
                STYLE_SUBHEADING => prefix.push_str("### "),
                STYLE_DOTTED_LIST | STYLE_DASHED_LIST => {
                    prefix.push_str(&"\t".repeat(indent as usize));
                    prefix.push_str("- ");
                }
                STYLE_NUMBERED_LIST => {
                    let count = numbering.get(&indent).copied().unwrap_or(0) + 1;
                    numbering.insert(indent, count);
                    prefix.push_str(&"\t".repeat(indent as usize));
                    prefix.push_str(&format!("{count}. "));
                }
                STYLE_CHECKBOX => {
                    prefix.push_str(&"\t".repeat(indent as usize));
                    let checked = style
                        .as_ref()
                        .and_then(|value| value.checklist.as_ref())
                        .is_some_and(|checklist| checklist.done != 0);
                    prefix.push_str(if checked { "- [x] " } else { "- [ ] " });
                }
                _ => {}
            }
        }
        output.push_str(&prefix);

        if is_code {
            for piece in paragraph {
                let text = piece.text.replace(OBJECT_REPLACEMENT, "");
                output.push_str(&text.replace(SOFT_RETURN, "\n"));
            }
        } else {
            // Paragraph alignment is dropped, as every other importer does: a
            // `<p>` wrapper would turn the Markdown inside it into literal text.
            let mut pieces = paragraph.into_iter().peekable();
            while let Some(piece) = pieces.next() {
                let Some(link) = text_link(&piece).map(str::to_owned) else {
                    append_piece(
                        &mut output,
                        &piece,
                        note_keys,
                        resolve,
                        &mut embedded,
                        &mut embedded_keys,
                        &mut tags,
                    );
                    continue;
                };
                // Runs that share a link but differ in format become one link.
                let mut raw = piece.text.clone();
                let mut inner = format_styles(&piece.text, &piece.attributes);
                while let Some(next) = pieces.next_if(|next| text_link(next) == Some(&link)) {
                    raw.push_str(&next.text);
                    inner.push_str(&format_styles(&next.text, &next.attributes));
                }
                output.push_str(&link_markdown(&inner, &raw, &link, note_keys));
            }
        }
        let _ = title;
    }

    if in_code {
        if !output.ends_with('\n') {
            output.push('\n');
        }
        output.push_str("```");
    }
    let body = output.trim().to_owned();
    RenderedNote {
        body,
        embedded,
        tags: tags.tags,
    }
}

fn append_piece(
    output: &mut String,
    piece: &Piece,
    note_keys: &HashMap<String, String>,
    resolve: &mut dyn FnMut(&str, &str) -> AttachmentResult,
    embedded: &mut Vec<super::Embedded>,
    embedded_keys: &mut HashSet<String>,
    tags: &mut TagCollector,
) {
    let Some(info) = piece.attributes.attachment_info.as_ref() else {
        output.push_str(&format_inline(&piece.text, &piece.attributes, note_keys));
        return;
    };
    let mut plain = String::new();
    for character in piece.text.chars() {
        if character == OBJECT_REPLACEMENT {
            if !plain.is_empty() {
                output.push_str(&format_inline(&plain, &piece.attributes, note_keys));
                plain.clear();
            }
            let converted = resolve(&info.attachment_identifier, &info.type_uti);
            output.push_str(&converted.markdown);
            for tag in converted.tags {
                tags.push(&tag);
            }
            for file in converted.embedded {
                if embedded_keys.insert(file.key.clone()) {
                    embedded.push(file);
                }
            }
        } else {
            plain.push(character);
        }
    }
    if !plain.is_empty() {
        output.push_str(&format_inline(&plain, &piece.attributes, note_keys));
    }
}

fn format_inline(
    text: &str,
    attributes: &AttributeRun,
    note_keys: &HashMap<String, String>,
) -> String {
    let styled = format_styles(text, attributes);
    match attributes.link.as_deref().filter(|link| !link.is_empty()) {
        Some(link) => link_markdown(&styled, text, link, note_keys),
        None => styled,
    }
}

/// The link a run of plain text carries, if any.
fn text_link(piece: &Piece) -> Option<&str> {
    if piece.attributes.attachment_info.is_some() {
        return None;
    }
    piece
        .attributes
        .link
        .as_deref()
        .filter(|link| !link.is_empty())
}

/// Formats a whole run at once, line by line, so markers wrap the run's words
/// together and never sit against the whitespace around them.
fn format_styles(text: &str, attributes: &AttributeRun) -> String {
    text.replace(SOFT_RETURN, "\n")
        .split('\n')
        .map(|line| around_whitespace(line, |core| format_word(core, attributes)))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Applies `wrap` to `text` without its surrounding whitespace.
fn around_whitespace(text: &str, wrap: impl FnOnce(&str) -> String) -> String {
    let core = text.trim();
    if core.is_empty() {
        return text.to_owned();
    }
    let leading = &text[..text.len() - text.trim_start().len()];
    let trailing = &text[text.trim_end().len()..];
    format!("{leading}{}{trailing}", wrap(core))
}

/// Links already-formatted `inner`, whose source text is `raw`. A bare URL
/// stays as it is, and so does a link to a note this import doesn't have.
fn link_markdown(
    inner: &str,
    raw: &str,
    link: &str,
    note_keys: &HashMap<String, String>,
) -> String {
    if raw.trim() == link {
        return inner.to_owned();
    }
    let target = match apple_note_identifier(link) {
        Some(identifier) => match note_keys.get(&identifier.to_ascii_lowercase()) {
            Some(key) => format!("kanso-import:note/{key}"),
            None => return inner.to_owned(),
        },
        None => escape_url(link),
    };
    around_whitespace(inner, |core| format!("[{core}]({target})"))
}

fn format_word(text: &str, attributes: &AttributeRun) -> String {
    let paragraph = attributes.paragraph_style.as_ref();
    let heading_text = paragraph.is_some_and(|style| {
        matches!(
            style.style_type,
            Some(STYLE_TITLE | STYLE_HEADING | STYLE_SUBHEADING)
        )
    });
    let redundant_heading_bold = heading_text && matches!(attributes.font_weight, Some(1 | 3));
    let mut output = escape_markdown_text(text);
    match attributes.font_weight.unwrap_or(0) {
        1 if !redundant_heading_bold => output = format!("**{output}**"),
        2 => output = format!("*{output}*"),
        3 if !redundant_heading_bold => output = format!("***{output}***"),
        3 => output = format!("*{output}*"),
        _ => {}
    }
    if attributes.strikethrough.unwrap_or(0) != 0 {
        output = format!("~~{output}~~");
    }
    if attributes.underlined.unwrap_or(0) != 0 {
        output = format!("<u>{output}</u>");
    }
    match attributes.superscript.unwrap_or(0) {
        1 => output = format!("<sup>{output}</sup>"),
        -1 => output = format!("<sub>{output}</sub>"),
        _ => {}
    }
    if let Some(color) = attributes.emphasis_color {
        let marker = match color {
            1 => "🟣",
            2 => "🔴",
            3 => "🟠",
            4 => "🟢",
            5 => "🔵",
            _ => "",
        };
        output = format!("=={marker}{output}==");
    }
    output
}

fn paragraphs(note: &Note) -> Vec<Vec<Piece>> {
    let mut result: Vec<Vec<Piece>> = vec![Vec::new()];
    let mut run_index = 0usize;
    let mut consumed_units = 0usize;
    let mut run_end = note
        .attribute_run
        .first()
        .map(|run| run.length.max(0) as usize)
        .unwrap_or(usize::MAX);
    let mut current = AttributeRun::default();

    for character in note.note_text.chars() {
        while run_index < note.attribute_run.len() && consumed_units >= run_end {
            run_index += 1;
            current = note
                .attribute_run
                .get(run_index)
                .cloned()
                .unwrap_or_default();
            let length = current.length.max(0) as usize;
            run_end = run_end.saturating_add(length);
            if length == 0 && run_index < note.attribute_run.len() {
                continue;
            }
            break;
        }
        if run_index == 0 {
            current = note.attribute_run.first().cloned().unwrap_or_default();
        }
        if character == '\n' || character == '\u{2029}' {
            result.push(Vec::new());
            consumed_units += character.len_utf16();
            continue;
        }
        let paragraph = result
            .last_mut()
            .expect("paragraph vector always has one entry");
        if let Some(piece) = paragraph
            .last_mut()
            .filter(|piece| same_style(&piece.attributes, &current))
        {
            piece.text.push(character);
        } else {
            let mut attributes = current.clone();
            attributes.length = 0;
            paragraph.push(Piece {
                text: character.to_string(),
                attributes,
            });
        }
        consumed_units += character.len_utf16();
    }
    result
}

fn same_style(left: &AttributeRun, right: &AttributeRun) -> bool {
    let mut left = left.clone();
    let mut right = right.clone();
    left.length = 0;
    right.length = 0;
    left == right
}

fn paragraph_style(paragraph: &[Piece]) -> Option<&ParagraphStyle> {
    paragraph
        .iter()
        .find_map(|piece| piece.attributes.paragraph_style.as_ref())
}

fn starts_nested_list(paragraphs: &[Vec<Piece>]) -> bool {
    let first = paragraphs.iter().find(|paragraph| !paragraph.is_empty());
    let Some(first) = first else { return false };
    let Some(style) = paragraph_style(first) else {
        return false;
    };
    let first_type = style.style_type.unwrap_or(-1);
    if !is_list_style(first_type) {
        return false;
    }
    let first_indent = style.indent_amount.unwrap_or(0);
    paragraphs
        .iter()
        .skip_while(|paragraph| paragraph.is_empty() || std::ptr::eq(*paragraph, first))
        .find(|paragraph| !paragraph.is_empty())
        .and_then(|paragraph| paragraph_style(paragraph))
        .is_some_and(|style| style.indent_amount.unwrap_or(0) > first_indent)
}

fn first_line(note: &Note) -> String {
    note.note_text
        .split('\n')
        .next()
        .unwrap_or_default()
        .replace(OBJECT_REPLACEMENT, "")
        .replace([SOFT_RETURN, '\u{2029}'], " ")
        .trim()
        .to_owned()
}

/// Whether the first line fits in a title whole. A longer one stays in the
/// body too, or the text past the title's limit would be lost.
fn first_line_fits_title(note: &Note) -> bool {
    first_line(note)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .count()
        <= MAX_TITLE_LEN
}

fn first_line_has_attachment(note: &Note) -> bool {
    note.note_text
        .split('\n')
        .next()
        .is_some_and(|line| line.contains(OBJECT_REPLACEMENT))
}

fn is_url_only(note: &Note) -> bool {
    let line = first_line(note);
    (line.starts_with("http://") || line.starts_with("https://"))
        && !line.chars().any(char::is_whitespace)
}

fn is_list_style(style: i32) -> bool {
    matches!(
        style,
        STYLE_DOTTED_LIST | STYLE_DASHED_LIST | STYLE_NUMBERED_LIST | STYLE_CHECKBOX
    )
}

fn strip_list_marker(value: &str) -> &str {
    let bytes = value.as_bytes();
    if bytes.len() > 1 && matches!(bytes[0], b'-' | b'+' | b'*') && bytes[1].is_ascii_whitespace() {
        return &value[2..];
    }
    let digits = bytes
        .iter()
        .take_while(|byte| byte.is_ascii_digit())
        .count();
    if digits > 0
        && bytes
            .get(digits)
            .is_some_and(|byte| matches!(byte, b'.' | b')'))
        && bytes.get(digits + 1).is_some_and(u8::is_ascii_whitespace)
    {
        return &value[digits + 2..];
    }
    value
}

fn apple_note_identifier(link: &str) -> Option<String> {
    let rest = link.strip_prefix("applenotes:note/")?;
    Some(rest.split(['?', '#']).next()?.to_owned())
}

fn escape_markdown_text(value: &str) -> String {
    value.replace('[', "\\[").replace(']', "\\]")
}

pub(super) fn escape_markdown_label(value: &str) -> String {
    value.replace(['[', ']'], "")
}

fn escape_url(value: &str) -> String {
    value
        .replace(')', "%29")
        .replace('(', "%28")
        .replace(' ', "%20")
}

pub(super) fn render_table(message: &MergeableDataProto) -> Result<String, String> {
    let Some(data) = message
        .mergable_data_object
        .as_ref()
        .and_then(|object| object.data.as_ref())
    else {
        return Err("has unsupported CRDT structure: table data is missing".to_owned());
    };
    let objects = &data.entry;
    let types = &data.type_item;
    let keys = &data.key_item;
    let uuids = &data.uuid_item;
    let root = objects.iter().find(|object| {
        object
            .custom_map
            .as_ref()
            .and_then(|map| map.type_index)
            .and_then(|index| types.get(index as usize))
            .is_some_and(|name| name == "com.apple.notes.ICTable")
    });
    let Some(root) = root.and_then(|entry| entry.custom_map.as_ref()) else {
        return Err("has unsupported CRDT structure: ICTable root is missing".to_owned());
    };
    let mut rows = None;
    let mut columns = None;
    let mut cell_data = None;

    for entry in &root.map_entry {
        let Some(name) = keys.get(entry.key as usize) else {
            continue;
        };
        let Some(object) = entry
            .value
            .as_ref()
            .and_then(|value| value.object_index)
            .and_then(|index| objects.get(index as usize))
        else {
            continue;
        };
        match name.as_str() {
            "crRows" => rows = ordering(object),
            "crColumns" => columns = ordering(object),
            "cellColumns" => cell_data = Some(object),
            _ => {}
        }
    }
    let Some(cell_data) = cell_data.and_then(|entry| entry.dictionary.as_ref()) else {
        return Err("has unsupported CRDT structure: cellColumns is missing".to_owned());
    };
    // The limit is checked before any location is worked out.
    let row_count = rows.map_or(0, ordered_count);
    let column_count = columns.map_or(0, ordered_count);
    if row_count == 0 || column_count == 0 {
        return Err("has unsupported CRDT structure: rows or columns are missing".to_owned());
    }
    if row_count.saturating_mul(column_count) > 10_000 {
        return Err("exceeds the 10000-cell rendering limit".to_owned());
    }
    let row_locations = rows.map_or_else(HashMap::new, |rows| find_locations(rows, objects, uuids));
    let column_locations = columns.map_or_else(HashMap::new, |columns| {
        find_locations(columns, objects, uuids)
    });
    let mut grid = vec![vec![String::new(); column_count]; row_count];
    for column in &cell_data.element {
        let Some(column_uuid) = column
            .key
            .as_ref()
            .and_then(|id| target_uuid(id, objects, uuids))
        else {
            continue;
        };
        let Some(&column_index) = column_locations.get(&column_uuid) else {
            continue;
        };
        let Some(rows) = column
            .value
            .as_ref()
            .and_then(|id| id.object_index)
            .and_then(|index| objects.get(index as usize))
            .and_then(|entry| entry.dictionary.as_ref())
        else {
            continue;
        };
        for row in &rows.element {
            let Some(row_uuid) = row
                .key
                .as_ref()
                .and_then(|id| target_uuid(id, objects, uuids))
            else {
                continue;
            };
            let Some(&row_index) = row_locations.get(&row_uuid) else {
                continue;
            };
            if row_index >= row_count || column_index >= column_count {
                continue;
            }
            let Some(note) = row
                .value
                .as_ref()
                .and_then(|id| id.object_index)
                .and_then(|index| objects.get(index as usize))
                .and_then(|entry| entry.note.as_ref())
            else {
                continue;
            };
            grid[row_index][column_index] = render_table_cell(note);
        }
    }
    let mut result = String::from("\n");
    for (index, row) in grid.iter().enumerate() {
        result.push_str("| ");
        result.push_str(&row.join(" | "));
        result.push_str(" |\n");
        if index == 0 {
            result.push('|');
            result.push_str(&" -- |".repeat(column_count));
            result.push('\n');
        }
    }
    result.push('\n');
    Ok(result)
}

fn ordering(entry: &proto::MergeableEntry) -> Option<&OrderedSetOrdering> {
    entry.ordered_set.as_ref()?.ordering.as_ref()
}

fn ordered_count(ordering: &OrderedSetOrdering) -> usize {
    ordering
        .array
        .as_ref()
        .map_or(0, |array| array.attachment.len())
}

/// Maps each row or column identity to its position in the ordering.
fn find_locations(
    ordering: &OrderedSetOrdering,
    objects: &[proto::MergeableEntry],
    uuids: &[Vec<u8>],
) -> HashMap<String, usize> {
    // A UUID listed twice keeps its first position.
    let mut positions = HashMap::new();
    for (index, attachment) in ordering
        .array
        .iter()
        .flat_map(|array| &array.attachment)
        .enumerate()
    {
        positions.entry(hex(&attachment.uuid)).or_insert(index);
    }
    let mut locations = HashMap::new();
    if let Some(contents) = ordering.contents.as_ref() {
        for element in &contents.element {
            let Some(key) = element
                .key
                .as_ref()
                .and_then(|id| target_uuid(id, objects, uuids))
            else {
                continue;
            };
            let Some(value) = element
                .value
                .as_ref()
                .and_then(|id| target_uuid(id, objects, uuids))
            else {
                continue;
            };
            if let Some(&index) = positions.get(&key) {
                locations.insert(value, index);
            }
        }
    }
    locations
}

fn target_uuid(
    id: &proto::ObjectId,
    objects: &[proto::MergeableEntry],
    uuids: &[Vec<u8>],
) -> Option<String> {
    let reference = id.object_index?;
    let object = objects.get(usize::try_from(reference).ok()?)?;
    let uuid_index = object
        .custom_map
        .as_ref()?
        .map_entry
        .first()?
        .value
        .as_ref()?
        .unsigned_integer_value?;
    uuids
        .get(usize::try_from(uuid_index).ok()?)
        .map(|value| hex(value))
}

fn render_table_cell(note: &Note) -> String {
    let rendered = paragraphs(note)
        .into_iter()
        .map(|paragraph| {
            let mut line = String::new();
            for piece in paragraph {
                if piece.attributes.attachment_info.is_some() {
                    line.push_str("(attachment)");
                } else {
                    line.push_str(&format_inline(
                        &piece.text.replace(OBJECT_REPLACEMENT, ""),
                        &piece.attributes,
                        &HashMap::new(),
                    ));
                }
            }
            line
        })
        .collect::<Vec<_>>()
        .join("<br>");
    rendered.trim().replace('|', "&#124;")
}

fn hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        result.push(HEX[(byte >> 4) as usize] as char);
        result.push(HEX[(byte & 0x0f) as usize] as char);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::apple_notes::proto::{
        Dictionary, DictionaryElement, MapEntry, MergeableDataObject, MergeableDataStore,
        MergeableEntry, MergeableMap, ObjectId, OrderedSet, OrderedSetArray, OrderedSetAttachment,
    };

    #[test]
    fn unsupported_table_structure_returns_an_error() {
        let message = MergeableDataProto {
            mergable_data_object: Some(MergeableDataObject {
                data: Some(MergeableDataStore {
                    entry: vec![MergeableEntry {
                        custom_map: Some(MergeableMap {
                            type_index: Some(0),
                            ..MergeableMap::default()
                        }),
                        ..MergeableEntry::default()
                    }],
                    type_item: vec!["com.apple.notes.ICTable".to_owned()],
                    ..MergeableDataStore::default()
                }),
                ..MergeableDataObject::default()
            }),
        };

        assert_eq!(
            render_table(&message).unwrap_err(),
            "has unsupported CRDT structure: cellColumns is missing"
        );
    }

    const LINK: &str = "https://example.test/x";

    fn run(text: &str, attributes: AttributeRun) -> (String, AttributeRun) {
        (text.to_owned(), attributes)
    }

    fn styled(style_type: i32, alignment: i32) -> AttributeRun {
        AttributeRun {
            paragraph_style: Some(ParagraphStyle {
                style_type: Some(style_type),
                alignment: Some(alignment),
                ..ParagraphStyle::default()
            }),
            ..AttributeRun::default()
        }
    }

    fn note(runs: &[(String, AttributeRun)]) -> Note {
        Note {
            note_text: runs.iter().map(|(text, _)| text.as_str()).collect(),
            attribute_run: runs
                .iter()
                .map(|(text, attributes)| AttributeRun {
                    length: text.encode_utf16().count() as i32,
                    ..attributes.clone()
                })
                .collect(),
        }
    }

    fn render(note: &Note, omit_first_line: bool) -> String {
        let mut resolve = |_: &str, _: &str| -> AttachmentResult { unreachable!() };
        render_note(note, "", "", &HashMap::new(), omit_first_line, &mut resolve).body
    }

    #[test]
    fn aligned_headings_list_items_and_paragraphs_stay_markdown() {
        let body = render(
            &note(&[
                run("Heading\n", styled(STYLE_HEADING, 1)),
                run("Item\n", styled(STYLE_DOTTED_LIST, 2)),
                run("Body\u{2028}second line", styled(-1, 1)),
            ]),
            false,
        );
        assert_eq!(body, "## Heading\n- Item\nBody\nsecond line");
    }

    #[test]
    fn a_link_over_several_words_is_one_link() {
        let linked = AttributeRun {
            link: Some(LINK.to_owned()),
            ..AttributeRun::default()
        };
        let bold = AttributeRun {
            font_weight: Some(1),
            ..AttributeRun::default()
        };
        let body = render(
            &note(&[
                run("See ", AttributeRun::default()),
                run("Apple Inc", linked),
                run(" and ", AttributeRun::default()),
                run("two words", bold),
            ]),
            false,
        );
        assert_eq!(body, format!("See [Apple Inc]({LINK}) and **two words**"));
    }

    /// A table builder: `rows` lists each row's UUID slot in order, and
    /// `contents` maps an ordering slot to an identity slot.
    fn table(rows: &[u64], contents: &[(u64, u64)], slots: u64) -> MergeableDataProto {
        let reference = |index: usize| ObjectId {
            object_index: Some(index as i32),
            ..ObjectId::default()
        };
        // Objects 0..4: root, rows, columns, cells; then one per UUID slot.
        let uuid_object = |slot: u64| reference(4 + slot as usize);
        let mut entry = vec![MergeableEntry::default(); 4];
        entry[0].custom_map = Some(MergeableMap {
            type_index: Some(0),
            map_entry: (0..3)
                .map(|key| MapEntry {
                    key,
                    value: Some(reference(key as usize + 1)),
                })
                .collect(),
        });
        let ordering = |order: &[u64], contents: &[(u64, u64)]| OrderedSet {
            ordering: Some(OrderedSetOrdering {
                array: Some(OrderedSetArray {
                    attachment: order
                        .iter()
                        .map(|slot| OrderedSetAttachment {
                            index: Some(0),
                            uuid: slot.to_be_bytes().to_vec(),
                        })
                        .collect(),
                    ..OrderedSetArray::default()
                }),
                contents: Some(Dictionary {
                    element: contents
                        .iter()
                        .map(|&(key, value)| DictionaryElement {
                            key: Some(uuid_object(key)),
                            value: Some(uuid_object(value)),
                        })
                        .collect(),
                }),
            }),
            ..OrderedSet::default()
        };
        entry[1].ordered_set = Some(ordering(rows, contents));
        entry[2].ordered_set = Some(ordering(&[0], &[]));
        entry[3].dictionary = Some(Dictionary::default());
        for slot in 0..slots {
            entry.push(MergeableEntry {
                custom_map: Some(MergeableMap {
                    type_index: Some(1),
                    map_entry: vec![MapEntry {
                        key: 0,
                        value: Some(ObjectId {
                            unsigned_integer_value: Some(slot),
                            ..ObjectId::default()
                        }),
                    }],
                }),
                ..MergeableEntry::default()
            });
        }
        MergeableDataProto {
            mergable_data_object: Some(MergeableDataObject {
                data: Some(MergeableDataStore {
                    entry,
                    key_item: ["crRows", "crColumns", "cellColumns"]
                        .map(str::to_owned)
                        .into(),
                    type_item: vec!["com.apple.notes.ICTable".to_owned()],
                    uuid_item: (0..slots).map(|slot| slot.to_be_bytes().to_vec()).collect(),
                }),
                ..MergeableDataObject::default()
            }),
        }
    }

    #[test]
    fn an_oversized_table_is_refused_before_its_rows_are_located() {
        // Each entry's row sits last in the ordering, so searching the whole
        // ordering per entry was quadratic; the limit now refuses it first.
        let rows: Vec<u64> = (1..60_000).chain([0]).collect();
        let contents = vec![(0, 0); 60_000];
        let message = table(&rows, &contents, 1);
        assert_eq!(
            render_table(&message).unwrap_err(),
            "exceeds the 10000-cell rendering limit"
        );
    }

    proptest::proptest! {
        /// Every location is the key's first position in the ordering, as a
        /// linear search over the ordering finds it.
        #[test]
        fn prop_locations_match_first_positions(
            order in proptest::collection::vec(0u64..8, 0..24),
            contents in proptest::collection::vec((0u64..8, 0u64..8), 0..24),
        ) {
            let message = table(&order, &contents, 8);
            let data = message.mergable_data_object.as_ref().unwrap().data.as_ref().unwrap();
            let rows = ordering(&data.entry[1]).unwrap();
            let found = find_locations(rows, &data.entry, &data.uuid_item);

            let mut expected = HashMap::new();
            for (key, value) in &contents {
                if let Some(index) = order.iter().position(|slot| slot == key) {
                    expected.insert(hex(&value.to_be_bytes()), index);
                }
            }
            proptest::prop_assert_eq!(found, expected);
        }

        /// No first-line text is lost: either the whole line is the title, or
        /// the line also stays in the body.
        #[test]
        fn prop_first_line_is_kept_whole_somewhere(
            words in proptest::collection::vec("[a-z]{1,12}", 1..40),
        ) {
            let first = words.join(" ");
            let source = note(&[run(&format!("{first}\n0 1 2"), AttributeRun::default())]);
            let title = crate::library::clean_title(&note_title(&source.note_text, ""));
            let body = render(&source, true);
            let long = first.chars().count() > MAX_TITLE_LEN;
            proptest::prop_assert_eq!(body.contains(&first), long);
            proptest::prop_assert!(long || title == first);
            proptest::prop_assert!(body.ends_with("0 1 2"));
        }

        /// However its words are formatted, text under one link is one link
        /// holding every word in order.
        #[test]
        fn prop_one_link_however_its_words_are_formatted(
            words in proptest::collection::vec(("[a-z]{1,8}", 0i32..4), 1..8),
        ) {
            let last = words.len() - 1;
            let runs: Vec<_> = words
                .iter()
                .enumerate()
                .map(|(index, (word, weight))| {
                    let text = if index == last { word.clone() } else { format!("{word} ") };
                    run(&text, AttributeRun {
                        font_weight: Some(*weight),
                        link: Some(LINK.to_owned()),
                        ..AttributeRun::default()
                    })
                })
                .collect();
            let body = render(&note(&runs), false);
            let target = format!("]({LINK})");
            proptest::prop_assert_eq!(body.matches(&target).count(), 1, "{}", body);
            let words: Vec<_> = words.iter().map(|(word, _)| word.as_str()).collect();
            proptest::prop_assert_eq!(
                body.replace('*', ""),
                format!("[{}]({LINK})", words.join(" "))
            );
        }
    }
}
