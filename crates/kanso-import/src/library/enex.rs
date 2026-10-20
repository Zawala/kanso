//! Evernote's `.enex` exports: one file per notebook, or a folder of them.
//!
//! An export is XML with one `<note>` after another, each carrying its
//! title, dates, tags, its content as ENML (Evernote's XHTML, inside CDATA)
//! and its attachments as base64 `<resource>`s. The file is streamed one
//! note at a time, and resources are decoded only when notes are written.
//! ENML becomes Markdown with [`enml_to_markdown`]; an `<en-media>` points
//! at its resource by MD5, which is also the attachment's key, so a picture
//! in several notes is one attachment.

use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader};
use std::path::{Path, PathBuf};

use anyhow::Context as _;
use base64::Engine as _;
use base64::alphabet;
use base64::engine::{DecodePaddingMode, GeneralPurpose, GeneralPurposeConfig};
use kanso_store::Attachment;
use quick_xml::events::{BytesRef, BytesStart, Event};
use quick_xml::reader::Reader;

use crate::library::tree::{TreeFile, extension, stem};
use crate::library::{Embedded, Skipped, Source, SourceNote, dates};

/// Resource data as Evernote writes it, wrapped at 76 columns; padding is
/// taken as optional.
const BASE64: GeneralPurpose = GeneralPurpose::new(
    &alphabet::STANDARD,
    GeneralPurposeConfig::new()
        .with_decode_padding_mode(DecodePaddingMode::Indifferent)
        .with_decode_allow_trailing_bits(true),
);

/// Errors in one export after which the rest of it is given up.
const MAX_ERRORS: usize = 100;

/// The placeholder for an `<en-crypt>` section.
pub const ENCRYPTED_PLACEHOLDER: &str = "[Encrypted content not imported]";

/// One or more `.enex` files.
pub struct EnexSource {
    name: String,
    exports: Vec<Export>,
    /// A folder of exports: each is a notebook.
    folder: bool,
    /// What opening the folder left out.
    unread: Vec<Skipped>,
    /// What the last pass over the notes left out.
    skipped: Vec<Skipped>,
}

/// One `.enex` file.
#[derive(Clone, Debug)]
struct Export {
    path: PathBuf,
    /// Its name in the folder (`Travel.enex`).
    file: String,
    /// Its name without `.enex`: the notebook's name.
    stem: String,
}

impl Export {
    fn new(path: PathBuf) -> Export {
        let file = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let stem = stem(&file).to_owned();
        Export { path, file, stem }
    }
}

/// Opens an `.enex` file (one notebook, named after the file), or a folder
/// of them (a notebook each, inside one named after the folder).
pub fn open(path: &Path) -> anyhow::Result<EnexSource> {
    if !path.is_dir() {
        let export = Export::new(path.to_path_buf());
        return Ok(EnexSource {
            name: export.stem.clone(),
            exports: vec![export],
            folder: false,
            unread: Vec::new(),
            skipped: Vec::new(),
        });
    }
    let mut exports = Vec::new();
    let mut unread = Vec::new();
    let entries =
        std::fs::read_dir(path).with_context(|| format!("cannot read {}", path.display()))?;
    for entry in entries {
        let entry = entry.with_context(|| format!("cannot read {}", path.display()))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') {
            continue;
        }
        if entry.path().is_dir() {
            unread.push(Skipped::new(
                format!("{name}/"),
                "folders inside the export folder aren't read",
            ));
        } else if extension(&name) == "enex" {
            exports.push(Export::new(entry.path()));
        } else {
            unread.push(Skipped::new(name, "not an Evernote export (.enex)"));
        }
    }
    anyhow::ensure!(
        !exports.is_empty(),
        "{} has no Evernote exports (.enex files)",
        path.display()
    );
    exports.sort_by(|a, b| a.file.cmp(&b.file));
    unread.sort_by(|a, b| a.path.cmp(&b.path));
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Evernote".to_owned());
    Ok(EnexSource {
        name,
        exports,
        folder: true,
        unread,
        skipped: Vec::new(),
    })
}

impl Source for EnexSource {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn files(&self) -> &[TreeFile] {
        &[]
    }

    fn notes(
        &mut self,
        full: bool,
        each: &mut dyn FnMut(SourceNote) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        self.skipped.clear();
        for export in &self.exports {
            let mut pass = Pass {
                export,
                folder: self.folder,
                full,
                skipped: &mut self.skipped,
            };
            pass.read(each)?;
        }
        Ok(())
    }

    fn read(&mut self, path: &str, _limit: u64) -> io::Result<Vec<u8>> {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{path} isn't in an Evernote export"),
        ))
    }

    fn skipped(&mut self) -> Vec<Skipped> {
        let mut skipped = self.unread.clone();
        skipped.append(&mut self.skipped);
        skipped
    }
}

/// A note as the export has it, before its content is converted.
#[derive(Debug, Default)]
struct RawNote {
    /// Elements open inside `<note>`, innermost last.
    stack: Vec<String>,
    title: String,
    content: Option<String>,
    created: String,
    updated: String,
    tags: Vec<String>,
    resources: Vec<RawResource>,
    /// Why the note can't be read, if it can't.
    broken: Option<String>,
}

#[derive(Debug, Default)]
struct RawResource {
    /// Base64, kept only when notes are written.
    data: String,
    mime: String,
    file_name: String,
}

impl RawNote {
    fn break_with(&mut self, reason: impl Into<String>) {
        if self.broken.is_none() {
            self.broken = Some(reason.into());
        }
    }

    fn start(&mut self, name: String) {
        match (self.stack.as_slice(), name.as_str()) {
            ([], "content") => {
                self.content.get_or_insert_with(String::new);
            }
            ([], "tag") => self.tags.push(String::new()),
            ([], "resource") => self.resources.push(RawResource::default()),
            _ => {}
        }
        self.stack.push(name);
    }

    /// Closes `name`; false when it closes the note itself.
    fn end(&mut self, name: &str) -> bool {
        match self.stack.iter().rposition(|open| open == name) {
            Some(at) => {
                if at + 1 != self.stack.len() {
                    self.break_with(format!("<{}> isn't closed", self.stack[at + 1]));
                }
                self.stack.truncate(at);
                true
            }
            None if name == "note" => {
                if let Some(open) = self.stack.last() {
                    let reason = format!("<{open}> isn't closed");
                    self.break_with(reason);
                }
                false
            }
            None => {
                self.break_with(format!("</{name}> closes nothing"));
                true
            }
        }
    }

