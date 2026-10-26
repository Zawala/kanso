// Kanso — kanso_text_context_menu.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_radii.dart';
import '../../configs/kanso_shadows.dart';
import '../../configs/kanso_spacing.dart';
import '../../utils/context_extensions.dart';
import 'kanso_divider.dart';
import 'kanso_pressable.dart';

/// The menu over selected or tapped text in a field: Cut, Copy, Paste and
/// Select All, in the buttons the field offers for what is selected, drawn
/// by Kanso so it is the same on iOS and Android.
///
/// It is made for `EditableText.contextMenuBuilder`: it sits above the
/// selection, or below it where there is no room above.
class KansoTextContextMenu extends StatelessWidget {
  const KansoTextContextMenu({required this.state, super.key});

  final EditableTextState state;

  /// The most the menu is wide, in points.
  static const double maxWidth = 320;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final anchors = state.contextMenuAnchors;
    final buttons = <(String, VoidCallback)>[
      for (final item in state.contextMenuButtonItems)
        if (item.onPressed != null)
          if (switch (item.type) {
                ContextMenuButtonType.cut => l10n.menuCut,
                ContextMenuButtonType.copy => l10n.menuCopy,
                ContextMenuButtonType.paste => l10n.menuPaste,
                ContextMenuButtonType.selectAll => l10n.menuSelectAll,
                _ => null,
              }
              case final label?)
            (label, item.onPressed!),
    ];
    if (buttons.isEmpty) {
      return const SizedBox.shrink();
    }
    return Padding(
      padding: EdgeInsets.all(MediaQuery.paddingOf(context).top),
      child: CustomSingleChildLayout(
        delegate: TextSelectionToolbarLayoutDelegate(
          anchorAbove: anchors.primaryAnchor - const Offset(0, KansoSpacing.xs),
          anchorBelow:
              (anchors.secondaryAnchor ?? anchors.primaryAnchor) +
              const Offset(0, KansoSpacing.xs),
        ),
        child: Container(
          constraints: const BoxConstraints(maxWidth: maxWidth),
          decoration: BoxDecoration(
            color: colors.elevated,
            borderRadius: KansoRadii.mediumBorder,
            border: Border.all(color: colors.divider),
            boxShadow: KansoShadows.floating(Theme.brightnessOf(context)),
          ),
          child: ClipRRect(
            borderRadius: KansoRadii.mediumBorder,
            child: IntrinsicHeight(
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: <Widget>[
                  for (var i = 0; i < buttons.length; i++) ...<Widget>[
                    if (i > 0) const KansoDivider(vertical: true),
                    KansoPressable(
                      onTap: () {
                        buttons[i].$2();
                      },
                      child: Padding(
                        padding: const EdgeInsets.symmetric(
                          horizontal: KansoSpacing.md,
                          vertical: KansoSpacing.sm,
                        ),
                        child: Text(
                          buttons[i].$1,
                          style: context.typography.body,
                        ),
                      ),
                    ),
                  ],
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}
