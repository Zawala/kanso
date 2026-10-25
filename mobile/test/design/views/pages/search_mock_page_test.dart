// Kanso — search_mock_page_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/design/views/pages/search_mock_page.dart';

import '../../../helpers/golden.dart';

void main() {
  group('SearchMockPage on a phone', () {
    testWidgets('search, light', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const SearchMockPage(),
        name: 'phone_search_light',
        size: phoneSize,
        brightness: Brightness.light,
      );
    });

    testWidgets('search, dark', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const SearchMockPage(),
        name: 'phone_search_dark',
        size: phoneSize,
        brightness: Brightness.dark,
      );
    });
  });
}
