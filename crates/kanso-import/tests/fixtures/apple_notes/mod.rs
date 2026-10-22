//! Rust-built Apple Notes Core Data fixtures. Every note and attachment here
//! is synthetic; no personal Notes database or exported data is committed.

use std::io::Write;
use std::path::{Path, PathBuf};

use flate2::Compression;
use flate2::GzBuilder;
use prost::Message;
use rusqlite::types::Value;
use rusqlite::{Connection, params_from_iter};

const ACCOUNT: i64 = 17;
const FOLDER: i64 = 29;
const NOTE: i64 = 42;
const ATTACHMENT: i64 = 65;
const MEDIA: i64 = 87;
const ACCOUNT_A: i64 = 1;
const ACCOUNT_B: i64 = 2;
pub const GOLDEN_WIRE_HEX: &str = "10011a08120248692a020802";

#[derive(Clone, Default)]
struct Run {
    text: String,
    style: Option<i32>,
    indent: Option<i32>,
    checked: Option<bool>,
    font_weight: Option<i32>,
    underlined: Option<bool>,
    strikethrough: Option<bool>,
    superscript: Option<i32>,
    emphasis_color: Option<i32>,
    alignment: Option<i32>,
    blockquote: bool,
    link: Option<String>,
    attachment: Option<(String, String)>,
}

impl Run {
    fn text(text: &str) -> Self {
        Self {
            text: text.to_owned(),
            ..Self::default()
        }
    }

    fn style(text: &str, style: i32) -> Self {
        Self {
            text: text.to_owned(),
            style: Some(style),
            ..Self::default()
        }
    }

    fn attachment(identifier: &str, uti: &str) -> Self {
        Self {
            text: "\u{fffc}".to_owned(),
            attachment: Some((identifier.to_owned(), uti.to_owned())),
            ..Self::default()
        }
    }
}

#[derive(Clone, PartialEq, Message)]
struct WireDocument {
    #[prost(int32, optional, tag = "2")]
    version: Option<i32>,
    #[prost(message, optional, tag = "3")]
    note: Option<WireNote>,
}

#[derive(Clone, PartialEq, Message)]
struct WireNote {
    #[prost(string, tag = "2")]
    note_text: String,
    #[prost(message, repeated, tag = "5")]
    attribute_run: Vec<WireAttributeRun>,
}

#[derive(Clone, PartialEq, Message)]
struct WireAttributeRun {
    #[prost(int32, tag = "1")]
    length: i32,
    #[prost(message, optional, tag = "2")]
    paragraph_style: Option<WireParagraphStyle>,
    #[prost(int32, optional, tag = "5")]
    font_weight: Option<i32>,
    #[prost(int32, optional, tag = "6")]
    underlined: Option<i32>,
    #[prost(int32, optional, tag = "7")]
    strikethrough: Option<i32>,
    #[prost(int32, optional, tag = "8")]
    superscript: Option<i32>,
    #[prost(string, optional, tag = "9")]
    link: Option<String>,
    #[prost(message, optional, tag = "12")]
    attachment_info: Option<WireAttachmentInfo>,
    #[prost(int32, optional, tag = "14")]
    emphasis_color: Option<i32>,
}

#[derive(Clone, PartialEq, Message)]
struct WireParagraphStyle {
    #[prost(int32, optional, tag = "1")]
    style_type: Option<i32>,
    #[prost(int32, optional, tag = "2")]
    alignment: Option<i32>,
    #[prost(int32, optional, tag = "4")]
    indent_amount: Option<i32>,
    #[prost(message, optional, tag = "5")]
    checklist: Option<WireChecklist>,
    #[prost(int32, optional, tag = "8")]
    blockquote: Option<i32>,
}

#[derive(Clone, PartialEq, Message)]
struct WireChecklist {
    #[prost(bytes = "vec", optional, tag = "1")]
    uuid: Option<Vec<u8>>,
    #[prost(int32, tag = "2")]
    done: i32,
}

#[derive(Clone, PartialEq, Message)]
struct WireAttachmentInfo {
    #[prost(string, tag = "1")]
    attachment_identifier: String,
    #[prost(string, tag = "2")]
    type_uti: String,
}

#[derive(Clone, PartialEq, Message)]
struct WireMergeableProto {
    #[prost(message, optional, tag = "2")]
    object: Option<WireMergeableObject>,
}

#[derive(Clone, PartialEq, Message)]
struct WireMergeableObject {
    #[prost(int32, optional, tag = "2")]
    version: Option<i32>,
    #[prost(message, optional, tag = "3")]
    data: Option<WireDataStore>,
}

