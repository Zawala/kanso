// Kanso — note_bottom_bar.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_bar.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';

/// The bar under a note on a phone: back and forward through the notes
/// opened, search and settings. A way that leads nowhere is dimmed.
class NoteBottomBar extends StatelessWidget {
  const NoteBottomBar({
    this.onGoBack,
    this.onGoForward,
    this.onSearch,
    this.onSettings,
    super.key,
  });

  final VoidCallback? onGoBack;
  final VoidCallback? onGoForward;
  final VoidCallback? onSearch;
  final VoidCallback? onSettings;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return KansoBottomBar(
      children: <Widget>[
        KansoIconButton(
          icon: KansoIcons.arrowLeft,
          semanticLabel: l10n.noteBack,
          onPressed: onGoBack,
        ),
        KansoIconButton(
          icon: KansoIcons.arrowRight,
          semanticLabel: l10n.noteForward,
          onPressed: onGoForward,
        ),
        KansoIconButton(
          icon: KansoIcons.search,
          semanticLabel: l10n.noteSearch,
          onPressed: onSearch,
        ),
        KansoIconButton(
          icon: KansoIcons.settings,
          semanticLabel: l10n.openSettings,
          onPressed: onSettings,
        ),
      ],
    );
  }
}
