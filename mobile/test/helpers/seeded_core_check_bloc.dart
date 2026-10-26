// Kanso — seeded_core_check_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:kanso/core/models/repos/abs/diagnostics_repository.dart';
import 'package:kanso/core/models/repos/abs/library_repository.dart';
import 'package:kanso/core_check/state/core_check/core_check_bloc.dart';
import 'package:mocktail/mocktail.dart';

class _MockLibrary extends Mock implements LibraryRepository {}

class _MockDiagnostics extends Mock implements DiagnosticsRepository {}

/// A `CoreCheckBloc` that starts in `seed` and asks nobody anything, for
/// pages whose picture depends on what the library said.
class SeededCoreCheckBloc extends CoreCheckBloc {
  SeededCoreCheckBloc(CoreCheckState seed)
    : super(_MockLibrary(), _MockDiagnostics()) {
    emit(seed);
  }
}
