// Kanso — load_status.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Whether what a screen shows has been read.
enum LoadStatus {
  /// Being read.
  loading,

  /// Read and shown.
  ready,

  /// Could not be read; the screen says so and offers to try again.
  failed,
}
