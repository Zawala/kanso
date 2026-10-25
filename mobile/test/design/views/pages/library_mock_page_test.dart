// Kanso — library_mock_page_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/design/views/pages/library_mock_page.dart';

import '../../../helpers/golden.dart';

void main() {
  group('LibraryMockPage on a phone', () {
    testWidgets('library, light', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const LibraryMockPage(),
        name: 'phone_library_light',
        size: phoneSize,
        brightness: Brightness.light,
      );
    });

    testWidgets('library, dark', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const LibraryMockPage(),
        name: 'phone_library_dark',
        size: phoneSize,
        brightness: Brightness.dark,
      );
    });

    testWidgets('library scrolled, light', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const LibraryMockPage(scrolled: true),
        name: 'phone_library_scrolled_light',
        size: phoneSize,
        brightness: Brightness.light,
      );
    });

    testWidgets('library scrolled, dark', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const LibraryMockPage(scrolled: true),
        name: 'phone_library_scrolled_dark',
        size: phoneSize,
        brightness: Brightness.dark,
      );
    });
  });
}