    fn text(&mut self, text: &str, full: bool) {
        let path: Vec<&str> = self.stack.iter().map(String::as_str).collect();
        match path.as_slice() {
            ["title"] => self.title.push_str(text),
            ["content"] => self.content.get_or_insert_with(String::new).push_str(text),
            ["created"] => self.created.push_str(text),
            ["updated"] => self.updated.push_str(text),
            ["tag"] => {
                if let Some(tag) = self.tags.last_mut() {
                    tag.push_str(text);
                }
            }
            ["resource", "data"] if full => {
                if let Some(r) = self.resources.last_mut() {
                    r.data.push_str(text);
                }
            }
            ["resource", "mime"] => {
                if let Some(r) = self.resources.last_mut() {
                    r.mime.push_str(text);
                }
            }
            ["resource", "resource-attributes", "file-name"] => {
                if let Some(r) = self.resources.last_mut() {
                    r.file_name.push_str(text);
                }
            }
            _ => {}
        }
    }
}

/// One pass over one export.
struct Pass<'a> {
    export: &'a Export,
    folder: bool,
    full: bool,
    skipped: &'a mut Vec<Skipped>,
}

impl Pass<'_> {
    fn read(
        &mut self,
        each: &mut dyn FnMut(SourceNote) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        let file = File::open(&self.export.path)
            .with_context(|| format!("cannot open {}", self.export.path.display()))?;
        let mut reader = Reader::from_reader(BufReader::new(file));
        let config = reader.config_mut();
        config.allow_dangling_amp = true;
        // Mismatched tags are found below, so one bad note is skipped
        // rather than ending the export.
        config.check_end_names = false;
        config.allow_unmatched_ends = true;
        let mut buf = Vec::new();
        let mut note: Option<RawNote> = None;
        let mut errors = 0;
        let mut last_error = None;
        loop {
            let event = reader.read_event_into(&mut buf);
            match event {
                Err(err) => {
                    errors += 1;
                    let at = reader.error_position();
                    if let Some(note) = note.as_mut() {
                        note.break_with(format!("it isn't well-formed XML ({err})"));
                    }
                    if errors > MAX_ERRORS || last_error == Some(reader.buffer_position()) {
                        if let Some(note) = note.take() {
                            self.skip(&note);
                        }
                        self.skipped.push(Skipped::new(
                            self.export.file.clone(),
                            format!("the rest of the file after byte {at} isn't readable XML"),
                        ));
                        break;
                    }
                    last_error = Some(reader.buffer_position());
                }
                Ok(Event::Eof) => {
                    if let Some(mut note) = note.take() {
                        note.break_with("the file ends in the middle of it");
                        self.skip(&note);
                    }
                    break;
                }
                Ok(Event::Start(e)) => {
                    let name = tag_name(e.name().as_ref());
                    match note.as_mut() {
                        Some(open) if name == "note" => {
                            open.break_with("it isn't closed");
                            self.skip(open);
                            note = Some(RawNote::default());
                        }
                        Some(open) => open.start(name),
                        None if name == "note" => note = Some(RawNote::default()),
                        None => {}
                    }
                }
                Ok(Event::Empty(e)) => {
                    if let Some(open) = note.as_mut() {
                        let name = tag_name(e.name().as_ref());
                        open.start(name.clone());
                        open.end(&name);
                    }
                }
                Ok(Event::End(e)) => {
                    if let Some(open) = note.as_mut() {
                        let name = tag_name(e.name().as_ref());
                        if !open.end(&name)
                            && let Some(done) = note.take()
                        {
                            self.emit(done, each)?;
                        }
                    }
                }
                Ok(Event::Text(t)) => {
                    if let Some(open) = note.as_mut() {
                        match t.decode() {
                            Ok(text) => open.text(&text, self.full),
                            Err(err) => open.break_with(format!("it isn't UTF-8 ({err})")),
                        }
                    }
                }
                Ok(Event::CData(t)) => {
                    if let Some(open) = note.as_mut() {
                        match t.decode() {
                            Ok(text) => open.text(&text, self.full),
                            Err(err) => open.break_with(format!("it isn't UTF-8 ({err})")),
                        }
                    }
                }
                Ok(Event::GeneralRef(r)) => {
                    if let Some(open) = note.as_mut() {
                        open.text(&reference(&r), self.full);
                    }
                }
                Ok(_) => {}
            }
            buf.clear();
        }
        Ok(())
    }

    /// What a skipped note is called in the report.
    fn label(&self, title: &str) -> String {
        let title = title.trim();
        let title = if title.is_empty() { "Untitled" } else { title };
        format!("{} › {title}", self.export.file)
    }

    fn skip(&mut self, note: &RawNote) {
        let reason = note
            .broken
            .clone()
            .unwrap_or_else(|| "it can't be read".to_owned());
        self.skipped
            .push(Skipped::new(self.label(&note.title), reason));
    }

    /// Hands a whole `<note>` on, or skips it.
    fn emit(
        &mut self,
        raw: RawNote,
        each: &mut dyn FnMut(SourceNote) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        if raw.broken.is_some() {
            self.skip(&raw);
            return Ok(());
        }
        let Some(content) = raw.content.as_deref() else {
            self.skipped
                .push(Skipped::new(self.label(&raw.title), "it has no content"));
            return Ok(());
        };
        let title = raw.title.trim().to_owned();
        let created = raw.created.trim();
        let file = if self.folder {
            &self.export.file
        } else {
            &self.export.stem
        };
        let mut note = SourceNote {
            key: format!("{file}/{title}@{created}"),
            folder: if self.folder {
                vec![self.export.stem.clone()]
            } else {
                Vec::new()
            },
            title,
            ..SourceNote::default()
        };
        if self.full {
            note.created = dates::parse(created);
            note.updated = dates::parse(&raw.updated);
            note.tags = raw
                .tags
                .iter()
                .map(|t| t.trim().to_owned())
                .filter(|t| !t.is_empty())
                .collect();
            let mut resources = Resources::new();
            for resource in raw.resources {
                let name = resource_name(&resource.file_name, resource.mime.trim());
                let mut data = resource.data;
                data.retain(|c| !c.is_ascii_whitespace());
                let bytes = match BASE64.decode(data.as_bytes()) {
                    Ok(bytes) => bytes,
                    Err(err) => {
                        self.skipped.push(Skipped::new(
                            format!("{} › {name}", self.label(&note.title)),
                            format!("its data isn't base64 ({err})"),
                        ));
                        continue;
                    }
                };
                let attachment = Attachment::new(bytes);
                let key = attachment.md5_hex();
                let mime = resource.mime.trim().to_owned();
                if resources.contains_key(&key) {
                    continue;
                }
                resources.insert(key.clone(), (name.clone(), mime.clone()));
                note.embedded.push(Embedded {
                    key,
                    name,
                    content_type: (!mime.is_empty()).then_some(mime),
                    data: attachment.data,
                });
            }
            let converted = convert(content, &resources);
            if converted.encrypted {
                self.skipped.push(Skipped::new(
                    self.label(&note.title),
                    "it has encrypted text, which Kanso can't read; the rest was imported",
                ));
            }
            if !converted.complete {
                self.skipped.push(Skipped::new(
                    self.label(&note.title),
                    "part of its content isn't well-formed ENML; what came before was imported",
                ));
            }
            note.body = with_unlinked(converted.markdown, &note.embedded);
        }
        each(note)
    }
}

