// Kanso — note_bottom_bar.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_bar.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';

/// The bar under a note on a phone: back, forward, search and settings.
class NoteBottomBar extends StatelessWidget {
  const NoteBottomBar({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return KansoBottomBar(
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
        KansoIconButton(
          icon: KansoIcons.search,
          semanticLabel: l10n.noteSearch,
          onPressed: () {},
        ),
        KansoIconButton(
          icon: KansoIcons.settings,
          semanticLabel: l10n.openSettings,
          onPressed: () {},
        ),
      ],
    );
  }
}