#[derive(Clone, PartialEq, Message)]
struct WireDataStore {
    #[prost(message, repeated, tag = "3")]
    entries: Vec<WireEntry>,
    #[prost(string, repeated, tag = "4")]
    keys: Vec<String>,
    #[prost(string, repeated, tag = "5")]
    types: Vec<String>,
    #[prost(bytes = "vec", repeated, tag = "6")]
    uuids: Vec<Vec<u8>>,
}

#[derive(Clone, PartialEq, Message)]
struct WireEntry {
    #[prost(message, optional, tag = "6")]
    dictionary: Option<WireDictionary>,
    #[prost(message, optional, tag = "10")]
    note: Option<WireNote>,
    #[prost(message, optional, tag = "13")]
    custom_map: Option<WireMap>,
    #[prost(message, optional, tag = "16")]
    ordered_set: Option<WireOrderedSet>,
}

#[derive(Clone, PartialEq, Message)]
struct WireMap {
    #[prost(int32, optional, tag = "1")]
    type_index: Option<i32>,
    #[prost(message, repeated, tag = "3")]
    entries: Vec<WireMapEntry>,
}

#[derive(Clone, PartialEq, Message)]
struct WireMapEntry {
    #[prost(int32, tag = "1")]
    key: i32,
    #[prost(message, optional, tag = "2")]
    value: Option<WireObjectId>,
}

#[derive(Clone, PartialEq, Message)]
struct WireObjectId {
    #[prost(uint64, optional, tag = "2")]
    unsigned_integer_value: Option<u64>,
    #[prost(string, optional, tag = "4")]
    string_value: Option<String>,
    #[prost(int32, optional, tag = "6")]
    object_index: Option<i32>,
}

#[derive(Clone, PartialEq, Message)]
struct WireDictionary {
    #[prost(message, repeated, tag = "1")]
    elements: Vec<WireDictionaryElement>,
}

#[derive(Clone, PartialEq, Message)]
struct WireDictionaryElement {
    #[prost(message, optional, tag = "1")]
    key: Option<WireObjectId>,
    #[prost(message, optional, tag = "2")]
    value: Option<WireObjectId>,
}

#[derive(Clone, PartialEq, Message)]
struct WireOrderedSet {
    #[prost(message, optional, tag = "1")]
    ordering: Option<WireOrdering>,
}

#[derive(Clone, PartialEq, Message)]
struct WireOrdering {
    #[prost(message, optional, tag = "1")]
    array: Option<WireOrderedArray>,
    #[prost(message, optional, tag = "2")]
    contents: Option<WireDictionary>,
}

#[derive(Clone, PartialEq, Message)]
struct WireOrderedArray {
    #[prost(message, repeated, tag = "2")]
    attachments: Vec<WireOrderedAttachment>,
}

#[derive(Clone, PartialEq, Message)]
struct WireOrderedAttachment {
    #[prost(int32, optional, tag = "1")]
    index: Option<i32>,
    #[prost(bytes = "vec", tag = "2")]
    uuid: Vec<u8>,
}

