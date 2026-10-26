// Kanso — haptics_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// The taps the phone gives the hand. The device's own haptics setting
/// applies: with it off, these do nothing.
abstract interface class HapticsRepository {
  /// A light tick for a choice: entering a selection, or toggling one.
  Future<void> selection();

  /// A light bump for something done: a swipe reaching the point where it
  /// acts, or a note pinned.
  Future<void> impact();
}
