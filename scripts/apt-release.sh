#!/usr/bin/env bash
# The release workflow's apt step (docs/APT.md): signs the apt index for a
# tagged release with the key scripts/apt-setup.sh made, into <out-dir>.
# Usage: scripts/apt-release.sh <dist-dir> <tag> <out-dir>
#
# Reads the key from APT_SIGNING_KEY (armored secret key) and its passphrase
# from APT_SIGNING_PASSPHRASE, and the suites and the key's fingerprint from
# packaging/linux/apt.conf. Without the key it publishes nothing and says so,
# so releases go on as before until the key exists. Writes published=true
# or false to GITHUB_OUTPUT when there is one.
#
# The key lives in a GnuPG home of its own under RUNNER_TEMP (or a temporary
# folder), removed on the way out; the workflow removes it again in an
# always() step.
set -euo pipefail

if [ $# -ne 3 ]; then
	echo "Usage: $0 <dist-dir> <tag> <out-dir>" >&2
	exit 2
fi
dist=$1
tag=$2
out=$3
root=$(cd "$(dirname "$0")/.." && pwd)
conf="$root/packaging/linux/apt.conf"

published() {
	if [ -n "${GITHUB_OUTPUT:-}" ]; then
		echo "published=$1" >>"$GITHUB_OUTPUT"
	fi
}

if [ -z "${APT_SIGNING_KEY:-}" ]; then
	echo "::notice::apt signing key not configured; skipping apt metadata"
	published false
	exit 0
fi

if [ ! -f "$conf" ]; then
	echo "::error::APT_SIGNING_KEY is set but $conf is missing; commit the files scripts/apt-setup.sh wrote."
	exit 1
fi
setting() { sed -n "s/^$1=//p" "$conf" | tail -1; }
suites=$(setting suites)
fingerprint=$(setting fingerprint)
if [ -z "$suites" ] || ! [[ $fingerprint =~ ^[0-9A-F]{40}$ ]]; then
	echo "::error::$conf needs suites=… and fingerprint=<40 hex digits>."
	exit 1
fi
for suite in ${suites//,/ }; do
	case $suite in
	stable) ;;
	*)
		echo "::error::$conf names the suite '$suite'; only stable is implemented (docs/APT.md)."
		exit 1
		;;
	esac
done

shopt -s nullglob
debs=("$dist"/kanso_*_amd64.deb "$dist"/kanso_*_arm64.deb)
shopt -u nullglob
if [ ${#debs[@]} -eq 0 ]; then
	echo "::warning::No .deb in $dist (the Package (Debian) step failed?); skipping apt metadata."
	published false
	exit 0
fi

export GNUPGHOME="${RUNNER_TEMP:-${TMPDIR:-/tmp}}/kanso-apt-gnupg"
rm -rf "$GNUPGHOME"
mkdir -m 700 "$GNUPGHOME"
cleanup() {
	gpgconf --kill gpg-agent 2>/dev/null || true
	rm -rf "$GNUPGHOME"
}
trap cleanup EXIT

(
	umask 077
	printf '%s' "${APT_SIGNING_PASSPHRASE:-}" >"$GNUPGHOME/passphrase"
)
printf '%s\n' "$APT_SIGNING_KEY" | gpg --batch --quiet --import
if ! gpg --batch --with-colons --list-secret-keys "$fingerprint" >/dev/null 2>&1; then
	echo "::error::APT_SIGNING_KEY isn't the key $conf names ($fingerprint)."
	exit 1
fi

for suite in ${suites//,/ }; do
	APT_SUITE=$suite \
		APT_SIGNING_FINGERPRINT=$fingerprint \
		APT_SIGNING_PASSPHRASE_FILE="$GNUPGHOME/passphrase" \
		"$root/scripts/apt-index.sh" "$dist" "$tag" "$out"
done
published true