pub fn build_full_fixture(directory: &Path) -> PathBuf {
    let (connection, database_path) = build_base(directory);
    add_note(
        &connection,
        100,
        "Overview stored",
        "note-overview",
        21,
        Some(700_000_000.0),
        Some(700_000_010.0),
        1,
        0,
        Some(note_payload(&[
            Run::style("Overview\n", 0),
            Run::style("Details\n", 1),
            Run {
                text: "bold".into(),
                font_weight: Some(1),
                ..Run::default()
            },
            Run::text(" and "),
            Run {
                text: "italic".into(),
                font_weight: Some(2),
                ..Run::default()
            },
            Run::text(" and "),
            Run {
                text: "both".into(),
                font_weight: Some(3),
                ..Run::default()
            },
            Run::text(" and "),
            Run {
                text: "under".into(),
                underlined: Some(true),
                ..Run::default()
            },
            Run::text(" "),
            Run {
                text: "up".into(),
                superscript: Some(1),
                ..Run::default()
            },
            Run::text(" "),
            Run {
                text: "down".into(),
                superscript: Some(-1),
                ..Run::default()
            },
            Run::text(" "),
            Run {
                text: "strike".into(),
                strikethrough: Some(true),
                ..Run::default()
            },
            Run::text(" "),
            Run {
                text: "highlight".into(),
                emphasis_color: Some(3),
                ..Run::default()
            },
            Run::text("\n"),
            Run {
                text: "web".into(),
                link: Some("https://example.test/path".into()),
                ..Run::default()
            },
            Run::text("\n"),
            Run::attachment("HASH-1", "com.apple.notes.inlinetextattachment.hashtag"),
            Run::text(" "),
            Run::attachment("HASH-2", "com.apple.notes.inlinetextattachment.hashtag"),
            Run::text(" "),
            Run::attachment("MENTION-1", "com.apple.notes.inlinetextattachment.mention"),
            Run::text("\n"),
            Run::attachment("LINK-1", "com.apple.notes.inlinetextattachment.link"),
            Run::text("\n"),
            Run::attachment("CARD-1", "public.url"),
            Run::text("\n"),
            Run {
                text: "Centered\n".into(),
                alignment: Some(1),
                ..Run::default()
            },
            Run {
                text: "Quoted\n".into(),
                blockquote: true,
                ..Run::default()
            },
            Run::style("First\n", 100),
            Run {
                text: "Child\n".into(),
                style: Some(100),
                indent: Some(1),
                ..Run::default()
            },
            Run {
                text: "Done\n".into(),
                style: Some(103),
                checked: Some(true),
                ..Run::default()
            },
            Run::style("Number one\n", 102),
            Run::style("Number two\n", 102),
            Run::style("let x = 1;\n", 4),
            Run::text("After code."),
        ])),
    );
    add_note(
        &connection,
        101,
        "Linked note",
        "note-target",
        22,
        Some(700_000_020.0),
        Some(700_000_030.0),
        0,
        0,
        Some(note_payload(&[
            Run::style("Linked note\n", 0),
            Run::text("Target body."),
        ])),
    );
    add_note(
        &connection,
        102,
        "Files",
        "note-files",
        21,
        Some(700_000_040.0),
        Some(700_000_050.0),
        0,
        0,
        Some(note_payload(&[
            Run::style("Files\n", 0),
            Run::attachment("IMAGE-1", "public.png"),
            Run::text("\n"),
            Run::attachment("PDF-1", "com.adobe.pdf"),
            Run::text("\n"),
            Run::attachment("AUDIO-1", "public.mp3"),
            Run::text("\n"),
            Run::attachment("VIDEO-1", "public.movie"),
            Run::text("\n"),
            Run::attachment("TABLE-1", "com.apple.notes.table"),
            Run::text("\n"),
            Run::attachment("SCAN-1", "com.apple.notes.gallery"),
            Run::text("\n"),
            Run::attachment("DRAWING-1", "com.apple.paper"),
            Run::text("\n"),
            Run::attachment("MISSING-1", "public.png"),
            Run::text("\n"),
            Run::attachment("MISSING-MEDIA-1", "public.png"),
            Run::text("\n"),
            Run::attachment("NOT-DOWNLOADED-1", "public.png"),
        ])),
    );
    add_note(
        &connection,
        103,
        "Deleted",
        "note-deleted",
        23,
        Some(700_000_060.0),
        Some(700_000_070.0),
        0,
        0,
        Some(note_payload(&[
            Run::style("Deleted\n", 0),
            Run::text("Kept in Trash."),
        ])),
    );
    add_note(
        &connection,
        104,
        "URL title",
        "note-url-title",
        22,
        Some(700_000_080.0),
        Some(700_000_080.0),
        0,
        0,
        Some(note_payload(&[
            Run::text("https://example.test/\n"),
            Run::text("Still in body."),
        ])),
    );
    add_note(
        &connection,
        105,
        "Nested list",
        "note-nested-list",
        21,
        Some(700_000_090.0),
        Some(700_000_090.0),
        0,
        0,
        Some(note_payload(&[
            Run {
                text: "Nested first\n".into(),
                style: Some(100),
                indent: Some(0),
                ..Run::default()
            },
            Run {
                text: "Deeper\n".into(),
                style: Some(100),
                indent: Some(1),
                ..Run::default()
            },
        ])),
    );
    add_note(
        &connection,
        106,
        "Attachment only",
        "note-attachment-only",
        21,
        Some(700_000_100.0),
        Some(700_000_100.0),
        0,
        0,
        Some(note_payload(&[
            Run::attachment("IMAGE-ONLY", "public.png"),
            Run::text("\n"),
        ])),
    );
    add_note(
        &connection,
        107,
        "Locked",
        "note-locked",
        22,
        None,
        None,
        0,
        1,
        Some(note_payload(&[Run::text("Secret")])),
    );
    add_note(
        &connection,
        108,
        "Body missing",
        "note-body-missing",
        22,
        None,
        None,
        0,
        0,
        None,
    );

    add_attachment(
        &connection,
        200,
        "HASH-1",
        None,
        Some("release"),
        None,
        None,
        None,
        None,
        None,
        None,
    );
    add_attachment(
        &connection,
        215,
        "HASH-2",
        None,
        Some("Release"),
        None,
        None,
        None,
        None,
        None,
        None,
    );
    add_attachment(
        &connection,
        201,
        "MENTION-1",
        None,
        Some("Ada"),
        None,
        None,
        None,
        None,
        None,
        None,
    );
    add_attachment(
        &connection,
        202,
        "LINK-1",
        None,
        Some("Linked note"),
        Some("applenotes:note/note-target"),
        None,
        None,
        None,
        None,
        None,
    );
    add_attachment(
        &connection,
        203,
        "CARD-1",
        None,
        None,
        None,
        Some("https://example.test/card"),
        Some("Example card"),
        None,
        None,
        None,
    );

    let table = table_payload();
    add_attachment(
        &connection,
        204,
        "TABLE-1",
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&table),
        None,
    );
    let scan = scan_payload("SCAN-PAGE-1");
    add_attachment(
        &connection,
        205,
        "SCAN-1",
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&scan),
        None,
    );
    add_attachment(
        &connection,
        206,
        "DRAWING-1",
        None,
        None,
        None,
        None,
        None,
        Some("GEN-1"),
        Some(b"drawing"),
        Some((90, 60)),
    );
    add_attachment(
        &connection,
        207,
        "IMAGE-1",
        Some(300),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );
    add_attachment(
        &connection,
        208,
        "PDF-1",
        Some(301),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );
    add_attachment(
        &connection,
        209,
        "AUDIO-1",
        Some(302),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );
    add_attachment(
        &connection,
        210,
        "VIDEO-1",
        Some(303),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );
    add_attachment(
        &connection,
        211,
        "IMAGE-ONLY",
        Some(304),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );
    add_attachment(
        &connection,
        212,
        "SCAN-PAGE-1",
        Some(305),
        None,
        None,
        None,
        None,
        None,
        None,
        Some((20, 30)),
    );
    add_attachment(
        &connection,
        213,
        "MISSING-MEDIA-1",
        Some(399),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );
    add_attachment(
        &connection,
        214,
        "NOT-DOWNLOADED-1",
        Some(306),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );
    add_media(&connection, 300, "MEDIA-IMAGE", "photo.png", None);
    add_media(&connection, 301, "MEDIA-PDF", "document.pdf", None);
    add_media(&connection, 302, "MEDIA-AUDIO", "voice.mp3", None);
    add_media(&connection, 303, "MEDIA-VIDEO", "clip.mp4", None);
    add_media(&connection, 304, "MEDIA-ONLY", "only.jpg", None);
    add_media(&connection, 305, "MEDIA-SCAN", "scan-original.jpg", None);
    add_media(
        &connection,
        306,
        "MEDIA-NOT-DOWNLOADED",
        "not-downloaded.png",
        None,
    );

    drop(connection);
    write_attachment(
        directory,
        "Accounts/ACC-A/Media/MEDIA-IMAGE/photo.png",
        b"PNG synthetic",
    );
    write_attachment(
        directory,
        "Accounts/ACC-A/Media/MEDIA-PDF/document.pdf",
        b"PDF synthetic",
    );
    write_attachment(
        directory,
        "Accounts/ACC-A/Media/MEDIA-AUDIO/voice.mp3",
        b"MP3 synthetic",
    );
    write_attachment(
        directory,
        "Accounts/ACC-A/Media/MEDIA-VIDEO/clip.mp4",
        b"MP4 synthetic",
    );
    write_attachment(
        directory,
        "Accounts/ACC-A/Media/MEDIA-ONLY/only.jpg",
        b"JPEG synthetic",
    );
    write_attachment(
        directory,
        "Accounts/ACC-A/Previews/SCAN-PAGE-1-1-20x30-0.jpeg",
        b"scan preview",
    );
    write_attachment(
        directory,
        "Accounts/ACC-A/FallbackImages/DRAWING-1/GEN-1/FallbackImage.png",
        b"drawing preview",
    );
    database_path
}

