// Kanso — note_preview_mock_page_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/design/views/pages/note_preview_mock_page.dart';

import '../../../helpers/golden.dart';

void main() {
  group('NotePreviewMockPage on a phone', () {
    testWidgets('note preview, light', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const NotePreviewMockPage(),
        name: 'phone_note_preview_light',
        size: phoneSize,
        brightness: Brightness.light,
      );
    });

    testWidgets('note preview, dark', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const NotePreviewMockPage(),
        name: 'phone_note_preview_dark',
        size: phoneSize,
        brightness: Brightness.dark,
      );
    });
  });
}
