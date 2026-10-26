// Kanso — day_grouping_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show Glados2, Glados3, any;
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/utils/day_grouping.dart';
import 'package:kanso/note_list/models/data/bucket.dart';
import 'package:kanso/note_list/models/data/note_list_entry.dart';

import '../../helpers/library_fixtures.dart';
import '../../helpers/property.dart';

const int _dayMs = 86400000;

/// The local day at a fixed offset from UTC.
int Function(int) _at(int offsetSeconds) =>
    (ms) => (ms + offsetSeconds * 1000) ~/ _dayMs;

Bucket _bucketOf(NoteRow row, ListShape shape, int Function(int) localDay) {
  if (shape.groupPinned && row.pinned) {
    return const Bucket.pinned();
  }
  return Bucket.day(
    localDay(shape.date == DateField.created ? row.createdMs : row.updatedMs),
  );
}

List<NoteRow> _notes(List<NoteListEntry> entries) => <NoteRow>[
  for (final entry in entries)
    if (entry is NoteEntry) entry.row,
];

void main() {
  group('localDayOf', () {
    test('is the same from midnight to the last minute of a day', () {
      final first = DateTime(2026, 10, 26).millisecondsSinceEpoch;
      final last = DateTime(2026, 10, 26, 23, 59).millisecondsSinceEpoch;
      final next = DateTime(2026, 10, 27).millisecondsSinceEpoch;
      expect(localDayOf(last), localDayOf(first));
      expect(localDayOf(next), localDayOf(first) + 1);
    });

    test('counts days from 1970-01-01', () {
      final epochDay = DateTime(1970).millisecondsSinceEpoch;
      expect(localDayOf(epochDay), 0);
      expect(
        localDayOf(DateTime(2026, 10, 26, 12).millisecondsSinceEpoch),
        20752,
      );
    });
  });

  group('groupRows', () {
    test('puts a header over each day, with Pinned first', () {
      final entries = groupRows(mockRows, mockShape);
      final headers = <Bucket>[
        for (final entry in entries)
          if (entry is HeaderEntry) entry.bucket,
      ];
      final today = localDayOf(mockNow.millisecondsSinceEpoch);
      expect(headers.first, const Bucket.pinned());
      expect(headers[1], Bucket.day(today));
      expect(headers[2], Bucket.day(today - 1));
      // Oct 22, 21, 12, 9: one header for the two notes of the 21st.
      expect(headers, hasLength(7));
      expect(_notes(entries), mockRows);
    });

    test('has no headers when the list is not sorted by date', () {
      final entries = groupRows(
        mockRows,
        const ListShape(
          sort: SortKey.title,
          byDate: false,
          date: DateField.updated,
          groupPinned: false,
        ),
      );
      expect(entries.every((entry) => entry is NoteEntry), isTrue);
      expect(entries, hasLength(mockRows.length));
    });

    test('keeps pinned notes among their days when it does not group them', () {
      final entries = groupRows(
        mockRows,
        const ListShape(
          sort: SortKey.updated,
          byDate: true,
          date: DateField.updated,
          groupPinned: false,
        ),
      );
      expect(
        entries.whereType<HeaderEntry>().map((entry) => entry.bucket),
        everyElement(isA<DayBucket>()),
      );
    });

    test('has nothing for an empty list', () {
      expect(groupRows(const <NoteRow>[], mockShape), isEmpty);
    });

    Glados3<List<NoteRow>, ListShape, int>(
      any.listedRows,
      any.listShape,
      any.utcOffsetSeconds,
      kansoExploreConfig(),
    ).test('dropping the headers gives back the rows, in order', (
      rows,
      shape,
      offset,
    ) {
      final entries = groupRows(rows, shape, localDay: _at(offset));
      expect(_notes(entries), rows);
    });

    Glados3<List<NoteRow>, ListShape, int>(
      any.listedRows,
      any.listShape,
      any.utcOffsetSeconds,
      kansoExploreConfig(),
    ).test('a header stands exactly where the bucket changes', (
      rows,
      shape,
      offset,
    ) {
      final localDay = _at(offset);
      final entries = groupRows(rows, shape, localDay: localDay);
      Bucket? header;
      var expected = 0;
      for (final entry in entries) {
        switch (entry) {
          case HeaderEntry(:final bucket):
            header = bucket;
          case NoteEntry(:final row):
            // Each note sits under the header of its own bucket.
            if (shape.byDate) {
              expect(header, _bucketOf(row, shape, localDay));
            } else {
              expect(header, isNull);
            }
        }
      }
      Bucket? last;
      for (final row in rows) {
        final bucket = _bucketOf(row, shape, localDay);
        if (shape.byDate && bucket != last) {
          expected++;
        }
        last = bucket;
      }
      expect(entries.whereType<HeaderEntry>(), hasLength(expected));
    });

    Glados3<List<NoteRow>, ListShape, int>(
      any.listedRows,
      any.listShape,
      any.utcOffsetSeconds,
      kansoExploreConfig(),
    ).test('has no header without notes under it, or two in a row', (
      rows,
      shape,
      offset,
    ) {
      final entries = groupRows(rows, shape, localDay: _at(offset));
      if (!shape.byDate) {
        expect(entries.whereType<HeaderEntry>(), isEmpty);
      }
      if (entries.isNotEmpty) {
        expect(entries.last, isA<NoteEntry>());
      }
      for (var i = 1; i < entries.length; i++) {
        expect(
          entries[i - 1] is HeaderEntry && entries[i] is HeaderEntry,
          isFalse,
        );
      }
    });

    Glados2<List<NoteRow>, int>(
      any.listedRows,
      any.utcOffsetSeconds,
      kansoExploreConfig(),
    ).test('has a Pinned header only for pinned notes, when it groups them', (
      rows,
      offset,
    ) {
      for (final groupPinned in <bool>[true, false]) {
        final shape = ListShape(
          sort: SortKey.updated,
          byDate: true,
          date: DateField.updated,
          groupPinned: groupPinned,
        );
        final entries = groupRows(rows, shape, localDay: _at(offset));
        final pinnedHeaders = entries.whereType<HeaderEntry>().where(
          (entry) => entry.bucket is PinnedBucket,
        );
        final anyPinned = rows.any((row) => row.pinned);
        expect(pinnedHeaders.length, groupPinned && anyPinned ? 1 : 0);
      }
    });
  });
}
