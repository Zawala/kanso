// Kanso — kanso_check_badge.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_icons.dart';
import '../../configs/kanso_sizes.dart';
import '../../utils/context_extensions.dart';
import 'kanso_icon.dart';

/// A check mark in a filled accent circle: the choice that is in use.
/// [semanticLabel] says so to a screen reader.
class KansoCheckBadge extends StatelessWidget {
  const KansoCheckBadge({required this.semanticLabel, super.key});

  final String semanticLabel;

  /// The badge's diameter.
  static const double diameter = 22;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    return Container(
      width: diameter,
      height: diameter,
      alignment: Alignment.center,
      decoration: BoxDecoration(color: colors.accent, shape: BoxShape.circle),
      child: KansoIcon(
        KansoIcons.check,
        size: KansoSizes.iconSmall,
        color: colors.buttonText,
        semanticLabel: semanticLabel,
      ),
    );
  }
}
