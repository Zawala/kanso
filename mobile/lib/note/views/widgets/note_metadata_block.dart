// Kanso — note_metadata_block.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/note_status.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/status_label.dart';
import '../../../core/utils/tag_row_color.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_status_glyph.dart';
import '../../../core/views/widgets/kanso_tag_chip.dart';

/// The facts of a note under its title: notebook and status, tags, and when
/// it was made and last changed.
class NoteMetadataBlock extends StatelessWidget {
  const NoteMetadataBlock({
    required this.notebook,
    required this.created,
    required this.updated,
    this.status = NoteStatus.none,
    this.tags = const <TagRow>[],
    super.key,
  });

  /// Where the note is, as a path (`Projects / Kanso`).
  final String notebook;

  /// When it was made and last changed, as the screen says it.
  final String created;
  final String updated;
  final NoteStatus status;
  final List<TagRow> tags;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final typography = context.typography;
    final l10n = context.l10n;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: <Widget>[
        Row(
          children: <Widget>[
            KansoIcon(
              KansoIcons.notebook,
              size: KansoSizes.iconSmall,
              color: colors.textMuted,
            ),
            const SizedBox(width: KansoSpacing.xs),
            Flexible(
              child: Text(
                notebook,
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
                style: typography.label.copyWith(color: colors.text),
              ),
            ),
            if (status != NoteStatus.none) ...<Widget>[
              const SizedBox(width: KansoSpacing.md),
              KansoStatusGlyph(status: status, size: KansoSizes.iconSmall),
              const SizedBox(width: KansoSpacing.xs),
              Text(
                l10n.statusName(status),
                maxLines: 1,
                style: typography.label.copyWith(color: colors.text),
              ),
            ],
          ],
        ),
        if (tags.isNotEmpty) ...<Widget>[
          const SizedBox(height: KansoSpacing.xs),
          Wrap(
            spacing: KansoSpacing.xs,
            runSpacing: KansoSpacing.xs,
            children: <Widget>[
              for (final tag in tags)
                KansoTagChip(name: tag.name, color: tag.tint),
            ],
          ),
        ],
        const SizedBox(height: KansoSpacing.xs),
        Wrap(
          spacing: KansoSpacing.md,
          runSpacing: KansoSpacing.xxs,
          children: <Widget>[
            Text('${l10n.noteMetaCreated} $created', style: typography.meta),
            Text('${l10n.noteMetaUpdated} $updated', style: typography.meta),
          ],
        ),
      ],
    );
  }
}
