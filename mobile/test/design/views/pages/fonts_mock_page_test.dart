// Kanso — fonts_mock_page_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/design/views/pages/fonts_mock_page.dart';

import '../../../helpers/golden.dart';

void main() {
  group('FontsMockPage on a phone', () {
    testWidgets('fonts, light', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const FontsMockPage(),
        name: 'phone_fonts_light',
        size: phoneSize,
        brightness: Brightness.light,
      );
    });

    testWidgets('fonts, dark', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const FontsMockPage(),
        name: 'phone_fonts_dark',
        size: phoneSize,
        brightness: Brightness.dark,
      );
    });

    testWidgets('fonts scrolled, light', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const FontsMockPage(scrolled: true),
        name: 'phone_fonts_scrolled_light',
        size: phoneSize,
        brightness: Brightness.light,
      );
    });

    testWidgets('fonts scrolled, dark', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const FontsMockPage(scrolled: true),
        name: 'phone_fonts_scrolled_dark',
        size: phoneSize,
        brightness: Brightness.dark,
      );
    });
  });
}
