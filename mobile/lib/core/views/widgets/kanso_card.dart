// Kanso — kanso_card.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_radii.dart';
import '../../configs/kanso_spacing.dart';
import '../../utils/context_extensions.dart';

/// A rounded surface with a hairline border, for content that stands apart
/// from the page.
class KansoCard extends StatelessWidget {
  const KansoCard({
    required this.child,
    this.padding = const EdgeInsets.all(KansoSpacing.md),
    this.color,
    this.borderColor,
    this.borderWidth = 1,
    super.key,
  });

  final Widget child;
  final EdgeInsetsGeometry padding;

  /// The fill; the elevated colour when null.
  final Color? color;

  /// The border's colour; the divider colour when null.
  final Color? borderColor;
  final double borderWidth;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    return Container(
      padding: padding,
      decoration: BoxDecoration(
        color: color ?? colors.elevated,
        borderRadius: KansoRadii.largeBorder,
        border: Border.all(
          color: borderColor ?? colors.divider,
          width: borderWidth,
        ),
      ),
      child: child,
    );
  }
}
