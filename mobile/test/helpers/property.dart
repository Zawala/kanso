// Kanso — property.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math';
import 'dart:ui';

import 'package:glados/glados.dart'
    show
        Any,
        AnyUtils,
        CombinableAny,
        ExploreConfig,
        Generator,
        GeneratorUtils,
        IntAnys,
        ListAnys;

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

/// The characters Markdown and search care about, plus letters and space.
const List<String> _markdownAlphabet = <String>[
  'a',
  'B',
  'y',
  's',
  'n',
  'c',
  ' ',
  '*',
  '`',
  '[',
  ']',
  '(',
  ')',
];

extension KansoMarkdownAny on Any {
  /// Short strings made of the characters Markdown and search care about, so
  /// marks pair up, break and overlap often.
  Generator<String> get markdownish {
    return listWithLengthInRange<String>(
      0,
      14,
      choose(_markdownAlphabet),
    ).map((chars) => chars.join());
  }
}
