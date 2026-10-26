// Kanso — library_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../bridge/api/types.dart';
import '../../data/application_error.dart';

/// The notes of the open library: what the Rust core knows about them.
///
/// A locked note's text, snippet and properties never come back while it is
/// closed; its title does (it shows in lists). Streams do not fail: they end
/// when the library closes.
abstract interface class LibraryRepository {
  /// The notes [query] picks, in its order.
  Future<Either<ApplicationError, List<NoteRow>>> notes(NoteQuery query);

  /// One note; its text is null while it is locked and closed.
  Future<Either<ApplicationError, NoteView>> note(String id);

  /// Saves [body] as the text of the note [id], which was at [baseRev].
  Future<Either<ApplicationError, Saved>> save({
    required String id,
    required String baseRev,
    required String body,
  });

  /// Makes a note.
  Future<Either<ApplicationError, Saved>> create(NewNote note);

  /// Moves the note to the trash.
  Future<Either<ApplicationError, Unit>> trash(String id);

  Future<Either<ApplicationError, List<NotebookRow>>> notebooks();

  Future<Either<ApplicationError, List<TagRow>>> tags();

  /// Notes matching [text], with where the words matched.
  Future<Either<ApplicationError, List<Hit>>> search(
    String text, {
    int limit = 50,
  });

  /// A note's front-matter properties; none for a closed locked note.
  Future<Either<ApplicationError, List<Property>>> properties(String id);

  /// The table a query view's [source] describes.
  Future<Either<ApplicationError, Table>> queryView(
    String source, {
    String? selfId,
  });

  /// The IDs of what changed, in batches, as it is written: by this phone, a
  /// sync or the background.
  Stream<List<String>> changes();
}
