// Kanso — contrast.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math' as math;
import 'dart:ui';

/// The WCAG 2.x relative luminance of [color], from 0 (black) to 1 (white).
///
/// Transparency is ignored: pass the colour as it looks over its background.
double relativeLuminance(Color color) {
  double linear(double channel) {
    return channel <= 0.04045
        ? channel / 12.92
        : math.pow((channel + 0.055) / 1.055, 2.4).toDouble();
  }

  return 0.2126 * linear(color.r) +
      0.7152 * linear(color.g) +
      0.0722 * linear(color.b);
}

/// The WCAG 2.x contrast ratio between [a] and [b], from 1 to 21.
///
/// It is symmetric. Body text needs 4.5 or more (7 for the highest level),
/// large text and icons 3 or more.
double contrastRatio(Color a, Color b) {
  final first = relativeLuminance(a);
  final second = relativeLuminance(b);
  final lighter = math.max(first, second);
  final darker = math.min(first, second);
  return (lighter + 0.05) / (darker + 0.05);
}
