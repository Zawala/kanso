#!/usr/bin/env bash
# Makes sure a Linux runner has the packages Kanso builds against.
# Usage: scripts/linux-deps.sh
#
# The self-hosted runners are containers where sudo may be blocked ("no new
# privileges"), so: nothing to do when every package is already there,
# apt-get directly when running as root, sudo otherwise, and a clear error
# naming the missing packages when none of that works. To skip the install
# on every run, bake these packages into the runner image.
set -euo pipefail

packages=(
	pkg-config clang cmake
	libxkbcommon-dev libxkbcommon-x11-dev
	libwayland-dev libvulkan-dev libegl-dev
	libx11-dev libx11-xcb-dev libxcb1-dev
	libxcb-randr0-dev libxcb-shape0-dev libxcb-shm0-dev libxcb-xfixes0-dev
	libfontconfig-dev libfreetype-dev
	libssl-dev libasound2-dev libdbus-1-dev libzstd-dev
)

missing=()
for package in "${packages[@]}"; do
	dpkg-query -W -f='${Status}' "$package" 2>/dev/null | grep -q 'install ok installed' ||
		missing+=("$package")
done

if [ ${#missing[@]} -eq 0 ]; then
	echo "Every build package is already installed."
	exit 0
fi
echo "Installing: ${missing[*]}"

if [ "$(id -u)" -eq 0 ]; then
	as_root=()
elif sudo -n true 2>/dev/null; then
	as_root=(sudo -n)
else
	echo "::error::This runner lacks ${missing[*]} and can't install them (not root, and sudo is blocked). Add them to the runner image."
	exit 1
fi

export DEBIAN_FRONTEND=noninteractive
"${as_root[@]}" apt-get update
"${as_root[@]}" apt-get install -y --no-install-recommends "${missing[@]}"
