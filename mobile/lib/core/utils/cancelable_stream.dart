// Kanso — cancelable_stream.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

/// [source] as a stream whose subscription can be cancelled at once.
///
/// The bridge's streams are generators waiting on the Rust side: cancelling
/// one does not finish until the library sends its next event, which may be
/// a long time. Awaiting that (as `Bloc.close` does) would hang. The stream
/// returned here lets go of [source] without waiting for it; the library
/// stops sending to a stream nobody listens to at its next event.
///
/// The stream is single-subscription, like [source].
Stream<T> cancelableStream<T>(Stream<T> source) {
  late StreamController<T> controller;
  // Cancelled in `onCancel`, below.
  // ignore: cancel_subscriptions
  StreamSubscription<T>? subscription;
  controller = StreamController<T>(
    onListen: () {
      subscription = source.listen(
        controller.add,
        onError: controller.addError,
        onDone: controller.close,
      );
    },
    onPause: () => subscription?.pause(),
    onResume: () => subscription?.resume(),
    onCancel: () {
      final upstream = subscription;
      subscription = null;
      if (upstream != null) {
        unawaited(upstream.cancel());
      }
    },
  );
  return controller.stream;
}
