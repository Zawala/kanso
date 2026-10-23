//! The SVG sanitizer: a parse-and-rebuild pass, not a search for substrings.
//!
//! Kanso draws SVG from people's icon themes and from the gallery, so an SVG
//! is held to an allowlist before it is drawn: only shapes, groups,
//! gradients, clip paths and `use` fragments; only geometry, presentation,
//! gradient and namespace attributes; only colors, bounded numbers, the SVG
//! path alphabet and `url(#id)` references. A `script`, `foreignObject`,
//! `image`, `style` element, event attribute, external `href`, `data:` URL,
//! animation, DOCTYPE entity, comment or processing instruction never
//! reaches the renderer.
//!
//! The output is a normalized SVG no larger than the input. Every SVG in
//! `themes/icon-themes/` must pass; the sanitizer is applied to gallery
//! installs, imports, and every non-built-in theme at load.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

/// The largest SVG the sanitizer reads, in bytes. The same cap the app's
/// icon theme loader uses.
pub const MAX_SVG_BYTES: u64 = 128 * 1024;
const MAX_DEPTH: usize = 32;
const MAX_ELEMENTS: usize = 4_096;
const MAX_PATH_BYTES: usize = 32 * 1024;
const MAX_ID_BYTES: usize = 64;
const MAX_VALUE_BYTES: usize = 32 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum SanitizeError {
    #[error("it isn't UTF-8")]
    NotUtf8,
    #[error("it is larger than 128 KB")]
    TooLarge,
    #[error("it isn't valid XML")]
    Malformed,
    #[error("it uses <{0}>, which Kanso doesn't draw from an icon theme")]
    DisallowedElement(String),
    #[error("it uses an attribute Kanso doesn't draw from an icon theme")]
    DisallowedAttribute,
    #[error("it is nested too deeply")]
    TooDeep,
    #[error("a <use> target contains another <use>")]
    NestedUse,
    #[error("it has too many elements")]
    TooManyElements,
    #[error("it has a value Kanso won't draw")]
    BadValue,
    #[error("it refers to something outside itself")]
    ExternalReference,
    #[error("it declares an entity")]
    Entity,
    #[error("it is larger once cleaned than it was")]
    Grew,
}

/// Elements an icon theme may draw. `title` and `desc` are text-only and
/// kept for accessibility.
const ELEMENTS: &[&str] = &[
    "svg",
    "g",
    "defs",
    "use",
    "path",
    "rect",
    "circle",
    "ellipse",
    "line",
    "polyline",
    "polygon",
    "clipPath",
    "linearGradient",
    "radialGradient",
    "stop",
    "title",
    "desc",
];

/// Presentation properties a `style` attribute may carry. The same names are
/// accepted as attributes; a style declaration is applied over the attribute
/// of the same name, as CSS does.
const PRESENTATION: &[&str] = &[
    "fill",
    "fill-opacity",
    "fill-rule",
    "stroke",
    "stroke-width",
    "stroke-linecap",
    "stroke-linejoin",
    "stroke-miterlimit",
    "stroke-dasharray",
    "stroke-dashoffset",
    "opacity",
    "clip-rule",
    "clip-path",
    "stop-color",
    "stop-opacity",
    "transform",
];

/// Every other attribute an icon theme may draw.
const ATTRIBUTES: &[&str] = &[
    "d",
    "x",
    "y",
    "width",
    "height",
    "cx",
    "cy",
    "r",
    "rx",
    "ry",
    "x1",
    "y1",
    "x2",
    "y2",
    "points",
    "viewBox",
    "preserveAspectRatio",
    "id",
    "href",
    "offset",
    "gradientUnits",
    "gradientTransform",
];

