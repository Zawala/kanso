#!/usr/bin/env bash
# Tests scripts/mobile-version.sh against made-up Cargo.toml files.
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
mkdir -p "$work/scripts"
cp "$here/mobile-version.sh" "$work/scripts/"

fail() {
	echo "test-mobile-version: $1" >&2
	exit 1
}
run() { "$work/scripts/mobile-version.sh" "$@" 2>&1; }
cargo() { printf '[workspace]\nmembers = []\n\n[workspace.package]\nversion = "%s"\nedition = "2021"\n\n[dependencies.x]\nversion = "9.9.9"\n' "$1" >"$work/Cargo.toml"; }

cargo 0.9.0
out="$(run --build-number 17)"
[ "$out" = $'name=0.9.0\nnumber=17' ] || fail "unexpected output: $out"
out="$(run --tag v0.9.0 --build-number 17)" || fail "a matching tag was refused"
out="$(KANSO_BUILD_NUMBER=5 run)" || fail "KANSO_BUILD_NUMBER wasn't read"
[ "$out" = $'name=0.9.0\nnumber=5' ] || fail "unexpected output: $out"

run --tag v0.9.1 --build-number 1 >/dev/null && fail "a tag for another version was accepted"
run >/dev/null && fail "a missing build number was accepted"
run --build-number 0 >/dev/null && fail "build number 0 was accepted"
run --build-number 1e3 >/dev/null && fail "a non-integer build number was accepted"
run --build-number 2100000000 >/dev/null && fail "a build number too big for versionCode was accepted"
cargo 0.9.0-rc.1
run --build-number 1 >/dev/null && fail "a pre-release version was accepted"
echo "test-mobile-version: ok"
