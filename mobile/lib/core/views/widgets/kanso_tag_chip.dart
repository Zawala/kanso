// Kanso — kanso_tag_chip.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_effects.dart';
import '../../configs/kanso_radii.dart';
import '../../configs/kanso_sizes.dart';
import '../../utils/context_extensions.dart';
import '../../utils/muted_tag_color.dart';

/// A tag: its name after a hash, in its own colour calmed to read on the
/// current theme, on a soft wash of the same colour.
class KansoTagChip extends StatelessWidget {
  const KansoTagChip({required this.name, required this.color, super.key});

  final String name;

  /// The tag's own colour, as stored; drawn calmed.
  final Color color;

  @override
  Widget build(BuildContext context) {
    final dark = context.brightness == Brightness.dark;
    final tint = mutedTagColor(color, dark: dark);
    return Container(
      height: KansoSizes.tagChip,
      padding: const EdgeInsets.symmetric(horizontal: 7),
      decoration: BoxDecoration(
        color: tint.withValues(alpha: KansoEffects.wash),
        borderRadius: KansoRadii.smallBorder,
      ),
      child: Center(
        widthFactor: 1,
        child: Text(
          '#$name',
          maxLines: 1,
          style: context.typography.meta.copyWith(
            color: tint,
            fontWeight: FontWeight.w500,
          ),
        ),
      ),
    );
  }
}
