// Kanso — seeded_appearance_bloc.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/preferences.dart';
import 'package:kanso/core/models/repos/abs/preferences_repository.dart';
import 'package:kanso/core/models/repos/abs/renderer_repository.dart';
import 'package:kanso/core/state/appearance/appearance_bloc.dart';
import 'package:mocktail/mocktail.dart';

class MockPreferencesRepository extends Mock implements PreferencesRepository {}

class MockRendererRepository extends Mock implements RendererRepository {}

/// A preferences repository that remembers nothing it is given back: loads
/// are the defaults and every update succeeds.
class ForgetfulPreferencesRepository implements PreferencesRepository {
  @override
  Future<Either<ApplicationError, Preferences>> load() async =>
      const Right<ApplicationError, Preferences>(Preferences());

  @override
  Future<Either<ApplicationError, Preferences>> update(
    Preferences Function(Preferences current) change,
  ) async => Right<ApplicationError, Preferences>(change(const Preferences()));
}

/// An `AppearanceBloc` that starts in `seed` rather than the defaults, for
/// pages whose picture depends on the person's choices. It saves nothing and
/// has no themes to read besides Kanso's.
class SeededAppearanceBloc extends AppearanceBloc {
  SeededAppearanceBloc(AppearanceState seed)
    : super(ForgetfulPreferencesRepository(), MockRendererRepository()) {
    emit(seed);
  }
}
