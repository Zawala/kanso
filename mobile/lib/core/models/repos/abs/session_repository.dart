// Kanso — session_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../bridge/api/types.dart';
import '../../data/application_error.dart';

/// The lock session. The lock password's keys stay in the Rust core.
abstract interface class SessionRepository {
  /// Opens the locked notes with [password]; false when it is wrong.
  Future<Either<ApplicationError, bool>> unlock(String password);

  /// Closes the locked notes.
  Future<Either<ApplicationError, Unit>> lock();

  Future<Either<ApplicationError, bool>> isUnlocked();

  /// Whether the note [id] can be read now.
  Future<Either<ApplicationError, bool>> isNoteUnlocked(String id);

  /// Locks the note [id] under [password].
  Future<Either<ApplicationError, Saved>> lockNote(String id, String password);

  /// Each time the locked notes open or close.
  Stream<LockEvent> lockEvents();
}
