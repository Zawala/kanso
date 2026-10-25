// Kanso — kanso_spacing.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Spacing, on an 8-point scale with half steps for dense rows.
abstract final class KansoSpacing {
  /// 4: icon-to-label gaps and dense list rows.
  static const double xxs = 4;

  /// 8
  static const double xs = 8;

  /// 12: the gutter inside a compact row.
  static const double sm = 12;

  /// 16: the page gutter on a phone.
  static const double md = 16;

  /// 24
  static const double lg = 24;

  /// 32
  static const double xl = 32;

  /// 48
  static const double xxl = 48;
}
