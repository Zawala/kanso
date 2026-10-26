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
  Future<Either<ApplicationError, List<NoteRow>>> notes(NoteQuery query) =>
      attempt(() => _library.notes(query: query));

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
  Future<Either<ApplicationError, List<TagRow>>> tags() =>
      attempt(_library.tags);

  @override
  Future<Either<ApplicationError, List<Hit>>> search(
    String text, {
    int limit = 50,
  }) => attempt(() => _library.search(text: text, limit: limit));

  @override
  Future<Either<ApplicationError, List<Property>>> properties(String id) =>
      attempt(() => _library.properties(id: id));

  @override
  Future<Either<ApplicationError, Table>> queryView(
    String source, {
    String? selfId,
  }) => attempt(() => _library.queryView(source: source, selfId: selfId));

  @override
  Stream<List<String>> changes() => cancelableStream(_library.changes());
}
