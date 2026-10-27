// Kanso — rust_library_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../bridge/api/library.dart';
import '../../../bridge/api/types.dart';
import '../../../utils/cancelable_stream.dart';
import '../../../utils/kanso_error_mapping.dart';
import '../../data/application_error.dart';
import '../abs/library_repository.dart';

/// [LibraryRepository] over the Rust core's library.
@LazySingleton(as: LibraryRepository)
class RustLibraryRepository implements LibraryRepository {
  RustLibraryRepository(this._library);

  final KansoLibrary _library;

  @override
  Future<Either<ApplicationError, NoteList>> list(ListQuery query) =>
      attempt(() => _library.list(query: query));

  @override
  Future<Either<ApplicationError, LibraryOverview>> overview() =>
      attempt(_library.overview);

  @override
  Future<Either<ApplicationError, Changed>> change(
    List<String> ids,
    NoteChange change,
  ) => attempt(() => _library.change(ids: ids, change: change));

  @override
  Future<Either<ApplicationError, int>> undo(int ticket) =>
      attempt(() => _library.undo(ticket: ticket));

  @override
  Future<Either<ApplicationError, NoteView>> note(String id) =>
      attempt(() => _library.note(id: id));

  @override
  Future<Either<ApplicationError, Saved>> save({
    required String id,
    required String baseRev,
    required String body,
  }) => attempt(() => _library.save(id: id, baseRev: baseRev, body: body));

  @override
  Future<Either<ApplicationError, Saved>> create(NewNote note) =>
      attempt(() => _library.create(note: note));

  @override
  Future<Either<ApplicationError, Unit>> trash(String id) => attempt(() async {
    await _library.trash(id: id);
    return unit;
  });

  @override
  Future<Either<ApplicationError, List<NotebookRow>>> notebooks() =>
      attempt(_library.notebooks);

  @override
  Future<Either<ApplicationError, NotebookRow>> createNotebook(
    String name, {
    String? parentId,
  }) => attempt(
    () => _library.createNotebook(name: name, parentId: parentId),
  );

  @override
  Future<Either<ApplicationError, List<TagRow>>> tags() =>
      attempt(_library.tags);

  @override
  Future<Either<ApplicationError, List<Hit>>> search(
    String text, {
    SearchScope scope = const SearchScope.all(),
    int limit = 50,
  }) => attempt(
    () => _library.search(text: text, scope: scope, limit: limit),
  );

  @override
  Future<Either<ApplicationError, List<Property>>> properties(String id) =>
      attempt(() => _library.properties(id: id));

  @override
  Future<Either<ApplicationError, Table>> queryView(
    String source, {
    String? selfId,
  }) => attempt(() => _library.queryView(source: source, selfId: selfId));

  @override
  Future<Either<ApplicationError, TaskBoard>> tasks(TaskListQuery query) =>
      attempt(() => _library.tasks(query: query));

  @override
  Future<Either<ApplicationError, int>> overdueTasks() =>
      attempt(_library.overdueTasks);

  @override
  Future<Either<ApplicationError, Ticked>> setTask({
    required String noteId,
    required int line,
    required String text,
    required bool checked,
  }) => attempt(
    () => _library.setTask(
      noteId: noteId,
      line: line,
      text: text,
      checked: checked,
    ),
  );

  @override
  Stream<List<String>> changes() => cancelableStream(_library.changes());
}
