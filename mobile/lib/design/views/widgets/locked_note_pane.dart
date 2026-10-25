// Kanso — locked_note_pane.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_message_card.dart';
import '../../models/data/design_layout.dart';
import '../../models/data/mock_data.dart';
import 'note_bottom_bar.dart';
import 'note_metadata_block.dart';
import 'note_top_bar.dart';

/// A locked note: its title (titles are not sealed), its facts greyed, and
/// in place of the text a calm card that offers to unlock it.
class LockedNotePane extends StatelessWidget {
  const LockedNotePane({required this.layout, super.key});

  final DesignLayout layout;

  /// How long a note stays unlocked, in minutes.
  static const int lockMinutes = 10;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    final phone = layout == DesignLayout.phone;
    final gutter = phone ? KansoSpacing.md : KansoSpacing.lg;
    return ColoredBox(
      color: colors.editor,
      child: Column(
        children: <Widget>[
          NoteTopBar(layout: layout, showToggle: false),
          Padding(
            padding: EdgeInsets.fromLTRB(
              gutter,
              KansoSpacing.xxs,
              gutter,
              0,
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: <Widget>[
                Row(
                  children: <Widget>[
                    Flexible(
                      child: Text(
                        MockData.lockedNoteTitle,
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: context.typography.noteTitle,
                      ),
                    ),
                    const SizedBox(width: KansoSpacing.xs),
                    KansoIcon(
                      KansoIcons.lock,
                      color: colors.textMuted,
                    ),
                  ],
                ),
                const SizedBox(height: KansoSpacing.sm),
                const NoteMetadataBlock(
                  notebook: MockData.lockedNoteNotebook,
                  created: MockData.lockedNoteCreated,
                  updated: MockData.lockedNoteUpdated,
                  dimmed: true,
                ),
              ],
            ),
          ),
          Expanded(
            child: Center(
              child: Padding(
                padding: EdgeInsets.symmetric(horizontal: gutter),
                child: KansoMessageCard(
                  icon: KansoIcons.lock,
                  title: l10n.lockedTitle,
                  message: l10n.lockedMessage(lockMinutes),
                  actions: <Widget>[
                    KansoButton(
                      label: l10n.lockedUnlock,
                      icon: KansoIcons.scanFace,
                      onPressed: () {},
                    ),
                    KansoButton(
                      label: l10n.lockedUsePassword,
                      variant: KansoButtonVariant.secondary,
                      onPressed: () {},
                    ),
                  ],
                ),
              ),
            ),
          ),
          if (phone) const NoteBottomBar(),
        ],
      ),
    );
  }
}
