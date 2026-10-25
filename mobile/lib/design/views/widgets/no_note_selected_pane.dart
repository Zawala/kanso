// Kanso — no_note_selected_pane.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_empty_state.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';

/// The note area of a tablet when no note is open: a bar without controls
/// and a short line that says what to do.
class NoNoteSelectedPane extends StatelessWidget {
  const NoNoteSelectedPane({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return ColoredBox(
      color: context.colors.editor,
      child: Column(
        children: <Widget>[
          const KansoTopBar(bordered: false),
          Expanded(
            child: KansoEmptyState(
              title: l10n.noNoteSelectedTitle,
              message: l10n.noNoteSelectedMessage,
            ),
          ),
        ],
      ),
    );
  }
}
