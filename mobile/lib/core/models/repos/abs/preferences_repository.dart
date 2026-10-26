// Kanso — preferences_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../data/application_error.dart';
import '../../data/preferences.dart';

/// What the person chose, kept on this phone.
abstract interface class PreferencesRepository {
  /// The saved preferences; the defaults when nothing was saved yet or the
  /// saved file cannot be read (the next change overwrites it).
  Future<Either<ApplicationError, Preferences>> load();

  /// Applies [change] to the saved preferences and saves the result, one
  /// change at a time in the order asked, so two parts of the app changing
  /// different choices cannot undo each other. The preferences as saved.
  Future<Either<ApplicationError, Preferences>> update(
    Preferences Function(Preferences current) change,
  );
}