/// `body` with links to the attachments it doesn't show yet, at the end.
fn with_unlinked(mut body: String, embedded: &[Embedded]) -> String {
    let links: Vec<String> = embedded
        .iter()
        .filter(|e| !body.contains(&format!("kanso-import:embedded/{}", e.key)))
        .map(|e| {
            let image = e
                .content_type
                .as_deref()
                .unwrap_or_else(|| crate::library::content_type(&e.name))
                .starts_with("image/");
            media_link(&e.key, &e.name, image, false)
        })
        .collect();
    if !links.is_empty() {
        if !body.is_empty() {
            body.push_str("\n\n");
        }
        body.push_str(&links.join("\n\n"));
    }
    body
}

/// A resource's file name, or one made from its type.
fn resource_name(file_name: &str, mime: &str) -> String {
    let name = file_name.trim();
    if !name.is_empty() {
        return name.to_owned();
    }
    let ext = match mime {
        "image/png" => "png",
        "image/jpeg" | "image/jpg" | "image/pjpeg" => "jpg",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/svg+xml" => "svg",
        "image/heic" => "heic",
        "application/pdf" => "pdf",
        "text/plain" => "txt",
        "text/html" => "html",
        "audio/mpeg" => "mp3",
        "audio/wav" | "audio/x-wav" => "wav",
        "audio/mp4" | "audio/x-m4a" => "m4a",
        "audio/amr" => "amr",
        "video/mp4" => "mp4",
        "video/quicktime" => "mov",
        _ => "",
    };
    let stem = if mime.starts_with("image/") {
        "image"
    } else {
        "attachment"
    };
    if ext.is_empty() {
        stem.to_owned()
    } else {
        format!("{stem}.{ext}")
    }
}

/// An element's name: lowercased, without a namespace prefix.
fn tag_name(raw: &[u8]) -> String {
    let name = String::from_utf8_lossy(raw);
    name.rsplit(':')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase()
}

/// The text an `&…;` reference stands for, or the reference itself when it
/// isn't known.
fn reference(r: &BytesRef<'_>) -> String {
    match r.decode() {
        Ok(name) => entity(&name).unwrap_or_else(|| format!("&{name};")),
        Err(_) => String::new(),
    }
}

/// The text of the entity `name` (without `&` and `;`): a character
/// reference, or one of the common HTML named entities.
pub fn entity(name: &str) -> Option<String> {
    if let Some(number) = name.strip_prefix('#') {
        let code = match number.strip_prefix(['x', 'X']) {
            Some(hex) => u32::from_str_radix(hex, 16).ok()?,
            None => number.parse().ok()?,
        };
        return char::from_u32(code)
            .filter(|&c| c != '\0')
            .map(String::from);
    }
    let text = match name {
        "amp" => "&",
        "lt" => "<",
        "gt" => ">",
        "quot" => "\"",
        "apos" => "'",
        "nbsp" => "\u{a0}",
        "ensp" => "\u{2002}",
        "emsp" => "\u{2003}",
        "thinsp" => "\u{2009}",
        "zwnj" => "\u{200c}",
        "zwj" => "\u{200d}",
        "lrm" => "\u{200e}",
        "rlm" => "\u{200f}",
        "shy" => "\u{ad}",
        "mdash" => "—",
        "ndash" => "–",
        "minus" => "−",
        "hellip" => "…",
        "lsquo" => "‘",
        "rsquo" => "’",
        "sbquo" => "‚",
        "ldquo" => "“",
        "rdquo" => "”",
        "bdquo" => "„",
        "laquo" => "«",
        "raquo" => "»",
        "lsaquo" => "‹",
        "rsaquo" => "›",
        "prime" => "′",
        "Prime" => "″",
        "bull" => "•",
        "middot" => "·",
        "deg" => "°",
        "times" => "×",
        "divide" => "÷",
        "plusmn" => "±",
        "frac12" => "½",
        "frac14" => "¼",
        "frac34" => "¾",
        "sup1" => "¹",
        "sup2" => "²",
        "sup3" => "³",
        "micro" => "µ",
        "permil" => "‰",
        "copy" => "©",
        "reg" => "®",
        "trade" => "™",
        "sect" => "§",
        "para" => "¶",
        "dagger" => "†",
        "Dagger" => "‡",
        "euro" => "€",
        "pound" => "£",
        "yen" => "¥",
        "cent" => "¢",
        "curren" => "¤",
        "iexcl" => "¡",
        "iquest" => "¿",
        "larr" => "←",
        "rarr" => "→",
        "uarr" => "↑",
        "darr" => "↓",
        "harr" => "↔",
        "lArr" => "⇐",
        "rArr" => "⇒",
        "hArr" => "⇔",
        "le" => "≤",
        "ge" => "≥",
        "ne" => "≠",
        "asymp" => "≈",
        "infin" => "∞",
        "check" => "✓",
        "hearts" => "♥",
        "star" => "☆",
        "agrave" => "à",
        "aacute" => "á",
        "acirc" => "â",
        "atilde" => "ã",
        "auml" => "ä",
        "aring" => "å",
        "aelig" => "æ",
        "ccedil" => "ç",
        "egrave" => "è",
        "eacute" => "é",
        "ecirc" => "ê",
        "euml" => "ë",
        "igrave" => "ì",
        "iacute" => "í",
        "icirc" => "î",
        "iuml" => "ï",
        "ntilde" => "ñ",
        "ograve" => "ò",
        "oacute" => "ó",
        "ocirc" => "ô",
        "otilde" => "õ",
        "ouml" => "ö",
        "oslash" => "ø",
        "ugrave" => "ù",
        "uacute" => "ú",
        "ucirc" => "û",
        "uuml" => "ü",
        "yacute" => "ý",
        "yuml" => "ÿ",
        "szlig" => "ß",
        "Agrave" => "À",
        "Aacute" => "Á",
        "Acirc" => "Â",
        "Auml" => "Ä",
        "Aring" => "Å",
        "Ccedil" => "Ç",
        "Egrave" => "È",
        "Eacute" => "É",
        "Ecirc" => "Ê",
        "Iacute" => "Í",
        "Ntilde" => "Ñ",
        "Oacute" => "Ó",
        "Ouml" => "Ö",
        "Oslash" => "Ø",
        "Uacute" => "Ú",
        "Uuml" => "Ü",
        _ => return None,
    };
    Some(text.to_owned())
}

