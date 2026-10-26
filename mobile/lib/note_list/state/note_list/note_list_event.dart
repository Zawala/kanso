// Kanso — note_list_event.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'note_list_bloc.dart';

@freezed
sealed class NoteListEvent with _$NoteListEvent {
  /// The list of [place] was opened: read the person's sort, then the notes,
  /// and keep them read.
  const factory NoteListEvent.started(ListPlace place) = NoteListStartedEvent;

  /// The person chose how to sort or narrow the list. A change of sort is
  /// saved for every list.
  const factory NoteListEvent.optionsChanged(ListOptions options) =
      NoteListOptionsChangedEvent;

  /// The library wrote [ids] (notes, notebooks or tags): read the list
  /// again, once the writes of a burst have stopped.
  const factory NoteListEvent.changesReceived(List<String> ids) =
      NoteListChangesReceivedEvent;

  /// The person asked to try again after a failure.
  const factory NoteListEvent.retried() = NoteListRetriedEvent;

  /// A long press on the note [id] began choosing several notes.
  const factory NoteListEvent.selectionStarted(String id) =
      NoteListSelectionStartedEvent;

  /// A tap on the note [id] while choosing: choose it, or let it go.
  const factory NoteListEvent.selectionToggled(String id) =
      NoteListSelectionToggledEvent;

  /// The person is done choosing.
  const factory NoteListEvent.selectionCleared() =
      NoteListSelectionClearedEvent;

  /// [change] to every note in [ids], in one write.
  const factory NoteListEvent.changeRequested(
    NoteChange change,
    List<String> ids,
  ) = NoteListChangeRequestedEvent;

  /// A swipe on a row reached the point where letting go acts.
  const factory NoteListEvent.swipeArmed() = NoteListSwipeArmedEvent;

  /// The person has read why a change failed.
  const factory NoteListEvent.errorDismissed() = NoteListErrorDismissedEvent;
}
