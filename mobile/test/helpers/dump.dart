// Kanso — dump.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:kanso/core/bridge/api/types.dart';

/// Everything [value] holds, as text, field by field.
///
/// The generated classes of the bridge print as `Instance of 'NoteView'`, so
/// a test that looks for a word in what a call returned can't use
/// `toString`. This covers every type the library returns; any other value
/// (an error, which Freezed prints in full) is printed as it is.
String dump(Object? value) {
  return switch (value) {
    Either<Object?, Object?>() => value.fold(dump, dump),
    List<Object?>() => '[${value.map(dump).join(', ')}]',
    NoteRow() =>
      'NoteRow(${value.id}, ${value.rev}, ${value.title}, ${value.snippet}, '
          '${value.notebookId}, ${value.status}, ${dump(value.tags)}, '
          '${value.pinned}, ${value.createdMs}, ${value.updatedMs}, '
          '${value.tasksDone}, ${value.tasksTotal}, ${value.locked})',
    NoteView() =>
      'NoteView(${value.id}, ${value.rev}, ${value.title}, ${value.body}, '
          '${value.lock}, ${value.notebookId}, ${dump(value.tags)}, '
          '${value.status}, ${value.pinned}, ${value.createdMs}, '
          '${value.updatedMs})',
    Hit() =>
      'Hit(${dump(value.row)}, ${dump(value.titleRanges)}, '
          '${dump(value.snippetRanges)})',
    Range16() => '${value.start}..${value.end}',
    Property() =>
      'Property(${value.key}, ${value.kind}, ${value.text}, '
          '${dump(value.items)})',
    Table() => 'Table(${dump(value.columns)}, ${dump(value.rows)})',
    TableRow() => 'TableRow(${value.id}, ${value.title}, ${dump(value.cells)})',
    Saved() => 'Saved(${value.id}, ${value.rev})',
    NotebookRow() =>
      'NotebookRow(${value.id}, ${value.name}, ${value.parentId}, '
          '${value.count}, ${value.locked}, ${value.icon}, ${value.color}, '
          '${value.role}, ${value.depth})',
    NoteList() => 'NoteList(${dump(value.rows)}, ${dump(value.shape)})',
    ListShape() =>
      'ListShape(${value.sort}, ${value.byDate}, ${value.date}, '
          '${value.groupPinned})',
    LibraryOverview() =>
      'LibraryOverview(${value.all}, ${value.pinned}, ${value.recent}, '
          '${value.trash})',
    TagRow() =>
      'TagRow(${value.id}, ${value.name}, ${value.color}, ${value.count})',
    _ => '$value',
  };
}
