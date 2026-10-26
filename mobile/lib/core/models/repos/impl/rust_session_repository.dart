// Kanso — rust_session_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../bridge/api/locks.dart';
import '../../../bridge/api/types.dart';
import '../../../utils/cancelable_stream.dart';
import '../../../utils/kanso_error_mapping.dart';
import '../../data/application_error.dart';
import '../abs/session_repository.dart';

/// [SessionRepository] over the Rust core's lock session.
@LazySingleton(as: SessionRepository)
class RustSessionRepository implements SessionRepository {
  RustSessionRepository(this._locks);

  final KansoLocks _locks;

  @override
  Future<Either<ApplicationError, bool>> unlock(String password) =>
      attempt(() => _locks.unlock(password: password));

  @override
  Future<Either<ApplicationError, Unit>> lock() => attempt(() async {
    await _locks.lock();
    return unit;
  });

  @override
  Future<Either<ApplicationError, bool>> isUnlocked() =>
      attempt(_locks.isUnlocked);

  @override
  Future<Either<ApplicationError, bool>> isNoteUnlocked(String id) =>
      attempt(() => _locks.isNoteUnlocked(id: id));

  @override
  Future<Either<ApplicationError, Saved>> lockNote(
    String id,
    String password,
  ) => attempt(() => _locks.lockNote(id: id, password: password));

  @override
  Stream<LockEvent> lockEvents() => cancelableStream(_locks.lockEvents());
}
