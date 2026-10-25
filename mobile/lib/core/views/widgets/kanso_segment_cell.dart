// Kanso — kanso_segment_cell.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_effects.dart';
import '../../configs/kanso_motion.dart';
import '../../configs/kanso_radii.dart';
import '../../configs/kanso_sizes.dart';
import '../../configs/kanso_spacing.dart';
import '../../utils/context_extensions.dart';
import 'kanso_icon.dart';
import 'kanso_pressable.dart';
import 'kanso_segment.dart';
import 'kanso_segmented.dart';

/// One choice inside a `KansoSegmented`.
class KansoSegmentCell<T> extends StatelessWidget {
  const KansoSegmentCell({
    required this.segment,
    required this.active,
    required this.onTap,
    super.key,
  });

  final KansoSegment<T> segment;
  final bool active;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final ink = active ? colors.text : colors.textSecondary;
    final glyph = segment.icon;
    final label = segment.label;
    return KansoPressable(
      onTap: onTap,
      semanticLabel: segment.semanticLabel ?? label,
      child: AnimatedContainer(
        duration: KansoMotion.resolve(context, KansoMotion.short),
        height: KansoSizes.touchTarget - 2 * KansoSegmented.track,
        constraints: BoxConstraints(
          minWidth: label == null ? KansoSizes.touchTarget : 0,
        ),
        padding: const EdgeInsets.symmetric(horizontal: KansoSpacing.sm),
        alignment: Alignment.center,
        decoration: BoxDecoration(
          color: active
              ? colors.accent.withValues(alpha: KansoEffects.wash)
              : null,
          borderRadius: KansoRadii.smallBorder,
        ),
        // At a large device text size a label can outgrow its cell; it
        // shrinks to fit rather than overflow, and stays as large as fits.
        child: FittedBox(
          child: Row(
            mainAxisSize: MainAxisSize.min,
            mainAxisAlignment: MainAxisAlignment.center,
            children: <Widget>[
              if (glyph != null)
                KansoIcon(
                  glyph,
                  size: KansoSizes.iconSmall + 2,
                  color: active ? colors.accent : ink,
                ),
              if (glyph != null && label != null)
                const SizedBox(width: KansoSpacing.xxs),
              if (label != null)
                Text(
                  label,
                  maxLines: 1,
                  style: context.typography.label.copyWith(color: ink),
                ),
            ],
          ),
        ),
      ),
    );
  }
}