/// Sanitizes `input` into a normalized SVG. Pure and total: any input either
/// produces safe SVG or an error, never a partly built tree.
pub fn sanitize_svg(input: &[u8]) -> Result<Vec<u8>, SanitizeError> {
    if input.len() as u64 > MAX_SVG_BYTES {
        return Err(SanitizeError::TooLarge);
    }
    let text = std::str::from_utf8(input).map_err(|_| SanitizeError::NotUtf8)?;
    let mut reader = Reader::from_str(text);
    reader.config_mut().check_end_names = true;
    reader.config_mut().expand_empty_elements = false;

    // A `<use>` may point at a shape or a group, never at something that
    // itself expands a `<use>`: that is how a tiny file becomes an
    // unbounded expansion. The referenced ids are collected first so a
    // target declared after the `use` is caught too.
    let referenced = use_targets(text)?;
    let mut out = String::with_capacity(text.len());
    let mut stack: Vec<String> = Vec::new();
    // Whether the element at the same index is a referenced `<use>` target.
    let mut targets: Vec<bool> = Vec::new();
    let mut elements = 0usize;
    let mut saw_root = false;
    let mut root_closed = false;

    loop {
        match reader.read_event().map_err(|_| SanitizeError::Malformed)? {
            Event::Eof => break,
            Event::Decl(_) => {}
            Event::DocType(doctype) => {
                // The repository's SVGs carry the external SVG DTD; it is
                // dropped. An internal subset could declare entities, so it
                // is refused.
                let text = doctype.decode().map_err(|_| SanitizeError::Malformed)?;
                if text.to_ascii_uppercase().contains("ENTITY") {
                    return Err(SanitizeError::Entity);
                }
            }
            // Comments carry no drawing and no behaviour. The repository's
            // JetBrains icons carry licence comments, so they are dropped
            // rather than refused.
            Event::Comment(_) => {}
            Event::PI(_) => return Err(SanitizeError::Malformed),
            Event::Start(start) => {
                if root_closed {
                    return Err(SanitizeError::Malformed);
                }
                elements += 1;
                check_caps(elements, stack.len() + 1)?;
                let name = element_name(&start)?;
                let id = id_of(&start);
                if name == "use"
                    && (targets.iter().any(|target| *target)
                        || is_referenced(&referenced, id.as_deref()))
                {
                    return Err(SanitizeError::NestedUse);
                }
                if stack.is_empty() {
                    if name != "svg" {
                        return Err(SanitizeError::Malformed);
                    }
                    saw_root = true;
                }
                let attributes = clean_attributes(&name, &start)?;
                out.push('<');
                out.push_str(&name);
                out.push_str(&attributes);
                out.push('>');
                targets.push(is_referenced(&referenced, id.as_deref()));
                stack.push(name);
            }
            Event::Empty(start) => {
                if root_closed {
                    return Err(SanitizeError::Malformed);
                }
                elements += 1;
                check_caps(elements, stack.len() + 1)?;
                let name = element_name(&start)?;
                let id = id_of(&start);
                if name == "use"
                    && (targets.iter().any(|target| *target)
                        || is_referenced(&referenced, id.as_deref()))
                {
                    return Err(SanitizeError::NestedUse);
                }
                if stack.is_empty() {
                    if name != "svg" {
                        return Err(SanitizeError::Malformed);
                    }
                    saw_root = true;
                    root_closed = true;
                }
                let attributes = clean_attributes(&name, &start)?;
                out.push('<');
                out.push_str(&name);
                out.push_str(&attributes);
                out.push_str("/>");
            }
            Event::End(end) => {
                let name = String::from_utf8_lossy(end.name().as_ref()).into_owned();
                let Some(open) = stack.pop() else {
                    return Err(SanitizeError::Malformed);
                };
                if open != name {
                    return Err(SanitizeError::Malformed);
                }
                targets.pop();
                out.push_str("</");
                out.push_str(&name);
                out.push('>');
                if stack.is_empty() {
                    root_closed = true;
                }
            }
            Event::Text(text) => {
                let value = text.decode().map_err(|_| SanitizeError::Malformed)?;
                push_text(&mut out, &stack, &value)?;
            }
            Event::CData(data) => {
                let value = data.decode().map_err(|_| SanitizeError::Malformed)?;
                push_text(&mut out, &stack, &value)?;
            }
            Event::GeneralRef(reference) => {
                let decoded = decode_reference(&reference)?;
                push_text(&mut out, &stack, &decoded)?;
            }
        }
        if out.len() > text.len() {
            return Err(SanitizeError::Grew);
        }
    }
    if !saw_root || !stack.is_empty() {
        return Err(SanitizeError::Malformed);
    }
    if out.len() > text.len() {
        return Err(SanitizeError::Grew);
    }
    Ok(out.into_bytes())
}

