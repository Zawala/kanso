// Kanso — navigation_history_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart'
    show Glados, Glados2, IntAnys, ListAnys, any;
import 'package:kanso/core/models/data/navigation_history.dart';

import '../../../helpers/property.dart';

bool _all(String id) => true;

NavigationHistory _visited(List<String> ids) {
  return ids.fold(
    const NavigationHistory(),
    (history, id) => history.visit(id),
  );
}

List<String> _ids(NavigationHistory history) => history.visits.$1;

/// The same history kept the plain way: one list of visits and the index of
/// the current one (the desktop's test model).
class _Model {
  List<String> visits = <String>[];
  int? at;

  String? get current => at == null ? null : visits[at!];

  void visit(String id) {
    if (current == id) {
      return;
    }
    final keep = at == null ? 0 : at! + 1;
    visits = visits.sublist(0, keep)..add(id);
    if (visits.length > mostVisits + 1) {
      visits.removeAt(0);
    }
    at = visits.length - 1;
  }

  /// Steps one way, dropping the visits passed over (all of that side's when
  /// there is none to land on).
  String? step({required bool back, required bool Function(String) live}) {
    final here = at;
    if (here == null) {
      return null;
    }
    final id = visits[here];
    bool wanted(int index) => visits[index] != id && live(visits[index]);
    if (back) {
      int? landing;
      for (var index = here - 1; index >= 0; index--) {
        if (wanted(index)) {
          landing = index;
          break;
        }
      }
      final firstPassed = landing == null ? 0 : landing + 1;
      visits.removeRange(firstPassed, here);
      at = firstPassed;
      if (landing == null) {
        return null;
      }
      at = landing;
    } else {
      int? landing;
      for (var index = here + 1; index < visits.length; index++) {
        if (wanted(index)) {
          landing = index;
          break;
        }
      }
      final past = landing ?? visits.length;
      visits.removeRange(here + 1, past);
      if (landing == null) {
        return null;
      }
      at = here + 1;
    }
    return visits[at!];
  }
}

void main() {
  group('NavigationHistory', () {
    test('back and forward walk the visits', () {
      var history = _visited(<String>['a', 'b', 'c']);
      var step = history.goBack(_all);
      expect(step.landed, 'b');
      step = step.history.goBack(_all);
      expect(step.landed, 'a');
      step = step.history.goBack(_all);
      expect(step.landed, isNull);
      history = step.history;
      expect(history.current, 'a');
      step = history.goForward(_all);
      expect(step.landed, 'b');
      step = step.history.goForward(_all);
      expect(step.landed, 'c');
      expect(step.history.goForward(_all).landed, isNull);
    });

    test('opening a note after going back drops the visits ahead', () {
      final back = _visited(<String>['a', 'b', 'c']).goBack(_all).history;
      final history = back.visit('d');
      expect(_ids(history), <String>['a', 'b', 'd']);
      expect(history.canGoForward(_all), isFalse);
      expect(history.goBack(_all).landed, 'b');
    });

    test('opening the open note again is not a visit', () {
      final history = _visited(<String>['a', 'a']);
      expect(_ids(history), <String>['a']);
      expect(history.canGoBack(_all), isFalse);
    });

    test('going back skips notes trashed or deleted', () {
      final history = _visited(<String>['a', 'b', 'c', 'd']);
      bool live(String id) => id != 'b' && id != 'c';
      expect(history.canGoBack(live), isTrue);
      final back = history.goBack(live);
      expect(back.landed, 'a');
      expect(back.history.goForward(live).landed, 'd');
      // With everything behind gone, there is nowhere to go back to.
      bool onlyD(String id) => id == 'd';
      expect(history.canGoBack(onlyD), isFalse);
      final none = history.goBack(onlyD);
      expect(none.landed, isNull);
      expect(none.history.current, 'd');
    });

    test('skipping a deleted note never lands on the open one', () {
      final history = _visited(<String>['a', 'b', 'a']);
      // Behind "a" are "b" (deleted) and "a" itself.
      final step = history.goBack((id) => id != 'b');
      expect(step.landed, isNull);
      expect(step.history.current, 'a');
    });

    test('the oldest visits are forgotten', () {
      final history = _visited(<String>[
        for (var n = 0; n < mostVisits + 5; n++) '$n',
      ]);
      final (visits, at) = history.visits;
      expect(visits, hasLength(mostVisits + 1));
      expect(visits.first, '4');
      expect(at, mostVisits);
    });

    test('lists the visits oldest first with the current one marked', () {
      final history = _visited(<String>['a', 'b', 'c']).goBack(_all).history;
      expect(history.visits.$1, <String>['a', 'b', 'c']);
      expect(history.visits.$2, 1);
      expect(const NavigationHistory().visits.$1, isEmpty);
      expect(const NavigationHistory().visits.$2, isNull);
    });

    Glados2<List<int>, int>(
      any.listWithLengthInRange(1, 30, any.intInRange(0, 8)),
      any.intInRange(0, 30),
      kansoExploreConfig(),
    ).test('going back then forward comes back to the same visits', (
      opens,
      steps,
    ) {
      final start = _visited(<String>[for (final n in opens) '$n']);
      var history = start;
      var taken = 0;
      while (taken < steps) {
        final step = history.goBack(_all);
        if (step.landed == null) {
          break;
        }
        history = step.history;
        taken++;
      }
      for (var i = 0; i < taken; i++) {
        final step = history.goForward(_all);
        expect(step.landed, isNotNull);
        history = step.history;
      }
      expect(history.visits.$1, start.visits.$1);
      expect(history.visits.$2, start.visits.$2);
    });

    Glados<List<(int, int)>>(
      any.listWithLengthInRange(0, 120, any.historyOp),
      kansoExploreConfig(),
    ).test('matches a model, whatever is opened, trashed and restored', (ops) {
      var history = const NavigationHistory();
      final model = _Model();
      final gone = <String>{};
      bool live(String id) => !gone.contains(id);
      for (final (kind, note) in ops) {
        final id = '$note';
        switch (kind) {
          case 0 || 1 || 2 || 3:
            // Only a live note opens.
            if (live(id)) {
              history = history.visit(id);
              model.visit(id);
            }
          case 4 || 5:
            final back = kind == 4;
            final before = history.current;
            final could = back
                ? history.canGoBack(live)
                : history.canGoForward(live);
            final step = back ? history.goBack(live) : history.goForward(live);
            final expected = model.step(back: back, live: live);
            expect(could, step.landed != null);
            expect(step.landed, expected);
            if (step.landed != null) {
              expect(
                live(step.landed!),
                isTrue,
                reason: 'landed on a gone note',
              );
              expect(step.landed, isNot(before));
              expect(step.history.current, step.landed);
            } else {
              expect(step.history.current, before);
            }
            history = step.history;
          default:
            if (note.isEven) {
              gone.add(id);
            } else {
              gone.remove(id);
            }
        }
        expect(history.visits.$1, model.visits);
        expect(history.visits.$2, model.at);
        expect(history.back.length, lessThanOrEqualTo(mostVisits));
      }
    });
  });
}