/// `text` with its `&…;` references replaced (attribute values).
fn unescape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find('&') {
        out.push_str(&rest[..at]);
        let after = &rest[at + 1..];
        let end = after
            .char_indices()
            .take(32)
            .find(|&(_, c)| c == ';' || c == '&')
            .filter(|&(_, c)| c == ';');
        match end.and_then(|(ix, _)| entity(&after[..ix]).map(|t| (ix, t))) {
            Some((ix, text)) => {
                out.push_str(&text);
                rest = &after[ix + 1..];
            }
            None => {
                out.push('&');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

// ---------------------------------------------------------------------------
// ENML to Markdown.

/// A note's resources by MD5 (hex): their file names and content types.
pub type Resources = HashMap<String, (String, String)>;

/// Markdown for a note's ENML, and what converting it came across.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Converted {
    pub markdown: String,
    /// It had `<en-crypt>` sections, which became [`ENCRYPTED_PLACEHOLDER`].
    pub encrypted: bool,
    /// False when the ENML stopped being readable part way; what came
    /// before is in `markdown`.
    pub complete: bool,
}

/// Markdown for a note's ENML. `<en-media>` becomes an image or a link
/// to `kanso-import:embedded/<md5>` when `resources` has its hash, and is
/// left out when it doesn't. Never fails: what can't be read is left out.
pub fn enml_to_markdown(enml: &str, resources: &Resources) -> String {
    convert(enml, resources).markdown
}

/// [`enml_to_markdown`], also saying what it came across.
pub fn convert(enml: &str, resources: &Resources) -> Converted {
    let (nodes, complete) = parse_enml(enml);
    let mut ctx = Ctx {
        resources,
        pipes: false,
        encrypted: false,
    };
    let blocks = render_flow(&nodes, &mut ctx);
    Converted {
        markdown: join_blocks(&blocks).trim().to_owned(),
        encrypted: ctx.encrypted,
        complete,
    }
}

/// ENML as a tree.
#[derive(Debug)]
enum Node {
    Text(String),
    Element(Element),
}

#[derive(Debug, Default)]
struct Element {
    name: String,
    attrs: Vec<(String, String)>,
    children: Vec<Node>,
}

impl Element {
    fn attr(&self, name: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }

    /// Whether its `style` has `decl` (`-en-codeblock:true`), ignoring
    /// spaces and case.
    fn style_has(&self, decl: &str) -> bool {
        self.attr("style").is_some_and(|style| {
            let style: String = style
                .chars()
                .filter(|c| !c.is_whitespace())
                .collect::<String>()
                .to_ascii_lowercase();
            style.contains(decl)
        })
    }
}

/// Elements nested deeper than this are read as their parent's content.
const MAX_DEPTH: usize = 128;

/// Elements that never have content, even when written `<br></br>`.
const VOID: &[&str] = &[
    "br", "hr", "img", "en-media", "en-todo", "wbr", "col", "input", "meta", "link", "area",
    "base", "param", "source", "track", "embed",
];

/// Elements left out with everything in them.
const SKIPPED: &[&str] = &[
    "style", "script", "head", "title", "meta", "link", "object", "embed", "iframe", "noscript",
    "button", "input", "select", "textarea", "map", "area", "applet", "base", "basefont", "frame",
    "frameset", "param", "audio", "video", "source", "track", "colgroup", "col",
];

/// Elements that hold blocks and are rendered as their content.
const CONTAINERS: &[&str] = &[
    "div",
    "p",
    "en-note",
    "li",
    "td",
    "th",
    "tr",
    "thead",
    "tbody",
    "tfoot",
    "caption",
    "section",
    "article",
    "header",
    "footer",
    "nav",
    "aside",
    "main",
    "figure",
    "figcaption",
    "center",
    "address",
    "dl",
    "dt",
    "dd",
    "details",
    "summary",
    "form",
    "fieldset",
    "legend",
    "hgroup",
    "body",
    "html",
];

/// Block elements with Markdown of their own.
const BLOCKS: &[&str] = &[
    "hr",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "ul",
    "ol",
    "blockquote",
    "pre",
    "table",
    "en-crypt",
];

fn is_block(name: &str) -> bool {
    CONTAINERS.contains(&name) || BLOCKS.contains(&name)
}

/// Whether anything inside `el` is a block.
fn has_block(el: &Element) -> bool {
    el.children.iter().any(|child| match child {
        Node::Element(e) => {
            !SKIPPED.contains(&e.name.as_str()) && (is_block(&e.name) || has_block(e))
        }
        Node::Text(_) => false,
    })
}

/// Reads ENML leniently: unclosed and stray tags are tolerated, and at the
/// first error what was read so far is kept. Returns the top-level nodes
/// and whether all of it was read.
fn parse_enml(enml: &str) -> (Vec<Node>, bool) {
    let mut reader = Reader::from_str(enml);
    let config = reader.config_mut();
    config.check_end_names = false;
    config.allow_unmatched_ends = true;
    config.allow_dangling_amp = true;
    config.check_comments = false;
    let mut stack: Vec<Element> = vec![Element::default()];
    let mut ignored = 0usize;
    let mut complete = true;
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let el = element(&e);
                if VOID.contains(&el.name.as_str()) {
                    push_node(&mut stack, Node::Element(el));
                } else if stack.len() > MAX_DEPTH {
                    ignored += 1;
                } else {
                    stack.push(el);
                }
            }
            Ok(Event::Empty(e)) => push_node(&mut stack, Node::Element(element(&e))),
            Ok(Event::End(e)) => {
                if ignored > 0 {
                    ignored -= 1;
                    continue;
                }
                let name = tag_name(e.name().as_ref());
                if let Some(at) = stack.iter().rposition(|el| el.name == name)
                    && at > 0
                {
                    while stack.len() > at {
                        close(&mut stack);
                    }
                }
            }
            Ok(Event::Text(t)) => {
                if let Ok(text) = t.decode() {
                    push_text(&mut stack, &text);
                }
            }
            Ok(Event::CData(t)) => {
                if let Ok(text) = t.decode() {
                    push_text(&mut stack, &text);
                }
            }
            Ok(Event::GeneralRef(r)) => push_text(&mut stack, &reference(&r)),
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(_) => {
                complete = false;
                break;
            }
        }
    }
    while stack.len() > 1 {
        close(&mut stack);
    }
    let root = stack.pop().unwrap_or_default();
    (root.children, complete)
}

fn element(e: &BytesStart<'_>) -> Element {
    let attrs = e
        .html_attributes()
        .flatten()
        .map(|a| {
            (
                tag_name(a.key.as_ref()),
                unescape(&String::from_utf8_lossy(&a.value)),
            )
        })
        .collect();
    Element {
        name: tag_name(e.name().as_ref()),
        attrs,
        children: Vec::new(),
    }
}

