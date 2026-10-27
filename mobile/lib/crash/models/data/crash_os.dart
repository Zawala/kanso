// Kanso — crash_os.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

import 'crash_os_name.dart';

part 'crash_os.freezed.dart';
part 'crash_os.g.dart';

/// The `os` object of a report.
@freezed
abstract class CrashOs with _$CrashOs {
  const factory CrashOs({
    required CrashOsName name,

    /// `18.1`, or `unknown`.
    required String version,
  }) = _CrashOs;

  factory CrashOs.fromJson(Map<String, dynamic> json) =>
      _$CrashOsFromJson(json);
}
