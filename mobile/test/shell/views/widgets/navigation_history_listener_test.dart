// Kanso — navigation_history_listener_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/browse_location.dart';
import 'package:kanso/core/state/navigation_history/navigation_history_bloc.dart';
import 'package:kanso/shell/views/widgets/navigation_history_listener.dart';
import 'package:mocktail/mocktail.dart';

class _MockHistoryBloc
    extends MockBloc<NavigationHistoryEvent, NavigationHistoryState>
    implements NavigationHistoryBloc {}

Future<GoRouter> _pump(
  WidgetTester tester,
  _MockHistoryBloc bloc,
  BrowseLocation? location,
) async {
  final router = GoRouter(
    routes: <RouteBase>[
      GoRoute(
        path: '/:rest(.*)',
        builder: (context, state) => BlocProvider<NavigationHistoryBloc>.value(
          value: bloc,
          child: NavigationHistoryListener(
            location: location,
            child: const SizedBox.shrink(),
          ),
        ),
      ),
    ],
  );
  addTearDown(router.dispose);
  await tester.pumpWidget(MaterialApp.router(routerConfig: router));
  return router;
}

void main() {
  setUpAll(() {
    registerFallbackValue(const NavigationHistoryEvent.consumed());
  });

  group('NavigationHistoryListener', () {
    testWidgets('opens the note history landed on where the person is, '
        'and says it did', (tester) async {
      final bloc = _MockHistoryBloc();
      whenListen(
        bloc,
        Stream<NavigationHistoryState>.fromIterable(
          const <NavigationHistoryState>[
            NavigationHistoryState(pendingNoteId: 'n7'),
          ],
        ),
        initialState: const NavigationHistoryState(),
      );
      final router = await _pump(
        tester,
        bloc,
        const BrowseLocation.place(ListPlace.pinned()),
      );
      await tester.pumpAndSettle();
      expect(
        router.routerDelegate.currentConfiguration.uri.toString(),
        '/pinned/note/n7',
      );
      verify(
        () => bloc.add(const NavigationHistoryEvent.consumed()),
      ).called(1);
    });

    testWidgets('a search stays a search', (tester) async {
      final bloc = _MockHistoryBloc();
      whenListen(
        bloc,
        Stream<NavigationHistoryState>.fromIterable(
          const <NavigationHistoryState>[
            NavigationHistoryState(pendingNoteId: 'n7'),
          ],
        ),
        initialState: const NavigationHistoryState(),
      );
      final router = await _pump(
        tester,
        bloc,
        const BrowseLocation.search(query: 'tea'),
      );
      await tester.pumpAndSettle();
      expect(
        router.routerDelegate.currentConfiguration.uri.toString(),
        '/search/note/n7?q=tea',
      );
    });

    testWidgets('with no place it opens the note in All Notes', (
      tester,
    ) async {
      final bloc = _MockHistoryBloc();
      whenListen(
        bloc,
        Stream<NavigationHistoryState>.fromIterable(
          const <NavigationHistoryState>[
            NavigationHistoryState(pendingNoteId: 'n7'),
          ],
        ),
        initialState: const NavigationHistoryState(),
      );
      final router = await _pump(tester, bloc, null);
      await tester.pumpAndSettle();
      expect(
        router.routerDelegate.currentConfiguration.uri.toString(),
        '/notes/note/n7',
      );
    });

    testWidgets('does nothing when there is no note to open', (tester) async {
      final bloc = _MockHistoryBloc();
      whenListen(
        bloc,
        Stream<NavigationHistoryState>.fromIterable(
          const <NavigationHistoryState>[
            NavigationHistoryState(canGoBack: true),
          ],
        ),
        initialState: const NavigationHistoryState(),
      );
      final router = await _pump(tester, bloc, null);
      await tester.pumpAndSettle();
      expect(router.routerDelegate.currentConfiguration.uri.toString(), '/');
      verifyNever(() => bloc.add(any()));
    });
  });
}
