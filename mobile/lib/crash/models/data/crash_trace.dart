// Kanso — crash_trace.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'crash_trace.freezed.dart';

/// What a parser makes of one raw crash: a type and the scrubbed frames,
/// plus whatever the raw crash says about itself that the report may keep.
///
/// Every string in it has already been through the whitelist.
@freezed
abstract class CrashTrace with _$CrashTrace {
  const factory CrashTrace({
    /// A type, signal or exception name, or `unknown`.
    required String type,

    /// Scrubbed frames, innermost first.
    required List<String> frames,

    /// The UTC day, `2026-10-26`, when the raw crash says it.
    String? day,

    /// The app version the raw crash says it ran, when it says.
    String? appVersion,

    /// The build the raw crash says it ran.
    String? appBuild,

    /// The OS version the raw crash says it ran on.
    String? osVersion,

    /// The device identifier the raw crash says it ran on.
    String? device,
  }) = _CrashTrace;
}
