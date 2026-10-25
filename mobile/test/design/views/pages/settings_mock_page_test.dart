// Kanso — settings_mock_page_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/design/views/pages/settings_mock_page.dart';

import '../../../helpers/golden.dart';

void main() {
  group('SettingsMockPage on a phone', () {
    testWidgets('settings, light', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const SettingsMockPage(),
        name: 'phone_settings_light',
        size: phoneSize,
        brightness: Brightness.light,
      );
    });

    testWidgets('settings, dark', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const SettingsMockPage(),
        name: 'phone_settings_dark',
        size: phoneSize,
        brightness: Brightness.dark,
      );
    });

    testWidgets('settings scrolled, light', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const SettingsMockPage(scrolled: true),
        name: 'phone_settings_scrolled_light',
        size: phoneSize,
        brightness: Brightness.light,
      );
    });

    testWidgets('settings scrolled, dark', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const SettingsMockPage(scrolled: true),
        name: 'phone_settings_scrolled_dark',
        size: phoneSize,
        brightness: Brightness.dark,
      );
    });
  });
}
