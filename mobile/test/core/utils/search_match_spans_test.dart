// Kanso — search_match_spans_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show Glados2, any;
import 'package:kanso/core/utils/search_match_spans.dart';

import '../../helpers/property.dart';

void main() {
  group('searchMatchSpans properties', () {
    Glados2<String, String>(
      any.markdownish,
      any.markdownish,
      kansoExploreConfig(),
    ).test('the spans joined give back the text', (text, query) {
      final spans = searchMatchSpans(text, query);
      expect(spans.map((span) => span.text).join(), text);
    });

    Glados2<String, String>(
      any.markdownish,
      any.markdownish,
      kansoExploreConfig(),
    ).test('no span is empty', (text, query) {
      expect(
        searchMatchSpans(text, query).every((span) => span.text.isNotEmpty),
        isTrue,
      );
    });

    Glados2<String, String>(
      any.markdownish,
      any.markdownish,
      kansoExploreConfig(),
    ).test('a matched span is the query, ignoring case', (text, query) {
      for (final span in searchMatchSpans(text, query)) {
        if (span.matched) {
          expect(span.text.toLowerCase(), query.toLowerCase());
        }
      }
    });

    Glados2<String, String>(
      any.markdownish,
      any.markdownish,
      kansoExploreConfig(),
    ).test('every occurrence of the query is matched', (text, query) {
      if (query.isEmpty) {
        return;
      }
      final spans = searchMatchSpans(text, query);
      final matched = spans.where((span) => span.matched).length;
      // Occurrences that do not overlap, counted from the left.
      var expected = 0;
      var cursor = 0;
      final haystack = text.toLowerCase();
      final needle = query.toLowerCase();
      while (true) {
        final found = haystack.indexOf(needle, cursor);
        if (found < 0) {
          break;
        }
        expected++;
        cursor = found + needle.length;
      }
      expect(matched, expected);
    });

    Glados2<String, String>(
      any.markdownish,
      any.markdownish,
      kansoExploreConfig(),
    ).test('two matched spans never touch a plain one of the same kind', (
      text,
      query,
    ) {
      final spans = searchMatchSpans(text, query);
      for (var i = 1; i < spans.length; i++) {
        if (!spans[i].matched) {
          expect(spans[i - 1].matched, isTrue);
        }
      }
    });
  });

  group('searchMatchSpans examples', () {
    test('marks every word, in any case', () {
      final spans = searchMatchSpans('Sync and sync', 'sync');
      expect(spans.map((s) => (s.text, s.matched)), <(String, bool)>[
        ('Sync', true),
        (' and ', false),
        ('sync', true),
      ]);
    });

    test('an empty query matches nothing', () {
      final spans = searchMatchSpans('abc', '');
      expect(spans.single.matched, isFalse);
    });

    test('an empty text has no spans', () {
      expect(searchMatchSpans('', 'a'), isEmpty);
    });
  });
}
