#!/usr/bin/env bash
# Looks inside the signed Android App Bundle that is about to go to Play
# (#48 step 13).
# Usage: scripts/mobile-release/aab-check.sh <app-release.aab> <mapping.txt> [--debug-ok]
#
# The bundle holds the library for each ABI (Play serves a phone only the one
# it runs), R8 has shrunk it (the mapping file exists and is not empty), and
# it is signed with the upload key, not the debug key the project falls back
# to when key.properties is missing. --debug-ok skips the signature check, for
# a build that has no upload key.
set -euo pipefail

[ "$#" -ge 2 ] || {
	echo "Usage: $0 <app-release.aab> <mapping.txt> [--debug-ok]" >&2
	exit 2
}
aab="$1" mapping="$2" debug_ok="${3:-}"
failed=0
fail() {
	echo "::error::$*"
	failed=1
}

[ -f "$aab" ] || {
	echo "::error::$aab doesn't exist."
	exit 1
}

entries="$(unzip -Z1 "$aab")"
for abi in arm64-v8a armeabi-v7a x86_64; do
	grep -Fqx "base/lib/$abi/libkanso_mobile_bridge.so" <<<"$entries" ||
		fail "$aab has no base/lib/$abi/libkanso_mobile_bridge.so."
done
grep -Fqx "base/manifest/AndroidManifest.xml" <<<"$entries" || fail "$aab has no base manifest."
[ -s "$mapping" ] || fail "R8's mapping file $mapping is missing or empty: the release build wasn't shrunk."

if [ "$debug_ok" != "--debug-ok" ]; then
	certificate="$(keytool -printcert -jarfile "$aab" 2>&1 || true)"
	grep -q '^Owner:' <<<"$certificate" || fail "$aab is not signed."
	if grep -q 'CN=Android Debug' <<<"$certificate"; then
		fail "$aab is signed with the debug key; the upload key wasn't used."
	fi
fi

signed="and a signature that isn't the debug key's"
[ "$debug_ok" = "--debug-ok" ] && signed="(signature not checked)"
[ "$failed" = 0 ] && echo "aab: $(basename "$aab") has the library for arm64-v8a, armeabi-v7a and x86_64, a mapping file $signed."
exit "$failed"
