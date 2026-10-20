//! A note as one self-contained HTML page, in a preview theme's colors:
//! math, diagrams and drawings inline as SVG, attached images as data URIs,
//! no scripts and nothing fetched (a Content Security Policy says so). It
//! opens anywhere, prints, and becomes a PDF.
//!
//! Raw HTML in a note is shown as text, except an attached image's `<img>`
//! tag; links into the app keep their text but go nowhere.

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
        Node::Paragraph(_) if attached_scene(node).is_some() => attached_scene(node)
            .and_then(|id| attachments(&id))
            .and_then(|(_, bytes)| String::from_utf8(bytes).ok())
            .and_then(|scene| drawn(Kind::Excalidraw, &scene))
            .map(|svg| Piece::Block(svg_markup(&svg, "kanso-drawing", 1.))),
        Node::Image(image) => attachment_id(&image.url)
            .map(|id| Piece::Inline(attached_image(&id, &image.alt, None, attachments))),
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

/// The note's body as HTML, without the page around it.
pub fn body(markdown: &str, palette: &Palette, attachments: Attachments<'_>) -> String {
    let options = parse_options();
    let Ok(root) = markdown::to_mdast(markdown, &options) else {
        return format!("<pre>{}</pre>", escape(markdown));
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
    let mut html = markdown::to_html_with_options(
        &text,
        &markdown::Options {
            parse: options,
            compile,
        },
    )
    .unwrap_or_else(|_| format!("<pre>{}</pre>", escape(markdown)));
    // Links into the app (and other schemes a browser can't follow) lose
    // their address, so they aren't links that go nowhere.
    html = html.replace("<a href=\"\">", "<a>");
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
@media print{{body{{max-width:none;padding:0}}pre,figure,table,img{{break-inside:avoid}}h1,h2,h3{{break-after:avoid}}}}
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
