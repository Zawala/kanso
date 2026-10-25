// Kanso — kanso_skeleton.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_radii.dart';
import '../../utils/context_extensions.dart';

/// A bar in the theme's fill colour that stands where text or a control
/// will be while it loads.
///
/// It is still on purpose: no shimmer, so a loading screen is calm, costs
/// nothing to draw and looks the same in a screenshot. Without a [width] it
/// takes the width it is given.
class KansoSkeleton extends StatelessWidget {
  const KansoSkeleton({this.width, this.height = 12, super.key});

  final double? width;
  final double height;

  @override
  Widget build(BuildContext context) {
    return ExcludeSemantics(
      child: Container(
        width: width,
        height: height,
        decoration: BoxDecoration(
          color: context.colors.fill,
          borderRadius: KansoRadii.smallBorder,
        ),
      ),
    );
  }
}
