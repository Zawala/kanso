// Kanso — kanso_drawer_controller.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// What a part of the page can do to the drawer it is in.
abstract interface class KansoDrawerController {
  /// Whether the drawer is open or on its way open.
  bool get isOpen;

  /// Slides the drawer out.
  void open();

  /// Slides the drawer away.
  void close();

  /// Opens the drawer if it is closed and closes it if it is open.
  void toggle();
}
