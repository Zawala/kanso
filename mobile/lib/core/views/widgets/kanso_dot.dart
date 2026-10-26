// Kanso — kanso_dot.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_sizes.dart';
import '../../utils/muted_tag_color.dart';

/// A small round dot in a tag's colour, calmed to read on the current theme.
class KansoDot extends StatelessWidget {
  const KansoDot({required this.color, super.key});

  /// The tag's own colour, as stored; drawn calmed.
  final Color color;

  @override
  Widget build(BuildContext context) {
    final dark = Theme.brightnessOf(context) == Brightness.dark;
    return Container(
      width: KansoSizes.dot,
      height: KansoSizes.dot,
      decoration: BoxDecoration(
        color: mutedTagColor(color, dark: dark),
        shape: BoxShape.circle,
      ),
    );
  }
}
