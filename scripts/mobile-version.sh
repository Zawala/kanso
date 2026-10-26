#!/usr/bin/env bash
# The version of the mobile app's release builds (#48 step 13,
# docs/RELEASING-MOBILE.md).
#
#   scripts/mobile-version.sh [--tag vX.Y.Z] [--build-number N]
#
# One source of truth: `[workspace.package] version` in the root Cargo.toml,
# which scripts/release.sh bumps for every Kanso release. The app's version
# name is that version (the iOS CFBundleShortVersionString and the Android
# versionName), so Kanso 0.9.0 is 0.9.0 on the desktop, the iPhone and
# Android. The build number is how the stores tell two builds of one version
# apart; it only ever grows. CI passes the workflow's run number. The
# version in mobile/pubspec.yaml is for local runs and is overridden by
# `--build-name` and `--build-number` in every release build.
#
# --tag checks the version against the tag that started the run, as the
# desktop release does. Prints `name=<x.y.z>` and `number=<n>` lines (the
# format of $GITHUB_OUTPUT).
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

die() {
	echo "mobile-version: $*" >&2
	exit 1
}

tag=""
number="${KANSO_BUILD_NUMBER:-}"
while [ "$#" -gt 0 ]; do
	case "$1" in
	--tag)
		tag="${2:-}"
		shift 2
		;;
	--build-number)
		number="${2:-}"
		shift 2
		;;
	*) die "unknown argument $1" ;;
	esac
done

# The first `version = "..."` of the file is [workspace.package]'s, as in
# scripts/release.sh and the release workflow.
name="$(sed -n 's/^version = "\(.*\)"/\1/p' "$root/Cargo.toml" | head -1)"
# Both stores take only dotted numbers: a pre-release suffix is refused.
[[ "$name" =~ ^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$ ]] ||
	die "the workspace version '$name' isn't x.y.z; the App Store and Play take no pre-release suffix"

if [ -n "$tag" ]; then
	[ "${tag#v}" = "$name" ] ||
		die "the tag says ${tag#v} but Cargo.toml says $name; bump [workspace.package] version first"
fi

[ -n "$number" ] || die "no build number: pass --build-number or set KANSO_BUILD_NUMBER"
[[ "$number" =~ ^[1-9][0-9]*$ ]] || die "the build number '$number' isn't a positive integer"
# Android's versionCode is a signed 32-bit integer and Play caps it at 2.1 billion.
[ "${#number}" -le 9 ] || die "the build number '$number' is too large for Android's versionCode"

echo "name=$name"
echo "number=$number"
