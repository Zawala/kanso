// Kanso — library_index_event.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'library_index_bloc.dart';

@freezed
sealed class LibraryIndexEvent with _$LibraryIndexEvent {
  /// The app started: read the library and keep it read.
  const factory LibraryIndexEvent.started() = LibraryIndexStartedEvent;

  /// The library wrote [ids] (notes, notebooks or tags): read it again, once
  /// the writes of a burst have stopped.
  const factory LibraryIndexEvent.changesReceived(List<String> ids) =
      LibraryIndexChangesReceivedEvent;

  /// The person asked to try again after a failure.
  const factory LibraryIndexEvent.retried() = LibraryIndexRetriedEvent;
}
