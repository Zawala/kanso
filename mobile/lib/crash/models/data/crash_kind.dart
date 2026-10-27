// Kanso — crash_kind.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:json_annotation/json_annotation.dart';

/// Where a crash report came from (`kind` in docs/CRASH-REPORTS.md).
enum CrashKind {
  /// `FlutterError.onError` or `PlatformDispatcher.onError`.
  @JsonValue('dart')
  dart('dart'),

  /// MetricKit's `MXCrashDiagnostic`, delivered at the next launch.
  @JsonValue('ios-native')
  iosNative('ios-native'),

  /// An `ApplicationExitInfo` with `REASON_CRASH_NATIVE`: a tombstone.
  @JsonValue('android-native')
  androidNative('android-native'),

  /// An `ApplicationExitInfo` with `REASON_ANR`: a text trace.
  @JsonValue('android-anr')
  androidAnr('android-anr'),

  /// An uncaught exception in Kotlin or Java.
  @JsonValue('android-jvm')
  androidJvm('android-jvm');

  const CrashKind(this.wire);

  /// The value written in the report.
  final String wire;
}
