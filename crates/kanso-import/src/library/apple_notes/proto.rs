//! Explicit prost subset of Apple Notes' stored messages.
//!
//! Derived from Obsidian Importer's Apple Notes descriptor and converter
//! (Copyright (c) 2023 Obsidian; MIT; see `LICENSE.apple-notes`). The original
//! descriptor includes a notice for its protobuf schema source, Three Planets
//! Software (Copyright (c) 2019). This file keeps only fields needed for note
//! text, formatting, attachments, tables, and scan pages. Unknown protobuf
//! fields are skipped by prost for forward compatibility.
//!
//! Supported subset: `Document` wraps a `Note`; note text is paired with
//! UTF-16-length `AttributeRun`s for paragraph style, font weight, underline,
//! strikethrough, superscript, links, color/highlight, and attachment metadata.
//! Paragraph styles include headings, alignment, indentation, checklists, and
//! blockquotes. Attachment records cover inline links/tags/mentions, URL cards,
//! media, drawings, tables, and scans. The mergeable-data subset decodes the
//! CRDT registers, lists, dictionaries, custom maps, ordered sets, and note
//! values needed to recover table cells and scan-page identifiers; it is not a
//! general decoder for every Apple Notes message or CRDT operation.

use std::io::Read;

use anyhow::Context as _;
use flate2::read::GzDecoder;
use prost::Message;

