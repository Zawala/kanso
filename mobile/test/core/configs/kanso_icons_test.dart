// Kanso — kanso_icons_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/configs/kanso_icons.dart';

void main() {
  group('KansoIcons', () {
    test('every icon has its SVG bundled', () {
      for (final icon in KansoIcons.values) {
        expect(
          File(icon.assetPath).existsSync(),
          isTrue,
          reason: '${icon.name} has no file at ${icon.assetPath}',
        );
      }
    });

    test('no two icons share an SVG', () {
      final paths = KansoIcons.values.map((icon) => icon.assetPath).toSet();
      expect(paths.length, KansoIcons.values.length);
    });

    test('every bundled SVG is used by an icon', () {
      final used = KansoIcons.values.map((icon) => icon.assetPath).toSet();
      final bundled = Directory('assets/icons/lucide')
          .listSync()
          .whereType<File>()
          .map((file) => file.path)
          .where((path) => path.endsWith('.svg'))
          .toSet();
      expect(bundled, used);
    });
  });
}
