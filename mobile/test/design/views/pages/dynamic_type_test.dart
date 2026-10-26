// Kanso — dynamic_type_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/configs/kanso_theme.dart';
import 'package:kanso/core/state/appearance/appearance_bloc.dart';
import 'package:kanso/design/models/data/design_layout.dart';
import 'package:kanso/design/models/data/design_screen.dart';
import 'package:kanso/design/utils/page_for_screen.dart';
import 'package:kanso/gen/l10n/app_localizations.dart';
import '../../../helpers/seeded_appearance_bloc.dart';

void main() {
  group('every phone screen at a large device text size', () {
    for (final scale in <double>[1.5, 2.5]) {
      for (final screen in DesignScreen.values) {
        testWidgets('${screen.name} at $scale lays out without overflow', (
          tester,
        ) async {
          tester.view
            ..physicalSize = const Size(393, 852)
            ..devicePixelRatio = 1;
          tester.platformDispatcher.textScaleFactorTestValue = scale;
          addTearDown(() {
            tester.view.reset();
            tester.platformDispatcher.clearTextScaleFactorTestValue();
          });
          final bloc = SeededAppearanceBloc(const AppearanceState());
          addTearDown(bloc.close);
          await tester.pumpWidget(
            BlocProvider<AppearanceBloc>.value(
              value: bloc,
              child: MaterialApp(
                theme: KansoTheme.light,
                localizationsDelegates: AppLocalizations.localizationsDelegates,
                supportedLocales: AppLocalizations.supportedLocales,
                home: pageForScreen(screen, DesignLayout.phone),
              ),
            ),
          );
          await tester.pumpAndSettle();
          expect(tester.takeException(), isNull);
        });
      }
    }
  });
}
