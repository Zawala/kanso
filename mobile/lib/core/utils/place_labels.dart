// Kanso — place_labels.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../gen/l10n/app_localizations.dart';
import '../bridge/api/types.dart';
import '../state/library_index/library_index_bloc.dart';

/// The name of a library place, in the app's language. A notebook and a tag
/// have their own names, which the library knows; here they are only the
/// word for what they are.
String placeLabel(AppLocalizations l10n, ListPlace place) => switch (place) {
  ListPlace_All() => l10n.libraryAllNotes,
  ListPlace_Pinned() => l10n.libraryPinned,
  ListPlace_Recent() => l10n.libraryRecent,
  ListPlace_Trash() => l10n.libraryTrash,
  ListPlace_Notebook() => l10n.libraryNotebooks,
  ListPlace_Tag() => l10n.libraryTags,
};

/// What a list of [place] is titled: the name of its notebook, or `#tag`,
/// as [index] knows it; otherwise the word for what the place is.
String placeTitle(
  AppLocalizations l10n,
  ListPlace place,
  LibraryIndexState index,
) {
  return switch (place) {
    ListPlace_Notebook(:final id) =>
      index.notebookName(id) ?? placeLabel(l10n, place),
    ListPlace_Tag(:final id) =>
      index.tag(id)?.name == null
          ? placeLabel(l10n, place)
          : '#${index.tag(id)!.name}',
    _ => placeLabel(l10n, place),
  };
}
