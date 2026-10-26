// Kanso — kanso_text_context_menu.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../utils/context_extensions.dart';
import 'kanso_text_menu_toolbar.dart';

/// The menu over selected or tapped text in a field: Cut, Copy, Paste and
/// Select All, in the buttons the field offers for what is selected, drawn
/// by Kanso so it is the same on iOS and Android.
///
/// It is made for `EditableText.contextMenuBuilder`: it sits above the
/// selection, or below it where there is no room above.
class KansoTextContextMenu extends StatelessWidget {
  const KansoTextContextMenu({
    required this.state,
    this.extra = const <(String, VoidCallback)>[],
    super.key,
  });

  final EditableTextState state;

  /// Buttons of the screen's own, before the field's (Open link).
  final List<(String, VoidCallback)> extra;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final buttons = <(String, VoidCallback)>[
      ...extra,
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
    return KansoTextMenuToolbar(
      anchors: state.contextMenuAnchors,
      buttons: buttons,
    );
  }
}
