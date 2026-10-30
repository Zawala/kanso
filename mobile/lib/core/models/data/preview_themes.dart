// Kanso — preview_themes.dart
// Created 2026-10-30.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// The themes a preview is drawn in: the interface, syntax and preview
/// themes in force, and whether they are the dark slot's. The core fits the
/// three together, as the app's own colours are.
typedef PreviewThemes = ({String ui, String syntax, String preview, bool dark});
