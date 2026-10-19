#!/usr/bin/env bash
# Gets the toolchain pinned in rust-toolchain.toml onto a CI runner.
# Usage: scripts/ci-rust.sh
#
# The self-hosted runners don't load a login shell, so ~/.cargo/bin is often
# off PATH even where Rust is installed, and fresh runners have no rustup at
# all. This puts ~/.cargo/bin on PATH for later steps, installs rustup for
# this user (no sudo) when it's missing, then installs the pinned toolchain.
set -euo pipefail

cargo_bin="${CARGO_HOME:-$HOME/.cargo}/bin"
export PATH="$cargo_bin:$PATH"
echo "$cargo_bin" >>"$GITHUB_PATH"

if ! command -v rustup >/dev/null 2>&1; then
	echo "No rustup on this runner; installing it for $(id -un)."
	case "$(uname -s)" in
	MINGW* | MSYS* | CYGWIN*)
		curl --proto '=https' --tlsv1.2 -sSfL -o "$RUNNER_TEMP/rustup-init.exe" \
			https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe
		"$RUNNER_TEMP/rustup-init.exe" -y --no-modify-path --default-toolchain none --profile minimal
		;;
	*)
		curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs |
			sh -s -- -y --no-modify-path --default-toolchain none --profile minimal
		;;
	esac
fi

rustup show active-toolchain || rustup toolchain install
