// Kanso — crash_configs.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Where reports go, and the limits of the inbox.
abstract final class CrashConfigs {
  /// The endpoint of docs/CRASH-REPORTS.md.
  static final Uri endpoint = Uri.https(
    'sync.kansoapp.app',
    '/v1/crash-reports',
  );

  /// The channel the native code answers `deviceModel` and `osVersion` on.
  static const String channel = 'app.kanso/crash';

  /// The most reports kept on the device; a new one pushes out the oldest.
  static const int maxPending = 10;

  /// The largest raw file read (a tombstone can be a few megabytes).
  static const int maxRawBytes = 16 * 1024 * 1024;

  /// How long a send may take.
  static const Duration sendTimeout = Duration(seconds: 20);
}
