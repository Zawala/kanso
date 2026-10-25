// Kanso — note_list_mock_page_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/design/views/pages/note_list_mock_page.dart';

import '../../../helpers/golden.dart';

void main() {
  group('NoteListMockPage on a phone', () {
    testWidgets('note list, light', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const NoteListMockPage(),
        name: 'phone_note_list_light',
        size: phoneSize,
        brightness: Brightness.light,
      );
    });

    testWidgets('note list, dark', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const NoteListMockPage(),
        name: 'phone_note_list_dark',
        size: phoneSize,
        brightness: Brightness.dark,
      );
    });
  });
}
