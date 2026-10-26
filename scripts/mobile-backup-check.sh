#!/usr/bin/env bash
# Checks that the Android app installed on a device or emulator opts out of
# backups (#48): its package flags must not list ALLOW_BACKUP.
# Usage: scripts/mobile-backup-check.sh [adb serial]
#
# `android:allowBackup="false"` in mobile/android/app/src/main/AndroidManifest.xml
# is what clears the flag. Device-to-device transfer on Android 12 and later
# is ruled by the data extraction rules, which mobile/test/platform/backup_exclusion_test.dart
# reads; this is the one check that asks the installed package itself.
#
# CI runs it in the Mobile workflow after the integration tests. `flutter test`
# uninstalls the app it ran, so CI installs the debug APK it built before this.
set -euo pipefail

package="app.kanso.kanso"
adb_args=()
if [ "$#" -ge 1 ]; then
  adb_args=(-s "$1")
fi

dump="$(adb "${adb_args[@]}" shell dumpsys package "$package")"
flags="$(printf '%s\n' "$dump" | grep -E '^[[:space:]]*(pkgFlags|flags)=\[' || true)"

if [ -z "$flags" ]; then
  echo "::error::$package is not installed on the device, or dumpsys lists no package flags." >&2
  exit 1
fi
printf '%s\n' "$flags"
if printf '%s\n' "$flags" | grep -q 'ALLOW_BACKUP'; then
  echo "::error::$package is installed with ALLOW_BACKUP: the library would be in device backups." >&2
  exit 1
fi
echo "$package does not allow backups."
