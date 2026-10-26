// Kanso — diagnostics_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../data/application_error.dart';

/// What the app can ask the Rust core about itself.
abstract interface class DiagnosticsRepository {
  /// The library's version.
  Future<Either<ApplicationError, String>> version();

  /// Whether the core can check a server's certificate now.
  Future<Either<ApplicationError, bool>> tlsReady();

  /// Makes the core panic inside its boundary; the result is an
  /// [InternalError] and the core keeps working. For tests on a device.
  Future<Either<ApplicationError, Unit>> panicProbe(String message);

  /// Writes a value to the phone's secret store (Keychain or Keystore),
  /// reads it back and removes it: the round trip the signed-in account's
  /// session takes, with nothing of the account's.
  Future<Either<ApplicationError, Unit>> secretsProbe();
}
