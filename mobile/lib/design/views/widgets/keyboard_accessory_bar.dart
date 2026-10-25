// Kanso — keyboard_accessory_bar.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_divider.dart';
import 'accessory_key.dart';

/// The bar of Markdown shortcuts above the keyboard: heading, list, task,
/// bold, link, image, `[[`, indent, outdent and undo. It scrolls sideways.
class KeyboardAccessoryBar extends StatelessWidget {
  const KeyboardAccessoryBar({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return ColoredBox(
      color: context.colors.sidebar,
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: <Widget>[
          const KansoDivider(),
          SizedBox(
            height: KansoSizes.touchTarget,
            child: LayoutBuilder(
              builder: (context, constraints) {
                final keys = <Widget>[
                  AccessoryKey(
                    icon: KansoIcons.hash,
                    semanticLabel: l10n.accessoryHeading,
                  ),
                  AccessoryKey(
                    icon: KansoIcons.list,
                    semanticLabel: l10n.accessoryList,
                  ),
                  AccessoryKey(
                    icon: KansoIcons.listChecks,
                    semanticLabel: l10n.accessoryTask,
                  ),
                  AccessoryKey(
                    icon: KansoIcons.bold,
                    semanticLabel: l10n.accessoryBold,
                  ),
                  AccessoryKey(
                    icon: KansoIcons.link,
                    semanticLabel: l10n.accessoryLink,
                  ),
                  AccessoryKey(
                    icon: KansoIcons.image,
                    semanticLabel: l10n.accessoryImage,
                  ),
                  AccessoryKey(
                    glyph: '[[',
                    semanticLabel: l10n.accessoryWikiLink,
                  ),
                  AccessoryKey(
                    icon: KansoIcons.indentIncrease,
                    semanticLabel: l10n.accessoryIndent,
                  ),
                  AccessoryKey(
                    icon: KansoIcons.indentDecrease,
                    semanticLabel: l10n.accessoryOutdent,
                  ),
                  AccessoryKey(
                    icon: KansoIcons.undo,
                    semanticLabel: l10n.accessoryUndo,
                  ),
                ];
                // Every key stays on screen: on a phone they share the
                // width; where there's room they keep their full size.
                final roomy =
                    constraints.maxWidth >=
                    keys.length * KansoSizes.touchTarget + KansoSpacing.xs * 2;
                return Padding(
                  padding: const EdgeInsets.symmetric(
                    horizontal: KansoSpacing.xs,
                  ),
                  child: Row(
                    children: <Widget>[
                      for (final key in keys)
                        roomy ? key : Expanded(child: key),
                    ],
                  ),
                );
              },
            ),
          ),
        ],
      ),
    );
  }
}
