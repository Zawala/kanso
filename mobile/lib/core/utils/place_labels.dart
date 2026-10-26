// Kanso — place_labels.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../gen/l10n/app_localizations.dart';
import '../bridge/api/types.dart';

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