pub fn build_golden_wire_fixture(directory: &Path) -> PathBuf {
    let (connection, database_path) = build_base(directory);
    // Independently specified protobuf wire bytes. Document(version=1, note={
    // noteText="Hi", attributeRun=[{length=2}]}) uses tags 2, 3, 2 and 5.
    let wire = decode_hex(GOLDEN_WIRE_HEX);
    add_note(
        &connection,
        100,
        "Stored title",
        "golden-note",
        21,
        Some(700_000_000.0),
        Some(700_000_000.0),
        0,
        0,
        Some(gzip(&wire)),
    );
    drop(connection);
    database_path
}

/// Rows marked for deletion, and inline attachments beyond tags, mentions
/// and note links.
pub fn build_deletion_and_inline_fixture(directory: &Path) -> PathBuf {
    let (connection, database_path) = build_base(directory);
    add_folder(
        &connection,
        24,
        "Purged",
        "purged-folder",
        None,
        ACCOUNT_A,
        0,
    );
    mark_for_deletion(&connection, 24);
    add_note(
        &connection,
        100,
        "Live",
        "note-live",
        21,
        None,
        None,
        0,
        0,
        Some(note_payload(&[
            Run::text("Live\n"),
            Run::text("Total "),
            Run::attachment(
                "CALC-1",
                "com.apple.notes.inlinetextattachment.calculateresult",
            ),
            Run::text(" and "),
            Run::attachment(
                "CALC-EMPTY",
                "com.apple.notes.inlinetextattachment.calculategraphexpression",
            ),
        ])),
    );
    add_note(
        &connection,
        101,
        "Purged note",
        "note-purged",
        21,
        None,
        None,
        0,
        0,
        Some(note_payload(&[Run::text("Purged\n"), Run::text("Gone.")])),
    );
    mark_for_deletion(&connection, 101);
    add_note(
        &connection,
        102,
        "Orphan",
        "note-in-purged-folder",
        24,
        None,
        None,
        0,
        0,
        Some(note_payload(&[Run::text("Orphan\n"), Run::text("Body.")])),
    );
    for (pk, identifier, alt_text) in [(200, "CALC-1", Some("= 42")), (201, "CALC-EMPTY", None)] {
        add_attachment(
            &connection,
            pk,
            identifier,
            None,
            alt_text,
            None,
            None,
            None,
            None,
            None,
            None,
        );
    }
    drop(connection);
    database_path
}

