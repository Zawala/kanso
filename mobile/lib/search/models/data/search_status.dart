// Kanso — search_status.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../state/search/search_bloc.dart' show SearchState;

/// What a search has to show.
enum SearchStatus {
  /// There is nothing typed to look for.
  idle,

  /// The notes are being searched, and nothing has been found yet.
  searching,

  /// The notes were searched; [SearchState.hits] is what was found, which
  /// may be nothing.
  ready,

  /// The last search failed.
  failed,
}
