// Kanso — licences_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../data/application_error.dart';
import '../../data/licence_package.dart';

/// The licences of what the app is made of.
///
/// A seam for tests: the licences come from the app's own registry.
// ignore: one_member_abstracts
abstract interface class LicencesRepository {
  /// Every package with its licences, by name.
  Future<Either<ApplicationError, List<LicencePackage>>> licences();
}