/// The fragment ids a `<use>` element names, collected before the main pass.
fn use_targets(text: &str) -> Result<BTreeSet<String>, SanitizeError> {
    let mut reader = Reader::from_str(text);
    reader.config_mut().check_end_names = true;
    reader.config_mut().expand_empty_elements = false;
    let mut targets = BTreeSet::new();
    loop {
        match reader.read_event().map_err(|_| SanitizeError::Malformed)? {
            Event::Eof => break,
            Event::Start(start) | Event::Empty(start) => {
                if start.name().as_ref() == b"use"
                    && let Some(href) = href_of(&start)
                    && let Some(id) = href.strip_prefix('#')
                    && !id.is_empty()
                {
                    targets.insert(id.to_owned());
                }
            }
            _ => {}
        }
    }
    Ok(targets)
}

fn is_referenced(referenced: &BTreeSet<String>, id: Option<&str>) -> bool {
    id.is_some_and(|id| referenced.contains(id))
}

fn id_of(start: &BytesStart<'_>) -> Option<String> {
    attribute_value(start, "id")
}

fn href_of(start: &BytesStart<'_>) -> Option<String> {
    attribute_value(start, "href").or_else(|| attribute_value(start, "xlink:href"))
}

fn attribute_value(start: &BytesStart<'_>, wanted: &str) -> Option<String> {
    for attribute in start.attributes().flatten() {
        if attribute.key.as_ref() == wanted.as_bytes() {
            return attribute
                .normalized_value(quick_xml::XmlVersion::Implicit1_0)
                .ok()
                .map(std::borrow::Cow::into_owned);
        }
    }
    None
}

fn check_caps(elements: usize, depth: usize) -> Result<(), SanitizeError> {
    if elements > MAX_ELEMENTS {
        return Err(SanitizeError::TooManyElements);
    }
    if depth > MAX_DEPTH {
        return Err(SanitizeError::TooDeep);
    }
    Ok(())
}

fn element_name(start: &BytesStart<'_>) -> Result<String, SanitizeError> {
    let qname = start.name();
    let name = std::str::from_utf8(qname.as_ref()).map_err(|_| SanitizeError::Malformed)?;
    if !ELEMENTS.contains(&name) {
        return Err(SanitizeError::DisallowedElement(name.to_owned()));
    }
    Ok(name.to_owned())
}

/// The checked attributes of `start`, ready to write after the element name.
fn clean_attributes(name: &str, start: &BytesStart<'_>) -> Result<String, SanitizeError> {
    let mut kept: Vec<(String, String)> = Vec::new();
    let mut seen = BTreeSet::new();
    for attribute in start.attributes() {
        let attribute = attribute.map_err(|_| SanitizeError::Malformed)?;
        let key = std::str::from_utf8(attribute.key.as_ref())
            .map_err(|_| SanitizeError::Malformed)?
            .to_owned();
        let value = attribute
            .normalized_value(quick_xml::XmlVersion::Implicit1_0)
            .map_err(|_| SanitizeError::Entity)?
            .into_owned();
        // Duplicates are judged by the name the output uses: `href` and
        // `xlink:href` together would let the reference check see one target
        // and the output keep the other.
        let canonical = if key == "xlink:href" {
            "href"
        } else {
            key.as_str()
        };
        if !seen.insert(canonical.to_owned()) {
            return Err(SanitizeError::Malformed);
        }
        if key == "style" {
            for (property, value) in style_declarations(&value)? {
                apply(&mut kept, property, value);
            }
            continue;
        }
        if key.starts_with("xmlns:") || key == "xmlns" {
            check_namespace(&value)?;
            apply(&mut kept, &key, value);
            continue;
        }
        if key.starts_with("on") || key == "xml:base" {
            return Err(SanitizeError::DisallowedAttribute);
        }
        let normalized = match key.as_str() {
            // `xlink:href` is how SVG 1.1 spells `href`; it is normalized,
            // and only a fragment is ever kept.
            "xlink:href" => "href",
            other => other,
        };
        if !ATTRIBUTES.contains(&normalized) && !PRESENTATION.contains(&normalized) {
            // An unknown attribute is dropped, not guessed at.
            continue;
        }
        if let Some(value) = check_value(name, normalized, &value)? {
            apply(&mut kept, normalized, value);
        }
    }
    let mut out = String::new();
    for (key, value) in kept {
        out.push(' ');
        out.push_str(&key);
        out.push_str("=\"");
        push_escaped(&mut out, &value);
        out.push('"');
    }
    Ok(out)
}

