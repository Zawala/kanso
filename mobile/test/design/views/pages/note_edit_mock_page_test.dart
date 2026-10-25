// Kanso — note_edit_mock_page_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/design/views/pages/note_edit_mock_page.dart';

import '../../../helpers/golden.dart';

void main() {
  group('NoteEditMockPage on a phone', () {
    testWidgets('note edit, light', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const NoteEditMockPage(),
        name: 'phone_note_edit_light',
        size: phoneSize,
        brightness: Brightness.light,
      );
    });

    testWidgets('note edit, dark', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const NoteEditMockPage(),
        name: 'phone_note_edit_dark',
        size: phoneSize,
        brightness: Brightness.dark,
      );
    });
  });
}
