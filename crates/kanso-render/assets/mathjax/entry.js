// MathJax 3 (mathjax-full 3.2.2, Apache-2.0) turning TeX into SVG with its
// liteDOM adaptor: no browser, no DOM, no timers, no network. Kanso runs the
// bundle in QuickJS (crates/kanso-render/src/math.rs).
//
// `scripts/build-mathjax.sh` bundles this file into mathjax-tex-svg.js.

import {mathjax} from 'mathjax-full/js/mathjax.js';
import {TeX} from 'mathjax-full/js/input/tex.js';
import {SVG} from 'mathjax-full/js/output/svg.js';
import {liteAdaptor} from 'mathjax-full/js/adaptors/liteAdaptor.js';
import {RegisterHTMLHandler} from 'mathjax-full/js/handlers/html.js';
import {AllPackages} from 'mathjax-full/js/input/tex/AllPackages.js';

const adaptor = liteAdaptor();
RegisterHTMLHandler(adaptor);

// Everything but the packages that load more code (there is nothing to load
// it from) and the two that hide mistakes: a formula with an error is shown
// as its source instead.
const skipped = ['require', 'autoload', 'noerrors', 'noundefined'];
const tex = new TeX({
  packages: AllPackages.filter((name) => !skipped.includes(name)),
  formatError(jax, error) {
    throw error;
  },
});
// Each formula carries its own glyphs, so it stands alone in an export.
const svg = new SVG({fontCache: 'local'});
const doc = mathjax.document('', {InputJax: tex, OutputJax: svg});

// One formula as `{"svg": "<svg …>"}` or `{"error": "…"}`.
globalThis.kansoTex2Svg = function (source, display) {
  try {
    const node = doc.convert(source, {display, em: 16, ex: 8, containerWidth: 1280});
    return JSON.stringify({svg: adaptor.innerHTML(node)});
  } catch (error) {
    return JSON.stringify({error: String((error && error.message) || error)});
  }
};
