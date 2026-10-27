// Kanso — crash_report.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

import 'crash_app.dart';
import 'crash_kind.dart';
import 'crash_os.dart';

part 'crash_report.freezed.dart';
part 'crash_report.g.dart';

/// A crash report, version 1: exactly the JSON of docs/CRASH-REPORTS.md and
/// no other field. It is only ever made by the scrubber (`CrashScrubber`),
/// which lets through what the page whitelists.
@freezed
abstract class CrashReport with _$CrashReport {
  @JsonSerializable(explicitToJson: true)
  const factory CrashReport({
    /// The version of the format; first, as on the page.
    required int v,
    required CrashKind kind,

    /// A type, signal or exception class name, or `unknown`.
    required String type,
    required CrashApp app,
    required CrashOs os,

    /// The device's model identifier, or `unknown`.
    required String device,

    /// The UTC day of the crash, `2026-10-26`.
    required String day,

    /// One to 128 frames, innermost first.
    required List<String> frames,
  }) = _CrashReport;

  factory CrashReport.fromJson(Map<String, dynamic> json) =>
      _$CrashReportFromJson(json);
}
