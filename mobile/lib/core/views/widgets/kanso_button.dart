// Kanso — kanso_button.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_effects.dart';
import '../../configs/kanso_icons.dart';
import '../../configs/kanso_radii.dart';
import '../../configs/kanso_sizes.dart';
import '../../configs/kanso_spacing.dart';
import '../../utils/context_extensions.dart';
import 'kanso_icon.dart';
import 'kanso_pressable.dart';

/// How loud a [KansoButton] is.
enum KansoButtonVariant {
  /// The one action the page is for: filled.
  primary,

  /// An alternative: a quiet fill.
  secondary,

  /// A way out: no fill.
  quiet,

  /// Something that cannot be undone: the danger colour on a wash.
  danger,
}

/// A button with a label and, optionally, an icon before it.
///
/// It is 48 points tall (36 when [compact]) and as wide as its parent allows
/// when [expanded]. Without [onPressed] it is drawn dimmed.
class KansoButton extends StatelessWidget {
  const KansoButton({
    required this.label,
    this.onPressed,
    this.variant = KansoButtonVariant.primary,
    this.icon,
    this.expanded = true,
    this.compact = false,
    super.key,
  });

  final String label;
  final VoidCallback? onPressed;
  final KansoButtonVariant variant;
  final KansoIcons? icon;
  final bool expanded;
  final bool compact;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final (fill, ink) = switch (variant) {
      KansoButtonVariant.primary => (colors.button, colors.buttonText),
      KansoButtonVariant.secondary => (colors.fill, colors.text),
      KansoButtonVariant.quiet => (null, colors.textSecondary),
      KansoButtonVariant.danger => (
        colors.danger.withValues(alpha: KansoEffects.wash),
        colors.danger,
      ),
    };
    final glyph = icon;
    final face = Container(
      height: compact ? KansoSizes.touchTarget - 8 : KansoSizes.button,
      padding: EdgeInsets.symmetric(
        horizontal: compact ? KansoSpacing.md : KansoSpacing.lg,
      ),
      decoration: BoxDecoration(
        color: fill,
        borderRadius: KansoRadii.mediumBorder,
      ),
      child: Center(
        widthFactor: 1,
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: <Widget>[
            if (glyph != null) ...<Widget>[
              KansoIcon(glyph, size: KansoSizes.iconSmall, color: ink),
              const SizedBox(width: KansoSpacing.xs),
            ],
            Flexible(
              child: Text(
                label,
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
                style: context.typography.listTitle.copyWith(color: ink),
              ),
            ),
          ],
        ),
      ),
    );
    return KansoPressable(
      onTap: onPressed,
      child: Opacity(
        opacity: onPressed == null ? KansoEffects.disabled : 1,
        child: expanded ? SizedBox(width: double.infinity, child: face) : face,
      ),
    );
  }
}
