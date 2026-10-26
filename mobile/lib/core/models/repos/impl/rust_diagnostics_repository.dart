// Kanso — rust_diagnostics_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../bridge/api/diagnostics.dart' as core;
import '../../../utils/kanso_error_mapping.dart';
import '../../data/application_error.dart';
import '../abs/diagnostics_repository.dart';

/// [DiagnosticsRepository] over the Rust core's diagnostics.
@LazySingleton(as: DiagnosticsRepository)
class RustDiagnosticsRepository implements DiagnosticsRepository {
  @override
  Future<Either<ApplicationError, String>> version() => attempt(core.version);

  @override
  Future<Either<ApplicationError, bool>> tlsReady() => attempt(core.tlsReady);

  @override
  Future<Either<ApplicationError, Unit>> panicProbe(String message) =>
      attempt(() async {
        await core.panicProbe(message: message);
        return unit;
      });

  @override
  Future<Either<ApplicationError, Unit>> secretsProbe() => attempt(() async {
    await core.secretsProbe();
    return unit;
  });
}
