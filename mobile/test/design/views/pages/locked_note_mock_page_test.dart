// Kanso — locked_note_mock_page_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/design/views/pages/locked_note_mock_page.dart';

import '../../../helpers/golden.dart';

void main() {
  group('LockedNoteMockPage on a phone', () {
    testWidgets('locked note, light', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const LockedNoteMockPage(),
        name: 'phone_locked_note_light',
        size: phoneSize,
        brightness: Brightness.light,
      );
    });

    testWidgets('locked note, dark', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const LockedNoteMockPage(),
        name: 'phone_locked_note_dark',
        size: phoneSize,
        brightness: Brightness.dark,
      );
    });
  });
}
