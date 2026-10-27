// Kanso — pending_crash_report.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

import 'crash_report.dart';

part 'pending_crash_report.freezed.dart';

/// A report kept on the device until the person decides.
@freezed
abstract class PendingCrashReport with _$PendingCrashReport {
  const factory PendingCrashReport({
    /// The file's name without its extension.
    required String id,
    required CrashReport report,

    /// The exact text that is sent, and shown to the person.
    required String body,
  }) = _PendingCrashReport;
}
