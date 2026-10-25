// Kanso — accessory_key.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_pressable.dart';

/// A key of the keyboard accessory bar: an [icon], or a text [glyph] when
/// Lucide has no icon for it.
class AccessoryKey extends StatelessWidget {
  const AccessoryKey({
    required this.semanticLabel,
    this.icon,
    this.glyph,
    this.onTap,
    super.key,
  }) : assert(
         (icon == null) != (glyph == null),
         'Give an icon or a glyph.',
       );

  final String semanticLabel;
  final KansoIcons? icon;
  final String? glyph;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final mark = icon;
    return KansoPressable(
      onTap: onTap,
      semanticLabel: semanticLabel,
      child: SizedBox.square(
        dimension: KansoSizes.touchTarget,
        child: Center(
          child: mark != null
              ? KansoIcon(mark, color: colors.textSecondary)
              : Text(
                  glyph!,
                  style: context.typography.mono.copyWith(
                    color: colors.textSecondary,
                    fontWeight: FontWeight.w700,
                  ),
                ),
        ),
      ),
    );
  }
}
