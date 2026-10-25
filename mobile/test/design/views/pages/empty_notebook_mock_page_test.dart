// Kanso — empty_notebook_mock_page_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/design/views/pages/empty_notebook_mock_page.dart';

import '../../../helpers/golden.dart';

void main() {
  group('EmptyNotebookMockPage on a phone', () {
    testWidgets('empty notebook, light', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const EmptyNotebookMockPage(),
        name: 'phone_empty_notebook_light',
        size: phoneSize,
        brightness: Brightness.light,
      );
    });

    testWidgets('empty notebook, dark', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const EmptyNotebookMockPage(),
        name: 'phone_empty_notebook_dark',
        size: phoneSize,
        brightness: Brightness.dark,
      );
    });
  });
}
