// Kanso — crash_capture.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/foundation.dart';

import '../models/repos/abs/crash_report_repository.dart';

/// Records Dart errors as crash reports, without changing what happens to
/// them: in debug mode they are still printed, and an uncaught asynchronous
/// error is still left to the engine's own handling.
///
/// Only the error's runtime type and its stack trace go to [reports]; the
/// error and its message are never read. Recording is fire-and-forget and
/// never throws.
void installCrashCapture(CrashReportRepository reports) {
  final previous = FlutterError.onError;
  FlutterError.onError = (details) {
    unawaited(reports.recordDartError(details.exception, _stack(details)));
    if (kDebugMode) {
      FlutterError.presentError(details);
    } else {
      previous?.call(details);
    }
  };
  final previousDispatcher = PlatformDispatcher.instance.onError;
  PlatformDispatcher.instance.onError = (error, stack) {
    unawaited(reports.recordDartError(error, stack));
    return previousDispatcher?.call(error, stack) ?? false;
  };
}

StackTrace _stack(FlutterErrorDetails details) =>
    details.stack ?? StackTrace.empty;
