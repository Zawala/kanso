// Kanso — kanso_effects.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Opacities that tint, dim or soften a colour.
abstract final class KansoEffects {
  /// A scrim over content behind a modal surface, on a light theme.
  static const double scrimLight = 0.32;

  /// A scrim over content behind a modal surface, on a dark theme.
  static const double scrimDark = 0.6;

  /// A tinted wash behind a tag or a search match.
  static const double wash = 0.18;

  /// A stronger wash, for a match in running text.
  static const double matchWash = 0.3;

  /// A control that cannot be used.
  static const double disabled = 0.45;

  /// A control while it is pressed.
  static const double pressed = 0.6;

  /// A shadow under a floating surface.
  static const double shadow = 0.2;
}
