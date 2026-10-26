// Kanso — theme_picker_goldens_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/theme_selection.dart';
import 'package:kanso/core/state/appearance/appearance_bloc.dart';
import 'package:kanso/themes/views/widgets/theme_picker_pane.dart';

import '../../../helpers/golden.dart';
import '../../../helpers/real_themes.dart';
import '../../../helpers/theme_picker_blocs.dart';

Widget _page(Widget child) {
  return Scaffold(resizeToAvoidBottomInset: false, body: child);
}

void main() {
  loadRealThemes();

  for (final brightness in Brightness.values) {
    final name = brightness.name;

    group('the theme picker pictures, $name', () {
      testWidgets('following the system', (tester) async {
        await expectGoldenOnBothPlatforms(
          tester,
          await withThemePicker(_page(const ThemePickerPane())),
          name: 'phone_theme_picker_$name',
          size: phoneSize,
          brightness: brightness,
        );
      });

      testWidgets('one theme, the system not followed', (tester) async {
        final nord = realPalette('nord');
        await expectGoldenOnBothPlatforms(
          tester,
          await withThemePicker(_page(const ThemePickerPane())),
          name: 'phone_theme_picker_one_theme_$name',
          size: phoneSize,
          brightness: brightness,
          palette: nord,
          appearance: AppearanceState(
            selection: const ThemeSelection(followSystem: false, ui: 'nord'),
            light: nord,
            dark: nord,
          ),
        );
      });
    });
  }
}
