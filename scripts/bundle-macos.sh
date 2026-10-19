#!/bin/sh
# Builds Kanso.app, or Kanso Dev.app with --dev (docs/DEVELOPING.md).
#
#   scripts/bundle-macos.sh                 target/release/Kanso.app
#   scripts/bundle-macos.sh --dev           target/kanso-dev/release/Kanso Dev.app
#   scripts/bundle-macos.sh --dev --install ...and copies it to ~/Applications
#
# Kanso Dev is its own app: bundle ID app.kanso.Kanso.dev, an icon with an
# amber DEV band, its own data folder and Keychain service, and no updates.
# It is never installed over /Applications/Kanso.app, which only ever comes
# from a release and updates itself.
#
# KANSO_SIGN_IDENTITY signs with that identity, the hardened runtime and a
# secure timestamp (the release workflow's Developer ID); without it the app
# is signed ad hoc, enough to run on this Mac.
set -eu

cd "$(dirname "$0")/.."

dev=0
install=0
for arg in "$@"; do
    case "$arg" in
    --dev) dev=1 ;;
    --install) install=1 ;;
    -h | --help)
        sed -n '2,16p' "$0" | sed 's/^# \{0,1\}//'
        exit 0
        ;;
    *)
        echo "bundle-macos: unknown option $arg" >&2
        exit 1
        ;;
    esac
done

if [ "$dev" = 1 ]; then
    # Its own target folder, so switching between the two never rebuilds
    # everything and a release build never picks up the dev flavour.
    target_dir=target/kanso-dev
    cargo build --release -p kanso-app --features dev-flavour --target-dir "$target_dir"
    name="Kanso Dev"
    bundle_id=app.kanso.Kanso.dev
else
    target_dir=target
    if [ "$install" = 1 ]; then
        echo "bundle-macos: Kanso itself installs from a release and updates itself; --install is for --dev builds." >&2
        exit 1
    fi
    cargo build --release -p kanso-app
    name=Kanso
    bundle_id=app.kanso.Kanso
fi

version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
app="$target_dir/release/$name.app"
rm -rf "$app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
cp "$target_dir/release/kanso" "$app/Contents/MacOS/kanso"

# The icon; regenerate Kanso's with assets/icon/make-icon.swift and iconutil.
if [ "$dev" = 1 ]; then
    iconset=$(mktemp -d)/Kanso.iconset
    mkdir -p "$iconset"
    KANSO_ICON_DEV=1 swift assets/icon/make-icon.swift "$iconset/icon_512x512@2x.png" 1024
    for size in 16 32 128 256 512; do
        sips -z "$size" "$size" "$iconset/icon_512x512@2x.png" --out "$iconset/icon_${size}x${size}.png" >/dev/null
        double=$((size * 2))
        sips -z "$double" "$double" "$iconset/icon_512x512@2x.png" --out "$iconset/icon_${size}x${size}@2x.png" >/dev/null
    done
    iconutil -c icns "$iconset" -o "$app/Contents/Resources/Kanso.icns"
    rm -rf "$(dirname "$iconset")"
else
    cp assets/icon/Kanso.icns "$app/Contents/Resources/Kanso.icns"
fi

cat > "$app/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleIdentifier</key>
    <string>$bundle_id</string>
    <key>CFBundleName</key>
    <string>$name</string>
    <key>CFBundleDisplayName</key>
    <string>$name</string>
    <key>CFBundleExecutable</key>
    <string>kanso</string>
    <key>CFBundleIconFile</key>
    <string>Kanso</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>$version</string>
    <key>CFBundleVersion</key>
    <string>$version</string>
    <key>LSMinimumSystemVersion</key>
    <string>13.0</string>
    <key>LSApplicationCategoryType</key>
    <string>public.app-category.productivity</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSSupportsAutomaticGraphicsSwitching</key>
    <true/>
</dict>
</plist>
PLIST

plutil -lint "$app/Contents/Info.plist" >/dev/null
if [ -n "${KANSO_SIGN_IDENTITY:-}" ]; then
    codesign --force --options runtime --timestamp --sign "$KANSO_SIGN_IDENTITY" "$app"
else
    # Ad hoc: macOS runs it on this Mac; not for distribution.
    codesign --force --sign - "$app"
fi
codesign --verify --deep --strict "$app"
echo "Built $app"

if [ "$install" = 1 ]; then
    dest="$HOME/Applications/$name.app"
    mkdir -p "$HOME/Applications"
    rm -rf "$dest"
    cp -R "$app" "$dest"
    echo "Installed $dest"
fi
