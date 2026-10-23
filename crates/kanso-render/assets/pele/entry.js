// Pele 1.9.0 (MIT, https://github.com/obsidianmd/pele) drawing Mermaid as
// SVG. Kanso runs pele.global.js, then this file, in QuickJS
// (crates/kanso-render/src/pele.rs). There's no canvas, so Pele measures
// text from its own table of Arial widths.

globalThis.kansoPele = function (source, font) {
  try {
    const drawn = pele.render(source, {
      fontFamily: font,
      fontSize: 14,
      responsive: false,
      // Notes are untrusted: no links or images in a drawing.
      links: false,
      images: false,
    });
    return JSON.stringify({ svg: drawn.svg });
  } catch (error) {
    return JSON.stringify({
      error: String((error && error.message) || error),
      code: (error && error.code) || null,
      type: (error && error.mermaidType) || null,
    });
  }
};
