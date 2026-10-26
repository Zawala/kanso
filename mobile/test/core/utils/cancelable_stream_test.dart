// Kanso — cancelable_stream_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show Glados, IntAnys, ListAnys, any;
import 'package:kanso/core/utils/cancelable_stream.dart';

import '../../helpers/property.dart';

void main() {
  group('cancelableStream', () {
    Glados(any.list(any.int), kansoExploreConfig()).test(
      'passes every event on in order, then ends',
      (events) async {
        final source = Stream<int>.fromIterable(events);
        expect(await cancelableStream(source).toList(), events);
      },
    );

    test('cancelling does not wait for a source that never finishes', () async {
      // Like the bridge's streams: cancelling waits for the next event.
      final source = StreamController<int>(
        onCancel: () => Completer<void>().future,
      );
      final heard = <int>[];
      final subscription = cancelableStream(source.stream).listen(heard.add);
      source.add(1);
      await Future<void>.delayed(Duration.zero);
      await subscription.cancel().timeout(const Duration(seconds: 1));
      expect(heard, <int>[1]);
    });

    test('an error of the source is an error of the stream', () async {
      final source = Stream<int>.error(StateError('x'));
      await expectLater(cancelableStream(source), emitsError(isStateError));
    });
  });
}
