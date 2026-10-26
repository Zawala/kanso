// Kanso — open_library.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:kanso/core/bridge/api/library.dart';
import 'package:kanso/core/bridge/api/types.dart';

/// How a suite gets a library of its own, empty: on this computer a folder in
/// the temporary directory, on a device one in the app's own storage.
typedef OpenLibrary = Future<KansoLibrary> Function();

/// Every note outside the trash, the newest edit first.
const ListQuery everyNote = ListQuery(
  place: ListPlace.all(),
  pinnedOnly: false,
  sort: SortKey.updated,
  descending: true,
);

/// The notes in the trash.
const ListQuery trashedNotes = ListQuery(
  place: ListPlace.trash(),
  pinnedOnly: false,
  sort: SortKey.updated,
  descending: true,
);

/// A lock password (at least 8 characters).
const String lockPassword = 'open sesame';

/// Makes a note of the given [title] and [body] and returns its ID.
Future<String> makeNote(
  KansoLibrary library,
  String title,
  String body,
) async {
  final saved = await library.create(
    note: NewNote(title: title, body: body),
  );
  return saved.id;
}
