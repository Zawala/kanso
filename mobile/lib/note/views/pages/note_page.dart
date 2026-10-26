// Kanso — note_page.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../shell/views/widgets/shell_back_button.dart';
import '../../../shell/views/widgets/shell_page.dart';

/// One note, opened from a place or a search.
class NotePage extends StatelessWidget {
  const NotePage({required this.noteId, this.inPane = false, super.key});

  final String noteId;

  /// Whether it is the note pane of a wide window rather than a page of its
  /// own, with nowhere to go back to.
  final bool inPane;

  @override
  Widget build(BuildContext context) {
    return ShellPage(
      title: '',
      leading: inPane ? null : const ShellBackButton(),
    );
  }
}
