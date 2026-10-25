// Kanso — note_preview_pane.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/note_status.dart';
import '../../../core/utils/context_extensions.dart';
import '../../models/data/design_layout.dart';
import '../../models/data/mock_data.dart';
import 'note_bottom_bar.dart';
import 'note_metadata_block.dart';
import 'note_preview_body.dart';
import 'note_top_bar.dart';

/// A note, rendered: its title, a compact block of facts and its text.
class NotePreviewPane extends StatelessWidget {
  const NotePreviewPane({required this.layout, super.key});

  final DesignLayout layout;

  @override
  Widget build(BuildContext context) {
    final phone = layout == DesignLayout.phone;
    final gutter = phone ? KansoSpacing.md : KansoSpacing.lg;
    return ColoredBox(
      color: context.colors.editor,
      child: Column(
        children: <Widget>[
          NoteTopBar(layout: layout),
          Expanded(
            child: ListView(
              padding: EdgeInsets.fromLTRB(
                gutter,
                KansoSpacing.xxs,
                gutter,
                KansoSpacing.lg,
              ),
              children: <Widget>[
                Text(
                  MockData.openNoteTitle,
                  style: context.typography.previewTitle,
                ),
                const SizedBox(height: KansoSpacing.sm),
                const NoteMetadataBlock(
                  notebook: MockData.openNoteNotebook,
                  status: NoteStatus.active,
                  tags: MockData.openNoteTags,
                  created: MockData.openNoteCreated,
                  updated: MockData.openNoteUpdated,
                ),
                const SizedBox(height: KansoSpacing.md),
                const NotePreviewBody(blocks: MockData.noteBlocks),
              ],
            ),
          ),
          if (phone) const NoteBottomBar(),
        ],
      ),
    );
  }
}
