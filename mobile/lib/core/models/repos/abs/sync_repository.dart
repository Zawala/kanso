// Kanso — sync_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../bridge/api/types.dart';
import '../../data/application_error.dart';

/// The account and sync. The session and the data key stay in the phone's
/// secret store, read by the Rust core.
abstract interface class SyncRepository {
  /// The server a new account is made on unless another is typed.
  Future<Either<ApplicationError, String>> defaultServer();

  Future<Either<ApplicationError, SyncStatus>> status();

  /// Asks the server at [url] whether it is a Kanso server.
  Future<Either<ApplicationError, ServerInfo>> checkServer(String url);

  Future<Either<ApplicationError, SyncStatus>> signInPassword({
    required String server,
    required String username,
    required String password,
    required String deviceName,
  });

  Future<Either<ApplicationError, Unit>> signOut();

  Future<Either<ApplicationError, SyncReportRow>> syncNow();

  /// The status now and each time it changes.
  Stream<SyncStatus> statusStream();

  /// The app is in front (true) or not. In front, sync runs after an edit and
  /// every so often; behind, it stops.
  Future<Either<ApplicationError, Unit>> setForeground({
    required bool foreground,
  });
}