pub fn build_malformed_fixture(directory: &Path) -> PathBuf {
    let (connection, database_path) = build_base(directory);
    add_note(
        &connection,
        100,
        "Missing body",
        "missing",
        21,
        None,
        None,
        0,
        0,
        None,
    );
    add_note(
        &connection,
        101,
        "Bad gzip",
        "bad-gzip",
        21,
        None,
        None,
        0,
        0,
        Some(b"not gzip".to_vec()),
    );
    add_note(
        &connection,
        102,
        "Bad protobuf",
        "bad-protobuf",
        21,
        None,
        None,
        0,
        0,
        Some(gzip(&[0x1a, 0x08, 0x12])),
    );
    add_note(
        &connection,
        103,
        "Locked without data",
        "locked",
        21,
        None,
        None,
        0,
        1,
        None,
    );
    drop(connection);
    database_path
}

fn build_base(directory: &Path) -> (Connection, PathBuf) {
    std::fs::create_dir_all(directory).expect("create fixture folder");
    let database_path = directory.join("NoteStore.sqlite");
    let connection = Connection::open(&database_path).expect("create fixture database");
    connection
        .execute_batch(
            "CREATE TABLE z_primarykey (Z_ENT INTEGER, Z_NAME TEXT);
         CREATE TABLE ziccloudsyncingobject (
           Z_PK INTEGER PRIMARY KEY, Z_ENT INTEGER,
           ZTITLE1 TEXT, ZTITLE2 TEXT, ZNAME TEXT, ZIDENTIFIER TEXT,
           ZALTTEXT TEXT, ZTOKENCONTENTIDENTIFIER TEXT, ZURLSTRING TEXT, ZTITLE TEXT,
           ZFOLDER INTEGER, ZFOLDERTYPE INTEGER, ZPARENT INTEGER, ZOWNER INTEGER,
           ZNOTE INTEGER, ZMEDIA INTEGER, ZMERGEABLEDATA1 BLOB, ZHANDWRITINGSUMMARY TEXT,
           ZFILENAME TEXT, ZTYPEUTI TEXT, ZGENERATION1 TEXT, ZFALLBACKIMAGEGENERATION TEXT,
           ZFALLBACKPDFGENERATION TEXT, ZSIZEWIDTH INTEGER, ZSIZEHEIGHT INTEGER,
           ZCREATIONDATE INTEGER, ZMODIFICATIONDATE INTEGER,
           ZCREATIONDATE1 REAL, ZCREATIONDATE2 REAL, ZCREATIONDATE3 REAL,
           ZMODIFICATIONDATE1 REAL, ZISPASSWORDPROTECTED INTEGER,
           ZISPINNED INTEGER, ZMARKEDFORDELETION INTEGER
         );
         CREATE TABLE zicnotedata (Z_PK INTEGER PRIMARY KEY, ZNOTE INTEGER, ZDATA BLOB);",
        )
        .expect("create Core Data tables");
    for (id, name) in [
        (ACCOUNT, "ICAccount"),
        (FOLDER, "ICFolder"),
        (NOTE, "ICNote"),
        (ATTACHMENT, "ICAttachment"),
        (MEDIA, "ICMedia"),
    ] {
        connection
            .execute(
                "INSERT INTO z_primarykey (Z_ENT, Z_NAME) VALUES (?1, ?2)",
                (id, name),
            )
            .expect("insert dynamic entity identifier");
    }
    insert_object(
        &connection,
        ACCOUNT_A,
        ACCOUNT,
        [("ZNAME", text("iCloud")), ("ZIDENTIFIER", text("ACC-A"))],
    );
    insert_object(
        &connection,
        ACCOUNT_B,
        ACCOUNT,
        [("ZNAME", text("On My Mac")), ("ZIDENTIFIER", text("ACC-B"))],
    );
    add_folder(&connection, 20, "Work", "work-folder", None, ACCOUNT_A, 0);
    add_folder(
        &connection,
        21,
        "Projects",
        "projects-folder",
        Some(20),
        ACCOUNT_A,
        0,
    );
    add_folder(
        &connection,
        22,
        "Inbox",
        "DefaultFolderNotes",
        None,
        ACCOUNT_B,
        0,
    );
    add_folder(
        &connection,
        23,
        "Recently Deleted",
        "trash-folder",
        None,
        ACCOUNT_B,
        1,
    );
    (connection, database_path)
}

