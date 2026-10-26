// Kanso — range_spans.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math' as math;

import '../bridge/api/types.dart';

/// A run of a text: [matched] when it is one of the places a search found
/// its words.
class RangeSpan {
  const RangeSpan(this.text, {required this.matched});

  final String text;
  final bool matched;
}

/// [text] cut into runs, the places in [ranges] marked as matches.
///
/// [ranges] are what the library reports for a search hit: where the words
/// matched, in UTF-16 units of [text], which is how a Dart string indexes
/// and so cuts it without any conversion. They may come in any order, touch
/// or overlap; each is cut to the text, and what is left is merged.
///
/// The runs joined give back [text]; the matched runs cover exactly the
/// ranges that fall inside it; no run is empty and no two runs that follow
/// each other are both matched or both not.
List<RangeSpan> rangeSpans(String text, List<Range16> ranges) {
  if (text.isEmpty) {
    return const <RangeSpan>[];
  }
  final clamped = <(int, int)>[
    for (final range in ranges)
      if (math.min(range.end, text.length) > math.max(range.start, 0))
        (math.max(range.start, 0), math.min(range.end, text.length)),
  ]..sort((a, b) => a.$1.compareTo(b.$1));
  final spans = <RangeSpan>[];
  var cursor = 0;
  var index = 0;
  while (index < clamped.length) {
    var (start, end) = clamped[index++];
    // What touches or overlaps this range is part of it.
    while (index < clamped.length && clamped[index].$1 <= end) {
      end = math.max(end, clamped[index].$2);
      index++;
    }
    if (start > cursor) {
      spans.add(RangeSpan(text.substring(cursor, start), matched: false));
    }
    spans.add(RangeSpan(text.substring(start, end), matched: true));
    cursor = end;
  }
  if (cursor < text.length) {
    spans.add(RangeSpan(text.substring(cursor), matched: false));
  }
  return spans;
}
