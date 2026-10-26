// Kanso — core_check_status.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// How far the core check has got.
enum CoreCheckStatus {
  /// Asking the library.
  loading,

  /// The library answered.
  ready,

  /// The library could not be read.
  failed,
}
