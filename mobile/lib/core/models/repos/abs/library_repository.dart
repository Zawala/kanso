// Kanso — library_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../bridge/api/types.dart';
import '../../data/application_error.dart';

/// The notes of the open library: what the Rust core knows about them.
///
/// A locked note's text, snippet, tags, status and properties never come back
/// while it is closed; its title and its lock do (they show in lists). A tag
/// place, a status narrowing and a tag search never list one. Streams do not
/// fail: they end when the library closes.
abstract interface class LibraryRepository {
  /// The notes [query] picks, in its order, with the layout the desktop
  /// gives that list (pinned first; Recent by edit time; headers by date).
  Future<Either<ApplicationError, NoteList>> list(ListQuery query);

  /// How many notes each place of the library lists.
  Future<Either<ApplicationError, LibraryOverview>> overview();

  /// Makes [change] to every note in [ids] in one write; how many notes it
  /// wrote. Unlocked notes into a locked notebook, and notes out of one, are
  /// a [LockedError]: those go one at a time.
  Future<Either<ApplicationError, int>> change(
    List<String> ids,
    NoteChange change,
  );

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

  /// The notebooks in tree order, with their depth and role.
  Future<Either<ApplicationError, List<NotebookRow>>> notebooks();

  /// Makes a notebook, inside [parentId] if given.
  Future<Either<ApplicationError, NotebookRow>> createNotebook(
    String name, {
    String? parentId,
  });

  Future<Either<ApplicationError, List<TagRow>>> tags();

  /// Notes matching [text] in [scope], with where the words matched.
  Future<Either<ApplicationError, List<Hit>>> search(
    String text, {
    SearchScope scope = const SearchScope.all(),
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