fn push_node(stack: &mut [Element], node: Node) {
    if let Some(top) = stack.last_mut() {
        top.children.push(node);
    }
}

fn push_text(stack: &mut [Element], text: &str) {
    if text.is_empty() {
        return;
    }
    if let Some(top) = stack.last_mut() {
        match top.children.last_mut() {
            Some(Node::Text(last)) => last.push_str(text),
            _ => top.children.push(Node::Text(text.to_owned())),
        }
    }
}

/// Closes the innermost open element into its parent.
fn close(stack: &mut Vec<Element>) {
    if stack.len() > 1
        && let Some(el) = stack.pop()
    {
        push_node(stack, Node::Element(el));
    }
}

/// What rendering needs to know.
struct Ctx<'a> {
    resources: &'a Resources,
    /// Inside a table cell, where `|` is escaped.
    pipes: bool,
    encrypted: bool,
}

/// A block of Markdown.
#[derive(Debug)]
struct Block {
    text: String,
    /// A task line from `<en-todo>`: consecutive ones make one list.
    task: bool,
}

impl Block {
    fn plain(text: String) -> Block {
        Block { text, task: false }
    }
}

/// Blocks with a blank line between them, task lines one under another.
fn join_blocks(blocks: &[Block]) -> String {
    let mut out = String::new();
    for (ix, block) in blocks.iter().enumerate() {
        if ix > 0 {
            let tight = block.task && blocks[ix - 1].task;
            out.push_str(if tight { "\n" } else { "\n\n" });
        }
        out.push_str(&block.text);
    }
    out
}

/// Inline Markdown being written: whitespace collapsed as HTML does, and
/// text escaped where it would otherwise read as Markdown.
#[derive(Debug, Default)]
struct Inline {
    out: String,
    /// Escape every `|` (in a table cell).
    pipes: bool,
    /// Nothing written on this line yet.
    at_line_start: bool,
    /// This line so far is only digits (so `1.` would make a list).
    digits_only: bool,
    pending_space: bool,
    pending_break: bool,
    /// Whitespace came before anything was written.
    leading_space: bool,
    /// The line began with a task marker.
    task_start: bool,
}

impl Inline {
    fn new(pipes: bool, at_line_start: bool) -> Inline {
        Inline {
            pipes,
            at_line_start,
            ..Inline::default()
        }
    }

    fn space(&mut self) {
        if self.out.is_empty() {
            self.leading_space = true;
        } else if !self.at_line_start {
            self.pending_space = true;
        }
    }

    fn br(&mut self) {
        if !self.out.is_empty() {
            self.pending_break = true;
        }
    }

    /// Writes the space or line break waiting for more content.
    fn begin(&mut self) {
        if self.pending_break {
            self.out.push_str("\\\n");
            self.at_line_start = true;
            self.digits_only = false;
        } else if self.pending_space {
            self.out.push(' ');
            self.digits_only = false;
        }
        self.pending_break = false;
        self.pending_space = false;
    }

    fn markup(&mut self, text: &str) {
        self.begin();
        self.out.push_str(text);
        self.at_line_start = false;
        self.digits_only = false;
    }

    fn text(&mut self, text: &str) {
        for (ix, c) in text.char_indices() {
            if c.is_whitespace() {
                self.space();
                continue;
            }
            self.begin();
            let rest = &text[ix + c.len_utf8()..];
            let escape = match c {
                '\\' | '*' | '_' | '`' | '[' | ']' | '<' => true,
                '&' => entity_like(rest),
                '~' => rest.starts_with('~') || self.out.ends_with('~'),
                '|' => self.pipes || self.at_line_start,
                '#' | '-' | '+' | '>' | '=' => self.at_line_start,
                '.' | ')' => self.digits_only,
                _ => false,
            };
            let digit = c.is_ascii_digit();
            self.digits_only = if self.at_line_start {
                digit
            } else {
                self.digits_only && digit
            };
            self.at_line_start = false;
            if escape {
                self.out.push('\\');
            }
            self.out.push(c);
        }
    }

    /// `child` between `open` and `close`, with the spaces around it kept.
    fn wrap(&mut self, open: &str, child: Inline, close: &str) {
        if child.leading_space {
            self.space();
        }
        let text = child.out.trim_end();
        if !text.is_empty() {
            self.markup(open);
            self.out.push_str(text);
            self.out.push_str(close);
        }
        if child.pending_break {
            self.br();
        } else if child.pending_space || child.out.len() != text.len() {
            self.space();
        }
    }

    /// Starts a task (`- [ ] `) on a line of its own.
    fn todo(&mut self, checked: bool) {
        self.pending_space = false;
        self.pending_break = false;
        if self.out.is_empty() {
            self.task_start = true;
        } else {
            self.out.push('\n');
        }
        self.out.push_str(if checked { "- [x] " } else { "- [ ] " });
        self.at_line_start = true;
        self.digits_only = false;
    }

    /// Writes an inline link or image, escaping a `!` before a link.
    fn link(&mut self, text: &str) {
        self.begin();
        if text.starts_with('[') && self.out.ends_with('!') {
            self.out.pop();
            self.out.push_str("\\!");
        }
        self.markup(text);
    }

    fn finish(self) -> String {
        self.out.trim_end().to_owned()
    }
}

/// Whether `&` followed by `rest` would read as an entity.
fn entity_like(rest: &str) -> bool {
    let name: String = rest
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '#')
        .collect();
    !name.is_empty() && rest[name.len()..].starts_with(';')
}