fn add_folder(
    connection: &Connection,
    pk: i64,
    title: &str,
    identifier: &str,
    parent: Option<i64>,
    owner: i64,
    kind: i64,
) {
    insert_object(
        connection,
        pk,
        FOLDER,
        [
            ("ZTITLE2", text(title)),
            ("ZIDENTIFIER", text(identifier)),
            ("ZPARENT", parent.map(Value::Integer).unwrap_or(Value::Null)),
            ("ZOWNER", Value::Integer(owner)),
            ("ZFOLDERTYPE", Value::Integer(kind)),
        ],
    );
}

fn mark_for_deletion(connection: &Connection, pk: i64) {
    connection
        .execute(
            "UPDATE ZICCloudSyncingObject SET ZMARKEDFORDELETION = 1 WHERE Z_PK = ?1",
            [pk],
        )
        .expect("mark Core Data object for deletion");
}

#[allow(clippy::too_many_arguments)]
fn add_note(
    connection: &Connection,
    pk: i64,
    title: &str,
    identifier: &str,
    folder: i64,
    created: Option<f64>,
    modified: Option<f64>,
    pinned: i64,
    locked: i64,
    body: Option<Vec<u8>>,
) {
    insert_object(
        connection,
        pk,
        NOTE,
        [
            ("ZTITLE1", text(title)),
            ("ZIDENTIFIER", text(identifier)),
            ("ZFOLDER", Value::Integer(folder)),
            (
                "ZCREATIONDATE1",
                created.map(Value::Real).unwrap_or(Value::Null),
            ),
            (
                "ZMODIFICATIONDATE1",
                modified.map(Value::Real).unwrap_or(Value::Null),
            ),
            ("ZISPASSWORDPROTECTED", Value::Integer(locked)),
            ("ZISPINNED", Value::Integer(pinned)),
            ("ZMARKEDFORDELETION", Value::Integer(0)),
        ],
    );
    if let Some(body) = body {
        connection
            .execute(
                "INSERT INTO zicnotedata (Z_PK, ZNOTE, ZDATA) VALUES (?1, ?1, ?2)",
                (pk, body),
            )
            .expect("insert compressed note body");
    }
}

#[allow(clippy::too_many_arguments)]
fn add_attachment(
    connection: &Connection,
    pk: i64,
    identifier: &str,
    media: Option<i64>,
    alt_text: Option<&str>,
    target: Option<&str>,
    url: Option<&str>,
    title: Option<&str>,
    fallback_image_generation: Option<&str>,
    mergeable: Option<&[u8]>,
    size: Option<(i64, i64)>,
) {
    let values = [
        ("ZIDENTIFIER", text(identifier)),
        ("ZMEDIA", media.map(Value::Integer).unwrap_or(Value::Null)),
        ("ZALTTEXT", alt_text.map(text).unwrap_or(Value::Null)),
        (
            "ZTOKENCONTENTIDENTIFIER",
            target.map(text).unwrap_or(Value::Null),
        ),
        ("ZURLSTRING", url.map(text).unwrap_or(Value::Null)),
        ("ZTITLE", title.map(text).unwrap_or(Value::Null)),
        (
            "ZFALLBACKIMAGEGENERATION",
            fallback_image_generation.map(text).unwrap_or(Value::Null),
        ),
        (
            "ZMERGEABLEDATA1",
            mergeable
                .map(|data| Value::Blob(data.to_vec()))
                .unwrap_or(Value::Null),
        ),
        (
            "ZSIZEWIDTH",
            size.map(|value| Value::Integer(value.0))
                .unwrap_or(Value::Integer(0)),
        ),
        (
            "ZSIZEHEIGHT",
            size.map(|value| Value::Integer(value.1))
                .unwrap_or(Value::Integer(0)),
        ),
        ("ZNOTE", Value::Integer(102)),
    ];
    insert_object(connection, pk, ATTACHMENT, values);
}

