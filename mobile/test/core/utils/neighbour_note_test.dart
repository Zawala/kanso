// Kanso — neighbour_note_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show BoolAny, Glados3, IntAnys, any;
import 'package:kanso/core/utils/neighbour_note.dart';

import '../../helpers/property.dart';

void main() {
  group('neighbourNote', () {
    const ids = <String>['a', 'b', 'c'];

    test('steps to the next and the previous note', () {
      expect(neighbourNote(ids, 'a', forward: true), 'b');
      expect(neighbourNote(ids, 'c', forward: false), 'b');
    });

    test('stays at the ends', () {
      expect(neighbourNote(ids, 'c', forward: true), isNull);
      expect(neighbourNote(ids, 'a', forward: false), isNull);
    });

    test('with no note open, forward is the first and back the last', () {
      expect(neighbourNote(ids, null, forward: true), 'a');
      expect(neighbourNote(ids, null, forward: false), 'c');
      expect(neighbourNote(ids, 'gone', forward: true), 'a');
      expect(neighbourNote(ids, 'gone', forward: false), 'c');
    });

    test('an empty list has nowhere to go', () {
      expect(neighbourNote(const <String>[], 'a', forward: true), isNull);
      expect(neighbourNote(const <String>[], null, forward: false), isNull);
    });

    Glados3<int, int, bool>(
      any.intInRange(1, 9),
      any.intInRange(0, 9),
      any.bool,
      kansoExploreConfig(),
    ).test('lands one place away, in the list, and back returns', (
      length,
      at,
      forward,
    ) {
      final list = <String>[for (var i = 0; i < length; i++) 'n$i'];
      final current = list[at % length];
      final next = neighbourNote(list, current, forward: forward);
      if (next == null) {
        expect(
          list.indexOf(current),
          forward ? list.length - 1 : 0,
        );
        return;
      }
      expect(list, contains(next));
      expect(
        list.indexOf(next) - list.indexOf(current),
        forward ? 1 : -1,
      );
      expect(neighbourNote(list, next, forward: !forward), current);
    });
  });
}
