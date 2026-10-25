// Kanso — property.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math';
import 'dart:ui';

import 'package:glados/glados.dart'
    show Any, CombinableAny, ExploreConfig, Generator, IntAnys;

/// Property tests use this configuration so a failure in CI reproduces
/// locally: the random source has a fixed seed.
ExploreConfig kansoExploreConfig() {
  return ExploreConfig(random: Random(20261025));
}

extension KansoAny on Any {
  /// Any opaque sRGB colour.
  Generator<Color> get opaqueColor => combine3(
    intInRange(0, 256),
    intInRange(0, 256),
    intInRange(0, 256),
    (int r, int g, int b) => Color.fromARGB(255, r, g, b),
  );
}
