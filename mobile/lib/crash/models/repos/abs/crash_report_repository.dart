// Kanso — crash_report_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../../core/models/data/application_error.dart';
import '../../data/crash_send_outcome.dart';
import '../../data/pending_crash_report.dart';

/// The reports kept on the device until the person decides what to do with
/// them.
abstract interface class CrashReportRepository {
  /// Keeps a report of a Dart error, made from the error's runtime type and
  /// its stack trace only: the error itself is never turned into text. Never
  /// throws, and the caller need not wait.
  Future<void> recordDartError(Object error, StackTrace stack);

  /// The reports waiting, oldest first. Raw crashes the native code left are
  /// turned into reports first, and the raw files deleted.
  Future<Either<ApplicationError, List<PendingCrashReport>>> pending();

  /// Deletes report [id] without sending it.
  Future<Either<ApplicationError, Unit>> discard(String id);

  /// Sends report [id], exactly as it is shown, and deletes it when the
  /// server stores it.
  Future<CrashSendOutcome> send(String id);
}
