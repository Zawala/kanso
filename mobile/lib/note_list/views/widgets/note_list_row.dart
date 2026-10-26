// Kanso — note_list_row.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/state/library_index/library_index_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/status_label.dart';
import '../../../core/utils/tag_row_color.dart';
import '../../../core/views/widgets/kanso_note_row.dart';
import '../../../core/views/widgets/kanso_tag_chip.dart';

/// A note of the library as a row of the note list: its title, a locked
/// note's lock, its text, the notebook it is in, its status and tags, and
/// when it was last changed.
///
/// A locked note shows its title and lock and nothing else of it.
class NoteListRow extends StatelessWidget {
  const NoteListRow({
    required this.row,
    required this.index,
    required this.time,
    this.open = false,
    this.checked,
    this.onTap,
    this.onLongPress,
    super.key,
  });

  final NoteRow row;

  /// The notebooks and tags the row's names are looked up in.
  final LibraryIndexState index;

  /// When the note was last changed, as the list says it.
  final String time;

  /// Whether the note is the one open beside the list.
  final bool open;

  /// While several notes are being chosen, whether this one is.
  final bool? checked;
  final VoidCallback? onTap;
  final VoidCallback? onLongPress;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return KansoNoteRow(
      title: row.title.trim().isEmpty ? l10n.noteUntitled : row.title,
      excerpt: row.locked ? l10n.noteListLocked : row.snippet,
      notebook: index.notebookPath(row.notebookId) ?? '',
      time: time,
      status: noteStatusOf(row.status),
      pinned: row.pinned,
      locked: row.locked,
      selected: open,
      checked: checked,
      onTap: onTap,
      onLongPress: onLongPress,
      tags: <KansoTagChip>[
        for (final id in row.tags)
          if (index.tag(id) case final tag?)
            KansoTagChip(name: tag.name, color: tag.tint),
      ],
    );
  }
}