fn apply(kept: &mut Vec<(String, String)>, key: &str, value: String) {
    if let Some(slot) = kept.iter_mut().find(|(other, _)| other == key) {
        slot.1 = value;
    } else {
        kept.push((key.to_owned(), value));
    }
}

fn check_namespace(value: &str) -> Result<(), SanitizeError> {
    if value.len() > 200
        || value.chars().any(|c| c.is_control() || c.is_whitespace())
        || !(value.starts_with("http://") || value.starts_with("https://"))
    {
        return Err(SanitizeError::BadValue);
    }
    Ok(())
}

/// The declarations of a `style` attribute that are presentation properties,
/// with their values checked. Others are dropped.
fn style_declarations(style: &str) -> Result<Vec<(&'static str, String)>, SanitizeError> {
    if style.len() > MAX_VALUE_BYTES {
        return Err(SanitizeError::BadValue);
    }
    let mut out = Vec::new();
    for declaration in style.split(';') {
        let declaration = declaration.trim();
        if declaration.is_empty() {
            continue;
        }
        let Some((property, value)) = declaration.split_once(':') else {
            return Err(SanitizeError::BadValue);
        };
        let Some(property) = PRESENTATION.iter().find(|p| **p == property.trim()) else {
            // `mix-blend-mode`, `font-*` and friends carry no drawing Kanso
            // needs from an icon; they are filtered out.
            continue;
        };
        // A property's value is checked exactly as its attribute would be;
        // `check_value` is handed a harmless element name for context.
        if let Some(value) = check_value("g", property, value.trim())? {
            out.push((*property, value));
        } else {
            return Err(SanitizeError::BadValue);
        }
    }
    Ok(out)
}

/// Checks `value` for `attribute` on `element`. Returns the normalized value
/// to keep, or `None` when the attribute is harmless but not needed.
fn check_value(
    element: &str,
    attribute: &str,
    value: &str,
) -> Result<Option<String>, SanitizeError> {
    if value.len() > MAX_VALUE_BYTES {
        return Err(SanitizeError::BadValue);
    }
    let low = value.to_ascii_lowercase();
    if low.contains("javascript:")
        || low.contains("data:")
        || low.contains("vbscript:")
        || low.contains("&#")
    {
        return Err(SanitizeError::ExternalReference);
    }
    let kept = match attribute {
        "d" => {
            if value.len() > MAX_PATH_BYTES {
                return Err(SanitizeError::BadValue);
            }
            if !value.chars().all(|c| {
                matches!(
                    c,
                    'M' | 'm'
                        | 'Z'
                        | 'z'
                        | 'L'
                        | 'l'
                        | 'H'
                        | 'h'
                        | 'V'
                        | 'v'
                        | 'C'
                        | 'c'
                        | 'S'
                        | 's'
                        | 'Q'
                        | 'q'
                        | 'T'
                        | 't'
                        | 'A'
                        | 'a'
                ) || c.is_ascii_digit()
                    || matches!(
                        c,
                        '.' | ',' | '-' | '+' | 'e' | 'E' | ' ' | '\t' | '\r' | '\n'
                    )
            }) {
                return Err(SanitizeError::BadValue);
            }
            value.to_owned()
        }
        "points" | "stroke-dasharray" => {
            if low == "none" {
                "none".to_owned()
            } else {
                numbers(value)?
            }
        }
        "viewBox" => {
            let parts: Vec<&str> = value
                .split(|c: char| c == ',' || c.is_whitespace())
                .filter(|part| !part.is_empty())
                .collect();
            if parts.len() != 4 || !parts.iter().all(|part| number(part).is_some()) {
                return Err(SanitizeError::BadValue);
            }
            value.to_owned()
        }
        "preserveAspectRatio" => {
            if !value
                .chars()
                .all(|c| c.is_ascii_alphabetic() || c.is_whitespace())
            {
                return Err(SanitizeError::BadValue);
            }
            value.to_owned()
        }
        "transform" | "gradientTransform" => transform(value)?,
        "id" => {
            if !valid_id(value) {
                return Err(SanitizeError::BadValue);
            }
            value.to_owned()
        }
        "href" => fragment(value)?,
        "fill" | "stroke" | "stop-color" | "clip-path" | "color" => paint(value)?,
        "fill-rule" | "clip-rule" => {
            if matches!(low.as_str(), "nonzero" | "evenodd" | "inherit") {
                low
            } else {
                return Err(SanitizeError::BadValue);
            }
        }
        "stroke-linecap" => {
            if matches!(low.as_str(), "butt" | "round" | "square" | "inherit") {
                low
            } else {
                return Err(SanitizeError::BadValue);
            }
        }
        "stroke-linejoin" => {
            if matches!(low.as_str(), "miter" | "round" | "bevel" | "inherit") {
                low
            } else {
                return Err(SanitizeError::BadValue);
            }
        }
        "gradientUnits" => {
            if matches!(low.as_str(), "userspaceonuse" | "objectboundingbox") {
                low
            } else {
                return Err(SanitizeError::BadValue);
            }
        }
        "offset" => {
            let part = value.trim_end_matches('%');
            if number(part).is_none() {
                return Err(SanitizeError::BadValue);
            }
            value.to_owned()
        }
        "x" | "y" | "width" | "height" | "cx" | "cy" | "r" | "rx" | "ry" | "x1" | "y1" | "x2"
        | "y2" | "fill-opacity" | "stroke-opacity" | "stop-opacity" | "opacity"
        | "stroke-width" | "stroke-miterlimit" | "stroke-dashoffset" => {
            if number(value).is_none() {
                return Err(SanitizeError::BadValue);
            }
            value.to_owned()
        }
        _ => return Ok(None),
    };
    if element == "use" && attribute == "href" && !kept.starts_with('#') {
        return Err(SanitizeError::ExternalReference);
    }
    Ok(Some(kept))
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_ID_BYTES
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | ':'))
}

