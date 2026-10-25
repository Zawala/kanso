// Kanso — license_registration_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:io';

import 'package:flutter/foundation.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/kanso_font_family.dart';
import 'package:kanso/core/utils/license_registration.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test('every bundled font and the icons are on the licence page', () async {
    registerBundledLicenses();
    final packages = <String>{};
    final texts = <String>[];
    await for (final entry in LicenseRegistry.licenses) {
      packages.addAll(entry.packages);
      texts.add(
        entry.paragraphs.map((paragraph) => paragraph.text).join('\n'),
      );
    }
    for (final font in KansoFontFamily.values) {
      if (font.bundled) {
        expect(packages, contains(font.family), reason: font.displayName);
      }
    }
    expect(packages, contains('Lucide'));
    expect(
      texts.where((text) => text.contains('SIL OPEN FONT LICENSE')),
      hasLength(KansoFontFamily.values.where((f) => f.bundled).length),
    );
  });

  test('every bundled family has its files in the app bundle', () {
    final pubspec = File('pubspec.yaml').readAsStringSync();
    for (final font in KansoFontFamily.values) {
      final family = font.family;
      if (family == null) {
        continue;
      }
      expect(pubspec, contains('family: $family'), reason: family);
    }
    final fonts = Directory('assets/fonts')
        .listSync()
        .map((entity) => entity.uri.pathSegments.last)
        .where((name) => name.endsWith('.ttf'));
    for (final name in fonts) {
      expect(pubspec, contains('assets/fonts/$name'), reason: name);
    }
  });
}
