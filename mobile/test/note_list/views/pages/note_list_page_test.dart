// Kanso — note_list_page_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/note_list/views/pages/note_list_page.dart';
import 'package:kanso/note_list/views/widgets/note_list_pane.dart';

import '../../../helpers/browse_blocs.dart';
import '../../../helpers/library_fixtures.dart';
import '../../../helpers/pump_routed.dart';
import '../../../helpers/register_page_blocs.dart';

void main() {
  group('NoteListPage', () {
    testWidgets('reads the notes of its place and shows them', (tester) async {
      final blocs = await BrowseBlocs.start();
      registerPageBlocs(blocs.library, null, blocs);
      await atMockNow(
        () => pumpRouted(
          tester,
          blocs.provide(
            const NoteListPage(place: ListPlace.pinned()),
          ),
        ),
      );
      expect(find.byType(NoteListPane), findsOneWidget);
      expect(blocs.library.listQueries.single.place, const ListPlace.pinned());
      expect(find.text('Weekly review'), findsOneWidget);
    });

    testWidgets('is a new list for a new place', (tester) async {
      final blocs = await BrowseBlocs.start();
      registerPageBlocs(blocs.library, null, blocs);
      final place = ValueNotifier<ListPlace>(const ListPlace.pinned());
      addTearDown(place.dispose);
      await atMockNow(
        () => pumpRouted(
          tester,
          blocs.provide(
            ValueListenableBuilder<ListPlace>(
              valueListenable: place,
              builder: (context, value, _) => NoteListPage(place: value),
            ),
          ),
        ),
      );
      place.value = const ListPlace.notebook(id: MockIds.work);
      await tester.pump();
      await tester.pump();
      expect(
        blocs.library.listQueries.map((query) => query.place),
        <ListPlace>[
          const ListPlace.pinned(),
          const ListPlace.notebook(id: MockIds.work),
        ],
      );
    });
  });
}