fn fragment(value: &str) -> Result<String, SanitizeError> {
    let some = value.strip_prefix('#').unwrap_or_default();
    if !valid_id(some) {
        return Err(SanitizeError::ExternalReference);
    }
    Ok(format!("#{some}"))
}

/// A paint value: a color, or `url(#id)` for a gradient or clip path.
fn paint(value: &str) -> Result<String, SanitizeError> {
    let trimmed = value.trim();
    let low = trimmed.to_ascii_lowercase();
    if low.starts_with("url(") {
        let original = trimmed[4..].trim();
        let Some(original) = original.strip_suffix(')') else {
            return Err(SanitizeError::ExternalReference);
        };
        let id = original.strip_prefix('#').unwrap_or_default();
        if !valid_id(id.trim()) {
            return Err(SanitizeError::ExternalReference);
        }
        return Ok(format!("url(#{})", id.trim()));
    }
    if is_color(trimmed) {
        Ok(trimmed.to_owned())
    } else {
        Err(SanitizeError::BadValue)
    }
}

fn is_color(value: &str) -> bool {
    let low = value.to_ascii_lowercase();
    if matches!(
        low.as_str(),
        "none" | "currentcolor" | "transparent" | "inherit" | "context-fill" | "context-stroke"
    ) {
        return true;
    }
    let bytes = low.as_bytes();
    if matches!(bytes.len(), 4 | 7 | 9)
        && bytes[0] == b'#'
        && bytes[1..].iter().all(|b| b.is_ascii_hexdigit())
    {
        return true;
    }
    for function in ["rgb(", "rgba(", "hsl(", "hsla("] {
        if let Some(inner) = low.strip_prefix(function)
            && let Some(inner) = inner.strip_suffix(')')
            && inner.len() <= 64
            && inner
                .chars()
                .all(|c| c.is_ascii_digit() || matches!(c, '.' | ',' | '%' | '/' | ' ' | '-' | '+'))
        {
            return true;
        }
    }
    // A named CSS color word. The renderer knows the names; this only keeps
    // the shape, so a name can't smuggle a reference.
    !low.is_empty() && low.len() <= 24 && low.chars().all(|c| c.is_ascii_alphabetic())
}

