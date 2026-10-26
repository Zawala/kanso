// Kanso — locked_note_pane.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_message_card.dart';

/// A locked note: its title (titles are not sealed) with its lock, and in
/// place of the text a calm card that says it is locked.
///
/// Nothing else of the note is shown (not its facts, and not its text), and
/// there is nothing to press: unlocking is not part of this screen yet.
class LockedNotePane extends StatelessWidget {
  const LockedNotePane({required this.title, required this.gutter, super.key});

  final String title;

  /// The space either side of the title.
  final double gutter;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        Padding(
          padding: EdgeInsets.fromLTRB(gutter, KansoSpacing.xxs, gutter, 0),
          child: Row(
            children: <Widget>[
              Flexible(
                child: Text(
                  title.trim().isEmpty ? l10n.noteUntitled : title,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: context.typography.previewTitle,
                ),
              ),
              const SizedBox(width: KansoSpacing.xs),
              KansoIcon(KansoIcons.lock, color: colors.textMuted),
            ],
          ),
        ),
        Expanded(
          // The card is centred, and scrolls when a large device text size
          // makes it taller than the room.
          child: LayoutBuilder(
            builder: (context, box) => SingleChildScrollView(
              padding: EdgeInsets.all(gutter),
              child: ConstrainedBox(
                constraints: BoxConstraints(
                  minHeight: box.maxHeight - 2 * gutter,
                ),
                child: Center(
                  child: KansoMessageCard(
                    icon: KansoIcons.lock,
                    title: l10n.lockedTitle,
                    message: l10n.lockedMessageSealed,
                    actions: const <Widget>[],
                  ),
                ),
              ),
            ),
          ),
        ),
      ],
    );
  }
}
