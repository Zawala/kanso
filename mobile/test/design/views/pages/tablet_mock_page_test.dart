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

    testWidgets('fonts, light', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const TabletMockPage(screen: DesignScreen.fonts),
        name: 'tablet_fonts_light',
        size: tabletLandscapeSize,
        brightness: Brightness.light,
      );
    });

    testWidgets('fonts, dark', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const TabletMockPage(screen: DesignScreen.fonts),
        name: 'tablet_fonts_dark',
        size: tabletLandscapeSize,
        brightness: Brightness.dark,
      );
    });

    testWidgets('fonts scrolled, light', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const TabletMockPage(screen: DesignScreen.fontsScrolled),
        name: 'tablet_fonts_scrolled_light',
        size: tabletLandscapeSize,
        brightness: Brightness.light,
      );
    });

    testWidgets('fonts scrolled, dark', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const TabletMockPage(screen: DesignScreen.fontsScrolled),
        name: 'tablet_fonts_scrolled_dark',
        size: tabletLandscapeSize,
        brightness: Brightness.dark,
      );
    });
  });
}
