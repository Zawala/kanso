// Kanso — search_event.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'search_bloc.dart';

@freezed
sealed class SearchEvent with _$SearchEvent {
  /// Search opened for [query] in [scope] ([notebookId] is the notebook it
  /// was opened from): look at once, and keep what is found up to date.
  const factory SearchEvent.started(
    String query,
    SearchScopeKind scope,
    String? notebookId,
  ) = SearchStartedEvent;

  /// The person typed: look for [text], once they have paused.
  const factory SearchEvent.queryChanged(String text) = SearchQueryChangedEvent;

  /// The person chose where to look.
  const factory SearchEvent.scopeChanged(SearchScopeKind scope) =
      SearchScopeChangedEvent;

  /// The person emptied the field.
  const factory SearchEvent.cleared() = SearchClearedEvent;

  /// The library wrote [ids]: look again, once the writes of a burst have
  /// stopped.
  const factory SearchEvent.changesReceived(List<String> ids) =
      SearchChangesReceivedEvent;

  /// The person asked to try again after a failure.
  const factory SearchEvent.retried() = SearchRetriedEvent;
}
