// Kanso — close_bloc_later.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc/bloc.dart';
import 'package:flutter_test/flutter_test.dart';

/// Closes [bloc] when the test ends, without waiting for it.
///
/// A bloc whose events are debounced never finishes closing inside a widget
/// test's fake clock, so awaiting `close` there would never return.
void closeBlocLater(BlocBase<Object?> bloc) {
  addTearDown(() => unawaited(bloc.close()));
}
