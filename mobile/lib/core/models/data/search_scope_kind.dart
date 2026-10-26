// Kanso — search_scope_kind.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Which part of a note a search looks in, as a search address names it
/// (`scope=titles`).
enum SearchScopeKind {
  /// Titles, text and tags.
  all,

  /// Titles only.
  titles,

  /// Tags only.
  tags,

  /// One notebook, the one the address names with `in`.
  notebook,
}
