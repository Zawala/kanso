// Kanso — kanso_bottom_bar.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_sizes.dart';
import '../../utils/context_extensions.dart';
import 'kanso_divider.dart';

/// The bar at the foot of a page: its [children] spread evenly.
///
/// It takes the home indicator's height as well, so its colour reaches the
/// bottom of the screen. Set [safeArea] to false when something else (the
/// keyboard) already covers that area.
class KansoBottomBar extends StatelessWidget {
  const KansoBottomBar({
    required this.children,
    this.backgroundColor,
    this.safeArea = true,
    super.key,
  });

  final List<Widget> children;
  final Color? backgroundColor;
  final bool safeArea;

  @override
  Widget build(BuildContext context) {
    return ColoredBox(
      color: backgroundColor ?? context.colors.editor,
      child: SafeArea(
        top: false,
        bottom: safeArea,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: <Widget>[
            const KansoDivider(),
            SizedBox(
              height: KansoSizes.barHeight,
              child: Row(
                mainAxisAlignment: MainAxisAlignment.spaceEvenly,
                children: children,
              ),
            ),
          ],
        ),
      ),
    );
  }
}
