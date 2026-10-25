// Kanso — error_mock_page_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/design/views/pages/error_mock_page.dart';

import '../../../helpers/golden.dart';

void main() {
  group('ErrorMockPage on a phone', () {
    testWidgets('error, light', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const ErrorMockPage(),
        name: 'phone_error_light',
        size: phoneSize,
        brightness: Brightness.light,
      );
    });

    testWidgets('error, dark', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const ErrorMockPage(),
        name: 'phone_error_dark',
        size: phoneSize,
        brightness: Brightness.dark,
      );
    });
  });
}
