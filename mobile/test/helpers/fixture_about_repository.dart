// Kanso — fixture_about_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:kanso/core/models/data/app_version.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/repos/abs/about_repository.dart';

/// An About repository for widget tests: version 0.1.0 build 3, and a
/// website that records how often it was opened.
class FixtureAboutRepository implements AboutRepository {
  /// How many times the website was opened.
  int websiteOpens = 0;

  @override
  Future<Either<ApplicationError, AppVersion>> version() async =>
      const Right<ApplicationError, AppVersion>(
        AppVersion(version: '0.1.0', build: '3'),
      );

  @override
  Future<Either<ApplicationError, Unit>> openWebsite() async {
    websiteOpens++;
    return const Right<ApplicationError, Unit>(unit);
  }
}
