#!/usr/bin/env bash
# Renders the pictures for the store listings (#48 step 13,
# mobile/store/README.md) from the golden-test harness: no simulator, so it
# runs in CI and on Linux.
#
#   scripts/mobile-store-render.sh [out-dir]
#
# Writes <out-dir>/<set>/NN-name.png for every set, and
# <out-dir>/play/feature-graphic.png and icon-512.png. out-dir defaults to
# mobile/store/screenshots/ (git-ignored). Then checks every PNG's size and
# that it is 24-bit RGB with no alpha channel.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
out="${1:-$root/mobile/store/screenshots}"
mkdir -p "$out"
out="$(cd "$out" && pwd)"

# set -> width x height; the sets' folders are wiped first so no stale picture stays.
sets=(
	"ios-iphone-6.9 1320x2868"
	"ios-iphone-6.5 1284x2778"
	"ios-ipad-13 2064x2752"
	"android-phone 1080x2160"
	"android-tablet-7 1200x1920"
	"android-tablet-10 1600x2560"
)
for entry in "${sets[@]}"; do rm -rf "${out:?}/${entry%% *}"; done
rm -rf "${out:?}/play"

(cd "$root/mobile" && KANSO_STORE_SHOTS_DIR="$out" flutter test --tags store-screenshots test/store/store_screenshots_test.dart)

# Prints "<width>x<height> <colour type>" of a PNG, read from its header.
png_info() {
	od -An -tu1 -j16 -N10 "$1" | awk '{ w = $1 * 16777216 + $2 * 65536 + $3 * 256 + $4; h = $5 * 16777216 + $6 * 65536 + $7 * 256 + $8; printf "%dx%d %d %d\n", w, h, $9, $10 }'
}

status=0
check() { # file want-size
	local info size depth ctype
	read -r size depth ctype <<<"$(png_info "$1")"
	if [ "$size" != "$2" ]; then
		echo "$1 is $size, wanted $2." >&2
		status=1
	elif [ "$ctype" != 2 ] || [ "$depth" != 8 ]; then
		echo "$1 is not 24-bit RGB without alpha (depth $depth, colour type $ctype)." >&2
		status=1
	else
		echo "${1#"$out"/} $size"
	fi
}

for entry in "${sets[@]}"; do
	name="${entry%% *}"
	want="${entry##* }"
	shots=("$out/$name"/*.png)
	if [ ! -f "${shots[0]}" ]; then
		echo "No pictures were made for $name." >&2
		status=1
		continue
	fi
	if [ "${#shots[@]}" -lt 4 ] || [ "${#shots[@]}" -gt 8 ]; then
		echo "$name has ${#shots[@]} pictures; the stores take 2 to 8 and we make 4 to 8." >&2
		status=1
	fi
	for shot in "${shots[@]}"; do check "$shot" "$want"; done
done
for file in "feature-graphic.png 1024x500" "icon-512.png 512x512"; do
	if [ -f "$out/play/${file% *}" ]; then
		check "$out/play/${file% *}" "${file#* }"
	else
		echo "$out/play/${file% *} is missing." >&2
		status=1
	fi
done
exit "$status"
