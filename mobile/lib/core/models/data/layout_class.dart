// Kanso — layout_class.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// How many panes the window has room for.
enum LayoutClass {
  /// One pane at a time: a phone, or a tablet window that is narrow.
  single,

  /// The note list beside the note; the library is a drawer.
  double,

  /// The library, the note list and the note side by side.
  triple,
}
