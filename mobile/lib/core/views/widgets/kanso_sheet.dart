// Kanso — kanso_sheet.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_radii.dart';
import '../../configs/kanso_shadows.dart';
import '../../configs/kanso_sizes.dart';
import '../../utils/context_extensions.dart';

/// A floating sheet: a rounded surface with a soft shadow, [width] wide,
/// for a tablet's modal pages such as settings.
class KansoSheet extends StatelessWidget {
  const KansoSheet({
    required this.child,
    this.width = KansoSizes.sheetWidth,
    this.height,
    super.key,
  });

  final Widget child;
  final double width;
  final double? height;

  @override
  Widget build(BuildContext context) {
    return Container(
      width: width,
      height: height,
      clipBehavior: Clip.antiAlias,
      decoration: BoxDecoration(
        color: context.colors.app,
        borderRadius: KansoRadii.largeBorder,
        border: Border.all(color: context.colors.divider),
        boxShadow: KansoShadows.sheet(Theme.brightnessOf(context)),
      ),
      // A sheet floats clear of the status bar and the home indicator.
      child: MediaQuery.removePadding(
        context: context,
        removeTop: true,
        removeBottom: true,
        child: child,
      ),
    );
  }
}
