// Kanso — kanso_note_row.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_icons.dart';
import '../../configs/kanso_sizes.dart';
import '../../configs/kanso_spacing.dart';
import '../../models/data/note_status.dart';
import '../../utils/context_extensions.dart';
import 'kanso_icon.dart';
import 'kanso_pressable.dart';
import 'kanso_status_glyph.dart';
import 'kanso_tag_chip.dart';

/// A note in a list: its title, two lines of its text, and a quiet line
/// with its notebook, status, tags and time.
///
/// The row has no divider of its own; the list draws one between rows.
/// [selected] is the note open beside the list on a tablet.
class KansoNoteRow extends StatelessWidget {
  const KansoNoteRow({
    required this.title,
    required this.excerpt,
    required this.notebook,
    required this.time,
    this.status = NoteStatus.none,
    this.tags = const <KansoTagChip>[],
    this.pinned = false,
    this.locked = false,
    this.selected = false,
    this.onTap,
    super.key,
  });

  final String title;
  final String excerpt;
  final String notebook;
  final String time;
  final NoteStatus status;
  final List<KansoTagChip> tags;
  final bool pinned;
  final bool locked;
  final bool selected;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final typography = context.typography;
    return KansoPressable(
      onTap: onTap,
      child: Container(
        color: selected ? colors.rowActive : null,
        padding: const EdgeInsets.symmetric(
          horizontal: KansoSpacing.md,
          vertical: KansoSpacing.sm,
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: <Widget>[
            Row(
              children: <Widget>[
                Expanded(
                  child: Text(
                    title,
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: typography.listTitle,
                  ),
                ),
                if (locked) ...<Widget>[
                  const SizedBox(width: KansoSpacing.xs),
                  KansoIcon(
                    KansoIcons.lock,
                    size: KansoSizes.glyph,
                    color: colors.textMuted,
                  ),
                ],
                if (pinned) ...<Widget>[
                  const SizedBox(width: KansoSpacing.xs),
                  KansoIcon(
                    KansoIcons.pin,
                    size: KansoSizes.glyph,
                    color: colors.accent,
                  ),
                ],
              ],
            ),
            const SizedBox(height: KansoSpacing.xxs),
            Text(
              excerpt,
              maxLines: 2,
              overflow: TextOverflow.ellipsis,
              style: typography.secondary,
            ),
            const SizedBox(height: KansoSpacing.xs),
            Row(
              children: <Widget>[
                Expanded(
                  child: Row(
                    children: <Widget>[
                      KansoIcon(
                        KansoIcons.notebook,
                        size: KansoSizes.glyph,
                        color: colors.textMuted,
                      ),
                      const SizedBox(width: KansoSpacing.xxs),
                      Flexible(
                        child: Text(
                          notebook,
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                          style: typography.meta,
                        ),
                      ),
                      if (status != NoteStatus.none) ...<Widget>[
                        const SizedBox(width: KansoSpacing.xs),
                        KansoStatusGlyph(status: status),
                      ],
                      for (final tag in tags) ...<Widget>[
                        const SizedBox(width: KansoSpacing.xs),
                        tag,
                      ],
                    ],
                  ),
                ),
                const SizedBox(width: KansoSpacing.xs),
                Text(time, maxLines: 1, style: typography.meta),
              ],
            ),
          ],
        ),
      ),
    );
  }
}
