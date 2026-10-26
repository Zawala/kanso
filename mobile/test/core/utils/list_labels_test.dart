// Kanso — list_labels_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show Glados2, IntAnys, any;
import 'package:intl/date_symbol_data_local.dart';
import 'package:kanso/core/utils/day_grouping.dart';
import 'package:kanso/core/utils/list_labels.dart';
import 'package:kanso/gen/l10n/app_localizations_en.dart';
import 'package:kanso/note_list/models/data/bucket.dart';

import '../../helpers/library_fixtures.dart';
import '../../helpers/property.dart';

final AppLocalizationsEn _l10n = AppLocalizationsEn();

int _day(int month, int day, [int year = 2026]) =>
    localDayOf(DateTime(year, month, day, 12).millisecondsSinceEpoch);

void main() {
  final today = _day(10, 26);

  setUpAll(() => initializeDateFormatting('en'));

  group('dayLabel', () {
    test('says Today and Yesterday', () {
      expect(dayLabel(_l10n, today, today: today), 'Today');
      expect(dayLabel(_l10n, today - 1, today: today), 'Yesterday');
    });

    test('says the date, without the year in the current year', () {
      expect(dayLabel(_l10n, _day(10, 12), today: today), 'Oct 12');
      expect(dayLabel(_l10n, _day(1, 3), today: today), 'Jan 3');
    });

    test('says the year in any other', () {
      expect(dayLabel(_l10n, _day(10, 12, 2025), today: today), 'Oct 12, 2025');
      expect(dayLabel(_l10n, _day(12, 31, 2027), today: today), 'Dec 31, 2027');
    });
  });

  group('bucketLabel', () {
    test('says Pinned for the pinned notes', () {
      expect(bucketLabel(_l10n, const Bucket.pinned(), today: today), 'Pinned');
    });

    test('says the day for a day', () {
      expect(bucketLabel(_l10n, Bucket.day(today), today: today), 'Today');
      expect(
        bucketLabel(_l10n, Bucket.day(_day(10, 21)), today: today),
        'Oct 21',
      );
    });
  });

  group('rowTimeLabel', () {
    test('is the time when it is today', () {
      expect(
        rowTimeLabel(
          _l10n,
          mockMs(10, 26, 9, 41),
          now: mockNow,
        ),
        '09:41',
      );
      expect(
        rowTimeLabel(_l10n, mockMs(10, 26, 0, 5), now: mockNow),
        '00:05',
      );
    });

    test('is the day as a header says it when it is not', () {
      expect(
        rowTimeLabel(_l10n, mockMs(10, 25, 18, 30), now: mockNow),
        'Yesterday',
      );
      expect(
        rowTimeLabel(_l10n, mockMs(10, 12, 10, 0), now: mockNow),
        'Oct 12',
      );
    });

    Glados2<int, int>(
      any.intInRange(1770000000000, 1790000000000),
      any.intInRange(1770000000000, 1790000000000),
      kansoExploreConfig(),
    ).test('agrees with the header of the same day', (at, nowMs) {
      final now = DateTime.fromMillisecondsSinceEpoch(nowMs);
      final todayNow = localDayOf(nowMs);
      final day = localDayOf(at);
      final header = bucketLabel(_l10n, Bucket.day(day), today: todayNow);
      final label = rowTimeLabel(_l10n, at, now: now);
      if (day == todayNow) {
        expect(header, 'Today');
        expect(label, matches(RegExp(r'^\d\d:\d\d$')));
      } else {
        expect(label, header);
      }
    });
  });

  group('dateTimeLabel and dateLabel', () {
    test("write a note's dates", () {
      expect(
        dateTimeLabel(_l10n, mockMs(10, 26, 9, 41), now: mockNow),
        'Today, 09:41',
      );
      expect(
        dateTimeLabel(_l10n, mockMs(10, 21, 14, 20), now: mockNow),
        'Oct 21, 14:20',
      );
      expect(dateLabel(_l10n, mockMs(10, 12, 8, 5)), 'Oct 12, 2026');
    });
  });
}
