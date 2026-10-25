#!/usr/bin/env bash
# Cargo's runner for the Android targets: copies one test binary to the
# device and runs it there (scripts/mobile-test.sh has already copied the
# fixtures to $KANSO_ANDROID_ROOT/repo, in the repository's own layout).
# Usage: android-run.sh <test binary> [arguments]
#
# `adb shell` returns the command's exit status, so a failing test fails the
# run. The crate's folder on the device is where the test reads its fixtures.
set -euo pipefail

binary="$1"
shift
root="${KANSO_ANDROID_ROOT:-/data/local/tmp/kanso}"
name="$(basename "$binary")"
rel="${CARGO_MANIFEST_DIR#"$KANSO_ROOT"/}"
crate="$root/repo/$rel"

adb push "$binary" "$root/bin/$name" >/dev/null
adb shell "chmod 755 $root/bin/$name"

# Each argument in single quotes for the device's shell.
quoted=""
for arg in "$@"; do
	quoted="$quoted '${arg//\'/\'\\\'\'}'"
done
exec adb shell "mkdir -p '$crate' && cd '$crate' && TMPDIR='$root/tmp' CARGO_MANIFEST_DIR='$crate' ${PROPTEST_RNG_SEED:+PROPTEST_RNG_SEED='$PROPTEST_RNG_SEED'} RUST_BACKTRACE=1 '$root/bin/$name' $quoted"
