// Kanso — library_pane_state.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'library_pane_bloc.dart';

@freezed
abstract class LibraryPaneState with _$LibraryPaneState {
  const factory LibraryPaneState({
    /// Whether the pane is shown beside the list.
    @Default(true) bool visible,
  }) = _LibraryPaneState;
}
