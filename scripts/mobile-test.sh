#!/usr/bin/env bash
# Runs the core's tests (#48) on the iOS Simulator or an Android device.
# Usage: scripts/mobile-test.sh ios-sim|android [cargo test arguments after --]
#
#   ios-sim  boots a simulator (KANSO_IOS_SIM_UDID, else the first iPhone),
#            builds for aarch64-apple-ios-sim and runs every test binary
#            in it with `xcrun simctl spawn`.
#   android  runs on the one device or emulator adb sees (ANDROID_SERIAL
#            picks one); its CPU decides the target: arm64-v8a is
#            aarch64-linux-android, x86_64 is x86_64-linux-android. The test
#            binaries and the fixtures they read are copied to the device.
#
# Each crate's library and integration tests run; doctests don't (cross
# doctests would go through the runner, and the desktop jobs run them). Cargo
# runs each test binary through scripts/mobile/*-run.sh, which pass on the
# environment the tests read (the property-test seed, the crate's folder).
# KANSO_TEST_BUILD_ONLY=1 (Android) builds the test binaries for
# KANSO_ANDROID_TARGET (default x86_64-linux-android) without a device, so CI
# can compile before it starts the emulator.
# Needs the Rust target and, for Android, the NDK (scripts/mobile/env.sh).
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=mobile/env.sh
. "$here/mobile/env.sh"
cd "$KANSO_ROOT"

platform="${1:-}"
[ "$#" -gt 0 ] && shift
[ "${1:-}" = "--" ] && shift

# Waits for the device, copies the fixtures to it and prints the Rust target
# its CPU runs. The test binaries and fixtures live in $KANSO_ANDROID_ROOT; the
# fixtures keep their places in the repository, so the tests' relative paths
# (../../cloud/test/fixtures) hold.
prepare_android_device() {
	command -v adb >/dev/null || {
		echo "adb isn't on PATH." >&2
		exit 1
	}
	adb wait-for-device
	local waited=0
	until [ "$(adb shell getprop sys.boot_completed | tr -d '\r')" = 1 ]; do
		sleep 2
		waited=$((waited + 2))
		[ "$waited" -lt 300 ] || {
			echo "The device didn't finish booting in five minutes." >&2
			exit 1
		}
	done
	local abi triple
	abi="$(adb shell getprop ro.product.cpu.abi | tr -d '\r')"
	case "$abi" in
	arm64-v8a) triple=aarch64-linux-android ;;
	x86_64) triple=x86_64-linux-android ;;
	armeabi-v7a) triple=armv7-linux-androideabi ;;
	*)
		echo "Unsupported device CPU: $abi" >&2
		exit 1
		;;
	esac
	local root="$KANSO_ANDROID_ROOT"
	adb shell "rm -rf $root/repo $root/bin && mkdir -p $root/repo $root/bin $root/tmp" >&2
	local dir
	for dir in cloud/test/fixtures themes crates/*/tests crates/*/proptest-regressions crates/kanso-render/assets; do
		[ -d "$KANSO_ROOT/$dir" ] || continue
		adb shell "mkdir -p $root/repo/$dir" >&2
		adb push "$KANSO_ROOT/$dir/." "$root/repo/$dir/" >&2
	done
	echo "$triple"
}

export KANSO_ANDROID_ROOT=/data/local/tmp/kanso

case "$platform" in
ios-sim)
	target=aarch64-apple-ios-sim
	udid="${KANSO_IOS_SIM_UDID:-}"
	if [ -z "$udid" ]; then
		udid="$(xcrun simctl list devices available -j | python3 -c '
import json, sys
runtimes = json.load(sys.stdin)["devices"]
for runtime in sorted(runtimes, reverse=True):
    for device in runtimes[runtime]:
        if device["name"].startswith("iPhone"):
            print(device["udid"])
            sys.exit(0)
sys.exit("no iPhone simulator is installed")
')"
	fi
	export KANSO_IOS_SIM_UDID="$udid"
	# `bootstatus -b` boots a shut-down simulator and waits for it.
	xcrun simctl bootstatus "$udid" -b >/dev/null
	runner="$here/mobile/ios-sim-run.sh"
	;;
android)
	if [ -n "${KANSO_TEST_BUILD_ONLY:-}" ]; then
		target="${KANSO_ANDROID_TARGET:-x86_64-linux-android}"
		runner=/bin/true
	else
		runner="$here/mobile/android-run.sh"
		target="$(prepare_android_device)"
	fi
	;;
*)
	echo "Usage: scripts/mobile-test.sh ios-sim|android" >&2
	exit 2
	;;
esac

upper="$(echo "$target" | tr '[:lower:]-' '[:upper:]_')"
export "CARGO_TARGET_${upper}_RUNNER=$runner"

# Library and integration tests of each crate the facade is built on, the
# facade (kanso-mobile) and the library the app loads (kanso-mobile-bridge),
# one crate at a time so each one's counts are its own. kanso-sync and
# kanso-ai are tested without their desktop features, as they are built.
log="${KANSO_TEST_LOG:-$(mktemp -t kanso-mobile-test)}"
: >"$log"
failed=0
run() {
	echo "== $*" | tee -a "$log"
	if ! cargo test --locked --target "$target" --lib --tests ${KANSO_TEST_BUILD_ONLY:+--no-run} "$@" 2>&1 | tee -a "$log"; then
		failed=1
	fi
}
# KANSO_TEST_CRATES narrows the run to some of them while working on one.
wanted=" ${KANSO_TEST_CRATES:-kanso-core kanso-store kanso-engine kanso-render kanso-collab kanso-import kanso-tls kanso-mobile kanso-mobile-bridge kanso-sync kanso-ai} "
for crate in kanso-core kanso-store kanso-engine kanso-render kanso-collab kanso-import kanso-tls kanso-mobile kanso-mobile-bridge; do
	case "$wanted" in *" $crate "*) run -p "$crate" ;; esac
done
for crate in kanso-sync kanso-ai; do
	case "$wanted" in *" $crate "*) run -p "$crate" --no-default-features ;; esac
done

echo
echo "== Results on $platform ($target)"
awk '
	/^== -p / { crate = $3 }
	/^test result:/ { passed[crate] += $4; failed[crate] += $6; ignored[crate] += $8 }
	END {
		for (c in passed) printf "%-14s %5d passed %3d failed %3d ignored\n", c, passed[c], failed[c], ignored[c]
	}
' "$log" | sort
echo "Full log: $log"
exit "$failed"
