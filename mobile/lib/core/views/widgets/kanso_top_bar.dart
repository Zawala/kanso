// Kanso — kanso_top_bar.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_sizes.dart';
import '../../configs/kanso_spacing.dart';
import '../../utils/context_extensions.dart';
import 'kanso_divider.dart';

/// The bar at the top of a page: [leading], a centred title and [trailing].
///
/// It takes the status bar's height as well, so its colour reaches the top
/// of the screen; the content sits below it.
class KansoTopBar extends StatelessWidget {
  const KansoTopBar({
    this.leading,
    this.title,
    this.subtitle,
    this.trailing = const <Widget>[],
    this.backgroundColor,
    this.bordered = true,
    super.key,
  });

  final Widget? leading;
  final String? title;

  /// A quiet line under the title.
  final String? subtitle;
  final List<Widget> trailing;

  /// The bar's colour; the colour of the page below when null.
  final Color? backgroundColor;

  /// A line along the bottom edge.
  final bool bordered;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final typography = context.typography;
    final name = title;
    final note = subtitle;
    return ColoredBox(
      color: backgroundColor ?? colors.editor,
      child: SafeArea(
        bottom: false,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: <Widget>[
            SizedBox(
              height: KansoSizes.barHeight,
              child: Padding(
                padding: const EdgeInsets.symmetric(
                  horizontal: KansoSpacing.xxs,
                ),
                child: NavigationToolbar(
                  leading: leading,
                  middle: name == null
                      ? null
                      : Column(
                          mainAxisSize: MainAxisSize.min,
                          children: <Widget>[
                            Text(
                              name,
                              maxLines: 1,
                              overflow: TextOverflow.ellipsis,
                              style: typography.barTitle,
                            ),
                            if (note != null)
                              Text(
                                note,
                                maxLines: 1,
                                overflow: TextOverflow.ellipsis,
                                style: typography.meta,
                              ),
                          ],
                        ),
                  trailing: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: trailing,
                  ),
                  middleSpacing: 0,
                ),
              ),
            ),
            if (bordered) const KansoDivider(),
          ],
        ),
      ),
    );
  }
}
