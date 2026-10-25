// Kanso — kanso_badge.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_effects.dart';
import '../../configs/kanso_radii.dart';
import '../../utils/context_extensions.dart';

/// How a [KansoBadge] looks.
enum KansoBadgeTone {
  /// A quiet fill, for a fact.
  neutral,

  /// The accent colour, for something that is on or yours.
  accent,
}

/// A short label in a small rounded box: "Built in", "Installed".
class KansoBadge extends StatelessWidget {
  const KansoBadge({
    required this.label,
    this.tone = KansoBadgeTone.neutral,
    super.key,
  });

  final String label;
  final KansoBadgeTone tone;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final accent = tone == KansoBadgeTone.accent;
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 7, vertical: 2),
      decoration: BoxDecoration(
        // A wash of the text colour, not the `fill` token: on a card in a
        // dark theme `fill` is the card's own colour and would vanish.
        color: accent
            ? colors.accent.withValues(alpha: KansoEffects.wash)
            : colors.text.withValues(alpha: KansoEffects.quietWash),
        borderRadius: KansoRadii.smallBorder,
      ),
      child: Text(
        label,
        maxLines: 1,
        style: context.typography.meta.copyWith(
          fontWeight: FontWeight.w500,
          color: accent ? colors.text : colors.textSecondary,
        ),
      ),
    );
  }
}
