// Kanso — flutter_licences_repository_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/foundation.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/repos/impl/flutter_licences_repository.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test(
    'groups the registry by package, by name, with the text joined',
    () async {
      LicenseRegistry.addLicense(() async* {
        yield const LicenseEntryWithLineBreaks(
          <String>['zeta', 'Alpha'],
          'First line.\n\nSecond paragraph.',
        );
        yield const LicenseEntryWithLineBreaks(<String>[
          'zeta',
        ], 'Other terms.');
      });
      final result = await FlutterLicencesRepository().licences();
      final packages = result.getOrElse(() => throw StateError('no licences'));
      final names = packages.map((package) => package.name).toList();
      expect(names, containsAllInOrder(<String>['Alpha', 'zeta']));
      final zeta = packages.firstWhere((package) => package.name == 'zeta');
      expect(zeta.licences, contains('Other terms.'));
      expect(zeta.licences.first, 'First line.\n\nSecond paragraph.');
      final sorted = [...names]
        ..sort((a, b) => a.toLowerCase().compareTo(b.toLowerCase()));
      expect(names, sorted);
    },
  );
}
