#!/usr/bin/env bash
# Makes the apt repository's signed index for one release (docs/APT.md):
# Packages for each architecture, Release, InRelease and Release.gpg, under
# the names the release assets go by (apt-stable-InRelease,
# apt-stable-main-binary-amd64-Packages.gz, …). The site redirects
# kansoapp.app/apt/dists/… to them, and pool/<tag>/<file>.deb to the .debs.
#
# Usage: scripts/apt-index.sh <dist-dir> <tag> <out-dir>
#
#   <dist-dir>  where the release's kanso_<version>_<arch>.deb files are
#   <tag>       the release's tag, vX.Y.Z: the .debs' folder in the pool
#   <out-dir>   where the index files go
#
# The key signs from GnuPG's home (GNUPGHOME), with:
#   APT_SIGNING_FINGERPRINT       the key to sign with
#   APT_SIGNING_PASSPHRASE_FILE   a file holding its passphrase (optional)
#   APT_SUITE                     the suite (default: stable)
#
# The index is made first, then Release over it, then the signatures, and
# nothing is touched after: Release's hashes are of the files as uploaded.
set -euo pipefail

if [ $# -ne 3 ]; then
	echo "Usage: $0 <dist-dir> <tag> <out-dir>" >&2
	exit 2
fi
dist=$(cd "$1" && pwd)
tag=$2
mkdir -p "$3"
out=$(cd "$3" && pwd)
suite=${APT_SUITE:-stable}
fingerprint=${APT_SIGNING_FINGERPRINT:?set APT_SIGNING_FINGERPRINT to the signing key}
architectures=(amd64 arm64)

if ! [[ $tag =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
	echo "$0: '$tag' isn't a release tag (vX.Y.Z)" >&2
	exit 2
fi
case $suite in
stable) ;;
*)
	echo "$0: unknown suite '$suite'; only stable is published" >&2
	exit 2
	;;
esac
for tool in apt-ftparchive gpg gzip; do
	command -v "$tool" >/dev/null || {
		echo "$0: $tool is missing (apt-ftparchive is in apt-utils)" >&2
		exit 1
	}
done

shopt -s nullglob
debs=("$dist"/kanso_*_amd64.deb "$dist"/kanso_*_arm64.deb)
shopt -u nullglob
if [ ${#debs[@]} -eq 0 ]; then
	echo "$0: no kanso_*_{amd64,arm64}.deb in $dist" >&2
	exit 1
fi

# The repository as apt sees it, in a scratch folder: pool/<tag>/*.deb, so
# each Packages entry's Filename is pool/<tag>/<file>.deb.
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
mkdir -p "$work/pool/$tag"
cp -- "${debs[@]}" "$work/pool/$tag/"

cd "$work"
for arch in "${architectures[@]}"; do
	dir="dists/$suite/main/binary-$arch"
	mkdir -p "$dir"
	apt-ftparchive --arch "$arch" packages "pool/$tag" >"$dir/Packages"
	gzip -9 -n -k "$dir/Packages"
done

# Over dists/<suite> only; written beside it, then moved in, so Release
# never lists itself.
apt-ftparchive \
	-o APT::FTPArchive::Release::Origin=Kanso \
	-o APT::FTPArchive::Release::Label=Kanso \
	-o APT::FTPArchive::Release::Suite="$suite" \
	-o APT::FTPArchive::Release::Codename="$suite" \
	-o APT::FTPArchive::Release::Components=main \
	-o APT::FTPArchive::Release::Architectures="${architectures[*]}" \
	-o APT::FTPArchive::Release::Description="Kanso: Markdown notes with sync and end-to-end encryption" \
	release "dists/$suite" >Release
mv Release "dists/$suite/Release"

sign=(gpg --batch --yes --local-user "$fingerprint" --digest-algo SHA512)
if [ -n "${APT_SIGNING_PASSPHRASE_FILE:-}" ]; then
	sign+=(--pinentry-mode loopback --passphrase-file "$APT_SIGNING_PASSPHRASE_FILE")
fi
"${sign[@]}" --clearsign --output "dists/$suite/InRelease" "dists/$suite/Release"
"${sign[@]}" --armor --detach-sign --output "dists/$suite/Release.gpg" "dists/$suite/Release"
gpg --batch --verify "dists/$suite/InRelease" 2>/dev/null
gpg --batch --verify "dists/$suite/Release.gpg" "dists/$suite/Release" 2>/dev/null

# Flattened: dists/stable/main/binary-amd64/Packages → apt-stable-main-binary-amd64-Packages.
cd "dists"
find "$suite" -type f | sort | while read -r file; do
	cp -- "$file" "$out/apt-${file//\//-}"
done
echo "apt index for $tag ($suite):"
ls -1 "$out"/apt-"$suite"-*
