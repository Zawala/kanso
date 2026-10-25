// Kanso — note_preview_mock_page_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/kanso_font_family.dart';
import 'package:kanso/core/models/data/kanso_fonts.dart';
import 'package:kanso/core/state/appearance/appearance_bloc.dart';
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

    testWidgets('note preview in Source Serif 4, light', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const NotePreviewMockPage(),
        name: 'phone_note_preview_serif_light',
        size: phoneSize,
        brightness: Brightness.light,
        appearance: const AppearanceState(
          fonts: KansoFonts(previewFont: KansoFontFamily.sourceSerif4),
        ),
      );
    });

    testWidgets('note preview in Source Serif 4, dark', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const NotePreviewMockPage(),
        name: 'phone_note_preview_serif_dark',
        size: phoneSize,
        brightness: Brightness.dark,
        appearance: const AppearanceState(
          fonts: KansoFonts(previewFont: KansoFontFamily.sourceSerif4),
        ),
      );
    });
  });
}
