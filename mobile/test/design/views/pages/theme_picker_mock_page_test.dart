// Kanso — theme_picker_mock_page_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/design/views/pages/theme_picker_mock_page.dart';

import '../../../helpers/golden.dart';

void main() {
  group('ThemePickerMockPage on a phone', () {
    testWidgets('theme picker, light', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const ThemePickerMockPage(),
        name: 'phone_theme_picker_light',
        size: phoneSize,
        brightness: Brightness.light,
      );
    });

    testWidgets('theme picker, dark', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const ThemePickerMockPage(),
        name: 'phone_theme_picker_dark',
        size: phoneSize,
        brightness: Brightness.dark,
      );
    });

    testWidgets('theme picker scrolled, light', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const ThemePickerMockPage(scrolled: true),
        name: 'phone_theme_picker_scrolled_light',
        size: phoneSize,
        brightness: Brightness.light,
      );
    });

    testWidgets('theme picker scrolled, dark', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const ThemePickerMockPage(scrolled: true),
        name: 'phone_theme_picker_scrolled_dark',
        size: phoneSize,
        brightness: Brightness.dark,
      );
    });
  });
}
