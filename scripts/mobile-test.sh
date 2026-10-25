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
# Needs the Rust target and, for Android, the NDK (scripts/mobile/env.sh).
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=mobile/env.sh
. "$here/mobile/env.sh"
cd "$KANSO_ROOT"

platform="${1:-}"
[ "$#" -gt 0 ] && shift
[ "${1:-}" = "--" ] && shift

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
	command -v adb >/dev/null || {
		echo "adb isn't on PATH." >&2
		exit 1
	}
	adb wait-for-device
	waited=0
	until [ "$(adb shell getprop sys.boot_completed | tr -d '\r')" = 1 ]; do
		sleep 2
		waited=$((waited + 2))
		[ "$waited" -lt 300 ] || {
			echo "The device didn't finish booting in five minutes." >&2
			exit 1
		}
	done
	case "$(adb shell getprop ro.product.cpu.abi | tr -d '\r')" in
	arm64-v8a) target=aarch64-linux-android ;;
	x86_64) target=x86_64-linux-android ;;
	armeabi-v7a) target=armv7-linux-androideabi ;;
	*)
		echo "Unsupported device CPU." >&2
		exit 1
		;;
	esac
	# Where the test binaries and their fixtures live on the device. The
	# fixtures keep their places in the repository, so the tests' relative
	# paths (../../cloud/test/fixtures) hold.
	device_root=/data/local/tmp/kanso
	export KANSO_ANDROID_ROOT="$device_root"
	adb shell "rm -rf $device_root/repo $device_root/bin && mkdir -p $device_root/repo $device_root/bin $device_root/tmp"
	push() {
		adb shell "mkdir -p $device_root/repo/$1"
		adb push "$KANSO_ROOT/$1/." "$device_root/repo/$1/" >/dev/null
	}
	push cloud/test/fixtures
	push themes
	for dir in crates/*/tests crates/*/proptest-regressions crates/kanso-render/assets; do
		[ -d "$dir" ] && push "$dir"
	done
	runner="$here/mobile/android-run.sh"
	;;
*)
	echo "Usage: scripts/mobile-test.sh ios-sim|android" >&2
	exit 2
	;;
esac

upper="$(echo "$target" | tr '[:lower:]-' '[:upper:]_')"
export "CARGO_TARGET_${upper}_RUNNER=$runner"

# Library and integration tests of each crate the facade is built on, one
# crate at a time so each one's counts are its own. kanso-sync and kanso-ai
# are tested without their desktop features, as they are built.
log="${KANSO_TEST_LOG:-$(mktemp -t kanso-mobile-test)}"
: >"$log"
failed=0
run() {
	echo "== $*" | tee -a "$log"
	if ! cargo test --locked --target "$target" --lib --tests "$@" 2>&1 | tee -a "$log"; then
		failed=1
	fi
}
# KANSO_TEST_CRATES narrows the run to some of them while working on one.
wanted=" ${KANSO_TEST_CRATES:-kanso-core kanso-store kanso-engine kanso-render kanso-collab kanso-import kanso-tls kanso-sync kanso-ai} "
for crate in kanso-core kanso-store kanso-engine kanso-render kanso-collab kanso-import kanso-tls; do
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