#[derive(Clone, PartialEq, Message)]
pub(super) struct NoteStoreProto {
    #[prost(message, optional, tag = "2")]
    pub document: Option<Document>,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct Document {
    #[prost(int32, optional, tag = "2")]
    pub version: Option<i32>,
    #[prost(message, optional, tag = "3")]
    pub note: Option<Note>,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct Note {
    #[prost(string, tag = "2")]
    pub note_text: String,
    #[prost(message, repeated, tag = "5")]
    pub attribute_run: Vec<AttributeRun>,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct AttributeRun {
    #[prost(int32, tag = "1")]
    pub length: i32,
    #[prost(message, optional, tag = "2")]
    pub paragraph_style: Option<ParagraphStyle>,
    #[prost(message, optional, tag = "3")]
    pub font: Option<Font>,
    #[prost(int32, optional, tag = "5")]
    pub font_weight: Option<i32>,
    #[prost(int32, optional, tag = "6")]
    pub underlined: Option<i32>,
    #[prost(int32, optional, tag = "7")]
    pub strikethrough: Option<i32>,
    #[prost(int32, optional, tag = "8")]
    pub superscript: Option<i32>,
    #[prost(string, optional, tag = "9")]
    pub link: Option<String>,
    #[prost(message, optional, tag = "10")]
    pub color: Option<Color>,
    #[prost(message, optional, tag = "12")]
    pub attachment_info: Option<AttachmentInfo>,
    #[prost(int32, optional, tag = "14")]
    pub emphasis_color: Option<i32>,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct ParagraphStyle {
    #[prost(int32, optional, tag = "1")]
    pub style_type: Option<i32>,
    #[prost(int32, optional, tag = "2")]
    pub alignment: Option<i32>,
    #[prost(int32, optional, tag = "4")]
    pub indent_amount: Option<i32>,
    #[prost(message, optional, tag = "5")]
    pub checklist: Option<Checklist>,
    #[prost(int32, optional, tag = "8")]
    pub blockquote: Option<i32>,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct Checklist {
    #[prost(bytes = "vec", optional, tag = "1")]
    pub uuid: Option<Vec<u8>>,
    #[prost(int32, tag = "2")]
    pub done: i32,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct Font {
    #[prost(string, optional, tag = "1")]
    pub font_name: Option<String>,
    #[prost(float, optional, tag = "2")]
    pub point_size: Option<f32>,
    #[prost(int32, optional, tag = "3")]
    pub font_hints: Option<i32>,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct Color {
    #[prost(float, tag = "1")]
    pub red: f32,
    #[prost(float, tag = "2")]
    pub green: f32,
    #[prost(float, tag = "3")]
    pub blue: f32,
    #[prost(float, tag = "4")]
    pub alpha: f32,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct AttachmentInfo {
    #[prost(string, tag = "1")]
    pub attachment_identifier: String,
    #[prost(string, tag = "2")]
    pub type_uti: String,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct MergeableDataProto {
    #[prost(message, optional, tag = "2")]
    pub mergable_data_object: Option<MergeableDataObject>,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct MergeableDataObject {
    #[prost(int32, optional, tag = "2")]
    pub version: Option<i32>,
    #[prost(message, optional, tag = "3")]
    pub data: Option<MergeableDataStore>,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct MergeableDataStore {
    #[prost(message, repeated, tag = "3")]
    pub entry: Vec<MergeableEntry>,
    #[prost(string, repeated, tag = "4")]
    pub key_item: Vec<String>,
    #[prost(string, repeated, tag = "5")]
    pub type_item: Vec<String>,
    #[prost(bytes = "vec", repeated, tag = "6")]
    pub uuid_item: Vec<Vec<u8>>,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct MergeableEntry {
    #[prost(message, optional, tag = "1")]
    pub register_latest: Option<RegisterLatest>,
    #[prost(message, optional, tag = "5")]
    pub list: Option<List>,
    #[prost(message, optional, tag = "6")]
    pub dictionary: Option<Dictionary>,
    #[prost(message, optional, tag = "10")]
    pub note: Option<Note>,
    #[prost(message, optional, tag = "13")]
    pub custom_map: Option<MergeableMap>,
    #[prost(message, optional, tag = "16")]
    pub ordered_set: Option<OrderedSet>,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct RegisterLatest {
    #[prost(message, optional, tag = "2")]
    pub contents: Option<ObjectId>,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct ObjectId {
    #[prost(uint64, optional, tag = "2")]
    pub unsigned_integer_value: Option<u64>,
    #[prost(string, optional, tag = "4")]
    pub string_value: Option<String>,
    #[prost(int32, optional, tag = "6")]
    pub object_index: Option<i32>,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct Dictionary {
    #[prost(message, repeated, tag = "1")]
    pub element: Vec<DictionaryElement>,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct DictionaryElement {
    #[prost(message, optional, tag = "1")]
    pub key: Option<ObjectId>,
    #[prost(message, optional, tag = "2")]
    pub value: Option<ObjectId>,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct MergeableMap {
    #[prost(int32, optional, tag = "1")]
    pub type_index: Option<i32>,
    #[prost(message, repeated, tag = "3")]
    pub map_entry: Vec<MapEntry>,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct MapEntry {
    #[prost(int32, tag = "1")]
    pub key: i32,
    #[prost(message, optional, tag = "2")]
    pub value: Option<ObjectId>,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct OrderedSet {
    #[prost(message, optional, tag = "1")]
    pub ordering: Option<OrderedSetOrdering>,
    #[prost(message, optional, tag = "2")]
    pub elements: Option<Dictionary>,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct OrderedSetOrdering {
    #[prost(message, optional, tag = "1")]
    pub array: Option<OrderedSetArray>,
    #[prost(message, optional, tag = "2")]
    pub contents: Option<Dictionary>,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct OrderedSetArray {
    #[prost(message, optional, tag = "1")]
    pub contents: Option<Note>,
    #[prost(message, repeated, tag = "2")]
    pub attachment: Vec<OrderedSetAttachment>,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct OrderedSetAttachment {
    #[prost(int32, optional, tag = "1")]
    pub index: Option<i32>,
    #[prost(bytes = "vec", tag = "2")]
    pub uuid: Vec<u8>,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct List {
    #[prost(message, repeated, tag = "1")]
    pub entry: Vec<ListEntry>,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct ListEntry {
    #[prost(message, optional, tag = "2")]
    pub id: Option<ObjectId>,
    #[prost(message, optional, tag = "3")]
    pub details: Option<ListEntryDetails>,
    #[prost(message, optional, tag = "4")]
    pub additional_details: Option<ListEntryDetails>,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct ListEntryDetails {
    #[prost(message, optional, tag = "1")]
    pub key: Option<ListEntryDetailsKey>,
    #[prost(message, optional, tag = "2")]
    pub id: Option<ObjectId>,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct ListEntryDetailsKey {
    #[prost(int32, optional, tag = "1")]
    pub type_index: Option<i32>,
    #[prost(int32, optional, tag = "2")]
    pub key: Option<i32>,
}

/// Decode a gzip protobuf with a hard cap on the expanded message size.
pub(super) fn decode_note(bytes: &[u8], max_output: usize) -> anyhow::Result<Note> {
    let store: NoteStoreProto = decode_gzip(bytes, max_output)?;
    store
        .document
        .and_then(|document| document.note)
        .context("protobuf note store has no note payload")
}

pub(super) fn decode_mergeable(
    bytes: &[u8],
    max_output: usize,
) -> anyhow::Result<MergeableDataProto> {
    decode_gzip(bytes, max_output)
}

fn decode_gzip<T: Message + Default>(bytes: &[u8], max_output: usize) -> anyhow::Result<T> {
    let mut decoder = GzDecoder::new(bytes);
    let mut output = Vec::with_capacity(max_output.min(64 * 1024));
    decoder
        .by_ref()
        .take(max_output as u64 + 1)
        .read_to_end(&mut output)
        .context("invalid gzip stream")?;
    anyhow::ensure!(
        output.len() <= max_output,
        "expanded protobuf exceeds the {max_output}-byte limit"
    );
    T::decode(output.as_slice()).context("invalid protobuf wire data")
}

pub(super) fn scan_attachment_identifiers(message: &MergeableDataProto) -> Vec<String> {
    let Some(data) = message
        .mergable_data_object
        .as_ref()
        .and_then(|object| object.data.as_ref())
    else {
        return Vec::new();
    };
    data.entry
        .iter()
        .filter_map(|entry| entry.custom_map.as_ref())
        .filter_map(|map| map.map_entry.first())
        .filter_map(|entry| entry.value.as_ref())
        .filter_map(|value| value.string_value.clone())
        .collect()
}