/// Text for a link's label or an image's alt text.
fn escape_label(text: &str, pipes: bool) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\\' | '[' | ']' | '*' | '_' | '`' | '<' => {
                out.push('\\');
                out.push(c);
            }
            '|' if pipes => out.push_str("\\|"),
            '\n' | '\r' | '\t' => out.push(' '),
            _ => out.push(c),
        }
    }
    out
}

/// A link destination that stays one: spaces, brackets and pipes encoded.
fn destination(href: &str) -> String {
    let mut out = String::with_capacity(href.len());
    for c in href.trim().chars() {
        match c {
            ' ' => out.push_str("%20"),
            '(' => out.push_str("%28"),
            ')' => out.push_str("%29"),
            '<' => out.push_str("%3C"),
            '>' => out.push_str("%3E"),
            '|' => out.push_str("%7C"),
            '\n' | '\r' | '\t' => {}
            _ => out.push(c),
        }
    }
    out
}

/// The Markdown for an attachment: an image or a link.
fn media_link(key: &str, name: &str, image: bool, pipes: bool) -> String {
    format!(
        "{}[{}](kanso-import:embedded/{key})",
        if image { "!" } else { "" },
        escape_label(name, pipes)
    )
}

/// Blocks being written, with the paragraph in progress.
struct Flow {
    blocks: Vec<Block>,
    line: Inline,
    /// `<br>`s since the last content.
    breaks: usize,
    pipes: bool,
}

impl Flow {
    fn new(pipes: bool) -> Flow {
        Flow {
            blocks: Vec::new(),
            line: Inline::new(pipes, true),
            breaks: 0,
            pipes,
        }
    }

    /// Ends the paragraph in progress.
    fn flush(&mut self) {
        let line = std::mem::replace(&mut self.line, Inline::new(self.pipes, true));
        self.breaks = 0;
        let task = line.task_start;
        let text = line.finish();
        if !text.is_empty() {
            self.blocks.push(Block { text, task });
        }
    }

    fn push(&mut self, block: Option<Block>) {
        self.flush();
        if let Some(block) = block.filter(|b| !b.text.is_empty()) {
            self.blocks.push(block);
        }
    }

    /// Before content: one `<br>` is a line break, more end the paragraph.
    fn before_content(&mut self) {
        if self.breaks >= 2 {
            self.flush();
        } else if self.breaks == 1 {
            self.line.br();
        }
        self.breaks = 0;
    }
}

fn render_flow(nodes: &[Node], ctx: &mut Ctx<'_>) -> Vec<Block> {
    let mut flow = Flow::new(ctx.pipes);
    for node in nodes {
        flow_node(node, &mut flow, ctx);
    }
    flow.flush();
    flow.blocks
}

fn flow_node(node: &Node, flow: &mut Flow, ctx: &mut Ctx<'_>) {
    let el = match node {
        Node::Text(text) => {
            if text.chars().any(|c| !c.is_whitespace()) {
                flow.before_content();
            }
            flow.line.text(text);
            return;
        }
        Node::Element(el) => el,
    };
    let name = el.name.as_str();
    match name {
        _ if SKIPPED.contains(&name) => {}
        "br" => flow.breaks += 1,
        "hr" => flow.push(Some(Block::plain("---".to_owned()))),
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            let level = usize::from(name.as_bytes()[1] - b'0');
            let block = heading(el, level, ctx);
            flow.push(block);
        }
        "ul" | "ol" => {
            let block = list(el, name == "ol", ctx);
            flow.push(block);
        }
        "blockquote" => {
            let block = quote(el, ctx);
            flow.push(block);
        }
        "pre" => flow.push(code_block(el)),
        "table" => {
            flow.flush();
            let blocks = table(el, ctx);
            flow.blocks.extend(blocks);
        }
        "en-crypt" => {
            ctx.encrypted = true;
            flow.push(Some(Block::plain(ENCRYPTED_PLACEHOLDER.to_owned())));
        }
        _ if el.style_has("-en-codeblock:true") => flow.push(code_block(el)),
        _ if CONTAINERS.contains(&name) => {
            flow.flush();
            for child in &el.children {
                flow_node(child, flow, ctx);
            }
            flow.flush();
        }
        _ if has_block(el) => {
            for child in &el.children {
                flow_node(child, flow, ctx);
            }
        }
        _ => {
            flow.before_content();
            inline_node(el, &mut flow.line, ctx, Style::default());
        }
    }
}

/// Formatting already open around inline content, so it isn't nested.
#[derive(Clone, Copy, Debug, Default)]
struct Style {
    bold: bool,
    italic: bool,
    strike: bool,
    link: bool,
}

fn inline_children(nodes: &[Node], line: &mut Inline, ctx: &mut Ctx<'_>, style: Style) {
    for node in nodes {
        match node {
            Node::Text(text) => line.text(text),
            Node::Element(el) => inline_node(el, line, ctx, style),
        }
    }
}

fn inline_node(el: &Element, line: &mut Inline, ctx: &mut Ctx<'_>, style: Style) {
    let name = el.name.as_str();
    match name {
        _ if SKIPPED.contains(&name) => {}
        "br" => line.br(),
        "b" | "strong" if !style.bold => {
            let style = Style {
                bold: true,
                ..style
            };
            wrap(el, "**", line, ctx, style);
        }
        "i" | "em" | "cite" | "dfn" | "var" if !style.italic => {
            let style = Style {
                italic: true,
                ..style
            };
            wrap(el, "*", line, ctx, style);
        }
        "s" | "strike" | "del" if !style.strike => {
            let style = Style {
                strike: true,
                ..style
            };
            wrap(el, "~~", line, ctx, style);
        }
        "code" | "tt" | "kbd" | "samp" => code_span(el, line),
        "a" if !style.link => link(el, line, ctx, style),
        "img" => {
            let src = el.attr("src").unwrap_or_default().trim();
            if src.starts_with("http://") || src.starts_with("https://") {
                let alt = escape_label(el.attr("alt").unwrap_or_default(), line.pipes);
                line.link(&format!("![{alt}]({})", destination(src)));
            }
        }
        "en-media" => {
            let hash = el
                .attr("hash")
                .unwrap_or_default()
                .trim()
                .to_ascii_lowercase();
            if let Some((file, mime)) = ctx.resources.get(&hash) {
                let mime = if mime.is_empty() {
                    el.attr("type").unwrap_or_default()
                } else {
                    mime
                };
                let image = mime.starts_with("image/");
                line.link(&media_link(&hash, file, image, line.pipes));
            }
        }
        "en-todo" => line.todo(el.attr("checked").is_some_and(|c| c.trim() == "true")),
        _ => inline_children(&el.children, line, ctx, style),
    }
}

fn wrap(el: &Element, marker: &str, line: &mut Inline, ctx: &mut Ctx<'_>, style: Style) {
    let mut child = Inline::new(line.pipes, false);
    inline_children(&el.children, &mut child, ctx, style);
    line.wrap(marker, child, marker);
}

fn link(el: &Element, line: &mut Inline, ctx: &mut Ctx<'_>, style: Style) {
    let href = el.attr("href").unwrap_or_default().trim();
    let mut child = Inline::new(line.pipes, false);
    let style = Style {
        link: true,
        ..style
    };
    inline_children(&el.children, &mut child, ctx, style);
    if href.is_empty() || href.to_ascii_lowercase().starts_with("javascript:") {
        line.wrap("", child, "");
        return;
    }
    if child.leading_space {
        line.space();
    }
    let text = child.out.trim_end();
    let text = if text.is_empty() {
        escape_label(href, line.pipes)
    } else {
        text.to_owned()
    };
    line.link(&format!("[{text}]({})", destination(href)));
    if child.pending_break {
        line.br();
    } else if child.pending_space {
        line.space();
    }
}

fn code_span(el: &Element, line: &mut Inline) {
    let text = plain_text(el);
    let mut text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if text.is_empty() {
        return;
    }
    if line.pipes {
        text = text.replace('|', "\\|");
    }
    let fence = "`".repeat(longest_run(&text, '`') + 1);
    let pad = if text.starts_with('`') || text.ends_with('`') {
        " "
    } else {
        ""
    };
    line.markup(&format!("{fence}{pad}{text}{pad}{fence}"));
}

