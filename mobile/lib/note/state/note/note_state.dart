// Kanso — note_state.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'note_bloc.dart';

@freezed
abstract class NoteState with _$NoteState {
  const factory NoteState({
    /// The note being shown.
    @Default('') String noteId,

    /// The built-in theme its text is highlighted in.
    @Default('') String themeId,
    @Default(NoteScreenStatus.loading) NoteScreenStatus status,

    /// The note as read. While it is locked and closed it has its title and
    /// its lock and no text, tags or status; null until it is read, and
    /// when there is none.
    NoteView? note,

    /// The text of the note, cut into lines with their styled runs; empty
    /// unless [status] is ready.
    @Default(<SourceLine>[]) List<SourceLine> lines,

    /// Why reading failed, while [status] is failed.
    ApplicationError? error,
  }) = _NoteState;
}
