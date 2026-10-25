// Kanso — kanso_chip.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_effects.dart';
import '../../configs/kanso_radii.dart';
import '../../configs/kanso_sizes.dart';
import '../../configs/kanso_spacing.dart';
import '../../utils/context_extensions.dart';
import 'kanso_pressable.dart';

/// A filter chip: a pill with a label, outlined in the accent colour when
/// [selected].
///
/// The pill is 32 points tall; the area that reacts to a touch is 44, so
/// chips in a wrapping row sit closer than their targets suggest.
class KansoChip extends StatelessWidget {
  const KansoChip({
    required this.label,
    this.selected = false,
    this.onTap,
    super.key,
  });

  final String label;
  final bool selected;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    return KansoPressable(
      onTap: onTap,
      child: Padding(
        padding: const EdgeInsets.symmetric(
          vertical: (KansoSizes.touchTarget - KansoSizes.chip) / 2,
        ),
        child: Container(
          height: KansoSizes.chip,
          padding: const EdgeInsets.symmetric(horizontal: KansoSpacing.sm),
          decoration: BoxDecoration(
            color: selected
                ? colors.accent.withValues(alpha: KansoEffects.wash)
                : null,
            borderRadius: KansoRadii.pillBorder,
            border: Border.all(
              color: selected ? colors.accent : colors.divider,
            ),
          ),
          child: Center(
            widthFactor: 1,
            child: Text(
              label,
              maxLines: 1,
              style: context.typography.label.copyWith(
                color: selected ? colors.text : colors.textSecondary,
              ),
            ),
          ),
        ),
      ),
    );
  }
}
