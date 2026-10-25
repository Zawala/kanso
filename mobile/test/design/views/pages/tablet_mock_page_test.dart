// Kanso — tablet_mock_page_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/design/models/data/design_screen.dart';
import 'package:kanso/design/views/pages/tablet_mock_page.dart';

import '../../../helpers/golden.dart';

void main() {
  group('TabletMockPage', () {
    testWidgets('note preview, light', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const TabletMockPage(screen: DesignScreen.notePreview),
        name: 'tablet_note_preview_light',
        size: tabletLandscapeSize,
        brightness: Brightness.light,
      );
    });

    testWidgets('note preview, dark', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const TabletMockPage(screen: DesignScreen.notePreview),
        name: 'tablet_note_preview_dark',
        size: tabletLandscapeSize,
        brightness: Brightness.dark,
      );
    });

    testWidgets('note edit, light', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const TabletMockPage(screen: DesignScreen.noteEdit),
        name: 'tablet_note_edit_light',
        size: tabletLandscapeSize,
        brightness: Brightness.light,
      );
    });

    testWidgets('note edit, dark', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const TabletMockPage(screen: DesignScreen.noteEdit),
        name: 'tablet_note_edit_dark',
        size: tabletLandscapeSize,
        brightness: Brightness.dark,
      );
    });

    testWidgets('search, light', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const TabletMockPage(screen: DesignScreen.search),
        name: 'tablet_search_light',
        size: tabletLandscapeSize,
        brightness: Brightness.light,
      );
    });

    testWidgets('search, dark', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const TabletMockPage(screen: DesignScreen.search),
        name: 'tablet_search_dark',
        size: tabletLandscapeSize,
        brightness: Brightness.dark,
      );
    });

    testWidgets('settings, light', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const TabletMockPage(screen: DesignScreen.settings),
        name: 'tablet_settings_light',
        size: tabletLandscapeSize,
        brightness: Brightness.light,
      );
    });

    testWidgets('settings, dark', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const TabletMockPage(screen: DesignScreen.settings),
        name: 'tablet_settings_dark',
        size: tabletLandscapeSize,
        brightness: Brightness.dark,
      );
    });

    testWidgets('settings scrolled, light', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const TabletMockPage(screen: DesignScreen.settingsScrolled),
        name: 'tablet_settings_scrolled_light',
        size: tabletLandscapeSize,
        brightness: Brightness.light,
      );
    });

    testWidgets('settings scrolled, dark', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const TabletMockPage(screen: DesignScreen.settingsScrolled),
        name: 'tablet_settings_scrolled_dark',
        size: tabletLandscapeSize,
        brightness: Brightness.dark,
      );
    });

    testWidgets('theme picker, light', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const TabletMockPage(screen: DesignScreen.themePicker),
        name: 'tablet_theme_picker_light',
        size: tabletLandscapeSize,
        brightness: Brightness.light,
      );
    });

    testWidgets('theme picker, dark', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const TabletMockPage(screen: DesignScreen.themePicker),
        name: 'tablet_theme_picker_dark',
        size: tabletLandscapeSize,
        brightness: Brightness.dark,
      );
    });

    testWidgets('theme picker scrolled, light', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const TabletMockPage(screen: DesignScreen.themePickerScrolled),
        name: 'tablet_theme_picker_scrolled_light',
        size: tabletLandscapeSize,
        brightness: Brightness.light,
      );
    });

    testWidgets('theme picker scrolled, dark', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const TabletMockPage(screen: DesignScreen.themePickerScrolled),
        name: 'tablet_theme_picker_scrolled_dark',
        size: tabletLandscapeSize,
        brightness: Brightness.dark,
      );
    });

    testWidgets('locked note, light', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const TabletMockPage(screen: DesignScreen.lockedNote),
        name: 'tablet_locked_note_light',
        size: tabletLandscapeSize,
        brightness: Brightness.light,
      );
    });

    testWidgets('locked note, dark', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const TabletMockPage(screen: DesignScreen.lockedNote),
        name: 'tablet_locked_note_dark',
        size: tabletLandscapeSize,
        brightness: Brightness.dark,
      );
    });

    testWidgets('error, light', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const TabletMockPage(screen: DesignScreen.error),
        name: 'tablet_error_light',
        size: tabletLandscapeSize,
        brightness: Brightness.light,
      );
    });

    testWidgets('error, dark', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const TabletMockPage(screen: DesignScreen.error),
        name: 'tablet_error_dark',
        size: tabletLandscapeSize,
        brightness: Brightness.dark,
      );
    });
  });
}
