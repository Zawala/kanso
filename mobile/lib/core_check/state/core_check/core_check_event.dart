// Kanso — core_check_event.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'core_check_bloc.dart';

@freezed
sealed class CoreCheckEvent with _$CoreCheckEvent {
  /// The page opened: ask the library, and listen for its changes.
  const factory CoreCheckEvent.started() = CoreCheckStartedEvent;

  /// The person asked for a new note.
  const factory CoreCheckEvent.noteCreated() = CoreCheckNoteCreatedEvent;

  /// The library reported that the notes [ids] changed.
  const factory CoreCheckEvent.changeReceived(List<String> ids) =
      CoreCheckChangeReceivedEvent;
}