/// The longest run of `c` in `text`.
fn longest_run(text: &str, c: char) -> usize {
    let mut longest = 0;
    let mut run = 0;
    for ch in text.chars() {
        if ch == c {
            run += 1;
            longest = longest.max(run);
        } else {
            run = 0;
        }
    }
    longest
}

/// The text in `el` as it reads, with a line per block and `<br>`.
fn plain_text(el: &Element) -> String {
    let mut out = String::new();
    plain_into(&el.children, &mut out);
    out
}

fn plain_into(nodes: &[Node], out: &mut String) {
    for node in nodes {
        match node {
            Node::Text(text) => {
                out.extend(
                    text.chars()
                        .filter(|&c| c != '\r')
                        .map(|c| if c == '\u{a0}' { ' ' } else { c }),
                );
            }
            Node::Element(el) => match el.name.as_str() {
                "br" => out.push('\n'),
                name if SKIPPED.contains(&name) => {}
                name if is_block(name) => {
                    if !out.is_empty() && !out.ends_with('\n') {
                        out.push('\n');
                    }
                    plain_into(&el.children, out);
                    if !out.ends_with('\n') {
                        out.push('\n');
                    }
                }
                _ => plain_into(&el.children, out),
            },
        }
    }
}

/// A fenced code block of `el`'s text.
fn code_block(el: &Element) -> Option<Block> {
    let text = plain_text(el);
    let text = text.trim_matches('\n').trim_end();
    if text.trim().is_empty() {
        return None;
    }
    let fence = "`".repeat((longest_run(text, '`') + 1).max(3));
    Some(Block::plain(format!("{fence}\n{text}\n{fence}")))
}

/// Blocks' text on one line.
fn one_line(blocks: &[Block]) -> String {
    blocks
        .iter()
        .map(|b| b.text.replace("\\\n", " ").replace('\n', " "))
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_owned()
}

fn heading(el: &Element, level: usize, ctx: &mut Ctx<'_>) -> Option<Block> {
    let mut text = if has_block(el) {
        one_line(&render_flow(&el.children, ctx))
    } else {
        let mut line = Inline::new(ctx.pipes, false);
        inline_children(&el.children, &mut line, ctx, Style::default());
        one_line(&[Block::plain(line.finish())])
    };
    if text.is_empty() {
        return None;
    }
    if text.ends_with('#') {
        // A closing `#` sequence would be dropped.
        let kept = text.trim_end_matches('#').len();
        text.insert(kept, '\\');
    }
    Some(Block::plain(format!("{} {text}", "#".repeat(level))))
}

/// `body` after `marker`, its other lines indented by `indent`.
fn list_item(marker: &str, indent: usize, body: &str) -> String {
    let pad = " ".repeat(indent);
    let mut out = String::new();
    for (ix, line) in body.lines().enumerate() {
        if ix == 0 {
            out.push_str(marker);
            out.push_str(line);
        } else {
            out.push('\n');
            if !line.is_empty() {
                out.push_str(&pad);
                out.push_str(line);
            }
        }
    }
    out.trim_end().to_owned()
}

fn list(el: &Element, ordered: bool, ctx: &mut Ctx<'_>) -> Option<Block> {
    let checklist = el.style_has("--en-todo:true");
    let mut number: u64 = el
        .attr("start")
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(1);
    let mut items: Vec<String> = Vec::new();
    let mut last_indent = 0;
    for child in &el.children {
        match child {
            Node::Text(text) if text.trim().is_empty() => {}
            Node::Element(nested) if nested.name == "ul" || nested.name == "ol" => {
                let Some(block) = list(nested, nested.name == "ol", ctx) else {
                    continue;
                };
                match items.last_mut() {
                    Some(last) => {
                        last.push('\n');
                        last.push_str(&list_item(
                            &" ".repeat(last_indent),
                            last_indent,
                            &block.text,
                        ));
                    }
                    None => items.push(block.text),
                }
            }
            _ => {
                let (nodes, checked) = match child {
                    Node::Element(li) if li.name == "li" => {
                        (li.children.as_slice(), li.style_has("--en-checked:true"))
                    }
                    other => (std::slice::from_ref(other), false),
                };
                let blocks = render_flow(nodes, ctx);
                if blocks.is_empty() {
                    continue;
                }
                let body = blocks
                    .iter()
                    .map(|b| b.text.as_str())
                    .collect::<Vec<_>>()
                    .join("\n");
                let (marker, indent) = if checklist {
                    let marker = if checked { "- [x] " } else { "- [ ] " };
                    (marker.to_owned(), 2)
                } else if ordered {
                    let marker = format!("{number}. ");
                    number += 1;
                    let indent = marker.len();
                    (marker, indent)
                } else {
                    ("- ".to_owned(), 2)
                };
                items.push(list_item(&marker, indent, &body));
                last_indent = indent;
            }
        }
    }
    (!items.is_empty()).then(|| Block::plain(items.join("\n")))
}

