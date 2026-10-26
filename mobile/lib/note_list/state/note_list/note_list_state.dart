// Kanso — note_list_state.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'note_list_bloc.dart';

@freezed
abstract class NoteListState with _$NoteListState {
  const factory NoteListState({
    /// The place being listed.
    @Default(ListPlace.all()) ListPlace place,

    /// How it is sorted and narrowed.
    @Default(ListOptions()) ListOptions options,
    @Default(NoteListStatus.loading) NoteListStatus status,

    /// The notes, in list order.
    @Default(<NoteRow>[]) List<NoteRow> rows,

    /// What the library says about laying [rows] out; null until read.
    ListShape? shape,

    /// [rows] under their headers.
    @Default(<NoteListEntry>[]) List<NoteListEntry> entries,

    /// The ids of the notes chosen, while the person is choosing several.
    @Default(<String>{}) Set<String> selected,

    /// Why reading the list failed, while [status] is failed.
    ApplicationError? error,

    /// Why the last change to notes failed, until it is dismissed.
    ApplicationError? actionError,
  }) = _NoteListState;

  const NoteListState._();

  /// Whether the person is choosing several notes.
  bool get selecting => selected.isNotEmpty;

  /// The notes chosen, in list order.
  List<NoteRow> get selectedRows => <NoteRow>[
    for (final row in rows)
      if (selected.contains(row.id)) row,
  ];

  /// Whether every note chosen is pinned: then a pin would unpin them.
  bool get selectionPinned {
    final chosen = selectedRows;
    return chosen.isNotEmpty && chosen.every((row) => row.pinned);
  }

  /// Whether the notes are in the trash: the only change is to move them
  /// out.
  bool get inTrash => place is ListPlace_Trash;
}
