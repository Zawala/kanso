// Kanso — query_ranges.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../core/bridge/api/types.dart';

/// Where [query] occurs in [text], ignoring case, as ranges of UTF-16 units:
/// what the library reports for a match, worked out by hand for the mocks.
List<Range16> queryRanges(String text, String query) {
  final ranges = <Range16>[];
  if (query.isEmpty) {
    return ranges;
  }
  final haystack = text.toLowerCase();
  final needle = query.toLowerCase();
  if (haystack.length != text.length) {
    return ranges;
  }
  var from = 0;
  while (true) {
    final at = haystack.indexOf(needle, from);
    if (at < 0) {
      return ranges;
    }
    ranges.add(Range16(start: at, end: at + needle.length));
    from = at + needle.length;
  }
}
