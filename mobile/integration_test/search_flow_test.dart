// Kanso — search_flow_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:kanso/core/views/widgets/kanso_search_input.dart';
import 'package:kanso/search/views/widgets/highlighted_text.dart';
import 'package:kanso/search/views/widgets/search_result_tile.dart';

import 'helpers/flow_driver.dart';
import 'helpers/seed_library.dart';

/// Searching: type a word and see what matched highlighted, narrow to
/// titles, open a hit.
///
/// Run it with `flutter test integration_test/search_flow_test.dart -d <device>`.
void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  // A title with a match is drawn in pieces, so the tile is found by the
  // note it shows.
  Finder tile(String title) => find.byWidgetPredicate(
    (widget) => widget is SearchResultTile && widget.hit.row.title == title,
  );

  testWidgets('search finds notes, narrows to titles and opens a hit', (
    tester,
  ) async {
    final semantics = tester.ensureSemantics();
    await startApp(tester);
    await seedLibrary();
    await pumpUntil(tester, inList('Weekly review'));

    await openSearch(tester);
    await tester.enterText(find.byType(KansoSearchInput), 'garden');
    await pumpUntil(tester, tile('Garden beds'));

    // What matched is on a wash of its own, in a title and in the text.
    expect(tile('Seed catalogue'), findsOneWidget);
    final washes = find.descendant(
      of: find.byType(SearchResultTile),
      matching: find.descendant(
        of: find.byType(HighlightedText),
        matching: find.byWidgetPredicate(
          (widget) =>
              widget is DecoratedBox &&
              widget.decoration is BoxDecoration &&
              (widget.decoration as BoxDecoration).borderRadius ==
                  BorderRadius.circular(HighlightedText.washRadius),
        ),
      ),
    );
    expect(washes, findsWidgets);
    expect(find.textContaining(SeededLibrary.secretBody), findsNothing);

    // Titles only: the note that has the word in its text drops out.
    await tester.tap(find.text('Titles'));
    await pumpUntilGone(tester, tile('Seed catalogue'));
    expect(tile('Garden beds'), findsOneWidget);

    // A hit opens its note.
    await tester.tap(tile('Garden beds'));
    await pumpUntil(tester, find.textContaining('Three raised beds'));
    expect(find.text('Garden'), findsWidgets);
    semantics.dispose();
  });
}
