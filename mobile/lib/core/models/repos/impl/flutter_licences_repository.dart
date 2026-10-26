// Kanso — flutter_licences_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter/foundation.dart';
import 'package:injectable/injectable.dart';

import '../../data/application_error.dart';
import '../../data/licence_package.dart';
import '../abs/licences_repository.dart';

/// The licences Flutter collects from the app's packages, its bundled fonts
/// and its icons (`LicenseRegistry`), grouped by package.
@LazySingleton(as: LicencesRepository)
class FlutterLicencesRepository implements LicencesRepository {
  @override
  Future<Either<ApplicationError, List<LicencePackage>>> licences() async {
    try {
      final byPackage = <String, List<String>>{};
      await for (final entry in LicenseRegistry.licenses) {
        final text = entry.paragraphs
            .map((paragraph) => paragraph.text)
            .join('\n\n');
        for (final package in entry.packages) {
          byPackage.putIfAbsent(package, () => <String>[]).add(text);
        }
      }
      final names = byPackage.keys.toList()
        ..sort((a, b) => a.toLowerCase().compareTo(b.toLowerCase()));
      return Right<ApplicationError, List<LicencePackage>>(<LicencePackage>[
        for (final name in names)
          LicencePackage(name: name, licences: byPackage[name]!),
      ]);
    } on Object catch (error) {
      return Left<ApplicationError, List<LicencePackage>>(
        ApplicationError.unexpected(message: '$error'),
      );
    }
  }
}
