// Kanso — library_index_status.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Whether the library's notebooks, tags and counts have been read.
enum LibraryIndexStatus {
  /// Being read for the first time.
  loading,

  /// Read; kept up to date as the library changes.
  ready,

  /// The last reading failed. What was read before, if anything, is kept.
  failed,
}
