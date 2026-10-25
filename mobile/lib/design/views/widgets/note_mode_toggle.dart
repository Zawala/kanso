// Kanso — note_mode_toggle.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_segment.dart';
import '../../../core/views/widgets/kanso_segmented.dart';

/// The switch between a note rendered and a note as text to edit.
class NoteModeToggle extends StatelessWidget {
  const NoteModeToggle({required this.editing, super.key});

  final bool editing;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return KansoSegmented<bool>(
      selected: editing,
      onChanged: (_) {},
      segments: <KansoSegment<bool>>[
        KansoSegment<bool>(
          value: false,
          icon: KansoIcons.eye,
          semanticLabel: l10n.notePreview,
        ),
        KansoSegment<bool>(
          value: true,
          icon: KansoIcons.pencil,
          semanticLabel: l10n.noteEdit,
        ),
      ],
    );
  }
}
