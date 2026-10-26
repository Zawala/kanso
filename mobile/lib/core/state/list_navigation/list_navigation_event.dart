// Kanso — list_navigation_event.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'list_navigation_bloc.dart';

@freezed
sealed class ListNavigationEvent with _$ListNavigationEvent {
  /// The list on screen now holds the notes [noteIds], in order.
  const factory ListNavigationEvent.listShown(List<String> noteIds) =
      ListNavigationListShownEvent;

  /// The person asked, from the keyboard, to pin the note [noteId] or unpin
  /// it.
  const factory ListNavigationEvent.pinRequested(String noteId) =
      ListNavigationPinRequestedEvent;
}
