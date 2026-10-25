// Kanso — kanso_scrim.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_effects.dart';

/// A veil over what is behind a drawer, a sheet or a card.
class KansoScrim extends StatelessWidget {
  const KansoScrim({this.child, super.key});

  final Widget? child;

  /// The veil's colour on a light theme: dark, with the warmth of paper.
  static const Color _warmDark = Color(0xFF2B2418);

  @override
  Widget build(BuildContext context) {
    final dark = Theme.brightnessOf(context) == Brightness.dark;
    return ColoredBox(
      color: dark
          ? Colors.black.withValues(alpha: KansoEffects.scrimDark)
          : _warmDark.withValues(alpha: KansoEffects.scrimLight),
      child: child,
    );
  }
}
