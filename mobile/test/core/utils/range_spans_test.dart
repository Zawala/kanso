// Kanso — range_spans_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart'
    show Any, CombinableAny, Generator, Glados2, IntAnys, ListAnys, any;
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/utils/range_spans.dart';

import '../../helpers/property.dart';

Range16 _range(int start, int end) => Range16(start: start, end: end);

extension _Ranges on Any {
  /// Ranges of any size and place: out of order, touching, overlapping, empty
  /// and past either end of the text.
  Generator<List<Range16>> get anyRanges => listWithLengthInRange<Range16>(
    0,
    6,
    combine2(
      intInRange(-3, 20),
      intInRange(-3, 12),
      (int start, int length) => _range(start, start + length),
    ),
  );
}

void main() {
  group('rangeSpans', () {
    test('has nothing for no text', () {
      expect(rangeSpans('', <Range16>[_range(0, 3)]), isEmpty);
    });

    test('is the whole text, unmatched, when nothing matched', () {
      final spans = rangeSpans('some text', const <Range16>[]);
      expect(spans.map((span) => (span.text, span.matched)), <(String, bool)>[
        ('some text', false),
      ]);
    });

    test('marks the places given, and leaves the rest between them', () {
      final spans = rangeSpans('Sync and sync again', <Range16>[
        _range(0, 4),
        _range(9, 13),
      ]);
      expect(spans.map((span) => (span.text, span.matched)), <(String, bool)>[
        ('Sync', true),
        (' and ', false),
        ('sync', true),
        (' again', false),
      ]);
    });

    test('merges ranges that touch or overlap, in any order', () {
      final spans = rangeSpans('abcdefgh', <Range16>[
        _range(4, 6),
        _range(0, 2),
        _range(2, 4),
        _range(5, 7),
      ]);
      expect(spans.map((span) => (span.text, span.matched)), <(String, bool)>[
        ('abcdefg', true),
        ('h', false),
      ]);
    });

    test('cuts a range that runs past the text, and drops one outside', () {
      final spans = rangeSpans('abc', <Range16>[_range(1, 99), _range(7, 9)]);
      expect(spans.map((span) => (span.text, span.matched)), <(String, bool)>[
        ('a', false),
        ('bc', true),
      ]);
      expect(rangeSpans('abc', <Range16>[_range(-5, 2)]).first.text, 'ab');
    });

    test('slices by UTF-16 units, as the library counts them', () {
      // A letter outside the basic plane is two units.
      const text = 'a\u{1F600}b sync';
      final start = text.indexOf('sync');
      final spans = rangeSpans(text, <Range16>[_range(start, start + 4)]);
      expect(spans.last.text, 'sync');
      expect(spans.last.matched, isTrue);
      expect(spans.first.text, 'a\u{1F600}b ');
    });

    Glados2<String, List<Range16>>(
      any.markdownish,
      any.anyRanges,
      kansoExploreConfig(),
    ).test('the runs rejoin to the text', (text, ranges) {
      expect(rangeSpans(text, ranges).map((span) => span.text).join(), text);
    });

    Glados2<String, List<Range16>>(
      any.markdownish,
      any.anyRanges,
      kansoExploreConfig(),
    ).test('no run is empty, and neighbours differ in match', (
      text,
      ranges,
    ) {
      final spans = rangeSpans(text, ranges);
      expect(spans.every((span) => span.text.isNotEmpty), isTrue);
      for (var i = 1; i < spans.length; i++) {
        expect(spans[i].matched, isNot(spans[i - 1].matched));
      }
    });

    Glados2<String, List<Range16>>(
      any.markdownish,
      any.anyRanges,
      kansoExploreConfig(),
    ).test('the matched runs are exactly the ranges, cut to the text', (
      text,
      ranges,
    ) {
      final wanted = List<bool>.filled(text.length, false);
      for (final range in ranges) {
        for (var i = range.start; i < range.end; i++) {
          if (i >= 0 && i < text.length) {
            wanted[i] = true;
          }
        }
      }
      final got = <bool>[
        for (final span in rangeSpans(text, ranges))
          for (var i = 0; i < span.text.length; i++) span.matched,
      ];
      expect(got, wanted);
    });
  });
}