fn quote(el: &Element, ctx: &mut Ctx<'_>) -> Option<Block> {
    let blocks = render_flow(&el.children, ctx);
    if blocks.is_empty() {
        return None;
    }
    let text = join_blocks(&blocks)
        .lines()
        .map(|line| {
            if line.is_empty() {
                ">".to_owned()
            } else {
                format!("> {line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    Some(Block::plain(text))
}

fn collect_rows<'a>(el: &'a Element, rows: &mut Vec<&'a Element>, captions: &mut Vec<&'a Element>) {
    for child in &el.children {
        if let Node::Element(e) = child {
            match e.name.as_str() {
                "tr" => rows.push(e),
                "thead" | "tbody" | "tfoot" => collect_rows(e, rows, captions),
                "caption" => captions.push(e),
                _ => {}
            }
        }
    }
}

/// A pipe table (after any caption), every cell on one line.
fn table(el: &Element, ctx: &mut Ctx<'_>) -> Vec<Block> {
    let mut rows = Vec::new();
    let mut captions = Vec::new();
    collect_rows(el, &mut rows, &mut captions);
    let mut blocks = Vec::new();
    for caption in captions {
        blocks.extend(render_flow(&caption.children, ctx));
    }
    let saved = ctx.pipes;
    ctx.pipes = true;
    let mut cells: Vec<Vec<String>> = Vec::new();
    for row in rows {
        let row_cells: Vec<String> = row
            .children
            .iter()
            .filter_map(|c| match c {
                Node::Element(cell) if cell.name == "td" || cell.name == "th" => {
                    Some(one_line(&render_flow(&cell.children, ctx)))
                }
                _ => None,
            })
            .collect();
        if !row_cells.is_empty() {
            cells.push(row_cells);
        }
    }
    ctx.pipes = saved;
    let width = cells.iter().map(Vec::len).max().unwrap_or(0);
    if width == 0 || cells.iter().flatten().all(String::is_empty) {
        return blocks;
    }
    let row_line = |row: &[String]| {
        let mut padded: Vec<&str> = row.iter().map(String::as_str).collect();
        padded.resize(width, "");
        format!("| {} |", padded.join(" | "))
    };
    let mut lines = vec![
        row_line(&cells[0]),
        format!("| {} |", vec!["---"; width].join(" | ")),
    ];
    lines.extend(cells[1..].iter().map(|row| row_line(row)));
    blocks.push(Block::plain(lines.join("\n")));
    blocks
}

#[cfg(test)]
mod tests {
    use super::*;

    fn md(enml: &str) -> String {
        enml_to_markdown(enml, &Resources::new())
    }

    #[test]
    fn named_and_numeric_entities() {
        assert_eq!(
            md(
                "<en-note><div>a&nbsp;b &mdash; c&ndash;d&hellip; &rsquo;&ldquo;x&rdquo; &#233;&#x41;</div></en-note>"
            ),
            "a b — c–d… ’“x” éA"
        );
        assert_eq!(
            md("<div>&bogus; &amp; &lt;b&gt;</div>"),
            "\\&bogus; & \\<b>"
        );
        assert_eq!(unescape("a&amp;b&quot;&nope;&"), "a&b\"&nope;&");
        assert_eq!(entity("#0"), None);
    }

    #[test]
    fn blocks_and_inline_formatting() {
        let enml = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE en-note SYSTEM "http://xml.evernote.com/pub/enml2.dtd">
<en-note><h1>Title</h1><div>Some <b>bold</b>, <i>italic</i>, <u>under</u> and <s>gone</s> text
with <code>a|b</code>.</div><div><br/></div><div>Line one<br/>line two</div>
<hr/><blockquote><div>Quoted</div><div>twice</div></blockquote>
<div><a href="https://example.com/a b">a link</a> <img src="https://example.com/p.png" alt="pic"/></div></en-note>"#;
        assert_eq!(
            md(enml),
            "# Title\n\nSome **bold**, *italic*, under and ~~gone~~ text with `a|b`.\n\n\
             Line one\\\nline two\n\n---\n\n> Quoted\n>\n> twice\n\n\
             [a link](https://example.com/a%20b) ![pic](https://example.com/p.png)"
        );
    }

    #[test]
    fn lists_tasks_and_checklists() {
        let enml = "<en-note><ul><li>one</li><li>two<ul><li>nested</li></ul></li></ul>\
            <ol start=\"3\"><li>three</li><li>four</li></ol>\
            <div><en-todo checked=\"true\"/>done</div><div><en-todo checked=\"false\"/>to do</div>\
            <ul style=\"--en-todo:true;\"><li style=\"--en-checked:true;\"><div>new done</div></li>\
            <li style=\"--en-checked:false;\"><div>new open</div></li></ul></en-note>";
        assert_eq!(
            md(enml),
            "- one\n- two\n  - nested\n\n3. three\n4. four\n\n- [x] done\n- [ ] to do\n\n\
             - [x] new done\n- [ ] new open"
        );
        // Evernote's own nesting: a list straight inside a list.
        assert_eq!(
            md("<ul><li>a</li><ul><li>b</li></ul><li>c</li></ul>"),
            "- a\n  - b\n- c"
        );
    }

    #[test]
    fn code_blocks_tables_and_crypt() {
        let enml = "<en-note><div style=\"box-sizing: border-box; -en-codeblock: true;\">\
            <div>fn main() {</div><div>&nbsp;&nbsp;go();</div><div><br/></div><div>}</div></div>\
            <pre>  raw *text*\n</pre>\
            <table><tr><th>A</th><th>B|C</th></tr><tr><td><div>1</div></td><td>2<br/>3</td></tr><tr><td>x</td></tr></table>\
            <en-crypt hint=\"h\">c2VjcmV0</en-crypt></en-note>";
        let converted = convert(enml, &Resources::new());
        assert_eq!(
            converted.markdown,
            "```\nfn main() {\n  go();\n\n}\n```\n\n```\n  raw *text*\n```\n\n\
             | A | B\\|C |\n| --- | --- |\n| 1 | 2 3 |\n| x |  |\n\n\
             [Encrypted content not imported]"
        );
        assert!(converted.encrypted);
        assert!(converted.complete);
    }

    #[test]
    fn media_becomes_embedded_links() {
        let mut resources = Resources::new();
        resources.insert("aa".into(), ("pic.png".into(), "image/png".into()));
        resources.insert("bb".into(), ("doc_v1.pdf".into(), "application/pdf".into()));
        let enml = "<en-note><div><en-media hash=\"AA\" type=\"image/png\"/></div>\
            <div>See <en-media hash=\"bb\" type=\"application/pdf\"></en-media></div>\
            <en-media hash=\"cc\" type=\"image/png\"/></en-note>";
        assert_eq!(
            enml_to_markdown(enml, &resources),
            "![pic.png](kanso-import:embedded/aa)\n\nSee [doc\\_v1.pdf](kanso-import:embedded/bb)"
        );
    }

    #[test]
    fn markdown_in_text_is_escaped() {
        assert_eq!(
            md("<div># not *a* [heading](x) `tick` 1_2</div><div>1. no list</div><div>- no</div>"),
            "\\# not \\*a\\* \\[heading\\](x) \\`tick\\` 1\\_2\n\n1\\. no list\n\n\\- no"
        );
        assert_eq!(md("<div>~~x~~ ~ok</div>"), "\\~\\~x\\~\\~ ~ok");
        assert_eq!(md("<div>Wow!<a href=\"u\">x</a></div>"), "Wow\\![x](u)");
    }

    #[test]
    fn broken_enml_keeps_what_came_before() {
        let converted = convert(
            "<en-note><div>kept</div><div a=\"1\" a=\"2\"<<</en-note>",
            &Resources::new(),
        );
        assert!(converted.markdown.starts_with("kept"), "{converted:?}");
        // Stray and unclosed tags are fine.
        assert_eq!(md("<div><b>bold</div></i>after"), "**bold**\n\nafter");
    }

    #[test]
    fn resource_names() {
        assert_eq!(resource_name("", "image/jpeg"), "image.jpg");
        assert_eq!(resource_name(" a.pdf ", "application/pdf"), "a.pdf");
        assert_eq!(resource_name("", "application/x-thing"), "attachment");
    }
}
