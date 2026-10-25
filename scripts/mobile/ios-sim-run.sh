#!/usr/bin/env bash
# Cargo's runner for aarch64-apple-ios-sim: runs one test binary in the
# booted simulator (scripts/mobile-test.sh sets KANSO_IOS_SIM_UDID).
# Usage: ios-sim-run.sh <test binary> [arguments]
#
# The simulator's processes see the Mac's files, so the fixtures are read where
# they are. `simctl spawn` passes on the variables that start with
# SIMCTL_CHILD_, without the prefix. It starts the binary in the Simulator's
# own folder, not the crate's as cargo would, and a shell in between can't
# help (the Mac's /bin/sh strips the variable the Simulator's loader needs),
# so the tests read their fixtures by absolute path (CARGO_MANIFEST_DIR).
set -euo pipefail

[ -n "${PROPTEST_RNG_SEED:-}" ] && export SIMCTL_CHILD_PROPTEST_RNG_SEED="$PROPTEST_RNG_SEED"
export SIMCTL_CHILD_CARGO_MANIFEST_DIR="${CARGO_MANIFEST_DIR:-}"
export SIMCTL_CHILD_RUST_BACKTRACE=1
export SIMCTL_CHILD_TMPDIR="${TMPDIR:-/tmp}"

exec xcrun simctl spawn "$KANSO_IOS_SIM_UDID" "$@"
