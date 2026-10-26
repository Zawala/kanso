#!/usr/bin/env bash
# Measures what the Kanso app weighs on a phone (#48): the real release
# library, kanso_mobile_bridge, as the Flutter app embeds it, and the whole
# download. Prints Markdown tables (CI appends them to the job summary).
# Usage: scripts/mobile-size.sh ios|android [--no-build]
#
#   ios      `flutter build ios --release --no-codesign`: arm64 only, the
#            library is Runner.app/Frameworks/kanso_mobile_bridge.framework.
#            Download estimate: the .app zipped as an .ipa is. Needs a Mac
#            with Xcode.
#   android  `flutter build apk --release --split-per-abi` for the platforms
#            in KANSO_SIZE_ANDROID_PLATFORMS (default android-arm,
#            android-arm64,android-x64); one APK per ABI, as the Play Store
#            delivers one. Needs the NDK (scripts/mobile/env.sh).
#   --no-build  measure what a build in mobile/build left.
#
# For each library: Raw is the file in the app (the hook builds it with
# [profile.mobile], which strips symbols); Stripped is the file after `strip`
# of the platform (nothing left that the loader doesn't read); Zip is the
# stripped file alone under `zip -9`; In package is what the file takes inside
# the .ipa-equivalent or the APK (an APK stores a library uncompressed, as
# Android maps it from the APK). The download estimate is the size of that
# package, and again under `gzip -9` (an APK stores its libraries uncompressed,
# so a store that compresses the transfer sends far less than the file). App
# Store delivery encrypts binaries, so its download is somewhat larger than
# the iOS zip here.
#
# Fails when the library zipped is over KANSO_MAX_LIB_ZIP bytes (default
# 12000000): a size like that means the hook's profile overrides stopped
# applying (scripts/mobile-parity.sh checks they match [profile.mobile]).
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=mobile/env.sh
. "$here/mobile/env.sh"
cd "$KANSO_ROOT"

platform="${1:-}"
[ "$#" -gt 0 ] && shift
build=1
[ "${1:-}" = "--no-build" ] && build=0
case "$platform" in
ios | android) ;;
*)
	echo "Usage: $0 ios|android [--no-build]" >&2
	exit 2
	;;
esac

export KANSO_MAX_LIB_ZIP="${KANSO_MAX_LIB_ZIP:-12000000}"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
lib=kanso_mobile_bridge

# The Markdown for one package: python reads the zip itself.
# Args: title, package zip, member of the library in it, library file, stripped
# library file, [label].
report() {
	python3 -I - "$@" <<'PY'
import os, subprocess, sys, zipfile

title, package, member, library, stripped, label = sys.argv[1:7]
limit = int(os.environ["KANSO_MAX_LIB_ZIP"])


def mb(n):
    return f"{n / 1e6:.2f} MB"


def zip9(path):
    return len(subprocess.run(["zip", "-9", "-q", "-", path], capture_output=True, check=True).stdout)


raw = os.path.getsize(library)
strip = os.path.getsize(stripped)
zipped = zip9(stripped)
with zipfile.ZipFile(package) as z:
    infos = {i.filename: i for i in z.infolist()}
    inpkg = infos[member].compress_size
    groups = {}
    for i in z.infolist():
        if i.is_dir():
            continue
        name = i.filename
        parts = name.split("/")
        if name.startswith("Payload/"):
            # Payload/Runner.app/Frameworks/<x>.framework/... and the rest.
            key = "/".join(parts[2:4]) if len(parts) > 3 and parts[2] == "Frameworks" else "(the rest of the app)"
        elif name.startswith("lib/"):
            key = "/".join(parts[:3])
        elif name.endswith(".dex"):
            key = "classes*.dex"
        elif name.startswith("assets/"):
            key = "assets/"
        else:
            key = "(resources, manifest, signing)"
        raw_n, comp_n = groups.get(key, (0, 0))
        groups[key] = (raw_n + i.file_size, comp_n + i.compress_size)
package_size = os.path.getsize(package)
with open(package, "rb") as f:
    gzipped = len(subprocess.run(["gzip", "-9", "-c"], stdin=f, capture_output=True, check=True).stdout)

print(f"### {title}\n")
print("| Library | Raw | Stripped | Zip | In package |")
print("|---|---:|---:|---:|---:|")
print(f"| {label} | {mb(raw)} | {mb(strip)} | {mb(zipped)} | {mb(inpkg)} |\n")
print(f"Package: **{mb(package_size)}**; under `gzip -9` again (a store that compresses its transfers): **{mb(gzipped)}**\n")
print("| Part of the package | Uncompressed | In package |")
print("|---|---:|---:|")
for key, (r, c) in sorted(groups.items(), key=lambda kv: -kv[1][1]):
    print(f"| {key} | {mb(r)} | {mb(c)} |")
print()
if zipped > limit:
    print(f"::error::{label} zipped is {zipped} bytes, over the limit of {limit}: the mobile profile probably did not apply.", file=sys.stderr)
    sys.exit(3)
PY
}

case "$platform" in
ios)
	app="$KANSO_ROOT/mobile/build/ios/iphoneos/Runner.app"
	if [ "$build" = 1 ]; then
		(cd mobile && flutter build ios --release --no-codesign) >&2
	fi
	binary="$app/Frameworks/$lib.framework/$lib"
	[ -f "$binary" ] || {
		echo "$binary doesn't exist: build first." >&2
		exit 1
	}
	# An .ipa is a zip of Payload/Runner.app.
	mkdir -p "$work/ipa/Payload"
	ditto "$app" "$work/ipa/Payload/Runner.app"
	(cd "$work/ipa" && zip -qry9 "$work/Runner.ipa" Payload)
	cp "$binary" "$work/$lib"
	strip -x -S "$work/$lib" 2>/dev/null
	report "iOS, release, arm64 (Runner.app, ipa-equivalent)" "$work/Runner.ipa" \
		"Payload/Runner.app/Frameworks/$lib.framework/$lib" "$binary" "$work/$lib" \
		"$lib.framework (aarch64-apple-ios)"
	;;
android)
	platforms="${KANSO_SIZE_ANDROID_PLATFORMS:-android-arm,android-arm64,android-x64}"
	if [ "$build" = 1 ]; then
		(cd mobile && flutter build apk --release --split-per-abi --target-platform "$platforms") >&2
	fi
	strip_tool="$(echo "${ANDROID_NDK_HOME:?no Android NDK found}"/toolchains/llvm/prebuilt/*/bin/llvm-strip)"
	for abi in armeabi-v7a arm64-v8a x86_64; do
		apk="$KANSO_ROOT/mobile/build/app/outputs/flutter-apk/app-$abi-release.apk"
		[ -f "$apk" ] || continue
		unzip -oq "$apk" "lib/$abi/lib$lib.so" -d "$work/$abi"
		raw="$work/$abi/lib/$abi/lib$lib.so"
		cp "$raw" "$work/$abi.so"
		"$strip_tool" --strip-unneeded "$work/$abi.so"
		report "Android, release, $abi (APK)" "$apk" "lib/$abi/lib$lib.so" "$raw" "$work/$abi.so" \
			"lib$lib.so ($abi)"
	done
	;;
esac
