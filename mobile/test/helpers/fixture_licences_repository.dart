// Kanso — fixture_licences_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/licence_package.dart';
import 'package:kanso/core/models/repos/abs/licences_repository.dart';

/// A few packages with licences, for widget tests.
class FixtureLicencesRepository implements LicencesRepository {
  @override
  Future<Either<ApplicationError, List<LicencePackage>>> licences() async =>
      const Right<ApplicationError, List<LicencePackage>>(<LicencePackage>[
        LicencePackage(
          name: 'Instrument Sans',
          licences: <String>['The SIL Open Font License, Version 1.1.'],
        ),
        LicencePackage(
          name: 'bloc',
          licences: <String>['The MIT License.'],
        ),
        LicencePackage(
          name: 'flutter',
          licences: <String>[
            'The BSD 3-Clause License.',
            'The BSD 3-Clause License, as used in the engine.',
          ],
        ),
      ]);
}
