// Kanso — kanso_scrim.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_effects.dart';

/// A veil over what is behind a drawer, a sheet or a card.
class KansoScrim extends StatelessWidget {
  const KansoScrim({this.child, super.key});

  final Widget? child;

  @override
  Widget build(BuildContext context) {
    final dark = Theme.brightnessOf(context) == Brightness.dark;
    return ColoredBox(
      color: Colors.black.withValues(
        alpha: dark ? KansoEffects.scrimDark : KansoEffects.scrimLight,
      ),
      child: child,
    );
  }
}
