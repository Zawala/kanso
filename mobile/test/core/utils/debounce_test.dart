// Kanso — debounce_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/utils/debounce.dart';

void main() {
  const gap = Duration(milliseconds: 30);

  group('debounceStream', () {
    test('lets only the last of a burst through, after the gap', () async {
      final source = StreamController<int>();
      final seen = <int>[];
      final done = Completer<void>();
      debounceStream(
        source.stream,
        gap,
      ).listen(seen.add, onDone: done.complete);
      source
        ..add(1)
        ..add(2)
        ..add(3);
      await Future<void>.delayed(gap * 2);
      expect(seen, <int>[3]);
      source
        ..add(4)
        ..add(5);
      await Future<void>.delayed(gap * 2);
      expect(seen, <int>[3, 5]);
      await source.close();
      await done.future;
    });

    test('delivers a held event before it ends', () async {
      final source = StreamController<int>();
      final seen = <int>[];
      final done = Completer<void>();
      debounceStream(
        source.stream,
        gap,
      ).listen(seen.add, onDone: done.complete);
      source.add(1);
      await source.close();
      await done.future;
      expect(seen, <int>[1]);
    });

    test('ends at once when nothing is held', () async {
      final source = StreamController<int>();
      final done = Completer<void>();
      debounceStream(source.stream, gap).listen((_) {}, onDone: done.complete);
      await source.close();
      await done.future;
    });

    test(
      'drops a held event when cancelled and lets go of the source',
      () async {
        final source = StreamController<int>();
        final seen = <int>[];
        final subscription = debounceStream(
          source.stream,
          gap,
        ).listen(seen.add);
        source.add(1);
        await subscription.cancel();
        expect(source.hasListener, isFalse);
        await Future<void>.delayed(gap * 2);
        expect(seen, isEmpty);
        await source.close();
      },
    );

    test('passes errors on', () async {
      final source = StreamController<int>();
      final errors = <Object>[];
      debounceStream(source.stream, gap).listen((_) {}, onError: errors.add);
      source.addError('broken');
      await Future<void>.delayed(Duration.zero);
      expect(errors, <Object>['broken']);
      await source.close();
    });
  });
}
