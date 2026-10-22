//! Converts Apple's styled text and CRDT tables to Markdown.
//!
//! The conversion rules are adapted from Obsidian Importer's Apple Notes
//! converter (Copyright (c) 2023 Obsidian; MIT; see `LICENSE.apple-notes`).
//! The supported subset and Kanso-specific attachment links are documented
//! here rather than relying on generated protobuf descriptors or runtime code.

use std::collections::{HashMap, HashSet};

use super::proto::{AttributeRun, MergeableDataProto, Note, ParagraphStyle};
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
            for piece in paragraph {
                append_piece(
                    &mut output,
                    &piece,
                    note_keys,
                    resolve,
                    &mut embedded,
                    &mut embedded_keys,
                    &mut tags,
                );
            }
        }

        if !in_code
            && let Some(alignment) = style
                .as_ref()
                .and_then(|value| value.alignment)
                .filter(|value| *value != 0)
        {
            let (start, end) = match alignment {
                1 => ("<p style=\"text-align:center;margin:0\">", "</p>"),
                2 => ("<p style=\"text-align:right;margin:0\">", "</p>"),
                3 => ("<p style=\"text-align:justify;margin:0\">", "</p>"),
                _ => ("<p style=\"text-align:left;margin:0\">", "</p>"),
            };
            // The prefix/content were appended directly; wrap this paragraph in-place.
            let end_index = output.len();
            let paragraph_start = output.rfind('\n').map_or(0, |position| position + 1);
            output.insert_str(paragraph_start, start);
            output.insert_str(end_index + start.len(), end);
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
    let mut output = String::new();
    let normalized = text.replace(SOFT_RETURN, "\n");
    let mut token = String::new();
    for character in normalized.chars() {
        if character.is_whitespace() {
            if !token.is_empty() {
                output.push_str(&format_word(&token, attributes, note_keys));
                token.clear();
            }
            output.push(character);
        } else {
            token.push(character);
        }
    }
    if !token.is_empty() {
        output.push_str(&format_word(&token, attributes, note_keys));
    }
    output
}

fn format_word(
    text: &str,
    attributes: &AttributeRun,
    note_keys: &HashMap<String, String>,
) -> String {
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
    if let Some(link) = attributes
        .link
        .as_deref()
        .filter(|link| !link.is_empty() && *link != text)
    {
        if let Some(identifier) = apple_note_identifier(link) {
            if let Some(key) = note_keys.get(&identifier.to_ascii_lowercase()) {
                output = format!("[{output}](kanso-import:note/{key})");
            }
        } else {
            output = format!("[{output}]({})", escape_url(link));
        }
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
    let mut row_locations = HashMap::new();
    let mut column_locations = HashMap::new();
    let mut row_count = 0usize;
    let mut column_count = 0usize;
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
            "crRows" => {
                (row_locations, row_count) = find_locations(object, objects, uuids);
            }
            "crColumns" => {
                (column_locations, column_count) = find_locations(object, objects, uuids);
            }
            "cellColumns" => cell_data = Some(object),
            _ => {}
        }
    }
    let Some(cell_data) = cell_data.and_then(|entry| entry.dictionary.as_ref()) else {
        return Err("has unsupported CRDT structure: cellColumns is missing".to_owned());
    };
    if row_count == 0 || column_count == 0 {
        return Err("has unsupported CRDT structure: rows or columns are missing".to_owned());
    }
    if row_count.saturating_mul(column_count) > 10_000 {
        return Err("exceeds the 10000-cell rendering limit".to_owned());
    }
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

fn find_locations(
    entry: &proto::MergeableEntry,
    objects: &[proto::MergeableEntry],
    uuids: &[Vec<u8>],
) -> (HashMap<String, usize>, usize) {
    let Some(ordering) = entry
        .ordered_set
        .as_ref()
        .and_then(|set| set.ordering.as_ref())
    else {
        return (HashMap::new(), 0);
    };
    let order: Vec<String> = ordering
        .array
        .as_ref()
        .map(|array| {
            array
                .attachment
                .iter()
                .map(|attachment| hex(&attachment.uuid))
                .collect()
        })
        .unwrap_or_default();
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
            if let Some(index) = order.iter().position(|uuid| uuid == &key) {
                locations.insert(value, index);
            }
        }
    }
    (locations, order.len())
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
        MergeableDataObject, MergeableDataStore, MergeableEntry, MergeableMap,
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
}
