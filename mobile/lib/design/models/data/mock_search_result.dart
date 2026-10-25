// Kanso — mock_search_result.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'mock_search_result.freezed.dart';

/// A search result: the note, its notebook and the text around the match.
@freezed
abstract class MockSearchResult with _$MockSearchResult {
  const factory MockSearchResult({
    required String title,
    required String notebook,
    required String snippet,
  }) = _MockSearchResult;
}