fn add_media(
    connection: &Connection,
    pk: i64,
    identifier: &str,
    filename: &str,
    generation: Option<&str>,
) {
    insert_object(
        connection,
        pk,
        MEDIA,
        [
            ("ZIDENTIFIER", text(identifier)),
            ("ZFILENAME", text(filename)),
            ("ZGENERATION1", generation.map(text).unwrap_or(Value::Null)),
        ],
    );
}

fn insert_object<const N: usize>(
    connection: &Connection,
    pk: i64,
    entity: i64,
    fields: [(&str, Value); N],
) {
    let mut columns = vec!["Z_PK", "Z_ENT"];
    columns.extend(fields.iter().map(|(name, _)| *name));
    let placeholders = (1..=columns.len())
        .map(|index| format!("?{index}"))
        .collect::<Vec<_>>();
    let sql = format!(
        "INSERT INTO ZICCloudSyncingObject ({}) VALUES ({})",
        columns.join(", "),
        placeholders.join(", ")
    );
    let mut values = vec![Value::Integer(pk), Value::Integer(entity)];
    values.extend(fields.into_iter().map(|(_, value)| value));
    connection
        .execute(&sql, params_from_iter(values.iter()))
        .expect("insert Core Data object");
}

fn note_payload(runs: &[Run]) -> Vec<u8> {
    let mut text = String::new();
    let mut attribute_run = Vec::new();
    for run in runs {
        let value = if run.attachment.is_some() {
            "\u{fffc}"
        } else {
            &run.text
        };
        text.push_str(value);
        let paragraph_style = if run.style.is_some()
            || run.indent.is_some()
            || run.checked.is_some()
            || run.alignment.is_some()
            || run.blockquote
        {
            Some(WireParagraphStyle {
                style_type: run.style,
                alignment: run.alignment,
                indent_amount: run.indent,
                checklist: run.checked.map(|checked| WireChecklist {
                    uuid: Some(vec![1]),
                    done: i32::from(checked),
                }),
                blockquote: run.blockquote.then_some(1),
            })
        } else {
            None
        };
        attribute_run.push(WireAttributeRun {
            length: value.encode_utf16().count() as i32,
            paragraph_style,
            font_weight: run.font_weight,
            underlined: run.underlined.map(i32::from),
            strikethrough: run.strikethrough.map(i32::from),
            superscript: run.superscript,
            link: run.link.clone(),
            attachment_info: run
                .attachment
                .as_ref()
                .map(|(identifier, uti)| WireAttachmentInfo {
                    attachment_identifier: identifier.clone(),
                    type_uti: uti.clone(),
                }),
            emphasis_color: run.emphasis_color,
        });
    }
    gzip(
        &WireDocument {
            version: Some(1),
            note: Some(WireNote {
                note_text: text,
                attribute_run,
            }),
        }
        .encode_to_vec(),
    )
}

fn scan_payload(identifier: &str) -> Vec<u8> {
    let entry = WireEntry {
        custom_map: Some(WireMap {
            type_index: Some(0),
            entries: vec![WireMapEntry {
                key: 0,
                value: Some(WireObjectId {
                    string_value: Some(identifier.to_owned()),
                    ..WireObjectId::default()
                }),
            }],
        }),
        ..WireEntry::default()
    };
    gzip(
        &WireMergeableProto {
            object: Some(WireMergeableObject {
                version: Some(1),
                data: Some(WireDataStore {
                    entries: vec![entry],
                    keys: vec![],
                    types: vec![],
                    uuids: vec![],
                }),
            }),
        }
        .encode_to_vec(),
    )
}

