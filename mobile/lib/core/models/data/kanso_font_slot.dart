// Kanso — kanso_font_slot.dart
// Created 2026-10-30.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Where a font is used: the app's own labels and lists, writing a note,
/// reading one, or the code in it.
enum KansoFontSlot {
  /// Menus, lists, labels and buttons.
  interface,

  /// Writing in the editor.
  editor,

  /// Reading a note.
  preview,

  /// Code spans and code blocks.
  code,
}