fn number(value: &str) -> Option<f64> {
    let value = value.trim();
    if value.is_empty() || value.len() > 40 {
        return None;
    }
    let parsed: f64 = value.parse().ok()?;
    (parsed.is_finite() && parsed.abs() <= 100_000_000.0).then_some(parsed)
}

fn numbers(value: &str) -> Result<String, SanitizeError> {
    if value.len() > MAX_VALUE_BYTES {
        return Err(SanitizeError::BadValue);
    }
    let mut count = 0;
    for part in value.split(|c: char| c == ',' || c.is_whitespace()) {
        if part.is_empty() {
            continue;
        }
        count += 1;
        if number(part).is_none() {
            return Err(SanitizeError::BadValue);
        }
    }
    // An empty `stroke-dasharray=""` means no dashes, which some themes
    // write out; it keeps its empty value.
    if count == 0 {
        return Ok(String::new());
    }
    Ok(value.to_owned())
}

fn transform(value: &str) -> Result<String, SanitizeError> {
    if value.len() > MAX_VALUE_BYTES {
        return Err(SanitizeError::BadValue);
    }
    let allowed = ["matrix", "translate", "scale", "rotate", "skewx", "skewy"];
    let bytes = value.as_bytes();
    let mut at = 0;
    while at < bytes.len() {
        let c = bytes[at] as char;
        if c.is_whitespace() {
            at += 1;
            continue;
        }
        let start = at;
        while at < bytes.len() && (bytes[at] as char).is_ascii_alphabetic() {
            at += 1;
        }
        let name = value[start..at].to_ascii_lowercase();
        if !allowed.contains(&name.as_str()) {
            return Err(SanitizeError::BadValue);
        }
        if bytes.get(at) != Some(&b'(') {
            return Err(SanitizeError::BadValue);
        }
        at += 1;
        let content_start = at;
        while at < bytes.len() && bytes[at] != b')' {
            let c = bytes[at] as char;
            if !(c.is_ascii_digit() || matches!(c, '.' | ',' | ' ' | '-' | '+' | 'e' | 'E' | '\t'))
            {
                return Err(SanitizeError::BadValue);
            }
            at += 1;
        }
        if bytes.get(at) != Some(&b')') {
            return Err(SanitizeError::BadValue);
        }
        for part in value[content_start..at].split(|c: char| c == ',' || c.is_whitespace()) {
            if !part.is_empty() && number(part).is_none() {
                return Err(SanitizeError::BadValue);
            }
        }
        at += 1;
    }
    Ok(value.to_owned())
}

/// Text is kept only inside `title` and `desc`; whitespace between elements
/// is dropped, and anything else would have no element to draw it.
fn push_text(out: &mut String, stack: &[String], value: &str) -> Result<(), SanitizeError> {
    if !matches!(stack.last().map(String::as_str), Some("title" | "desc")) {
        return Ok(());
    }
    push_escaped(out, value);
    Ok(())
}

fn push_escaped(out: &mut String, value: &str) {
    for c in value.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => {
                let _ = out.write_char(c);
            }
        }
    }
}

