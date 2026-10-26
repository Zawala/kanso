// Kanso — browse_blocs.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:clock/clock.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart' show addTearDown;
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/library_index_status.dart';
import 'package:kanso/core/models/repos/abs/haptics_repository.dart';
import 'package:kanso/core/state/library_index/library_index_bloc.dart';
import 'package:kanso/core/state/navigation_history/navigation_history_bloc.dart';
import 'package:kanso/core/state/sync_status/sync_status_bloc.dart';
import 'package:kanso/note_list/models/data/note_list_status.dart';
import 'package:kanso/note_list/state/note_list/note_list_bloc.dart';

import 'close_bloc_later.dart';
import 'fixture_library_repository.dart';
import 'fixture_sync_repository.dart';
import 'library_fixtures.dart';
import 'seeded_appearance_bloc.dart';

/// Runs [body] with the clock at [mockNow], the time of the mock content.
Future<T> atMockNow<T>(Future<T> Function() body) =>
    withClock(Clock.fixed(mockNow), body);

/// A haptics repository that remembers what it was asked for.
class RecordingHaptics implements HapticsRepository {
  int selections = 0;
  int impacts = 0;

  @override
  Future<void> selection() async => selections++;

  @override
  Future<void> impact() async => impacts++;
}

/// The BLoCs the browsing screens read, over the mock library: the shared
/// ones, and a list of a place when a test makes one.
class BrowseBlocs {
  BrowseBlocs._(
    this.library,
    this.index,
    this.history,
    this.sync,
    this.syncRepository,
  );

  /// The mock library (see [FixtureLibraryRepository]).
  final FixtureLibraryRepository library;
  final LibraryIndexBloc index;
  final NavigationHistoryBloc history;
  final SyncStatusBloc sync;
  final FixtureSyncRepository syncRepository;
  final RecordingHaptics haptics = RecordingHaptics();

  /// Makes them, with the library read, and closes them when the test ends.
  static Future<BrowseBlocs> start([
    FixtureLibraryRepository? library,
    FixtureSyncRepository? syncRepository,
  ]) async {
    final fixture = library ?? FixtureLibraryRepository();
    addTearDown(() => unawaited(fixture.close()));
    final index = LibraryIndexBloc(fixture)
      ..add(const LibraryIndexEvent.started());
    final history = NavigationHistoryBloc(fixture);
    closeBlocLater(index);
    closeBlocLater(history);
    final account = syncRepository ?? FixtureSyncRepository();
    final sync = SyncStatusBloc(account)..add(const SyncStatusEvent.started());
    closeBlocLater(sync);
    await index.stream.firstWhere(
      (state) => state.status != LibraryIndexStatus.loading,
    );
    if (sync.state.status == null) {
      await sync.stream.firstWhere((state) => state.status != null);
    }
    return BrowseBlocs._(fixture, index, history, sync, account);
  }

  /// A list of [place], read; or only made when [read] is false, so that it
  /// is still loading.
  Future<NoteListBloc> listOf(
    ListPlace place, {
    bool read = true,
  }) async {
    final bloc = NoteListBloc(
      library,
      ForgetfulPreferencesRepository(),
      haptics,
    );
    closeBlocLater(bloc);
    if (read) {
      bloc.add(NoteListEvent.started(place));
      await bloc.stream.firstWhere(
        (state) => state.status != NoteListStatus.loading,
      );
    }
    return bloc;
  }

  /// [child] with these BLoCs above it, and [list] if given.
  Widget provide(Widget child, {NoteListBloc? list}) {
    return MultiBlocProvider(
      providers: <BlocProvider<dynamic>>[
        BlocProvider<LibraryIndexBloc>.value(value: index),
        BlocProvider<NavigationHistoryBloc>.value(value: history),
        BlocProvider<SyncStatusBloc>.value(value: sync),
        if (list != null) BlocProvider<NoteListBloc>.value(value: list),
      ],
      child: child,
    );
  }
}
