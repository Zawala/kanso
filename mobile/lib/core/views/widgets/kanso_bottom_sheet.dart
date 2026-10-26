// Kanso — kanso_bottom_sheet.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_radii.dart';
import '../../configs/kanso_shadows.dart';
import '../../configs/kanso_sizes.dart';
import '../../configs/kanso_spacing.dart';
import '../../utils/context_extensions.dart';

/// The panel of a sheet that rises from the foot of the screen: a rounded
/// surface that is as tall as its [child] and at most nine tenths of the
/// screen, no wider than a tablet's sheet and centred when the screen is
/// wider. It takes the home indicator's height as well.
///
/// The veil behind it and the way it moves in are the route's
/// (`showKansoBottomSheet`).
class KansoBottomSheet extends StatelessWidget {
  const KansoBottomSheet({required this.child, super.key});

  final Widget child;

  /// How much of the screen's height the panel may take.
  static const double heightFraction = 0.9;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    return LayoutBuilder(
      builder: (context, constraints) {
        return Align(
          alignment: Alignment.bottomCenter,
          child: ConstrainedBox(
            constraints: BoxConstraints(
              maxWidth: KansoSizes.sheetWidth,
              maxHeight: constraints.maxHeight * heightFraction,
            ),
            child: DecoratedBox(
              decoration: BoxDecoration(
                color: colors.app,
                borderRadius: const BorderRadius.vertical(
                  top: Radius.circular(KansoRadii.large),
                ),
                border: Border.all(color: colors.divider),
                boxShadow: KansoShadows.sheet(Theme.brightnessOf(context)),
              ),
              child: ClipRRect(
                borderRadius: const BorderRadius.vertical(
                  top: Radius.circular(KansoRadii.large),
                ),
                child: Padding(
                  padding: EdgeInsets.only(
                    bottom: MediaQuery.paddingOf(context).bottom,
                  ),
                  child: Column(
                    mainAxisSize: MainAxisSize.min,
                    children: <Widget>[
                      const SizedBox(height: KansoSpacing.xs),
                      Container(
                        width: 36,
                        height: 4,
                        decoration: BoxDecoration(
                          color: colors.divider,
                          borderRadius: KansoRadii.pillBorder,
                        ),
                      ),
                      Flexible(child: child),
                    ],
                  ),
                ),
              ),
            ),
          ),
        );
      },
    );
  }
}
