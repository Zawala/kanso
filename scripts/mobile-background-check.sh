#!/usr/bin/env bash
# Proves the background sync entry (#48) runs with no Flutter: builds the
# mobile library for this computer, compiles a small C program that calls
# `kanso_background_sync` (mobile/rust/tests/c_abi/main.c), and runs it. The
# program is a plain C process, with no Dart and no JVM in it, as the iOS and
# Android background jobs are when the system wakes the app.
#
# Also checks the library exports the symbol. Usage:
#   scripts/mobile-background-check.sh
#
# The library is the debug build of the cdylib; it is the same code as the
# phones' (the entry point does not depend on the profile).
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$here/.." && pwd)"
cd "$root"

target_dir="${CARGO_TARGET_DIR:-$root/target}"
cargo build --locked -p kanso-mobile-bridge

case "$(uname -s)" in
Darwin)
	lib="$target_dir/debug/libkanso_mobile_bridge.dylib"
	library_path_var=DYLD_LIBRARY_PATH
	;;
*)
	lib="$target_dir/debug/libkanso_mobile_bridge.so"
	library_path_var=LD_LIBRARY_PATH
	;;
esac
[ -f "$lib" ] || {
	echo "::error::$lib wasn't built."
	exit 1
}

# The symbol is defined (exported) by the library itself, not left to a
# dependency's rlib (rust-lang/rust#47384).
if ! nm -g "$lib" | grep -E ' T _?kanso_background_sync$' >/dev/null; then
	echo "::error::$lib doesn't export kanso_background_sync."
	exit 1
fi

work="$(mktemp -d -t kanso-c-abi)"
trap 'rm -rf "$work"' EXIT
cc -Wall -Wextra -Werror -o "$work/c_abi" mobile/rust/tests/c_abi/main.c \
	-L "$(dirname "$lib")" -lkanso_mobile_bridge
env "$library_path_var=$(dirname "$lib")" "$work/c_abi"
echo "kanso_background_sync answered from a plain C process."
