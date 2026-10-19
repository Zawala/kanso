#!/bin/sh
# Installs Kanso for the current user from an unpacked release archive:
# the binary in ~/.local/bin, the launcher in ~/.local/share/applications
# and the icon in ~/.local/share/icons. Run it again to update; run it with
# --uninstall to remove those three files (your notes stay where they are).
set -eu

here=$(cd "$(dirname "$0")" && pwd)
bin_dir="$HOME/.local/bin"
data_home="${XDG_DATA_HOME:-$HOME/.local/share}"
apps_dir="$data_home/applications"
icon_dir="$data_home/icons/hicolor/512x512/apps"

if [ "${1:-}" = "--uninstall" ]; then
    rm -f "$bin_dir/kanso" "$apps_dir/kanso.desktop" "$icon_dir/kanso.png"
    echo "Removed Kanso. Your notes are still in ${XDG_DATA_HOME:-~/.local/share}/kanso."
    exit 0
fi

mkdir -p "$bin_dir" "$apps_dir" "$icon_dir"
install -m 755 "$here/kanso" "$bin_dir/kanso"
install -m 644 "$here/kanso.png" "$icon_dir/kanso.png"
# The launcher runs the installed binary by its full path, since a desktop
# session's PATH often leaves out ~/.local/bin.
sed "s|^Exec=kanso|Exec=\"$bin_dir/kanso\"|" "$here/kanso.desktop" >"$apps_dir/kanso.desktop"
chmod 644 "$apps_dir/kanso.desktop"
command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$apps_dir" || true
command -v gtk-update-icon-cache >/dev/null 2>&1 && gtk-update-icon-cache -q "$data_home/icons/hicolor" || true

echo "Installed Kanso in $bin_dir/kanso; it's in your applications menu."
