// Kanso — empty_notebook_pane.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_empty_state.dart';
import '../../models/data/design_layout.dart';
import '../../models/data/mock_data.dart';
import 'note_list_top_bar.dart';

/// The list of a notebook with no notes: a quiet message and the one thing
/// to do next.
class EmptyNotebookPane extends StatelessWidget {
  const EmptyNotebookPane({required this.layout, super.key});

  final DesignLayout layout;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return ColoredBox(
      color: context.colors.list,
      child: Column(
        children: <Widget>[
          NoteListTopBar(
            layout: layout,
            title: MockData.emptyNotebookName,
            subtitle: l10n.noteListCount(0),
          ),
          Expanded(
            child: KansoEmptyState(
              title: l10n.emptyNotebookTitle(MockData.emptyNotebookName),
              message: l10n.emptyNotebookMessage,
              action: KansoButton(
                label: l10n.noteListNewNote,
                icon: KansoIcons.plus,
                expanded: false,
                onPressed: () {},
              ),
            ),
          ),
        ],
      ),
    );
  }
}
