// Kanso — search_pane_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/browse_location.dart';
import 'package:kanso/core/models/data/search_scope_kind.dart';
import 'package:kanso/core/views/widgets/kanso_chip.dart';
import 'package:kanso/core/views/widgets/kanso_search_input.dart';
import 'package:kanso/core/views/widgets/kanso_skeleton.dart';
import 'package:kanso/injection.dart';
import 'package:kanso/library/views/widgets/library_pane.dart';
import 'package:kanso/metrics/models/repos/abs/usage_metrics_repository.dart';
import 'package:kanso/note/views/widgets/note_pane.dart';
import 'package:kanso/search/views/widgets/highlighted_text.dart';
import 'package:kanso/search/views/widgets/search_pane.dart';
import 'package:kanso/search/views/widgets/search_result_tile.dart';
import 'package:kanso/shell/views/widgets/list_note_layout.dart';
import 'package:kanso/shell/views/widgets/tablet_layout.dart';

import '../../../helpers/browse_blocs.dart';
import '../../../helpers/fake_usage_metrics.dart';
import '../../../helpers/fixture_library_repository.dart';
import '../../../helpers/golden.dart';
import '../../../helpers/library_fixtures.dart';
import '../../../helpers/pump_routed.dart';

Widget _scaffold(Widget pane) {
  return Scaffold(resizeToAvoidBottomInset: false, body: pane);
}

Iterable<BoxDecoration> _washes(WidgetTester tester) {
  return tester
      .widgetList<DecoratedBox>(
        find.descendant(
          of: find.byType(HighlightedText),
          matching: find.byType(DecoratedBox),
        ),
      )
      .map((box) => box.decoration)
      .whereType<BoxDecoration>();
}

/// The chips and which of them is chosen.
Map<String, bool> _chips(WidgetTester tester) => <String, bool>{
  for (final chip in tester.widgetList<KansoChip>(find.byType(KansoChip)))
    chip.label: chip.selected,
};

