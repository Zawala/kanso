// Kanso — note_list_refresh_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/cupertino.dart' show CupertinoSliverRefreshControl;
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/views/widgets/kanso_refresh_control.dart';
import 'package:kanso/core/views/widgets/kanso_refresh_indicator.dart';
import 'package:kanso/note_list/views/widgets/note_list_pane.dart';

import '../../../helpers/browse_blocs.dart';
import '../../../helpers/fixture_sync_repository.dart';
import '../../../helpers/pump_routed.dart';

void main() {
  group('pull to sync', () {
    testWidgets('has no control on a phone with no account', (tester) async {
      final blocs = await BrowseBlocs.start();
      final list = await blocs.listOf(const ListPlace.all());
      await atMockNow(
        () => pumpRouted(
          tester,
          blocs.provide(const NoteListPane(), list: list),
        ),
      );
      expect(
        find.byType(KansoRefreshControl, skipOffstage: false),
        findsNothing,
      );
      expect(
        find.byType(CupertinoSliverRefreshControl, skipOffstage: false),
        findsNothing,
      );
      // The list bounces, and nothing syncs.
      await tester.drag(find.text('Weekly review'), const Offset(0, 300));
      await tester.pumpAndSettle();
      expect(blocs.syncRepository.syncs, 0);
    });

    testWidgets('syncs when the list is pulled down, while signed in', (
      tester,
    ) async {
      final blocs = await BrowseBlocs.start(
        null,
        FixtureSyncRepository(current: signedInStatus),
      );
      final list = await blocs.listOf(const ListPlace.all());
      await atMockNow(
        () => pumpRouted(
          tester,
          blocs.provide(const NoteListPane(), list: list),
        ),
      );
      expect(
        find.byType(KansoRefreshControl, skipOffstage: false),
        findsOneWidget,
      );
      await tester.drag(find.text('Weekly review'), const Offset(0, 300));
      await tester.pumpAndSettle();
      expect(blocs.syncRepository.syncs, 1);
    });

    testWidgets('holds the list down, spinning, until the sync ends', (
      tester,
    ) async {
      final account = FixtureSyncRepository(current: signedInStatus)
        ..gate = Completer<void>();
      final blocs = await BrowseBlocs.start(null, account);
      final list = await blocs.listOf(const ListPlace.all());
      await atMockNow(
        () => pumpRouted(
          tester,
          blocs.provide(const NoteListPane(), list: list),
        ),
      );
      final restingTop = tester.getTopLeft(find.text('Weekly review')).dy;
      await tester.drag(find.text('Weekly review'), const Offset(0, 300));
      await tester.pump(const Duration(milliseconds: 600));
      expect(account.syncs, 1);
      expect(find.byType(KansoRefreshIndicator), findsOneWidget);
      expect(
        tester.getTopLeft(find.text('Weekly review')).dy,
        greaterThan(restingTop),
      );

      account.gate!.complete();
      await tester.pumpAndSettle();
      expect(
        tester.getTopLeft(find.text('Weekly review')).dy,
        moreOrLessEquals(restingTop, epsilon: 1),
      );
    });

    testWidgets('a short pull does not sync', (tester) async {
      final blocs = await BrowseBlocs.start(
        null,
        FixtureSyncRepository(current: signedInStatus),
      );
      final list = await blocs.listOf(const ListPlace.all());
      await atMockNow(
        () => pumpRouted(
          tester,
          blocs.provide(const NoteListPane(), list: list),
        ),
      );
      await tester.drag(find.text('Weekly review'), const Offset(0, 40));
      await tester.pumpAndSettle();
      expect(blocs.syncRepository.syncs, 0);
    });
  });
}
