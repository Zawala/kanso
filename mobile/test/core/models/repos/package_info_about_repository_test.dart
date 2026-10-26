// Kanso — package_info_about_repository_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/configs/kanso_links.dart';
import 'package:kanso/core/models/repos/impl/package_info_about_repository.dart';
import 'package:package_info_plus/package_info_plus.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test('the version and build come from the package information', () async {
    PackageInfo.setMockInitialValues(
      appName: 'Kanso',
      packageName: 'app.kanso.kanso',
      version: '1.2.3',
      buildNumber: '45',
      buildSignature: '',
    );
    final result = await PackageInfoAboutRepository().version();
    final version = result.getOrElse(() => throw StateError('no version'));
    expect(version.version, '1.2.3');
    expect(version.build, '45');
  });

  test('the website is the https address of Kanso', () {
    expect(KansoLinks.website.scheme, 'https');
    expect(KansoLinks.website.host, 'kansoapp.app');
  });
}
