// Kanso — design_gallery_page_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/configs/kanso_theme.dart';
import 'package:kanso/core/models/data/appearance_mode.dart';
import 'package:kanso/core/state/appearance/appearance_bloc.dart';
import 'package:kanso/design/views/pages/design_gallery_page.dart';
import 'package:kanso/gen/l10n/app_localizations.dart';

Future<AppearanceBloc> _pump(WidgetTester tester) async {
  final bloc = AppearanceBloc();
  addTearDown(bloc.close);
  tester.view
    ..physicalSize = const Size(393, 2400)
    ..devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  await tester.pumpWidget(
    BlocProvider<AppearanceBloc>.value(
      value: bloc,
      child: MaterialApp(
        theme: KansoTheme.light,
        localizationsDelegates: AppLocalizations.localizationsDelegates,
        supportedLocales: AppLocalizations.supportedLocales,
        home: const DesignGalleryPage(),
      ),
    ),
  );
  await tester.pumpAndSettle();
  return bloc;
}

void main() {
  group('DesignGalleryPage', () {
    testWidgets('lists every phone and tablet screen', (tester) async {
      await _pump(tester);
      expect(find.text('Phone'), findsOneWidget);
      expect(find.text('Tablet'), findsOneWidget);
      expect(find.text('Library'), findsOneWidget);
      expect(find.text('Note list'), findsOneWidget);
      // Settings and the theme picker are listed once per layout.
      expect(find.text('Settings'), findsNWidgets(2));
      expect(find.text('Theme picker'), findsNWidgets(2));
      expect(find.text('Error card'), findsNWidgets(2));
    });

    testWidgets('the appearance control dispatches the chosen mode', (
      tester,
    ) async {
      final bloc = await _pump(tester);
      expect(bloc.state.mode, AppearanceMode.system);
      await tester.tap(find.text('Dark'));
      await tester.pumpAndSettle();
      expect(bloc.state.mode, AppearanceMode.dark);
      await tester.tap(find.text('Light'));
      await tester.pumpAndSettle();
      expect(bloc.state.mode, AppearanceMode.light);
    });

    testWidgets('opens a screen and comes back', (tester) async {
      await _pump(tester);
      await tester.tap(find.text('Locked note').first);
      await tester.pumpAndSettle();
      expect(find.text('This note is locked'), findsOneWidget);
      tester.state<NavigatorState>(find.byType(Navigator)).pop();
      await tester.pumpAndSettle();
      expect(find.text('Design proposal'), findsOneWidget);
    });
  });
}
