// Kanso — muted_tag_color.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math' as math;
import 'dart:ui';

const double _saturationMax = 0.42;
const (double, double) _lightnessDark = (0.50, 0.76);
const (double, double) _lightnessLight = (0.32, 0.50);
const (double, double) _lightnessSpread = (0.40, 0.60);

/// [color] calmed for reading on a [dark] or light surface, as the desktop
/// app does: the hue stays, the saturation is compressed toward a cap and
/// the lightness is moved into a band that reads on the surface.
///
/// Both keep their order instead of being cut off: a stronger colour stays
/// stronger and a darker one stays darker. The stored colour is unchanged.
Color mutedTagColor(Color color, {required bool dark}) {
  final (hue, saturation, lightness) = toHsl(color);
  final (low, high) = dark ? _lightnessDark : _lightnessLight;
  final (from, to) = _lightnessSpread;
  final spread = ((lightness - from) / (to - from)).clamp(0.0, 1.0);
  return _fromHsl(
    hue,
    math.min(saturation, _saturationMax * math.sqrt(saturation)),
    low + (high - low) * spread,
  );
}

/// [color] as hue in degrees `0..360`, saturation and lightness in `0..1`.
(double, double, double) toHsl(Color color) {
  final r = color.r;
  final g = color.g;
  final b = color.b;
  final max = math.max(r, math.max(g, b));
  final min = math.min(r, math.min(g, b));
  final lightness = (max + min) / 2;
  final chroma = max - min;
  if (chroma == 0) {
    return (0, 0, lightness);
  }
  final saturation = chroma / (1 - (2 * lightness - 1).abs());
  final double hue;
  if (max == r) {
    hue = 60 * (((g - b) / chroma) % 6);
  } else if (max == g) {
    hue = 60 * ((b - r) / chroma + 2);
  } else {
    hue = 60 * ((r - g) / chroma + 4);
  }
  return (hue, math.min(saturation, 1), lightness);
}

Color _fromHsl(double hue, double saturation, double lightness) {
  final chroma = (1 - (2 * lightness - 1).abs()) * saturation;
  final sector = (hue % 360) / 60;
  final x = chroma * (1 - ((sector % 2) - 1).abs());
  final (r, g, b) = switch (sector.floor()) {
    0 => (chroma, x, 0.0),
    1 => (x, chroma, 0.0),
    2 => (0.0, chroma, x),
    3 => (0.0, x, chroma),
    4 => (x, 0.0, chroma),
    _ => (chroma, 0.0, x),
  };
  final m = lightness - chroma / 2;
  int byte(double v) => ((v + m).clamp(0.0, 1.0) * 255).round();
  return Color.fromARGB(255, byte(r), byte(g), byte(b));
}
