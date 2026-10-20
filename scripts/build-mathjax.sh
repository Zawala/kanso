#!/usr/bin/env bash
# Rebuilds crates/kanso-render/assets/mathjax/mathjax-tex-svg.js, the MathJax
# bundle the preview typesets math with, from entry.js beside it.
#
# Needs Node (for npm) and Bun (for bundling). The result is committed, so a
# Kanso build never needs either.
set -euo pipefail

here="$(cd "$(dirname "$0")/.." && pwd)"
assets="$here/crates/kanso-render/assets/mathjax"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

cp "$assets/entry.js" "$work/entry.js"
cd "$work"
npm init -y >/dev/null
npm install --no-audit --no-fund mathjax-full@3.2.2 >/dev/null
bun build entry.js --minify --format=iife --target=browser --outfile="$assets/mathjax-tex-svg.js"
cp node_modules/mathjax-full/LICENSE "$assets/LICENSE"
ls -l "$assets/mathjax-tex-svg.js"
