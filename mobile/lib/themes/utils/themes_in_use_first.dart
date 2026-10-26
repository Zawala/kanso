// Kanso — themes_in_use_first.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../core/models/data/theme_selection.dart';
import '../models/data/theme_preview.dart';

/// [themes] with the ones in use under [selection] first, so their outline
/// is the first thing seen, and the rest after them; each group keeps the
/// order it came in (the desktop's).
List<ThemePreview> themesInUseFirst(
  List<ThemePreview> themes,
  ThemeSelection selection,
) {
  return <ThemePreview>[
    ...themes.where((theme) => selection.isSelected(theme.id)),
    ...themes.where((theme) => !selection.isSelected(theme.id)),
  ];
}
