//! A note as one self-contained HTML page, in a preview theme's colors:
//! math, diagrams and drawings inline as SVG, attached images as data URIs,
//! no scripts and nothing fetched (a Content Security Policy says so). It
//! opens anywhere, prints, and becomes a PDF.
//!
//! Raw HTML in a note is shown as text, except an attached image's `<img>`
//! tag; links into the app keep their text but go nowhere, and wiki-links
//! (`[[Title]]`, `[[Title#Heading]]`, `[[Title|alias]]`) show as their label.

use std::fmt::Write as _;
use std::hash::{Hash as _, Hasher as _};

use base64::Engine as _;
use markdown::mdast::Node;

use crate::{Job, Kind, Palette, Rgb, Svg, escape, render_blocking};

/// An attachment's content type and bytes, by ID (`file:…`).
pub type Attachments<'a> = &'a dyn Fn(&str) -> Option<(String, Vec<u8>)>;

/// The parse options the preview uses, with math.
fn parse_options() -> markdown::ParseOptions {
    let mut options = markdown::ParseOptions::gfm();
    options.constructs.frontmatter = false;
    options.constructs.math_text = true;
    options.constructs.math_flow = true;
    options
}

/// What a piece of the note becomes.
enum Piece {
    /// Stands alone, in place of a paragraph.
    Block(String),
    /// Sits in a line.
    Inline(String),
}

/// The attachment ID an `inkdrop://file:` URL points at. Pure.
pub fn attachment_id(url: &str) -> Option<String> {
    let id = url.strip_prefix("inkdrop://file:")?;
    let id = id.split(['?', '#', '/', ')', ' ']).next()?;
    (!id.is_empty()).then(|| format!("file:{id}"))
}

/// An `<img>` tag's `src`, `alt` and `width`. Pure.
pub fn img_tag(html: &str) -> Option<(String, String, Option<u32>)> {
    let html = html.trim();
    let rest = html
        .strip_prefix("<img")
        .or_else(|| html.strip_prefix("<IMG"))?;
    if !rest.starts_with(|c: char| c.is_whitespace()) {
        return None;
    }
    let rest = rest.trim_end_matches('>').trim_end_matches('/');
    let (mut src, mut alt, mut width) = (None, String::new(), None);
    let mut chars = rest;
    loop {
        chars = chars.trim_start();
        if chars.is_empty() {
            break;
        }
        let name_end = chars
            .find(|c: char| c == '=' || c.is_whitespace())
            .unwrap_or(chars.len());
        let name = chars[..name_end].to_ascii_lowercase();
        chars = chars[name_end..].trim_start();
        let Some(after) = chars.strip_prefix('=') else {
            continue;
        };
        let after = after.trim_start();
        let (value, tail) = match after.chars().next() {
            Some(q @ ('"' | '\'')) => {
                let end = after[1..].find(q)? + 1;
                (&after[1..end], &after[end + 1..])
            }
            _ => {
                let end = after.find(char::is_whitespace).unwrap_or(after.len());
                (&after[..end], &after[end..])
            }
        };
        match name.as_str() {
            "src" => src = Some(value.to_owned()),
            "alt" => alt = value.to_owned(),
            "width" => width = value.trim_end_matches("px").parse().ok(),
            _ => {}
        }
        chars = tail;
    }
    Some((src?, alt, width))
}

/// An image's alt text without an Obsidian-style width after a `|`
/// (`Cat|300`, or `Cat|300x200`, whose height is left to the image), and
/// that width. Pure.
pub fn split_image_width(alt: &str) -> (&str, Option<u32>) {
    let Some((text, size)) = alt.rsplit_once('|') else {
        return (alt, None);
    };
    let size = size.trim();
    let width = size.split_once(['x', 'X']).map_or(size, |(w, _)| w);
    if width.is_empty() || width.len() > 5 || !width.bytes().all(|b| b.is_ascii_digit()) {
        return (alt, None);
    }
    match width.parse::<u32>() {
        Ok(width) if width > 0 => (text.trim_end(), Some(width)),
        _ => (alt, None),
    }
}

