#!/bin/bash
# Kanso — ci_post_clone.sh
# Created 2026-10-29.
# Licensed under AGPL-3.0-or-later (see LICENSE).
#
# Xcode Cloud runs this after cloning the repository, before it builds the
# iPhone and iPad app (docs/RELEASING-MOBILE.md, "iOS builds in Xcode Cloud").
# An Xcode Cloud runner has Xcode but neither Flutter nor Rust, and the app's
# Rust library is built by the Flutter native-assets hook (mobile/hook/build.dart)
# inside Xcode's own build, so this installs both and leaves them where that
# build finds them. Signing is Xcode Cloud's (cloud-managed), not this script's.
#
# The version is the root Cargo.toml's, as in every release build
# (scripts/mobile-version.sh); the build number is Xcode Cloud's, which only
# grows for the workflow.
set -euo pipefail

root="${CI_PRIMARY_REPOSITORY_PATH:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)}"
cd "$root"

# The Flutter release the app is pinned to (mobile/pubspec.yaml).
flutter_version="$(sed -n 's/^  flutter: \([0-9][0-9.]*\)$/\1/p' mobile/pubspec.yaml | head -n 1)"
[ -n "$flutter_version" ] || {
	echo "ci_post_clone: no pinned Flutter version in mobile/pubspec.yaml" >&2
	exit 1
}

echo "ci_post_clone: submodules"
git submodule update --init --recursive

echo "ci_post_clone: Rust"
if ! command -v rustup >/dev/null 2>&1; then
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs |
		sh -s -- -y --profile minimal --default-toolchain none
fi
# shellcheck disable=SC1091
. "$HOME/.cargo/env"
# The toolchain and the iOS targets mobile/rust/rust-toolchain.toml names.
(cd mobile/rust && rustup toolchain install)

# Xcode's build phases don't read this script's PATH, and the native-assets
# hook runs cargo from them, so put the tools where every process looks.
for tool in cargo rustc rustup; do
	ln -sf "$HOME/.cargo/bin/$tool" "/usr/local/bin/$tool" 2>/dev/null ||
		sudo -n ln -sf "$HOME/.cargo/bin/$tool" "/usr/local/bin/$tool" 2>/dev/null ||
		echo "ci_post_clone: couldn't link $tool into /usr/local/bin"
done

echo "ci_post_clone: Flutter $flutter_version"
if [ ! -d "$HOME/flutter" ]; then
	git clone --depth 1 --branch "$flutter_version" https://github.com/flutter/flutter.git "$HOME/flutter"
fi
export PATH="$HOME/flutter/bin:$PATH"
flutter --version
flutter precache --ios

echo "ci_post_clone: dependencies and the generated Xcode configuration"
cd mobile
flutter pub get

version="$(../scripts/mobile-version.sh --build-number "${CI_BUILD_NUMBER:-1}")"
name="$(printf '%s\n' "$version" | sed -n 's/^name=//p')"
number="$(printf '%s\n' "$version" | sed -n 's/^number=//p')"
echo "ci_post_clone: version $name ($number)"

# Writes ios/Flutter/Generated.xcconfig (FLUTTER_ROOT, the version, the build
# number) without building, so Xcode Cloud's own archive step uses them.
flutter build ios --config-only --release --no-codesign \
	--build-name="$name" --build-number="$number"

# Builds the Rust library once here, where cargo is on PATH, so the archive
# step finds the hook's cached output if it can't run cargo itself.
flutter build ios --release --no-codesign \
	--build-name="$name" --build-number="$number"
