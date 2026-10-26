// Kanso — settings_pane_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/kanso_font_family.dart';
import 'package:kanso/core/models/data/kanso_text_size.dart';
import 'package:kanso/core/state/appearance/appearance_bloc.dart';
import 'package:kanso/design/views/widgets/settings_pane.dart';

import '../../../helpers/pump.dart';
import '../../../helpers/seeded_appearance_bloc.dart';

Future<AppearanceBloc> _pump(WidgetTester tester) async {
  final bloc = SeededAppearanceBloc(const AppearanceState());
  addTearDown(bloc.close);
  tester.view
    ..physicalSize = const Size(393, 2400)
    ..devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  await pumpKanso(
    tester,
    BlocProvider<AppearanceBloc>.value(
      value: bloc,
      child: const SizedBox(width: 393, height: 2400, child: SettingsPane()),
    ),
  );
  return bloc;
}

void main() {
  group('SettingsPane', () {
    testWidgets('has a Fonts row under Appearance and no Editor font row', (
      tester,
    ) async {
      await _pump(tester);
      expect(find.text('Fonts'), findsOneWidget);
      expect(find.text('Editor font'), findsNothing);
      final fonts = tester.getTopLeft(find.text('Fonts')).dy;
      expect(fonts, greaterThan(tester.getTopLeft(find.text('Appearance')).dy));
      expect(fonts, lessThan(tester.getTopLeft(find.text('Sync')).dy));
    });

    testWidgets('the Fonts row sums up the choice and follows it', (
      tester,
    ) async {
      final bloc = await _pump(tester);
      expect(
        find.text('Instrument Sans \u00b7 JetBrains Mono'),
        findsOneWidget,
      );
      bloc
        ..add(const AppearanceEvent.editorFontChanged(KansoFontFamily.system))
        ..add(
          const AppearanceEvent.codeFontChanged(KansoFontFamily.ibmPlexMono),
        );
      await tester.pumpAndSettle();
      expect(
        find.text('System \u00b7 Instrument Sans \u00b7 IBM Plex Mono'),
        findsOneWidget,
      );
    });

    testWidgets('the Text size row shows the size chosen', (tester) async {
      final bloc = await _pump(tester);
      expect(find.text('Default'), findsOneWidget);
      bloc.add(const AppearanceEvent.textSizeChanged(KansoTextSize.larger));
      await tester.pumpAndSettle();
      expect(find.text('Larger'), findsOneWidget);
    });
  });
}
