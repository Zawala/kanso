// Kanso — fixture_sync_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/repos/abs/sync_repository.dart';

/// The status of a phone with no account.
const SyncStatus signedOutStatus = SyncStatus(
  phase: SyncPhase.signedOut,
  conflicts: 0,
  skippedCount: 0,
  account: SyncAccount.signedOut(),
);

/// The status of a phone signed in to an account, with nothing to do.
const SyncStatus signedInStatus = SyncStatus(
  phase: SyncPhase.idle,
  username: 'ana',
  server: 'https://example.invalid',
  conflicts: 0,
  skippedCount: 0,
  account: SyncAccount.signedIn(),
);

/// A sync service for widget tests: it has the status a test gives it,
/// counts the syncs it is asked for, and can hold one until the test lets
/// it go.
class FixtureSyncRepository implements SyncRepository {
  FixtureSyncRepository({this.current = signedOutStatus});

  /// The status the account has.
  SyncStatus current;

  /// How many times a sync was asked for.
  int syncs = 0;

  /// When set, a sync waits for it.
  Completer<void>? gate;

  @override
  Future<Either<ApplicationError, SyncReportRow>> syncNow() async {
    syncs++;
    await gate?.future;
    return const Right<ApplicationError, SyncReportRow>(
      SyncReportRow(
        pulled: 0,
        pushed: 0,
        conflicts: 0,
        pending: 0,
        queued: false,
      ),
    );
  }

  @override
  Stream<SyncStatus> statusStream() => Stream<SyncStatus>.value(current);

  /// Whether the app was last said to be in front.
  bool? foreground;

  @override
  Future<Either<ApplicationError, Unit>> setForeground({
    required bool foreground,
  }) async {
    this.foreground = foreground;
    return const Right<ApplicationError, Unit>(unit);
  }

  @override
  Future<Either<ApplicationError, SyncStatus>> status() async =>
      Right<ApplicationError, SyncStatus>(current);

  @override
  Future<Either<ApplicationError, String>> defaultServer() async =>
      const Right<ApplicationError, String>('https://sync.kansoapp.app');

  @override
  Future<Either<ApplicationError, ServerInfo>> checkServer(String url) =>
      throw UnimplementedError();

  @override
  Future<Either<ApplicationError, SyncStatus>> signInPassword({
    required String server,
    required String username,
    required String password,
    required String deviceName,
  }) => throw UnimplementedError();

  @override
  Future<Either<ApplicationError, Unit>> signOut() =>
      throw UnimplementedError();
}
