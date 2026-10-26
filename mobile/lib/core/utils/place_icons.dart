// Kanso — place_icons.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../bridge/api/types.dart';
import '../configs/kanso_icons.dart';

/// The icon of a library place.
KansoIcons placeIcon(ListPlace place) => switch (place) {
  ListPlace_All() => KansoIcons.files,
  ListPlace_Pinned() => KansoIcons.pin,
  ListPlace_Recent() => KansoIcons.clock,
  ListPlace_Trash() => KansoIcons.trash,
  ListPlace_Notebook() => KansoIcons.notebook,
  ListPlace_Tag() => KansoIcons.hash,
};
