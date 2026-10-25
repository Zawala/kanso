// Kanso — mock_note_row.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/views/widgets/kanso_note_row.dart';
import '../../../core/views/widgets/kanso_tag_chip.dart';
import '../../models/data/mock_note.dart';

/// A [MockNote] as a row of the note list.
class MockNoteRow extends StatelessWidget {
  const MockNoteRow({required this.note, this.selected = false, super.key});

  final MockNote note;
  final bool selected;

  @override
  Widget build(BuildContext context) {
    return KansoNoteRow(
      title: note.title,
      excerpt: note.excerpt,
      notebook: note.notebook,
      time: note.time,
      status: note.status,
      pinned: note.pinned,
      locked: note.locked,
      selected: selected,
      tags: <KansoTagChip>[
        for (final tag in note.tags)
          KansoTagChip(name: tag.name, color: tag.color),
      ],
    );
  }
}
