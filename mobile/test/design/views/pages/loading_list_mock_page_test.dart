// Kanso — loading_list_mock_page_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/design/views/pages/loading_list_mock_page.dart';

import '../../../helpers/golden.dart';

void main() {
  group('LoadingListMockPage on a phone', () {
    testWidgets('loading list, light', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const LoadingListMockPage(),
        name: 'phone_loading_list_light',
        size: phoneSize,
        brightness: Brightness.light,
      );
    });

    testWidgets('loading list, dark', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const LoadingListMockPage(),
        name: 'phone_loading_list_dark',
        size: phoneSize,
        brightness: Brightness.dark,
      );
    });
  });
}
