// Kanso — note_event.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'note_bloc.dart';

@freezed
sealed class NoteEvent with _$NoteEvent {
  /// The note [id] was opened, to be shown in the built-in theme [themeId]:
  /// read it, and keep it read.
  const factory NoteEvent.started(String id, String themeId) = NoteStartedEvent;

  /// The library wrote [ids]: read the note again if it is one of them,
  /// once the writes of a burst have stopped.
  const factory NoteEvent.changesReceived(List<String> ids) =
      NoteChangesReceivedEvent;

  /// The theme the text is highlighted in changed to [themeId].
  const factory NoteEvent.syntaxThemeChanged(String themeId) =
      NoteSyntaxThemeChangedEvent;

  /// The person asked to try again after a failure.
  const factory NoteEvent.retried() = NoteRetriedEvent;
}