/// The label a wiki-link's inside (`Title`, `Title#Heading` or
/// `Title|alias`, as compiled HTML) shows: the alias, else the title and
/// heading. `None` when it has no words. Pure.
fn wiki_label(inner: &str) -> Option<String> {
    let label = match inner.split_once('|') {
        Some((_, alias)) if !alias.trim().is_empty() => alias.trim().to_owned(),
        Some((target, _)) => heading_label(target),
        None => heading_label(inner),
    };
    let words = label.replace(['<', '>'], "");
    (!words.trim().is_empty()).then_some(label)
}

/// `Title#Heading` as "Title › Heading", or the heading alone.
fn heading_label(target: &str) -> String {
    match target.split_once('#') {
        Some((title, heading)) if title.trim().is_empty() => heading.trim().to_owned(),
        Some((title, heading)) if heading.trim().is_empty() => title.trim().to_owned(),
        Some((title, heading)) => format!("{} › {}", title.trim(), heading.trim()),
        None => target.trim().to_owned(),
    }
}

/// Inline tags a wiki-link's text may hold, as Markdown compiles it.
const WIKI_INLINE_TAGS: [&str; 6] = ["<em>", "</em>", "<strong>", "</strong>", "<del>", "</del>"];

/// The wiki-link at the start of `html` (which starts `[[`): its label and
/// how long it is. Pure.
fn wiki_link_at(html: &str) -> Option<(String, usize)> {
    let close = html.get(2..)?.find("]]")? + 2;
    let inner = &html[2..close];
    if inner.contains('\n') || inner.contains("[[") {
        return None;
    }
    // Only inline emphasis inside: no code, links or block tags.
    let mut rest = inner;
    while let Some(at) = rest.find('<') {
        let tag_end = rest[at..].find('>')? + at + 1;
        if !WIKI_INLINE_TAGS.contains(&&rest[at..tag_end]) {
            return None;
        }
        rest = &rest[tag_end..];
    }
    Some((wiki_label(inner)?, close + 2))
}

/// `[[Title]]`, `[[Title#Heading]]` and `[[Title|alias]]` in compiled
/// HTML, outside code, as their label: a standalone page has no note to
/// link to. Pure.
pub fn wiki_links(html: &str) -> String {
    if !html.contains("[[") {
        return html.to_owned();
    }
    let mut out = String::with_capacity(html.len());
    let mut in_code = 0usize;
    let mut rest = html;
    while let Some(c) = rest.chars().next() {
        if c == '<' {
            let end = rest.find('>').map_or(rest.len(), |e| e + 1);
            let tag = rest[..end].to_ascii_lowercase();
            if tag.starts_with("<code") || tag.starts_with("<pre") {
                in_code += 1;
            } else if tag.starts_with("</code") || tag.starts_with("</pre") {
                in_code = in_code.saturating_sub(1);
            }
            out.push_str(&rest[..end]);
            rest = &rest[end..];
            continue;
        }
        if in_code == 0
            && rest.starts_with("[[")
            && let Some((label, len)) = wiki_link_at(rest)
        {
            let _ = write!(out, r#"<span class="kanso-wikilink">{label}</span>"#);
            rest = &rest[len..];
            continue;
        }
        out.push(c);
        rest = &rest[c.len_utf8()..];
    }
    out
}

/// Whether a fence's info string names `want`.
fn fence_is(lang: &Option<String>, want: &str) -> bool {
    lang.as_deref()
        .is_some_and(|l| l.trim().eq_ignore_ascii_case(want))
}

/// The attached Excalidraw scene a paragraph is, if it is only a link (or
/// image) to an `inkdrop://file:` named `….excalidraw`. Pure.
pub fn attached_scene(node: &Node) -> Option<String> {
    let Node::Paragraph(paragraph) = node else {
        return None;
    };
    let (url, label) = match paragraph.children.as_slice() {
        [Node::Link(link)] => (link.url.as_str(), node_text(&link.children)),
        [Node::Image(image)] => (image.url.as_str(), image.alt.clone()),
        _ => return None,
    };
    let label = label.trim().to_ascii_lowercase();
    (label.ends_with(".excalidraw") || label.ends_with(".excalidraw.json"))
        .then(|| attachment_id(url))
        .flatten()
}

/// The formula a paragraph is, if it is only one `$$…$$`: display math as
/// KaTeX writes it on one line. `markdown` is the paragraph's source. Pure.
pub fn display_math_paragraph<'a>(node: &'a Node, markdown: &str) -> Option<&'a str> {
    let Node::Paragraph(paragraph) = node else {
        return None;
    };
    let [Node::InlineMath(math)] = paragraph.children.as_slice() else {
        return None;
    };
    let markdown = markdown.trim();
    (markdown.len() > 4 && markdown.starts_with("$$") && markdown.ends_with("$$"))
        .then_some(math.value.as_str())
}

