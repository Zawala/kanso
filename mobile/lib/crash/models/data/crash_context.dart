// Kanso — crash_context.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

import 'crash_app.dart';
import 'crash_os.dart';

part 'crash_context.freezed.dart';

/// What a report says about the build and the phone it ran on.
@freezed
abstract class CrashContext with _$CrashContext {
  const factory CrashContext({
    required CrashApp app,
    required CrashOs os,
    required String device,
  }) = _CrashContext;
}
