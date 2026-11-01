#!/usr/bin/env bash
# Makes sure a Linux runner has what scripts/apt-index.sh needs:
# apt-ftparchive (apt-utils) and gpg (gnupg).
# Usage: scripts/apt-tools.sh
#
# As scripts/linux-deps.sh does: nothing when both are there, apt-get as
# root, sudo otherwise, and a clear error when none of that works (the
# self-hosted runners are containers where sudo may be blocked).
set -euo pipefail

missing=()
command -v apt-ftparchive >/dev/null || missing+=(apt-utils)
command -v gpg >/dev/null || missing+=(gnupg)

if [ ${#missing[@]} -eq 0 ]; then
	echo "apt-ftparchive and gpg are installed."
	exit 0
fi
echo "Installing: ${missing[*]}"

if [ "$(id -u)" -eq 0 ]; then
	export DEBIAN_FRONTEND=noninteractive
	apt-get update
	apt-get install -y --no-install-recommends "${missing[@]}"
elif sudo -n true 2>/dev/null; then
	export DEBIAN_FRONTEND=noninteractive
	sudo -n apt-get update
	sudo -n apt-get install -y --no-install-recommends "${missing[@]}"
else
	echo "::error::This runner lacks ${missing[*]} and can't install them (not root, and sudo is blocked). Add them to the runner image."
	exit 1
fi
