// Kanso — navigation_history_state.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'navigation_history_bloc.dart';

@freezed
abstract class NavigationHistoryState with _$NavigationHistoryState {
  const factory NavigationHistoryState({
    /// The visits behind the open note, the open note and the visits ahead.
    @Default(NavigationHistory()) NavigationHistory history,

    /// The note going back or forward landed on, for the screen to open.
    String? pendingNoteId,

    /// Whether going back would land on a note that is still there.
    @Default(false) bool canGoBack,

    /// Whether going forward would land on a note that is still there.
    @Default(false) bool canGoForward,
  }) = _NavigationHistoryState;
}
