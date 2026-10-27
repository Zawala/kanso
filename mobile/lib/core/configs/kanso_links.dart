// Kanso — kanso_links.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// The addresses outside the app that it points to.
abstract final class KansoLinks {
  /// Kanso's website.
  static final Uri website = Uri.parse('https://kansoapp.app');

  /// What changed in each version of Kanso, on the website.
  static final Uri releaseNotes = Uri.parse('https://kansoapp.app/releases');

  /// Help with using Kanso, on the website.
  static final Uri help = Uri.parse('https://kansoapp.app/help');
}
