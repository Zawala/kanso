// Kanso — app_version.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'app_version.freezed.dart';

/// The version of the app that is running.
@freezed
abstract class AppVersion with _$AppVersion {
  const factory AppVersion({
    /// The version people know it by: `0.1.0`.
    required String version,

    /// The number of the build: `1`.
    required String build,

    /// The Shorebird patch the app is running; null without Shorebird or
    /// before the first patch.
    int? patch,
  }) = _AppVersion;
}
