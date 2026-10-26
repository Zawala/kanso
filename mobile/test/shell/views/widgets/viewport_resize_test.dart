// Kanso — viewport_resize_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/layout_class.dart';
import 'package:kanso/core/router/kanso_routes.dart';
import 'package:kanso/core/views/widgets/kanso_drawer.dart';
import 'package:kanso/core/views/widgets/kanso_sheet.dart';
import 'package:kanso/note/views/pages/note_page.dart';
import 'package:kanso/note/views/widgets/no_note_selected_pane.dart';
import 'package:kanso/note_list/views/widgets/note_list_pane.dart';
import 'package:kanso/search/views/widgets/search_pane.dart';
import 'package:kanso/settings/views/pages/settings_page.dart';
import 'package:kanso/shell/views/widgets/library_navigation.dart';

import '../../../helpers/browse_blocs.dart';
import '../../../helpers/fixture_library_repository.dart';
import '../../../helpers/kanso_viewport.dart';
import '../../../helpers/library_fixtures.dart';
import '../../../helpers/pump_browse_app.dart';
import '../../../helpers/real_themes.dart';
import '../../../helpers/resize_viewport.dart';

final String _syncNote =
    '/notes/note/${Uri.encodeComponent(MockIds.syncDesign)}';

Future<BrowseApp> _open(
  WidgetTester tester,
  KansoViewport viewport, {
  String at = '/notes',
  FixtureLibraryRepository? library,
}) {
  return atMockNow(
    () => pumpBrowseApp(
      tester,
      viewport.size,
      at: at,
      library: library,
      devicePixelRatio: viewport.devicePixelRatio,
      safeArea: viewport.safeArea,
      tabletDevice: viewport.tablet,
    ),
  );
}

/// What a window must show of the browse shell, by its layout class.
void _expectLayout(KansoViewport viewport, BrowseApp app) {
  final reason = viewport.name;
  expect(app.layoutClass.value, viewport.layout, reason: reason);
  expect(
    find.byType(LibraryNavigation).evaluate().isNotEmpty,
    viewport.layout == LayoutClass.triple,
    reason: '$reason: the library is a pane on three panes only',
  );
  expect(
    find.byType(KansoDrawer, skipOffstage: false).evaluate().isNotEmpty,
    viewport.layout != LayoutClass.triple,
    reason: '$reason: the library is a drawer on one or two panes',
  );
}

double _scrolled(WidgetTester tester) {
  final scrollable = find.descendant(
    of: find.byType(NoteListPane),
    matching: find.byType(Scrollable),
  );
  return tester.state<ScrollableState>(scrollable.first).position.pixels;
}

