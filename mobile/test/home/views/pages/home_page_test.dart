// Kanso — home_page_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/home/views/pages/home_page.dart';

import '../../../helpers/golden.dart';

void main() {
  group('HomePage', () {
    testWidgets('light, phone', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const HomePage(),
        name: 'home_page_light_phone',
        size: phoneSize,
        brightness: Brightness.light,
      );
    });

    testWidgets('dark, phone', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const HomePage(),
        name: 'home_page_dark_phone',
        size: phoneSize,
        brightness: Brightness.dark,
      );
    });

    testWidgets('shows the app name and the placeholder line', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const HomePage(),
        name: 'home_page_light_tablet',
        size: tabletLandscapeSize,
        brightness: Brightness.light,
      );
      expect(find.text('Kanso'), findsOneWidget);
      expect(find.textContaining('on its way'), findsOneWidget);
    });
  });
}
