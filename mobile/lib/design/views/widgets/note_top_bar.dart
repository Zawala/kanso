// Kanso — note_top_bar.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../models/data/design_layout.dart';
import 'note_mode_toggle.dart';

/// The bar above a note: the way back (on a tablet, back and forward), the
/// preview and edit switch and the info button.
class NoteTopBar extends StatelessWidget {
  const NoteTopBar({
    required this.layout,
    this.editing = false,
    this.showToggle = true,
    super.key,
  });

  final DesignLayout layout;
  final bool editing;
  final bool showToggle;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final phone = layout == DesignLayout.phone;
    return KansoTopBar(
      bordered: false,
      leading: phone
          ? KansoIconButton(
              icon: KansoIcons.chevronLeft,
              semanticLabel: l10n.noteBack,
              onPressed: () {},
            )
          : Row(
              mainAxisSize: MainAxisSize.min,
              children: <Widget>[
                KansoIconButton(
                  icon: KansoIcons.arrowLeft,
                  semanticLabel: l10n.noteBack,
                  onPressed: () {},
                ),
                KansoIconButton(
                  icon: KansoIcons.arrowRight,
                  semanticLabel: l10n.noteForward,
                  onPressed: () {},
                ),
              ],
            ),
      trailing: <Widget>[
        if (showToggle) NoteModeToggle(editing: editing),
        KansoIconButton(
          icon: KansoIcons.info,
          semanticLabel: l10n.noteInfo,
          onPressed: () {},
        ),
      ],
    );
  }
}