void main() {
  loadRealThemes();

  const walk = <KansoViewport>[
    KansoViewport.iPadLandscape,
    KansoViewport.splitViewTwoThirds,
    KansoViewport.splitViewHalf,
    KansoViewport.slideOver,
    KansoViewport.splitViewHalf,
    KansoViewport.splitViewTwoThirds,
    KansoViewport.iPadLandscape,
  ];

  group('a window that changes size', () {
    testWidgets('keeps the open note through Split View and Slide Over', (
      tester,
    ) async {
      final app = await _open(
        tester,
        KansoViewport.iPadLandscape,
        at: _syncNote,
      );
      for (final viewport in walk) {
        await resizeToViewport(tester, viewport);
        expect(tester.takeException(), isNull, reason: viewport.name);
        _expectLayout(viewport, app);
        expect(app.where, _syncNote, reason: viewport.name);
        expect(
          tester.widget<NotePage>(find.byType(NotePage)).noteId,
          MockIds.syncDesign,
          reason: viewport.name,
        );
        expect(
          tester.widget<NotePage>(find.byType(NotePage)).inPane,
          viewport.layout != LayoutClass.single,
          reason: viewport.name,
        );
        expect(find.text('Sync design notes'), findsWidgets);
      }
    });

    testWidgets('keeps the list where it was scrolled while a list is shown', (
      tester,
    ) async {
      final app = await _open(tester, KansoViewport.iPadLandscape);
      // Not so far that a taller window, with less to scroll, moves it.
      await tester.drag(find.byType(NoteListPane), const Offset(0, -150));
      await tester.pumpAndSettle();
      final before = _scrolled(tester);
      expect(before, greaterThan(0));

      // Wide windows keep the list in its own pane.
      for (final viewport in <KansoViewport>[
        KansoViewport.splitViewTwoThirds,
        KansoViewport.iPadLandscape,
        KansoViewport.iPadPortrait,
        KansoViewport.iPadLandscape,
      ]) {
        await resizeToViewport(tester, viewport);
        expect(tester.takeException(), isNull, reason: viewport.name);
        expect(_scrolled(tester), before, reason: viewport.name);
        expect(app.where, '/notes');
      }
    });

    testWidgets('has the list under the note on one pane and beside it again', (
      tester,
    ) async {
      final app = await _open(tester, KansoViewport.iPadLandscape);
      await tester.drag(find.byType(NoteListPane), const Offset(0, -150));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Lisbon in spring'));
      await tester.pumpAndSettle();
      final opened = app.where;
      expect(opened, contains('/note/'));

      await resizeToViewport(tester, KansoViewport.slideOver);
      expect(tester.takeException(), isNull);
      expect(app.where, opened);
      expect(find.byType(NotePage), findsOneWidget);
      // Going back leaves the list the note was opened from.
      app.router.pop();
      await tester.pumpAndSettle();
      expect(app.where, '/notes');
      expect(find.byType(NoteListPane), findsOneWidget);
      expect(tester.takeException(), isNull);

      await resizeToViewport(tester, KansoViewport.iPadLandscape);
      expect(find.byType(NoteListPane), findsOneWidget);
      expect(find.byType(NoNoteSelectedPane), findsOneWidget);
    });

    testWidgets('turns an iPad round with the note and the list in place', (
      tester,
    ) async {
      final app = await _open(
        tester,
        KansoViewport.iPadPortrait,
        at: _syncNote,
      );
      for (final viewport in <KansoViewport>[
        KansoViewport.iPadLandscape,
        KansoViewport.iPadPortrait,
        KansoViewport.iPad13Portrait,
        KansoViewport.androidTablet,
        KansoViewport.androidTabletPortrait,
        KansoViewport.iPadPortrait,
      ]) {
        await resizeToViewport(tester, viewport);
        expect(tester.takeException(), isNull, reason: viewport.name);
        _expectLayout(viewport, app);
        expect(app.where, _syncNote, reason: viewport.name);
        expect(find.byType(NoteListPane), findsOneWidget);
        expect(find.byType(NotePage), findsOneWidget);
      }
    });

    testWidgets('moves the library between a pane and a drawer', (
      tester,
    ) async {
      await _open(tester, KansoViewport.iPadPortrait);
      // Two panes: the menu opens the drawer, which has the library.
      await tester.tap(find.bySemanticsLabel('Library'));
      await tester.pumpAndSettle();
      expect(find.byType(LibraryNavigation), findsOneWidget);
      expect(
        tester.getSize(find.byType(LibraryNavigation)).width,
        closeTo(280, 1),
      );

      // Turned sideways while it is open: the library is a pane, and no
      // veil is left over the list.
      await resizeToViewport(tester, KansoViewport.iPadLandscape);
      expect(tester.takeException(), isNull);
      expect(find.byType(LibraryNavigation), findsOneWidget);
      expect(find.bySemanticsLabel('Library'), findsNothing);
      await tester.tap(find.text('Sourdough starter'));
      await tester.pumpAndSettle();
      expect(find.byType(NotePage), findsOneWidget);
    });

    testWidgets('has the library in a drawer of one pane at Slide Over', (
      tester,
    ) async {
      await _open(tester, KansoViewport.slideOver);
      await tester.tap(find.bySemanticsLabel('Library'));
      await tester.pumpAndSettle();
      expect(find.byType(LibraryNavigation), findsOneWidget);
      final drawer = tester.getSize(find.byType(LibraryNavigation));
      expect(drawer.width, lessThan(KansoViewport.slideOver.size.width));
      expect(tester.takeException(), isNull);
    });

    testWidgets('keeps a search and its results through the sizes', (
      tester,
    ) async {
      final app = await _open(
        tester,
        KansoViewport.iPadLandscape,
        at: '/search?q=sync',
      );
      for (final viewport in walk) {
        await resizeToViewport(tester, viewport);
        expect(tester.takeException(), isNull, reason: viewport.name);
        expect(app.where, '/search?q=sync', reason: viewport.name);
        expect(find.byType(SearchPane), findsOneWidget);
        expect(find.text('5 results'), findsOneWidget, reason: viewport.name);
      }
    });

    testWidgets('keeps the settings sheet in every window of a tablet', (
      tester,
    ) async {
      final app = await _open(tester, KansoViewport.iPadLandscape);
      unawaited(app.router.push<void>(KansoRoutes.settings));
      await tester.pumpAndSettle();
      for (final viewport in walk) {
        await resizeToViewport(tester, viewport);
        expect(tester.takeException(), isNull, reason: viewport.name);
        expect(find.byType(KansoSheet), findsOneWidget, reason: viewport.name);
        expect(find.byType(SettingsPage), findsOneWidget);
        final sheet = tester.getRect(find.byType(KansoSheet));
        final window = Offset.zero & viewport.size;
        expect(
          sheet.left >= 0 &&
              sheet.top >= 0 &&
              sheet.right <= window.right &&
              sheet.bottom <= window.bottom,
          isTrue,
          reason: '${viewport.name}: the sheet $sheet is in the window',
        );
        expect(sheet.width, lessThanOrEqualTo(viewport.size.width));
      }
    });

    testWidgets('says "device", not "phone", on a tablet', (tester) async {
      final library = FixtureLibraryRepository()
        ..noteFailure = const ApplicationError.storage(message: 'Disk error.');
      await _open(
        tester,
        KansoViewport.iPadLandscape,
        at: _syncNote,
        library: library,
      );
      expect(find.textContaining('on this device'), findsOneWidget);
      expect(find.textContaining('phone'), findsNothing);
    });
  });
}
