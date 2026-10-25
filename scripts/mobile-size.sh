#!/usr/bin/env bash
# Measures what the bare library weighs on a phone (#48): builds
# kanso-mobile-bridge's dynamic library (the facade and the core, without
# Flutter or the app) with the mobile profile for iOS and Android, unwinding
# and aborting, and prints a Markdown table.
# Usage: scripts/mobile-size.sh [output folder] [target...]
#
# A phone's library is the dylib for iOS (standing in for the static library
# the app links with dead stripping) and the .so for Android. The sizes are
# the raw file, `gzip -9` and `zip -9` (what the stores deliver is smaller
# than the file). The unwinding cost is read from the sections that exist
# for it: `__unwind_info`, `__eh_frame` and `__gcc_except_tab` on iOS,
# `.eh_frame`, `.eh_frame_hdr` and `.gcc_except_table` on Android.
#
# Each build overwrites the last in the target folder, so the library is
# copied out. Needs the Rust targets and the NDK (scripts/mobile/env.sh).
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=mobile/env.sh
. "$here/mobile/env.sh"
cd "$KANSO_ROOT"

out="${1:-$KANSO_ROOT/target/mobile-size}"
[ "$#" -gt 0 ] && shift
if [ "$#" -gt 0 ]; then
	targets=("$@")
else
	targets=(aarch64-apple-ios aarch64-linux-android)
fi
mkdir -p "$out"
target_dir="${CARGO_TARGET_DIR:-$KANSO_ROOT/target}"

readelf="$(echo "$ANDROID_NDK_HOME"/toolchains/llvm/prebuilt/*/bin/llvm-readelf)"

# Bytes of the named sections of a library, summed.
section_bytes() {
	local file="$1" target="$2"
	case "$target" in
	*apple*)
		otool -l "$file" | python3 -c '
import re, sys
name, total = None, 0
for line in sys.stdin:
    m = re.match(r"\s+sectname (\S+)", line)
    if m:
        name = m.group(1)
    m = re.match(r"\s+size 0x([0-9a-f]+)", line)
    if m and name in ("__unwind_info", "__eh_frame", "__gcc_except_tab"):
        total += int(m.group(1), 16)
print(total)'
		;;
	*)
		"$readelf" -S -W "$file" | python3 -c '
import re, sys
total = 0
for line in sys.stdin:
    m = re.search(r"\]\s+(\.eh_frame|\.eh_frame_hdr|\.gcc_except_table)\s+\S+\s+\S+\s+\S+\s+([0-9a-f]+)", line)
    if m:
        total += int(m.group(2), 16)
print(total)'
		;;
	esac
}

echo "| Target | Panic | Raw | gzip -9 | zip -9 | Unwind sections |"
echo "|---|---|---:|---:|---:|---:|"
for target in "${targets[@]}"; do
	case "$target" in
	*apple*) lib=libkanso_mobile_bridge.dylib ;;
	*) lib=libkanso_mobile_bridge.so ;;
	esac
	for panic in unwind abort; do
		(
			export CARGO_PROFILE_MOBILE_PANIC="$panic"
			# Only the dynamic library: the crate also builds a static one.
			cargo rustc --locked --profile mobile --target "$target" -p kanso-mobile-bridge --lib --crate-type cdylib >&2
		)
		built="$target_dir/$target/mobile/$lib"
		copy="$out/$target-$panic-$lib"
		cp "$built" "$copy"
		raw="$(stat -f %z "$copy" 2>/dev/null || stat -c %s "$copy")"
		gz="$(gzip -9 -c "$copy" | wc -c | tr -d ' ')"
		rm -f "$copy.zip"
		(cd "$out" && zip -9 -q "$(basename "$copy").zip" "$(basename "$copy")")
		zp="$(stat -f %z "$copy.zip" 2>/dev/null || stat -c %s "$copy.zip")"
		unwind="$(section_bytes "$copy" "$target")"
		echo "| $target | $panic | $raw | $gz | $zp | $unwind |"
	done
done
