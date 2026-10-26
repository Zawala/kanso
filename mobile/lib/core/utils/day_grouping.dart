// Kanso — day_grouping.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../note_list/models/data/bucket.dart';
import '../../note_list/models/data/note_list_entry.dart';
import '../bridge/api/types.dart';

/// The local day of the instant [ms] (milliseconds since the epoch): the
/// number of days from 1970-01-01 to its date in the phone's time zone.
///
/// Each instant is turned into local time by itself, so a list that spans
/// a change of clocks still puts every note under the right day.
int localDayOf(int ms) {
  final local = DateTime.fromMillisecondsSinceEpoch(ms);
  return DateTime.utc(
        local.year,
        local.month,
        local.day,
      ).millisecondsSinceEpoch ~/
      Duration.millisecondsPerDay;
}

/// [rows], in list order, with a header wherever the bucket changes, when
/// [shape] says the list is sorted by date. Pinned notes are under one
/// "Pinned" header when [shape] groups them; the others under the day of
/// the date the list is sorted by. [localDay] turns an instant into its
/// local day.
///
/// Dropping the headers gives back [rows]; there is never a header with
/// nothing under it, or two in a row; a list that is not sorted by date has
/// none.
List<NoteListEntry> groupRows(
  List<NoteRow> rows,
  ListShape shape, {
  int Function(int ms) localDay = localDayOf,
}) {
  final entries = <NoteListEntry>[];
  Bucket? current;
  for (final row in rows) {
    if (shape.byDate) {
      final bucket = shape.groupPinned && row.pinned
          ? const Bucket.pinned()
          : Bucket.day(
              localDay(
                shape.date == DateField.created ? row.createdMs : row.updatedMs,
              ),
            );
      if (bucket != current) {
        entries.add(NoteListEntry.header(bucket));
        current = bucket;
      }
    }
    entries.add(NoteListEntry.note(row));
  }
  return entries;
}
