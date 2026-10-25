// Kanso — search_match_spans.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// A run of a text: [matched] when it is one of the words searched for.
class SearchSpan {
  const SearchSpan(this.text, {required this.matched});

  final String text;
  final bool matched;
}

/// [text] cut into runs, the case-insensitive occurrences of [query] marked
/// as matches.
///
/// The runs joined give back [text]; a matched run is exactly an occurrence
/// of [query]; no run is empty. An empty [query] matches nothing.
List<SearchSpan> searchMatchSpans(String text, String query) {
  if (text.isEmpty) {
    return const <SearchSpan>[];
  }
  final needle = query.toLowerCase();
  final haystack = text.toLowerCase();
  // Lowercasing can change a string's length (some scripts); then the
  // offsets of the lowercase copy do not hold for the original.
  if (needle.isEmpty || haystack.length != text.length) {
    return <SearchSpan>[SearchSpan(text, matched: false)];
  }
  final spans = <SearchSpan>[];
  var cursor = 0;
  while (true) {
    final found = haystack.indexOf(needle, cursor);
    if (found < 0) {
      break;
    }
    if (found > cursor) {
      spans.add(SearchSpan(text.substring(cursor, found), matched: false));
    }
    final end = found + needle.length;
    spans.add(SearchSpan(text.substring(found, end), matched: true));
    cursor = end;
  }
  if (cursor < text.length) {
    spans.add(SearchSpan(text.substring(cursor), matched: false));
  }
  return spans;
}
