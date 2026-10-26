// Kanso — note_edit_pane.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../editor/views/widgets/completion_popup.dart';
import '../../../editor/views/widgets/keyboard_accessory_bar.dart';
import '../../../editor/views/widgets/properties_row.dart';
import '../../../note/models/data/note_mode.dart';
import '../../../note/views/widgets/note_mode_toggle.dart';
import '../../../note/views/widgets/note_top_bar.dart';
import '../../models/data/design_layout.dart';
import '../../models/data/mock_data.dart';
import 'keyboard_placeholder.dart';
import 'note_source_body.dart';

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
          NoteTopBar(
            phone: layout == DesignLayout.phone,
            onBack: () {},
            onGoBack: () {},
            onGoForward: () {},
            trailing: <Widget>[
              NoteModeToggle(mode: NoteMode.edit, onChanged: (_) {}),
              KansoIconButton(
                icon: KansoIcons.info,
                semanticLabel: context.l10n.noteInfo,
                onPressed: () {},
              ),
            ],
          ),
          Expanded(
            child: ListView(
              padding: EdgeInsets.symmetric(horizontal: gutter),
              children: <Widget>[
                Text(
                  MockData.openNoteTitle,
                  style: context.typography.editorTitle,
                ),
                const PropertiesRow(count: MockData.editPropertiesCount),
                const NoteSourceBody(
                  blocks: MockData.editBlocks,
                  typing: MockData.editTyping,
                ),
                const SizedBox(height: KansoSpacing.xs),
                CompletionPopup(
                  completion: Completion(
                    kind: CompletionKind.wiki,
                    replace: const Range16(start: 0, end: 0),
                    items: <CompletionItem>[
                      for (final choice in MockData.completions)
                        CompletionItem(
                          label: choice.title,
                          detail: choice.notebook,
                          highlight: const Range16(start: 0, end: 2),
                          insert: '',
                          newNote: false,
                        ),
                    ],
                  ),
                  activeIndex: 0,
                  onChoose: (_) {},
                ),
              ],
            ),
          ),
          KeyboardAccessoryBar(
            editorContext: const EditorContext(
              inList: true,
              inTable: false,
              inCode: false,
              completionHere: false,
              history: HistoryState(canUndo: true, canRedo: false),
            ),
            onCommand: (_) {},
            onUndo: () {},
            onRedo: () {},
          ),
          const KeyboardPlaceholder(),
        ],
      ),
    );
  }
}