fn table_payload() -> Vec<u8> {
    let mut entries = vec![WireEntry::default(); 18];
    let uuids = vec![vec![1], vec![2], vec![3], vec![4]];
    entries[0].custom_map = Some(WireMap {
        type_index: Some(0),
        entries: vec![map_ref(0, 1), map_ref(1, 2), map_ref(2, 3)],
    });
    entries[1].ordered_set = Some(ordered_set(0, 1, 1));
    entries[2].ordered_set = Some(ordered_set(2, 3, 3));
    entries[3].dictionary = Some(WireDictionary {
        elements: vec![dict_ref(12, 4)],
    });
    entries[4].dictionary = Some(WireDictionary {
        elements: vec![dict_ref(13, 5)],
    });
    entries[5].note = Some(WireNote {
        note_text: "Cell value".to_owned(),
        attribute_run: vec![WireAttributeRun {
            length: 10,
            ..WireAttributeRun::default()
        }],
    });
    for (object_index, uuid_index) in [
        (6, 0),
        (7, 1),
        (8, 2),
        (9, 3),
        (10, 1),
        (11, 3),
        (12, 3),
        (13, 1),
    ] {
        entries[object_index].custom_map = Some(WireMap {
            type_index: Some(1),
            entries: vec![WireMapEntry {
                key: 0,
                value: Some(WireObjectId {
                    unsigned_integer_value: Some(uuid_index),
                    ..WireObjectId::default()
                }),
            }],
        });
    }
    // Ordered row and column UUIDs are UUID slots 0 and 2. Their CRDT
    // mappings point to row/column identities in slots 1 and 3.
    entries[1].ordered_set = Some(WireOrderedSet {
        ordering: Some(WireOrdering {
            array: Some(WireOrderedArray {
                attachments: vec![ordered_attachment(&uuids[0])],
            }),
            contents: Some(WireDictionary {
                elements: vec![dict_ref(6, 7)],
            }),
        }),
    });
    entries[2].ordered_set = Some(WireOrderedSet {
        ordering: Some(WireOrdering {
            array: Some(WireOrderedArray {
                attachments: vec![ordered_attachment(&uuids[2])],
            }),
            contents: Some(WireDictionary {
                elements: vec![dict_ref(8, 9)],
            }),
        }),
    });
    entries[3].dictionary = Some(WireDictionary {
        elements: vec![dict_ref(11, 4)],
    });
    entries[4].dictionary = Some(WireDictionary {
        elements: vec![dict_ref(10, 5)],
    });
    gzip(
        &WireMergeableProto {
            object: Some(WireMergeableObject {
                version: Some(1),
                data: Some(WireDataStore {
                    entries,
                    keys: vec!["crRows".into(), "crColumns".into(), "cellColumns".into()],
                    types: vec![
                        "com.apple.notes.ICTable".into(),
                        "com.apple.CRDT.NSUUID".into(),
                    ],
                    uuids,
                }),
            }),
        }
        .encode_to_vec(),
    )
}

fn ordered_set(
    order_uuid_index: usize,
    target_uuid_index: usize,
    ref_object_index: usize,
) -> WireOrderedSet {
    let _ = (order_uuid_index, target_uuid_index, ref_object_index);
    WireOrderedSet::default()
}

fn map_ref(key: i32, object_index: i32) -> WireMapEntry {
    WireMapEntry {
        key,
        value: Some(WireObjectId {
            object_index: Some(object_index),
            ..WireObjectId::default()
        }),
    }
}

fn dict_ref(key_object_index: i32, value_object_index: i32) -> WireDictionaryElement {
    WireDictionaryElement {
        key: Some(WireObjectId {
            object_index: Some(key_object_index),
            ..WireObjectId::default()
        }),
        value: Some(WireObjectId {
            object_index: Some(value_object_index),
            ..WireObjectId::default()
        }),
    }
}

fn ordered_attachment(uuid: &[u8]) -> WireOrderedAttachment {
    WireOrderedAttachment {
        index: Some(0),
        uuid: uuid.to_vec(),
    }
}

fn gzip(bytes: &[u8]) -> Vec<u8> {
    let mut encoder = GzBuilder::new()
        .mtime(0)
        .write(Vec::new(), Compression::new(6));
    encoder
        .write_all(bytes)
        .expect("write deterministic gzip payload");
    encoder.finish().expect("finish deterministic gzip payload")
}

fn decode_hex(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|&[high, low]| {
            let digit = |byte: u8| match byte {
                b'0'..=b'9' => byte - b'0',
                b'a'..=b'f' => byte - b'a' + 10,
                _ => panic!("invalid golden hex fixture"),
            };
            digit(high) * 16 + digit(low)
        })
        .collect()
}

fn write_attachment(root: &Path, relative: &str, bytes: &[u8]) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().expect("attachment parent"))
        .expect("create media folder");
    std::fs::write(path, bytes).expect("write synthetic attachment");
}

fn text(value: &str) -> Value {
    Value::Text(value.to_owned())
}
