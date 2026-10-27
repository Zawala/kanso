// Kanso — fixture_library_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/repos/abs/library_repository.dart';

import 'library_fixtures.dart';

/// A library of the mock content, for widget and golden tests: every call
/// answers from a field a test can change, and records what it was asked.
class FixtureLibraryRepository implements LibraryRepository {
  FixtureLibraryRepository() {
    for (final row in mockRows) {
      if (row.id == MockIds.syncDesign) {
        notes[row.id] = mockOpenNote;
      } else if (row.locked) {
        notes[row.id] = mockLockedNote;
      }
    }
  }

  List<NotebookRow> notebookRows = mockNotebooks;
  List<TagRow> tagRows = mockTags;
  LibraryOverview overviewValue = mockOverview;

  /// What `list` answers; a failure shows the list's error state.
  Either<ApplicationError, NoteList> listResult =
      Right<ApplicationError, NoteList>(mockNoteList);

  /// The notes `note` knows; others are not found.
  final Map<String, NoteView> notes = <String, NoteView>{};

  /// What `search` answers.
  Either<ApplicationError, List<Hit>> searchResult =
      Right<ApplicationError, List<Hit>>(mockHits);

  /// What `change` answers.
  Either<ApplicationError, Changed> changeResult =
      const Right<ApplicationError, Changed>(
        Changed(count: 1, undo: 1, message: 'Pinned 1 note.'),
      );

  /// What `undo` answers.
  Either<ApplicationError, int> undoResult = const Right<ApplicationError, int>(
    1,
  );

  /// The tickets `undo` was asked to take back.
  final List<int> undone = <int>[];

  /// When set, `list` waits for it: a list that is still loading.
  Completer<void>? listGate;

  /// When set, `search` waits for it: a search that is still running.
  Completer<void>? searchGate;

  /// When set, `note` waits for it: a note that is still loading.
  Completer<void>? noteGate;

  /// When set, `note` fails with it for every note.
  ApplicationError? noteFailure;

  /// When set, `notebooks` waits for it: a library that is still loading.
  Completer<void>? notebooksGate;

  /// When set, `notebooks` fails with it: a library that cannot be read.
  ApplicationError? notebooksFailure;

  final List<ListQuery> listQueries = <ListQuery>[];
  final List<(List<String>, NoteChange)> changesMade =
      <(List<String>, NoteChange)>[];
  final List<(String, SearchScope)> searches = <(String, SearchScope)>[];
  final StreamController<List<String>> changeStream =
      StreamController<List<String>>.broadcast();

  Future<void> close() => changeStream.close();

  @override
  Future<Either<ApplicationError, NoteList>> list(ListQuery query) async {
    listQueries.add(query);
    await listGate?.future;
    return listResult;
  }

  @override
  Future<Either<ApplicationError, LibraryOverview>> overview() async =>
      Right<ApplicationError, LibraryOverview>(overviewValue);

  @override
  Future<Either<ApplicationError, int>> undo(int ticket) async {
    undone.add(ticket);
    return undoResult;
  }

  @override
  Future<Either<ApplicationError, Changed>> change(
    List<String> ids,
    NoteChange change,
  ) async {
    changesMade.add((ids, change));
    return changeResult;
  }

  @override
  Future<Either<ApplicationError, NoteView>> note(String id) async {
    await noteGate?.future;
    final failure = noteFailure;
    if (failure != null) {
      return Left<ApplicationError, NoteView>(failure);
    }
    final note = notes[id];
    return note == null
        ? const Left<ApplicationError, NoteView>(
            ApplicationError.notFound(message: 'Not found.'),
          )
        : Right<ApplicationError, NoteView>(note);
  }

  @override
  Future<Either<ApplicationError, List<NotebookRow>>> notebooks() async {
    await notebooksGate?.future;
    final failure = notebooksFailure;
    return failure == null
        ? Right<ApplicationError, List<NotebookRow>>(notebookRows)
        : Left<ApplicationError, List<NotebookRow>>(failure);
  }

  @override
  Future<Either<ApplicationError, List<TagRow>>> tags() async =>
      Right<ApplicationError, List<TagRow>>(tagRows);

  @override
  Future<Either<ApplicationError, List<Hit>>> search(
    String text, {
    SearchScope scope = const SearchScope.all(),
    int limit = 50,
  }) async {
    searches.add((text, scope));
    await searchGate?.future;
    return searchResult;
  }

  @override
  Stream<List<String>> changes() => changeStream.stream;

  @override
  Future<Either<ApplicationError, Saved>> save({
    required String id,
    required String baseRev,
    required String body,
  }) => throw UnimplementedError();

  @override
  Future<Either<ApplicationError, Saved>> create(NewNote note) =>
      throw UnimplementedError();

  @override
  Future<Either<ApplicationError, Unit>> trash(String id) =>
      throw UnimplementedError();

  @override
  Future<Either<ApplicationError, NotebookRow>> createNotebook(
    String name, {
    String? parentId,
  }) => throw UnimplementedError();

  @override
  Future<Either<ApplicationError, List<Property>>> properties(String id) =>
      throw UnimplementedError();

  @override
  Future<Either<ApplicationError, Table>> queryView(
    String source, {
    String? selfId,
  }) => throw UnimplementedError();
}
