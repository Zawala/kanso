// Kanso — list_navigation_state.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'list_navigation_bloc.dart';

@freezed
abstract class ListNavigationState with _$ListNavigationState {
  const factory ListNavigationState({
    /// The ids of the notes of the list on screen, in order.
    @Default(<String>[]) List<String> noteIds,
  }) = _ListNavigationState;
}
