// Kanso — kanso_motion.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

/// Durations and easing for every animation in the app.
///
/// Read durations through [resolve] so that a person who turned animations
/// off gets none.
abstract final class KansoMotion {
  /// 150ms: a press, a toggle, a hover.
  static const Duration fast = Duration(milliseconds: 150);

  /// 200ms: a small control changing state.
  static const Duration short = Duration(milliseconds: 200);

  /// 300ms: content entering, leaving or swapping.
  static const Duration content = Duration(milliseconds: 300);

  /// 450ms: a large surface expanding.
  static const Duration complex = Duration(milliseconds: 450);

  /// Things that appear: decelerate into place.
  static const Curve enter = Curves.easeOutCubic;

  /// Things that leave: accelerate away.
  static const Curve exit = Curves.easeInCubic;

  /// Things that move or change in place.
  static const Curve standard = Curves.easeInOutCubic;

  /// [duration], or [Duration.zero] when animations are disabled.
  static Duration resolve(BuildContext context, Duration duration) {
    return MediaQuery.disableAnimationsOf(context) ? Duration.zero : duration;
  }
}
