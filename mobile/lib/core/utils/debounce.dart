// Kanso — debounce.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';

/// [source] with each event held for [duration] and dropped if another comes
/// first: only the last of a burst gets through, [duration] after it. When
/// [source] ends, an event still held is delivered before the stream ends.
Stream<T> debounceStream<T>(Stream<T> source, Duration duration) {
  late StreamController<T> controller;
  StreamSubscription<T>? subscription;
  Timer? timer;
  var sourceDone = false;
  controller = StreamController<T>(
    onListen: () {
      subscription = source.listen(
        (event) {
          timer?.cancel();
          timer = Timer(duration, () {
            controller.add(event);
            if (sourceDone) {
              unawaited(controller.close());
            }
          });
        },
        onError: controller.addError,
        onDone: () {
          sourceDone = true;
          if (!(timer?.isActive ?? false)) {
            unawaited(controller.close());
          }
        },
      );
    },
    onPause: () => subscription?.pause(),
    onResume: () => subscription?.resume(),
    onCancel: () {
      timer?.cancel();
      return subscription?.cancel();
    },
  );
  return controller.stream;
}

/// An event transformer that waits [duration] after the last of a burst of
/// events and then handles it, abandoning a handler still running for an
/// earlier one.
EventTransformer<E> debounceRestartable<E>(Duration duration) {
  return (events, mapper) =>
      restartable<E>().call(debounceStream(events, duration), mapper);
}
