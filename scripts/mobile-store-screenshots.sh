#!/usr/bin/env bash
# Takes the pictures for the store listings (#48 step 13,
# mobile/store/README.md) with the app running on a simulator or emulator.
#
#   scripts/mobile-store-screenshots.sh <set> <device> [--build-first]
#
# <set> names the listing and the folder under mobile/store/screenshots/:
#   ios-iphone-6.9   iPhone 17 Pro Max      1320 x 2868
#   ios-ipad-13      iPad Pro 13-inch (M5)  2064 x 2752
#   android-phone    an emulator at 1080 x 1920
#   android-tablet   an emulator at 1600 x 2560 (a 10-inch tablet)
# <device> is what `flutter devices` calls it (a simulator name or UDID, or
# emulator-5554). The device must already be booted at the size of the set;
# the script checks the pictures' size and fails if it is wrong.
#
# The pictures are written to mobile/store/screenshots/<set>/ (git-ignored).
# They are not made in CI: a simulator's picture of the app is checked by
# eye before it goes to a store.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
set_name="${1:-}"
device="${2:-}"
[ -n "$set_name" ] && [ -n "$device" ] || {
	sed -n '2,16p' "$0" | sed 's/^# \{0,1\}//' >&2
	exit 2
}

case "$set_name" in
ios-iphone-6.9) want="1320x2868" ;;
ios-ipad-13) want="2064x2752" ;;
android-phone) want="1080x1920" ;;
android-tablet) want="1600x2560" ;;
*)
	echo "Unknown set '$set_name'." >&2
	exit 2
	;;
esac

out="$root/mobile/store/screenshots/$set_name"
rm -rf "$out"
mkdir -p "$out"
cd "$root/mobile"

KANSO_SCREENSHOT_DIR="$out" flutter drive \
	--driver=test_driver/store_screenshot_driver.dart \
	--target=integration_test/store_screenshots_test.dart \
	-d "$device"

shots=("$out"/*.png)
[ -f "${shots[0]}" ] || {
	echo "No pictures were made." >&2
	exit 1
}
status=0
for shot in "${shots[@]}"; do
	size="$(sips -g pixelWidth -g pixelHeight "$shot" | awk '/pixelWidth/ {w=$2} /pixelHeight/ {h=$2} END {print w "x" h}')"
	if [ -n "$want" ] && [ "$size" != "$want" ]; then
		echo "$shot is $size, the listing wants $want." >&2
		status=1
	else
		echo "$(basename "$shot") $size"
	fi
done
exit "$status"
