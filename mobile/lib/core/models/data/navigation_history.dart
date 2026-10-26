// Kanso — navigation_history.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'navigation_history.freezed.dart';

/// The most visits kept behind the open note; older ones are forgotten.
const int mostVisits = 100;

/// Back and forward between notes, as in a browser: every note opened is a
/// visit, going back and forward walks the visits, and opening a note after
/// going back drops the visits ahead. It is the desktop's `NavHistory`,
/// rule for rule, and one window's own.
///
/// Notes trashed or deleted since they were visited are skipped, and so is a
/// visit to the note already open; whether a note is still there is for the
/// caller to say, as `live`.
@freezed
abstract class NavigationHistory with _$NavigationHistory {
  const factory NavigationHistory({
    /// The visits behind the open note, the nearest last.
    @Default(<String>[]) List<String> back,

    /// The note of the current visit.
    String? current,

    /// The visits ahead of the open note, the nearest last.
    @Default(<String>[]) List<String> forward,
  }) = _NavigationHistory;

  const NavigationHistory._();

  /// Note [id] was opened some way other than going back or forward: it
  /// becomes the current visit, and the visits ahead are dropped. The note
  /// already current stays as it is.
  NavigationHistory visit(String id) {
    if (current == id) {
      return this;
    }
    final behind = <String>[...back, ?current];
    if (behind.length > mostVisits) {
      behind.removeAt(0);
    }
    return NavigationHistory(back: behind, current: id);
  }

  /// The visits, oldest first, and where the current one is in them.
  (List<String>, int?) get visits {
    final all = <String>[...back];
    final at = current == null ? null : all.length;
    if (current != null) {
      all.add(current!);
    }
    all.addAll(forward.reversed);
    return (all, at);
  }

  /// Whether going back would land anywhere: on a note that [live] says is
  /// still there and is not the open one.
  bool canGoBack(bool Function(String id) live) => _reachable(back, live);

  /// Whether going forward would land anywhere.
  bool canGoForward(bool Function(String id) live) => _reachable(forward, live);

  /// The visits on the way back, the nearest first.
  Iterable<String> get candidatesBack => back.reversed;

  /// The visits on the way forward, the nearest first.
  Iterable<String> get candidatesForward => forward.reversed;

  /// Steps back to the nearest earlier visit whose note [live] says is
  /// still there. The note landed on is null when there is none; the visits
  /// stepped over are dropped either way.
  ({NavigationHistory history, String? landed}) goBack(
    bool Function(String id) live,
  ) {
    final (remaining, landed) = _step(back, live);
    return (
      history: landed == null
          ? copyWith(back: remaining)
          : NavigationHistory(
              back: remaining,
              current: landed,
              forward: <String>[...forward, ?current],
            ),
      landed: landed,
    );
  }

  /// Steps forward, as [goBack] steps back.
  ({NavigationHistory history, String? landed}) goForward(
    bool Function(String id) live,
  ) {
    final (remaining, landed) = _step(forward, live);
    return (
      history: landed == null
          ? copyWith(forward: remaining)
          : NavigationHistory(
              back: <String>[...back, ?current],
              current: landed,
              forward: remaining,
            ),
      landed: landed,
    );
  }

  bool _reachable(List<String> visits, bool Function(String id) live) {
    return visits.any((id) => id != current && live(id));
  }

  /// Pops the nearest visit of [side] that is live and another note; what is
  /// left of [side], and that visit.
  (List<String>, String?) _step(
    List<String> side,
    bool Function(String id) live,
  ) {
    final remaining = <String>[...side];
    while (remaining.isNotEmpty) {
      final id = remaining.removeLast();
      if (id == current || !live(id)) {
        continue;
      }
      return (remaining, id);
    }
    return (remaining, null);
  }
}
