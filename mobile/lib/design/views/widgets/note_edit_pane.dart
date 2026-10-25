// Kanso — note_edit_pane.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../models/data/design_layout.dart';
import '../../models/data/mock_data.dart';
import 'keyboard_accessory_bar.dart';
import 'keyboard_placeholder.dart';
import 'note_source_body.dart';
import 'note_top_bar.dart';
import 'properties_row.dart';
import 'wiki_completion_popup.dart';

/// A note as Markdown source with the keyboard up: the properties
/// collapsed above the text, a link to another note being typed with its
/// completions open, and the accessory bar over the keyboard.
class NoteEditPane extends StatelessWidget {
  const NoteEditPane({required this.layout, super.key});

  final DesignLayout layout;

  @override
  Widget build(BuildContext context) {
    final gutter = layout == DesignLayout.phone
        ? KansoSpacing.md
        : KansoSpacing.lg;
    return ColoredBox(
      color: context.colors.editor,
      child: Column(
        children: <Widget>[
          NoteTopBar(layout: layout, editing: true),
          Expanded(
            child: ListView(
              padding: EdgeInsets.symmetric(horizontal: gutter),
              children: <Widget>[
                Text(
                  MockData.openNoteTitle,
                  style: context.typography.noteTitle,
                ),
                const PropertiesRow(count: MockData.editPropertiesCount),
                const NoteSourceBody(
                  blocks: MockData.editBlocks,
                  typing: MockData.editTyping,
                ),
                const SizedBox(height: KansoSpacing.xs),
                const WikiCompletionPopup(
                  typed: 'Sy',
                  completions: MockData.completions,
                ),
              ],
            ),
          ),
          const KeyboardAccessoryBar(),
          const KeyboardPlaceholder(),
        ],
      ),
    );
  }
}
