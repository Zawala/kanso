// Kanso — note_not_found_pane.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_empty_state.dart';

/// What a link or a history entry to a note that is not there (any more)
/// opens: a quiet message, and nothing to press; the bars around it lead
/// back.
class NoteNotFoundPane extends StatelessWidget {
  const NoteNotFoundPane({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return KansoEmptyState(
      title: l10n.noteNotFoundTitle,
      message: l10n.noteNotFoundMessage,
    );
  }
}