fn node_text(nodes: &[Node]) -> String {
    nodes.iter().map(Node::to_string).collect()
}

fn svg_markup(svg: &Svg, class: &str, scale: f32) -> String {
    let style = match svg.baseline {
        Some(baseline) if class == "kanso-math" => format!(
            " style=\"vertical-align:{}px\"",
            crate::svg::number(-(svg.height - baseline) * scale)
        ),
        _ => String::new(),
    };
    format!(r#"<span class="{class}"{style}>{}</span>"#, svg.markup)
}

fn data_uri(content_type: &str, bytes: &[u8]) -> String {
    format!(
        "data:{};base64,{}",
        content_type,
        base64::engine::general_purpose::STANDARD.encode(bytes)
    )
}

/// The image an attachment is, as an `<img>`; its alt text if it is gone.
fn attached_image(id: &str, alt: &str, width: Option<u32>, attachments: Attachments<'_>) -> String {
    let safe_type = |t: &str| t.starts_with("image/") && !t.contains(['"', '<', '>']);
    match attachments(id) {
        Some((content_type, bytes)) if safe_type(&content_type) => format!(
            r#"<img src="{}" alt="{}"{}>"#,
            data_uri(&content_type, &bytes),
            escape(alt),
            width
                .map(|w| format!(r#" width="{w}""#))
                .unwrap_or_default()
        ),
        _ => format!(r#"<span class="kanso-missing">{}</span>"#, escape(alt)),
    }
}

/// Finds what the note's Markdown can't say in HTML itself, depth first.
fn collect(
    node: &Node,
    source: &str,
    palette: &Palette,
    attachments: Attachments<'_>,
    out: &mut Vec<(std::ops::Range<usize>, Piece)>,
) {
    let range = node.position().map(|p| p.start.offset..p.end.offset);
    // The node as written.
    let written = range
        .clone()
        .and_then(|r| source.get(r))
        .unwrap_or_default();
    let drawn = |kind: Kind, code: &str| -> Option<Svg> {
        match render_blocking(&Job::new(kind, code, *palette)).as_ref() {
            Ok(svg) => Some(svg.clone()),
            Err(_) => None,
        }
    };
    let piece = match node {
        Node::Math(math) => drawn(Kind::DisplayMath, &math.value)
            .map(|svg| Piece::Block(svg_markup(&svg, "kanso-math-display", 1.))),
        Node::InlineMath(math) => {
            let range = range.clone().unwrap_or_default();
            let span = source.get(range.clone()).unwrap_or_default();
            let next = source.get(range.end..).and_then(|rest| rest.chars().next());
            if crate::is_inline_math(span, next) {
                drawn(Kind::InlineMath, &math.value)
                    .map(|svg| Piece::Inline(svg_markup(&svg, "kanso-math", 1.)))
            } else {
                // Dollar amounts, not math: the text as written.
                Some(Piece::Inline(escape(span)))
            }
        }
        Node::Code(code) if fence_is(&code.lang, "math") => drawn(Kind::DisplayMath, &code.value)
            .map(|svg| Piece::Block(svg_markup(&svg, "kanso-math-display", 1.))),
        Node::Code(code) if fence_is(&code.lang, "mermaid") => drawn(Kind::Mermaid, &code.value)
            .map(|svg| Piece::Block(svg_markup(&svg, "kanso-diagram", 1.))),
        Node::Code(code) if fence_is(&code.lang, "excalidraw") => {
            drawn(Kind::Excalidraw, &code.value)
                .map(|svg| Piece::Block(svg_markup(&svg, "kanso-drawing", 1.)))
        }
        Node::Paragraph(_) if display_math_paragraph(node, written).is_some() => {
            display_math_paragraph(node, written)
                .and_then(|formula| drawn(Kind::DisplayMath, formula))
                .map(|svg| Piece::Block(svg_markup(&svg, "kanso-math-display", 1.)))
        }
        Node::Paragraph(_) if attached_scene(node).is_some() => attached_scene(node)
            .and_then(|id| attachments(&id))
            .and_then(|(_, bytes)| String::from_utf8(bytes).ok())
            .and_then(|scene| drawn(Kind::Excalidraw, &scene))
            .map(|svg| Piece::Block(svg_markup(&svg, "kanso-drawing", 1.))),
        Node::Image(image) => {
            let (alt, width) = split_image_width(&image.alt);
            match attachment_id(&image.url) {
                Some(id) => Some(Piece::Inline(attached_image(&id, alt, width, attachments))),
                // A web image given a width: the same tag Markdown would
                // write, with the width and without the `|300`.
                None => width
                    .filter(|_| {
                        image.url.starts_with("https://") || image.url.starts_with("http://")
                    })
                    .map(|width| {
                        Piece::Inline(format!(
                            r#"<img src="{}" alt="{}" width="{width}">"#,
                            escape(&image.url),
                            escape(alt)
                        ))
                    }),
            }
        }
        Node::Html(html) => img_tag(&html.value).and_then(|(src, alt, width)| {
            attachment_id(&src)
                .map(|id| Piece::Inline(attached_image(&id, &alt, width, attachments)))
        }),
        _ => None,
    };
    match (piece, range) {
        (Some(piece), Some(range)) if source.get(range.clone()).is_some() => {
            out.push((range, piece))
        }
        _ => {
            for child in node.children().into_iter().flatten() {
                collect(child, source, palette, attachments, out);
            }
        }
    }
}

/// `markdown` as plain paragraphs, for text markdown-rs can't parse: its
/// lines as written, a blank line between paragraphs.
fn plain_html(markdown: &str) -> String {
    markdown
        .split("\n\n")
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .map(|p| format!("<p>{}</p>\n", escape(p).replace('\n', "<br>\n")))
        .collect()
}

/// The note's body as HTML, without the page around it.
pub fn body(markdown: &str, palette: &Palette, attachments: Attachments<'_>) -> String {
    let options = parse_options();
    let Ok(root) = crate::markdown_safe::to_mdast(markdown, &options) else {
        return plain_html(markdown);
    };
    let mut pieces = Vec::new();
    collect(&root, markdown, palette, attachments, &mut pieces);
    pieces.sort_by_key(|(range, _)| range.start);
    // A placeholder no note will contain: it survives Markdown as a word.
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    markdown.hash(&mut hasher);
    let nonce = format!("{:x}", hasher.finish());
    let token = |n: usize| format!("kansopiece{nonce}n{n}x");
    let mut text = String::with_capacity(markdown.len());
    let mut at = 0;
    let mut kept = Vec::new();
    for (range, piece) in pieces {
        if range.start < at {
            continue;
        }
        text.push_str(&markdown[at..range.start]);
        text.push_str(&token(kept.len()));
        at = range.end;
        kept.push(piece);
    }
    text.push_str(&markdown[at..]);
    let compile = markdown::CompileOptions {
        gfm_tagfilter: true,
        ..markdown::CompileOptions::default()
    };
    let Ok(mut html) = crate::markdown_safe::to_html_with_options(
        &text,
        &markdown::Options {
            parse: options,
            compile,
        },
    ) else {
        return plain_html(markdown);
    };
    // Links into the app (and other schemes a browser can't follow) lose
    // their address, so they aren't links that go nowhere.
    html = html.replace("<a href=\"\">", "<a>");
    html = wiki_links(&html);
    for (n, piece) in kept.into_iter().enumerate() {
        let token = token(n);
        html = match piece {
            Piece::Block(markup) => {
                let paragraph = format!("<p>{token}</p>");
                if html.contains(&paragraph) {
                    html.replacen(&paragraph, &format!("<figure>{markup}</figure>"), 1)
                } else {
                    html.replacen(&token, &markup, 1)
                }
            }
            Piece::Inline(markup) => html.replacen(&token, &markup, 1),
        };
    }
    html
}

fn css(p: &Palette) -> String {
    let c = |rgb: Rgb| rgb.hex();
    let quiet = c(p.border.over(p.background, 0.5));
    format!(
        r#":root{{color-scheme:{scheme}}}
*{{box-sizing:border-box}}
html{{background:{bg};-webkit-print-color-adjust:exact;print-color-adjust:exact}}
body{{margin:0 auto;max-width:46rem;padding:3rem 2rem 4rem;background:{bg};color:{text};font:16px/1.65 -apple-system,BlinkMacSystemFont,"Segoe UI",Inter,"Helvetica Neue",Arial,sans-serif;-webkit-font-smoothing:antialiased}}
header h1{{font-size:2rem;line-height:1.25;margin:0 0 1.5rem}}
h1,h2,h3,h4,h5,h6{{line-height:1.3;margin:1.8em 0 .5em}}
h1{{font-size:1.8rem}}h2{{font-size:1.35rem}}h3{{font-size:1.15rem}}h4,h5,h6{{font-size:1rem}}
p,ul,ol,pre,table,blockquote,figure{{margin:0 0 .9em}}
a{{color:{accent};text-decoration:underline;text-underline-offset:2px}}
a:not([href]){{color:inherit;text-decoration-style:dotted}}
code,pre{{font-family:ui-monospace,"SF Mono",Menlo,Consolas,"DejaVu Sans Mono",monospace;font-size:.88em}}
code{{background:{surface};border-radius:4px;padding:.1em .3em}}
pre{{background:{surface};border:1px solid {quiet};border-radius:8px;padding:.9em 1em;overflow-x:auto;line-height:1.5}}
pre code{{background:none;padding:0;font-size:inherit}}
blockquote{{border-left:3px solid {quiet};margin-left:0;padding-left:14px;color:{muted}}}
table{{border-collapse:collapse;display:block;overflow-x:auto}}
th,td{{border:1px solid {border};padding:.35em .7em;text-align:left}}
th{{background:{surface};font-weight:600}}
hr{{border:0;border-top:1px solid {border};margin:2em 0}}
img{{max-width:100%;height:auto}}
li>input[type=checkbox]{{margin:0 .5em 0 -1.4em}}
ul.contains-task-list{{list-style:none}}
li.task-list-item{{list-style:none}}
figure{{margin-left:0;margin-right:0;overflow-x:auto}}
figure svg{{max-width:100%;height:auto}}
.kanso-math svg{{display:inline-block}}
.kanso-math-display{{display:block;text-align:center;margin:.4em 0}}
.kanso-diagram,.kanso-drawing{{display:block}}
.kanso-missing{{color:{muted};font-style:italic}}
.kanso-wikilink{{border-bottom:1px dotted {muted}}}
@media print{{html,body{{background:none}}body{{max-width:none;padding:0}}pre,figure,table,img{{break-inside:avoid}}h1,h2,h3{{break-after:avoid}}}}
"#,
        scheme = if p.dark { "dark" } else { "light" },
        bg = c(p.background),
        text = c(p.text),
        muted = c(p.muted),
        accent = c(p.accent),
        border = c(p.border),
        surface = c(p.surface),
    )
}

/// A whole page: the note's title and body, styled, standalone.
pub fn page(
    title: &str,
    markdown: &str,
    palette: &Palette,
    attachments: Attachments<'_>,
) -> String {
    let body = body(markdown, palette, attachments);
    let mut html = String::with_capacity(body.len() + 4096);
    let title = title.trim();
    let shown_title = if title.is_empty() { "Untitled" } else { title };
    let _ = write!(
        html,
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta http-equiv="Content-Security-Policy" content="default-src 'none'; img-src data:; style-src 'unsafe-inline'">
<meta name="generator" content="Kanso">
<title>{title}</title>
<style>{css}</style>
</head>
<body>
<header><h1>{title}</h1></header>
<main>
{body}</main>
</body>
</html>
"#,
        title = escape(shown_title),
        css = css(palette),
    );
    html
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn none(_: &str) -> Option<(String, Vec<u8>)> {
        None
    }

    #[test]
    fn a_note_becomes_a_standalone_page() {
        let note = "Intro with $x^2$ inline.\n\n$$\n\\int_0^1 x\\,dx\n$$\n\n```mermaid\ngraph TD\n  A --> B\n```\n\n- [x] done\n- [ ] open\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\n<script>alert(1)</script>\n\n[app link](inkdrop://note/abc) and [web](https://example.com)\n";
        let html = page("My <note>", note, &Palette::LIGHT, &none);
        assert!(html.starts_with("<!doctype html>"));
        assert!(html.contains("<title>My &lt;note&gt;</title>"));
        assert!(html.contains("class=\"kanso-math\""));
        assert!(html.contains("class=\"kanso-math-display\""));
        assert!(html.contains("class=\"kanso-diagram\""));
        assert!(html.contains("<table>"));
        assert!(html.contains("type=\"checkbox\""));
        // No scripts, and the policy forbids them anyway.
        assert!(!html.contains("<script"));
        assert!(html.contains("default-src 'none'"));
        assert!(html.contains("<a>app link</a>"));
        assert!(html.contains("href=\"https://example.com\""));
        assert!(!html.contains("kansopiece"));
    }

    #[test]
    fn one_line_double_dollars_are_display_math() {
        let html = body(
            "$$\\sum_{k=1}^n k$$\n\nText $$x$$ inline.\n",
            &Palette::LIGHT,
            &none,
        );
        assert_eq!(html.matches("class=\"kanso-math-display\"").count(), 1);
        assert_eq!(html.matches("class=\"kanso-math\"").count(), 1);
        assert!(!html.contains("$$"));
    }

    proptest! {
        /// Invariants: a paragraph is display math exactly when it is one
        /// `$$…$$` and nothing else, and the formula is what is inside.
        #[test]
        fn prop_display_math_paragraphs(formula in "[a-z0-9+=^]{1,12}", before in "[A-Za-z ]{0,6}") {
            let markdown = format!("{before}$${formula}$$");
            let tree = markdown::to_mdast(&markdown, &parse_options()).unwrap();
            let Some(paragraph) = tree.children().and_then(|c| c.first()) else {
                return Ok(());
            };
            let found = display_math_paragraph(paragraph, &markdown);
            if before.trim().is_empty() {
                prop_assert_eq!(found, Some(formula.as_str()));
            } else {
                prop_assert_eq!(found, None);
            }
        }
    }

    #[test]
    fn attachments_are_inlined() {
        let png = vec![0x89, b'P', b'N', b'G'];
        let attachments =
            |id: &str| (id == "file:abc").then(|| ("image/png".to_owned(), png.clone()));
        let html = body(
            "![Cat](inkdrop://file:abc)\n\n<img src=\"inkdrop://file:abc\" width=\"240\">\n\n![Gone](inkdrop://file:zzz)\n",
            &Palette::DARK,
            &attachments,
        );
        assert_eq!(
            html.matches("src=\"data:image/png;base64,iVBORw==\"")
                .count(),
            2,
            "{html}"
        );
        assert!(html.contains("width=\"240\""));
        assert!(html.contains("kanso-missing\">Gone<"));
    }

    #[test]
    fn diagrams_that_cannot_be_drawn_stay_code() {
        let html = body(
            "```mermaid\nsparkleDiagram\n  a\n```\n",
            &Palette::LIGHT,
            &none,
        );
        assert!(html.contains("<pre><code class=\"language-mermaid\">"));
        let html = body("Price $5 and $10.\n", &Palette::LIGHT, &none);
        assert!(html.contains("$5"), "{html}");
    }

    /// markdown 1.0.0 panics on this; the export shows it as plain text.
    #[test]
    fn text_markdown_rs_panics_on_exports_as_plain_paragraphs() {
        let html = body("Para\n---\n---\na\n===\n\nNext <b>", &Palette::LIGHT, &none);
        assert_eq!(
            html,
            "<p>Para<br>\n---<br>\n---<br>\na<br>\n===</p>\n<p>Next &lt;b&gt;</p>\n"
        );
    }

    #[test]
    fn wiki_links_export_as_their_labels() {
        let html = body(
            "See [[Plans]], [[Plans#Next steps]], [[Plans|the plan]] and [[#Intro]].\n\n`[[kept]]` stays code.\n",
            &Palette::LIGHT,
            &none,
        );
        assert!(html.contains(r#"<span class="kanso-wikilink">Plans</span>,"#), "{html}");
        assert!(html.contains(r#"<span class="kanso-wikilink">Plans › Next steps</span>"#));
        assert!(html.contains(r#"<span class="kanso-wikilink">the plan</span>"#));
        assert!(html.contains(r#"<span class="kanso-wikilink">Intro</span>"#));
        assert!(html.contains("<code>[[kept]]</code>"), "{html}");
        let html = body("A [[*big* idea]] here, and [[]] stays.\n", &Palette::LIGHT, &none);
        assert!(html.contains(r#"<span class="kanso-wikilink"><em>big</em> idea</span>"#), "{html}");
        assert!(html.contains("[[]]"));
    }

    #[test]
    fn image_widths_after_a_bar() {
        assert_eq!(split_image_width("Cat|300"), ("Cat", Some(300)));
        assert_eq!(split_image_width("Cat |300x200"), ("Cat", Some(300)));
        assert_eq!(split_image_width("|120"), ("", Some(120)));
        assert_eq!(split_image_width("a|b"), ("a|b", None));
        assert_eq!(split_image_width("Cat|0"), ("Cat|0", None));
        assert_eq!(split_image_width("Cat|9999999"), ("Cat|9999999", None));
        assert_eq!(split_image_width("Cat"), ("Cat", None));
        let png = vec![0x89, b'P', b'N', b'G'];
        let attachments =
            |id: &str| (id == "file:abc").then(|| ("image/png".to_owned(), png.clone()));
        let html = body(
            "![Cat|300](inkdrop://file:abc)\n\n![Web|200](https://example.com/a.png)\n",
            &Palette::LIGHT,
            &attachments,
        );
        assert!(html.contains(r#"alt="Cat" width="300""#), "{html}");
        assert!(
            html.contains(r#"<img src="https://example.com/a.png" alt="Web" width="200">"#),
            "{html}"
        );
        assert!(!html.contains("|300") && !html.contains("|200"));
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(128))]

        /// Invariant: no wiki-link survives export as literal brackets,
        /// whatever its title, heading or alias.
        #[test]
        fn prop_wiki_links_leave_no_brackets(
            title in "[A-Za-z0-9][A-Za-z0-9 ,.'&-]{0,20}",
            heading in prop::option::of("[A-Za-z0-9][A-Za-z0-9 ]{0,10}"),
            alias in prop::option::of("[A-Za-z0-9][A-Za-z0-9 !?]{0,10}"),
            before in "([A-Za-z][A-Za-z ]{0,9})?",
            after in "[A-Za-z .]{0,10}",
            heading_line in any::<bool>(),
        ) {
            let mut link = title.clone();
            if let Some(heading) = &heading {
                link = format!("{link}#{heading}");
            }
            if let Some(alias) = &alias {
                link = format!("{link}|{alias}");
            }
            let line = format!("{before}[[{link}]]{after}");
            let markdown = if heading_line { format!("## {line}\n") } else { format!("{line}\n\n- {line}\n") };
            let html = page("t", &markdown, &Palette::LIGHT, &none);
            prop_assert!(!html.contains("[["), "{}", html);
            prop_assert!(html.contains("kanso-wikilink"));
        }
    }

    #[test]
    fn img_tags_read() {
        assert_eq!(
            img_tag(r#"<img src="inkdrop://file:a" alt='A b' width="240px">"#),
            Some(("inkdrop://file:a".into(), "A b".into(), Some(240)))
        );
        assert_eq!(img_tag("<imgx src=a>"), None);
        assert_eq!(img_tag("<img alt=a>"), None);
        assert_eq!(
            img_tag("<img src=a/>"),
            Some(("a".into(), String::new(), None))
        );
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        /// Invariants: any note exports to a page with no script element
        /// and no placeholder left behind.
        #[test]
        fn prop_pages_have_no_scripts(text in "(\\PC|\n|<script>|\\$|`|\\[|\\]|\\(|\\))*") {
            let html = page("t", &text, &Palette::LIGHT, &none);
            let lower = html.to_ascii_lowercase();
            prop_assert!(!lower.contains("<script"));
            prop_assert!(!html.contains("kansopiece"));
        }

        /// Round trip: an attachment ID inside a URL is found again.
        #[test]
        fn prop_attachment_ids_round_trip(id in "[A-Za-z0-9_-]{1,20}", tail in prop::option::of("[?#][a-z=]{0,6}")) {
            let url = format!("inkdrop://file:{id}{}", tail.unwrap_or_default());
            prop_assert_eq!(attachment_id(&url), Some(format!("file:{id}")));
        }
    }
}
