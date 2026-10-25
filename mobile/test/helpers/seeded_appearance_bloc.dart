// Kanso — seeded_appearance_bloc.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:kanso/core/state/appearance/appearance_bloc.dart';

/// An `AppearanceBloc` that starts in `seed` rather than the defaults, for
/// pages whose picture depends on the person's choices.
class SeededAppearanceBloc extends AppearanceBloc {
  SeededAppearanceBloc(AppearanceState seed) {
    emit(seed);
  }
}
