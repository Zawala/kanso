#!/bin/sh
# Checks every theme in this folder and brings the generated files up to
# date:
#
#   1. reads each theme with Kanso's own parser and checks it is legible
#      (`kanso theme check`), and, when `npx` is available, validates it
#      against schema/kanso-theme-v1.json with ajv;
#   2. writes index.json (`kanso theme index`);
#   3. renders each theme's three cards to thumbnails/<file>.png with the
#      app's headless snapshot harness.
#
# It needs a Kanso checkout to build the tools: this folder's parent when it
# lives in one (themes/ in the Kanso repository), or $KANSO_REPO.
#
#   themes/scripts/check.sh            # check, index and thumbnails
#   themes/scripts/check.sh --no-thumbs
set -eu

here=$(cd "$(dirname "$0")/.." && pwd)
repo=${KANSO_REPO:-$(cd "$here/.." && pwd)}
if [ ! -f "$repo/crates/kanso-app/Cargo.toml" ]; then
    echo "Set KANSO_REPO to a Kanso checkout; $repo isn't one." >&2
    exit 1
fi
thumbs=1
[ "${1:-}" = "--no-thumbs" ] && thumbs=0

cd "$repo"
cargo build -q -p kanso-app
kanso="$repo/target/debug/kanso"

themes=$(find "$here" -maxdepth 1 -name '*.json' ! -name index.json | sort)
# shellcheck disable=SC2086
"$kanso" theme check $themes

if command -v npx >/dev/null 2>&1 && [ "${KANSO_SKIP_AJV:-}" = "" ]; then
    for theme in $themes; do
        npx --yes ajv-cli@5 validate --spec=draft2020 --strict=false \
            -s "$here/schema/kanso-theme-v1.json" -d "$theme" >/dev/null
    done
    echo "schema: every theme validates"
fi

"$kanso" theme index "$here" --out "$here/index.json"

if [ "$thumbs" = 1 ]; then
    KANSO_THEMES_DIR="$here" KANSO_THEME_THUMBS_DIR="$here/thumbnails" \
        cargo test -q -p kanso-app theme_thumbnails -- --ignored --nocapture
fi
