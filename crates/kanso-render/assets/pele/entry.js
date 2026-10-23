// Pele 1.9.0 (MIT, https://github.com/obsidianmd/pele) drawing Mermaid as
// SVG. Kanso runs MathJax's bundle, then pele.global.js, then this file, in
// one QuickJS engine (crates/kanso-render/src/js.rs). There's no canvas, so
// Pele measures text from its own table of Arial widths.

// `$$…$$` in a label, typeset by MathJax (`kansoTex2Svg`, from
// assets/mathjax/entry.js) at the label's size. Null shows the TeX source.
function kansoMath(tex, size) {
  if (typeof globalThis.kansoTex2Svg !== 'function') return null;
  const answer = JSON.parse(globalThis.kansoTex2Svg(tex, true));
  if (!answer.svg) return null;
  let svg = answer.svg;
  // MathJax sizes in ex, and an ex is half the font size.
  const ex = (name) => {
    const found = svg.match(new RegExp(' ' + name + '="([0-9.]+)ex"'));
    return found ? (parseFloat(found[1]) * size) / 2 : 0;
  };
  const width = ex('width');
  const height = ex('height');
  if (!(width > 0 && height > 0)) return null;
  svg = svg
    .replace(/ width="[0-9.]+ex"/, ' width="' + width + '"')
    .replace(/ height="[0-9.]+ex"/, ' height="' + height + '"')
    .replace(/ style="[^"]*"/, '');
  return { svg, width, height };
}

globalThis.kansoPele = function (source, font) {
  try {
    const drawn = pele.render(source, {
      fontFamily: font,
      fontSize: 14,
      responsive: false,
      // Notes are untrusted: no links or images in a drawing.
      links: false,
      images: false,
      math: kansoMath,
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
