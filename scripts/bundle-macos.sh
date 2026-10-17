#!/bin/sh
# Builds Kanso.app in target/release/. Usage: scripts/bundle-macos.sh
set -eu

cd "$(dirname "$0")/.."
cargo build --release -p kanso-app

version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
app=target/release/Kanso.app
rm -rf "$app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
cp target/release/kanso "$app/Contents/MacOS/kanso"

cat > "$app/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleIdentifier</key>
    <string>app.kanso.Kanso</string>
    <key>CFBundleName</key>
    <string>Kanso</string>
    <key>CFBundleDisplayName</key>
    <string>Kanso</string>
    <key>CFBundleExecutable</key>
    <string>kanso</string>
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
# Ad-hoc signature so macOS runs it locally; not for distribution.
codesign --force --sign - "$app"
echo "Built $app"
echo "To install: cp -R $app /Applications/"
