// Kanso — design_gallery_page_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/configs/kanso_theme.dart';
import 'package:kanso/core/models/data/kanso_font_family.dart';
import 'package:kanso/core/state/appearance/appearance_bloc.dart';
import 'package:kanso/design/models/data/gallery_appearance.dart';
import 'package:kanso/design/views/pages/design_gallery_page.dart';
import 'package:kanso/design/views/widgets/loading_list_pane.dart';
import 'package:kanso/gen/l10n/app_localizations.dart';

import '../../../helpers/seeded_appearance_bloc.dart';

Future<AppearanceBloc> _pump(WidgetTester tester) async {
  final bloc = SeededAppearanceBloc(const AppearanceState());
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

/// Lets the appearance BLoC finish a choice: it reads the themes and saves
/// before the next one, in microtasks that a single pump runs.
Future<void> _settle(WidgetTester tester) async {
  for (var i = 0; i < 5; i++) {
    await tester.pump();
  }
  await tester.pumpAndSettle();
}

void main() {
  group('DesignGalleryPage', () {
    testWidgets('lists every phone and tablet screen', (tester) async {
      await _pump(tester);
      expect(find.text('Rust core'), findsOneWidget);
      expect(find.text('Phone'), findsOneWidget);
      expect(find.text('Tablet'), findsOneWidget);
      expect(find.text('Library'), findsOneWidget);
      expect(find.text('Note list'), findsOneWidget);
      // Settings and the theme picker are listed once per layout.
      expect(find.text('Settings'), findsNWidgets(2));
      expect(find.text('Theme picker'), findsNWidgets(2));
      expect(find.text('Error card'), findsNWidgets(2));
      // The fonts, the loading list and the empty notebook have both.
      expect(find.text('Fonts'), findsNWidgets(2));
      expect(find.text('Fonts, scrolled'), findsNWidgets(2));
      expect(find.text('Note list, loading'), findsNWidgets(2));
      expect(find.text('Empty notebook'), findsNWidgets(2));
    });

    testWidgets('the appearance control dispatches the chosen mode', (
      tester,
    ) async {
      final bloc = await _pump(tester);
      expect(GalleryAppearance.of(bloc.state), GalleryAppearance.system);
      await tester.tap(find.text('Dark'));
      await _settle(tester);
      expect(GalleryAppearance.of(bloc.state), GalleryAppearance.dark);
      expect(bloc.state.selection.ui, 'kanso-dark');
      await tester.tap(find.text('Light'));
      await _settle(tester);
      expect(GalleryAppearance.of(bloc.state), GalleryAppearance.light);
      await tester.tap(find.text('System'));
      await _settle(tester);
      expect(GalleryAppearance.of(bloc.state), GalleryAppearance.system);
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

    testWidgets('opens Fonts and a font chosen there restyles the notes', (
      tester,
    ) async {
      final bloc = await _pump(tester);
      await tester.tap(find.text('Fonts').first);
      await tester.pumpAndSettle();
      expect(find.text('Editor font'), findsOneWidget);
      await tester.tap(find.text('Literata').last);
      await tester.pumpAndSettle();
      expect(bloc.state.fonts.previewFont, KansoFontFamily.literata);
    });

    testWidgets('opens the empty notebook and the loading list', (
      tester,
    ) async {
      await _pump(tester);
      await tester.tap(find.text('Empty notebook').first);
      await tester.pumpAndSettle();
      expect(find.text('No notes in Someday yet'), findsOneWidget);
      tester.state<NavigatorState>(find.byType(Navigator)).pop();
      await tester.pumpAndSettle();
      await tester.tap(find.text('Note list, loading').first);
      await tester.pumpAndSettle();
      expect(find.byType(LoadingListPane), findsOneWidget);
    });
  });
}