void main() {
  group('SearchPane', () {
    testWidgets('asks for words while there are none', (tester) async {
      final blocs = await BrowseBlocs.start();
      final search = await blocs.searchOf('');
      await pumpRouted(
        tester,
        blocs.provide(const SearchPane(), search: search),
      );
      expect(find.text('Search your notes'), findsOneWidget);
      expect(find.byType(KansoSearchInput), findsOneWidget);
      expect(_chips(tester), <String, bool>{
        'All': true,
        'Titles': false,
        'Tags': false,
      });
      expect(find.text('Cancel'), findsOneWidget);
      // The field has the cursor.
      expect(
        tester
            .widget<EditableText>(find.byType(EditableText))
            .focusNode
            .hasFocus,
        isTrue,
      );
    });

    testWidgets('searches for what is typed, once the person pauses', (
      tester,
    ) async {
      final blocs = await BrowseBlocs.start();
      final search = await blocs.searchOf('');
      await atMockNow(
        () => pumpRouted(
          tester,
          blocs.provide(const SearchPane(), search: search),
        ),
      );
      await tester.enterText(find.byType(EditableText), 'syn');
      await tester.enterText(find.byType(EditableText), 'sync');
      await tester.pump(const Duration(milliseconds: 100));
      expect(blocs.library.searches, isEmpty);
      await tester.pump(const Duration(milliseconds: 100));
      await tester.pump();
      await tester.pump();
      expect(blocs.library.searches.single.$1, 'sync');
      expect(find.text('5 results'), findsOneWidget);
      expect(
        find.textContaining('design notes', findRichText: true),
        findsOneWidget,
      );
      expect(find.text('Projects / Kanso'), findsOneWidget);
      // What matched is on a wash, in titles and in the text around it.
      expect(_washes(tester), isNotEmpty);
      expect(
        find.descendant(
          of: find.byType(HighlightedText),
          matching: find.text('Sync'),
        ),
        findsWidgets,
      );
    });

    testWidgets('starts with the words it was opened with, searched', (
      tester,
    ) async {
      final blocs = await BrowseBlocs.start();
      final search = await blocs.searchOf('sync');
      await atMockNow(
        () => pumpRouted(
          tester,
          blocs.provide(const SearchPane(initialQuery: 'sync'), search: search),
        ),
      );
      expect(
        tester.widget<EditableText>(find.byType(EditableText)).controller.text,
        'sync',
      );
      expect(find.text('5 results'), findsOneWidget);
    });

    testWidgets('a chip chooses where to look, and looks at once', (
      tester,
    ) async {
      final blocs = await BrowseBlocs.start();
      final search = await blocs.searchOf('sync');
      await atMockNow(
        () => pumpRouted(
          tester,
          blocs.provide(const SearchPane(initialQuery: 'sync'), search: search),
        ),
      );
      await tester.tap(find.text('Titles'));
      await tester.pump();
      await tester.pump();
      expect(blocs.library.searches.last.$2, const SearchScope.titles());
      expect(_chips(tester)['Titles'], isTrue);
      expect(_chips(tester)['All'], isFalse);
      await tester.tap(find.text('Tags'));
      await tester.pump();
      await tester.pump();
      expect(blocs.library.searches.last.$2, const SearchScope.tags());
    });

    testWidgets(
      'has the notebook chip only for a search opened in a notebook',
      (
        tester,
      ) async {
        final blocs = await BrowseBlocs.start();
        final search = await blocs.searchOf(
          'sync',
          scope: SearchScopeKind.notebook,
          notebookId: MockIds.work,
        );
        await atMockNow(
          () => pumpRouted(
            tester,
            blocs.provide(
              const SearchPane(initialQuery: 'sync'),
              search: search,
            ),
          ),
        );
        expect(_chips(tester), <String, bool>{
          'All': false,
          'Titles': false,
          'Tags': false,
          'This notebook': true,
        });
        expect(
          blocs.library.searches.single.$2,
          const SearchScope.notebook(id: MockIds.work),
        );
      },
    );

    testWidgets('a result opens its note in the search', (tester) async {
      final metrics = FakeUsageMetrics();
      getIt.registerSingleton<UsageMetricsRepository>(metrics);
      addTearDown(() => getIt.unregister<UsageMetricsRepository>());
      final blocs = await BrowseBlocs.start();
      final search = await blocs.searchOf('sync');
      final router = await atMockNow(
        () => pumpRouted(
          tester,
          blocs.provide(const SearchPane(initialQuery: 'sync'), search: search),
        ),
      );
      await tester.tap(find.text('Weekly review'));
      await tester.pumpAndSettle();
      expect(
        whereIs(router),
        const BrowseLocation.search(
          query: 'sync',
        ).withNote(MockIds.weeklyReview).location,
      );
      await tester.pump();
      expect(blocs.history.state.history.current, MockIds.weeklyReview);
      // Typing counted nothing; opening a result counts the search once.
      expect(metrics.recorded, [UsageFeature.search]);
    });

    testWidgets('marks the result whose note is open beside it', (
      tester,
    ) async {
      final blocs = await BrowseBlocs.start();
      final search = await blocs.searchOf('sync');
      await atMockNow(
        () => pumpRouted(
          tester,
          blocs.provide(
            const SearchPane(
              initialQuery: 'sync',
              openNoteId: MockIds.weeklyReview,
            ),
            search: search,
          ),
        ),
      );
      final open = <String>[
        for (final tile in tester.widgetList<SearchResultTile>(
          find.byType(SearchResultTile),
        ))
          if (tile.open) tile.hit.row.id,
      ];
      expect(open, <String>[MockIds.weeklyReview]);
    });

    testWidgets('a locked note is found by its title, and shows nothing more', (
      tester,
    ) async {
      final blocs = await BrowseBlocs.start();
      blocs.library.searchResult = Right<ApplicationError, List<Hit>>(<Hit>[
        mockHit(mockRows.firstWhere((row) => row.locked), 'journal'),
      ]);
      final search = await blocs.searchOf('journal');
      await pumpRouted(
        tester,
        blocs.provide(
          const SearchPane(initialQuery: 'journal'),
          search: search,
        ),
      );
      expect(find.text('Locked'), findsOneWidget);
      expect(find.textContaining(mockSecret, findRichText: true), findsNothing);
      // The word searched for is marked in the title.
      expect(_washes(tester), hasLength(1));
    });

    testWidgets('says nothing matched, in the words searched for', (
      tester,
    ) async {
      final blocs = await BrowseBlocs.start();
      blocs.library.searchResult = const Right<ApplicationError, List<Hit>>(
        <Hit>[],
      );
      final search = await blocs.searchOf('zzz');
      await pumpRouted(
        tester,
        blocs.provide(const SearchPane(initialQuery: 'zzz'), search: search),
      );
      expect(find.text('No notes match'), findsOneWidget);
      expect(find.textContaining('“zzz”'), findsOneWidget);
      expect(find.text('0 results'), findsOneWidget);
    });

    testWidgets('shows still bars while it searches, and says so', (
      tester,
    ) async {
      final handle = tester.ensureSemantics();
      final library = FixtureLibraryRepository()
        ..searchGate = Completer<void>();
      final blocs = await BrowseBlocs.start(library);
      final search = await blocs.searchOf('sync', wait: false);
      await pumpRouted(
        tester,
        blocs.provide(const SearchPane(initialQuery: 'sync'), search: search),
      );
      expect(find.bySemanticsLabel('Searching'), findsOneWidget);
      expect(find.byType(KansoSkeleton), findsWidgets);
      library.searchGate!.complete();
      await tester.pump();
      await tester.pump();
      expect(find.byType(KansoSkeleton), findsNothing);
      expect(find.text('5 results'), findsOneWidget);
      handle.dispose();
    });

    testWidgets('offers to try again after a failure', (tester) async {
      final blocs = await BrowseBlocs.start();
      blocs.library.searchResult = const Left<ApplicationError, List<Hit>>(
        ApplicationError.storage(message: 'Disk error.'),
      );
      final search = await blocs.searchOf('sync');
      await pumpRouted(
        tester,
        blocs.provide(const SearchPane(initialQuery: 'sync'), search: search),
      );
      expect(find.text("The search didn't run"), findsOneWidget);
      expect(find.textContaining('Disk error'), findsNothing);
      blocs.library.searchResult = Right<ApplicationError, List<Hit>>(mockHits);
      await tester.tap(find.text('Try again'));
      await tester.pump();
      await tester.pump();
      await tester.pumpAndSettle();
      expect(find.text('5 results'), findsOneWidget);
    });

    testWidgets('clearing the field goes back to asking for words', (
      tester,
    ) async {
      final blocs = await BrowseBlocs.start();
      final search = await blocs.searchOf('sync');
      final handle = tester.ensureSemantics();
      await atMockNow(
        () => pumpRouted(
          tester,
          blocs.provide(const SearchPane(initialQuery: 'sync'), search: search),
        ),
      );
      await tester.tap(find.bySemanticsLabel('Clear search'));
      await tester.pump();
      await tester.pump();
      expect(find.text('Search your notes'), findsOneWidget);
      expect(find.text('5 results'), findsNothing);
      handle.dispose();
    });

    testWidgets('follows the library as it changes', (tester) async {
      final blocs = await BrowseBlocs.start();
      final search = await blocs.searchOf('sync');
      await atMockNow(
        () => pumpRouted(
          tester,
          blocs.provide(const SearchPane(initialQuery: 'sync'), search: search),
        ),
      );
      blocs.library.searchResult = Right<ApplicationError, List<Hit>>(
        mockHits.sublist(2),
      );
      blocs.library.changeStream.add(<String>['note:new']);
      await tester.pump(const Duration(milliseconds: 150));
      await tester.pump();
      await tester.pumpAndSettle();
      expect(find.text('3 results'), findsOneWidget);
    });

    testWidgets('cancel leaves for All Notes when there is nowhere to go back '
        'to', (tester) async {
      final blocs = await BrowseBlocs.start();
      final search = await blocs.searchOf('');
      final router = await pumpRouted(
        tester,
        blocs.provide(const SearchPane(), search: search),
      );
      await tester.tap(find.text('Cancel'));
      await tester.pumpAndSettle();
      expect(whereIs(router), '/notes');
    });
  });

  group('SearchPane pictures', () {
    for (final brightness in Brightness.values) {
      final name = brightness.name;

      testWidgets('with results, $name', (tester) async {
        final blocs = await BrowseBlocs.start();
        final search = await blocs.searchOf('sync');
        await atMockNow(
          () => expectGoldenOnBothPlatforms(
            tester,
            blocs.provide(
              _scaffold(const SearchPane(initialQuery: 'sync')),
              search: search,
            ),
            name: 'phone_search_$name',
            size: phoneSize,
            brightness: brightness,
            // The person has tapped the field: it has the cursor.
            interact: (tester) async {
              await tester.tap(find.byType(EditableText));
            },
          ),
        );
      });

      testWidgets('with nothing typed, $name', (tester) async {
        final blocs = await BrowseBlocs.start();
        final search = await blocs.searchOf('');
        await expectGoldenOnBothPlatforms(
          tester,
          blocs.provide(_scaffold(const SearchPane()), search: search),
          name: 'phone_search_idle_$name',
          size: phoneSize,
          brightness: brightness,
        );
      });

      testWidgets('while it searches, $name', (tester) async {
        final library = FixtureLibraryRepository()
          ..searchGate = Completer<void>();
        final blocs = await BrowseBlocs.start(library);
        final search = await blocs.searchOf('sync', wait: false);
        await expectGoldenOnBothPlatforms(
          tester,
          blocs.provide(
            _scaffold(const SearchPane(initialQuery: 'sync')),
            search: search,
          ),
          name: 'phone_search_loading_$name',
          size: phoneSize,
          brightness: brightness,
        );
      });

      testWidgets('with no results, $name', (tester) async {
        final blocs = await BrowseBlocs.start();
        blocs.library.searchResult = const Right<ApplicationError, List<Hit>>(
          <Hit>[],
        );
        final search = await blocs.searchOf('zzz');
        await expectGoldenOnBothPlatforms(
          tester,
          blocs.provide(
            _scaffold(const SearchPane(initialQuery: 'zzz')),
            search: search,
          ),
          name: 'phone_search_empty_$name',
          size: phoneSize,
          brightness: brightness,
        );
      });

      testWidgets('after a failure, $name', (tester) async {
        final blocs = await BrowseBlocs.start();
        blocs.library.searchResult = const Left<ApplicationError, List<Hit>>(
          ApplicationError.storage(message: 'Disk error.'),
        );
        final search = await blocs.searchOf('sync');
        await expectGoldenOnBothPlatforms(
          tester,
          blocs.provide(
            _scaffold(const SearchPane(initialQuery: 'sync')),
            search: search,
          ),
          name: 'phone_search_failed_$name',
          size: phoneSize,
          brightness: brightness,
        );
      });

      testWidgets('finding a locked note, $name', (tester) async {
        final blocs = await BrowseBlocs.start();
        blocs.library.searchResult = Right<ApplicationError, List<Hit>>(
          <Hit>[
            mockHit(mockRows.firstWhere((row) => row.locked), 'journal'),
            mockHits.first,
          ],
        );
        final search = await blocs.searchOf('journal');
        await expectGoldenOnBothPlatforms(
          tester,
          blocs.provide(
            _scaffold(const SearchPane(initialQuery: 'journal')),
            search: search,
          ),
          name: 'phone_search_locked_$name',
          size: phoneSize,
          brightness: brightness,
        );
      });

      testWidgets('three panes, $name', (tester) async {
        final blocs = await BrowseBlocs.start();
        final search = await blocs.searchOf('sync');
        final note = await blocs.noteOf(
          MockIds.syncDesign,
          themeId: 'kanso-$name',
        );
        await atMockNow(
          () => expectGoldenOnBothPlatforms(
            tester,
            blocs.provide(
              _scaffold(
                const TabletLayout(
                  library: LibraryPane(
                    location: BrowseLocation.search(query: 'sync'),
                  ),
                  list: SearchPane(
                    initialQuery: 'sync',
                    openNoteId: MockIds.syncDesign,
                  ),
                  note: NotePane(inPane: true),
                ),
              ),
              search: search,
              note: note,
            ),
            name: 'tablet_search_$name',
            size: tabletLandscapeSize,
            brightness: brightness,
          ),
        );
      });

      testWidgets('two panes, $name', (tester) async {
        final blocs = await BrowseBlocs.start();
        final search = await blocs.searchOf('sync');
        final note = await blocs.noteOf(
          MockIds.syncDesign,
          themeId: 'kanso-$name',
        );
        await atMockNow(
          () => expectGoldenOnBothPlatforms(
            tester,
            blocs.provide(
              _scaffold(
                const ListNoteLayout(
                  list: SearchPane(
                    initialQuery: 'sync',
                    openNoteId: MockIds.syncDesign,
                  ),
                  note: NotePane(inPane: true),
                ),
              ),
              search: search,
              note: note,
            ),
            name: 'tablet_portrait_search_$name',
            size: tabletPortraitSize,
            brightness: brightness,
          ),
        );
      });
    }
  });
}