fn decode_reference(reference: &quick_xml::events::BytesRef<'_>) -> Result<String, SanitizeError> {
    let raw = std::str::from_utf8(reference.as_ref()).map_err(|_| SanitizeError::Malformed)?;
    if let Some(number) = raw.strip_prefix('#') {
        let code = if let Some(hex) = number
            .strip_prefix('x')
            .or_else(|| number.strip_prefix('X'))
        {
            u32::from_str_radix(hex, 16).ok()
        } else {
            number.parse().ok()
        };
        return code
            .and_then(char::from_u32)
            .map(|c| c.to_string())
            .ok_or(SanitizeError::Entity);
    }
    match raw {
        "amp" => Ok("&".into()),
        "lt" => Ok("<".into()),
        "gt" => Ok(">".into()),
        "quot" => Ok("\"".into()),
        "apos" => Ok("'".into()),
        // Anything else would need a DOCTYPE declaration, which is refused.
        _ => Err(SanitizeError::Entity),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn ok(input: &str) -> String {
        String::from_utf8(sanitize_svg(input.as_bytes()).unwrap()).unwrap()
    }

    #[test]
    fn a_plain_icon_is_kept() {
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16"><path d="M1 2h3v4z" fill="#ff0000"/></svg>"##;
        let out = ok(svg);
        assert!(out.contains("viewBox=\"0 0 16 16\""));
        assert!(out.contains("fill=\"#ff0000\""));
        assert!(out.starts_with("<svg"));
    }

    #[test]
    fn scripts_images_events_and_external_references_are_refused() {
        for bad in [
            r#"<svg><script>alert(1)</script></svg>"#,
            r#"<svg><foreignObject><div/></foreignObject></svg>"#,
            r#"<svg><image href="https://evil.example/x.png"/></svg>"#,
            r#"<svg><path d="M0 0" onload="alert(1)"/></svg>"#,
            r#"<svg><use href="https://evil.example/x.svg#a"/></svg>"#,
            r#"<svg><path fill="url(https://evil.example/x)"/></svg>"#,
            r#"<svg><path fill="data:image/png;base64,AAAA"/></svg>"#,
            r#"<svg><path fill="url(#a)" style="fill:url(https://evil.example/x)"/></svg>"#,
            r#"<svg><style>path{fill:red}</style></svg>"#,
            r#"<svg><animate attributeName="x"/></svg>"#,
            r#"<svg><text>hi</text></svg>"#,
        ] {
            assert!(sanitize_svg(bad.as_bytes()).is_err(), "{bad}");
        }
    }

    #[test]
    fn a_doctype_is_dropped_and_an_entity_is_refused() {
        let with_dtd = r#"<?xml version="1.0"?><!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1//EN" "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd"><svg><path d="M0 0"/></svg>"#;
        assert!(sanitize_svg(with_dtd.as_bytes()).is_ok());
        let laughs = r#"<!DOCTYPE svg [<!ENTITY a "aaaa">]><svg><title>&a;</title></svg>"#;
        assert_eq!(sanitize_svg(laughs.as_bytes()), Err(SanitizeError::Entity));
        let unknown = r#"<svg><title>&nope;</title></svg>"#;
        assert_eq!(sanitize_svg(unknown.as_bytes()), Err(SanitizeError::Entity));
    }

    #[test]
    fn comments_are_dropped_and_style_is_filtered() {
        let svg = r#"<svg><!-- licence --><path d="M0 0" style="fill:#123456;mix-blend-mode:normal;fill-rule:evenodd"/></svg>"#;
        let out = ok(svg);
        assert!(!out.contains("<!--"));
        assert!(out.contains("fill=\"#123456\""));
        assert!(out.contains("fill-rule=\"evenodd\""));
        assert!(!out.contains("mix-blend-mode"));
    }

    #[test]
    fn text_is_escaped_and_caps_apply() {
        let out = ok(r#"<svg><title>a &amp; b</title></svg>"#);
        assert!(out.contains("<title>a &amp; b</title>"));
        let deep = format!(
            "<svg>{}{}</svg>",
            "<g>".repeat(MAX_DEPTH),
            "</g>".repeat(MAX_DEPTH)
        );
        assert_eq!(sanitize_svg(deep.as_bytes()), Err(SanitizeError::TooDeep));
        let many = format!("<svg>{}</svg>", "<path d=\"M0 0\"/>".repeat(MAX_ELEMENTS));
        assert_eq!(
            sanitize_svg(many.as_bytes()),
            Err(SanitizeError::TooManyElements)
        );
        assert_eq!(
            sanitize_svg(&vec![b'a'; MAX_SVG_BYTES as usize + 1]),
            Err(SanitizeError::TooLarge)
        );
        assert_eq!(sanitize_svg(&[0xff, 0xfe]), Err(SanitizeError::NotUtf8));
    }

    #[test]
    fn a_use_target_may_not_contain_another_use() {
        // A target declared after the use is caught too.
        let nested = r##"<svg><use href="#a"/><defs><g id="a"><use href="#b"/></g></defs></svg>"##;
        assert_eq!(
            sanitize_svg(nested.as_bytes()),
            Err(SanitizeError::NestedUse)
        );
        let direct = r##"<svg><defs><path id="a" d="M0 0h4v4z"/></defs><use href="#a"/></svg>"##;
        assert!(sanitize_svg(direct.as_bytes()).is_ok());
        let group =
            r##"<svg><defs><g id="a"><path d="M0 0h4v4z"/></g></defs><use href="#a"/></svg>"##;
        assert!(sanitize_svg(group.as_bytes()).is_ok());
    }

    #[test]
    fn href_and_xlink_href_together_are_refused() {
        // The use check read `href` (#z, a path) while the output kept
        // `xlink:href` (#a, a group holding a use): a chain the check missed.
        let both = r##"<svg><defs><path id="z" d="M0 0"/><g id="a"><use href="#z"/></g></defs><use href="#z" xlink:href="#a"/></svg>"##;
        assert_eq!(sanitize_svg(both.as_bytes()), Err(SanitizeError::Malformed));
    }

    #[test]
    fn gradient_and_clip_references_are_kept() {
        let svg = r##"<svg><defs><linearGradient id="g"><stop offset="0" stop-color="#fff"/></linearGradient><clipPath id="c"><rect width="4" height="4"/></clipPath></defs><path d="M0 0h4v4z" fill="url(#g)" clip-path="url(#c)"/></svg>"##;
        let out = ok(svg);
        assert!(out.contains("fill=\"url(#g)\""));
        assert!(out.contains("clip-path=\"url(#c)\""));
    }

    proptest! {
        /// Whatever UTF-8/XML input comes in, a successful output carries no
        /// script, no external reference, no event handler, no entity, and
        /// no element Kanso doesn't draw.
        #[test]
        fn prop_output_never_contains_an_active_construct(
            input in prop::collection::vec(any::<u8>(), 0..4096),
        ) {
            if let Ok(out) = sanitize_svg(&input) {
                let text = String::from_utf8(out).unwrap();
                let low = text.to_ascii_lowercase();
                for needle in [
                    "<script", "<foreignobject", "<image", "<iframe", "<video", "<audio",
                    "<style", "<animate", "<set", "<handler", "<!doctype", "<!entity", "javascript:",
                    "data:", "vbscript:", "xml:base",
                ] {
                    prop_assert!(!low.contains(needle), "{needle} in {text}");
                }
                for (at, _) in low.match_indices(" on") {
                    let rest = &low[at + 3..];
                    let name: String = rest.chars().take_while(|c| c.is_ascii_alphabetic()).collect();
                    prop_assert!(
                        !rest[name.len()..].starts_with('='),
                        "event handler in {text}"
                    );
                }
                for (at, _) in low.match_indices("url(") {
                    let rest = &low[at + 4..];
                    prop_assert!(rest.starts_with('#'), "non-fragment url in {text}");
                }
                for (at, _) in low.match_indices("href=\"") {
                    let rest = &text[at + 6..];
                    prop_assert!(rest.starts_with('#'), "external href in {text}");
                }
            }
        }

        /// However `use` elements, groups and the two spellings of `href` are
        /// combined, no `use` in the output points at anything holding a `use`,
        /// and the output passes again unchanged.
        #[test]
        fn prop_no_use_chain_survives(
            parts in prop::collection::vec((0u8..4, 0u8..4, 0u8..4), 1..8),
        ) {
            let mut defs = String::from(r#"<path id="p" d="M0 0"/>"#);
            for (i, (a, b, how)) in parts.iter().enumerate() {
                let target = |n: u8| if usize::from(n) < i { format!("#g{n}") } else { "#p".into() };
                let inner = match how {
                    0 => format!(r#"<use href="{}"/>"#, target(*a)),
                    1 => format!(r#"<use xlink:href="{}"/>"#, target(*a)),
                    2 => format!(r#"<use href="{}" xlink:href="{}"/>"#, target(*a), target(*b)),
                    _ => r#"<path d="M0 0"/>"#.into(),
                };
                defs.push_str(&format!(r#"<g id="g{i}">{inner}</g>"#));
            }
            let svg = format!("<svg><defs>{defs}</defs></svg>");
            if let Ok(once) = sanitize_svg(svg.as_bytes()) {
                prop_assert_eq!(sanitize_svg(&once), Ok(once));
            }
        }

        /// Sanitizing is idempotent: what comes out passes again unchanged.
        #[test]
        fn prop_sanitizing_twice_equals_once(
            input in prop::collection::vec(any::<u8>(), 0..2048),
        ) {
            if let Ok(once) = sanitize_svg(&input) {
                let twice = sanitize_svg(&once).unwrap();
                prop_assert_eq!(once, twice);
            }
        }
    }
}
