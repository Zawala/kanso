// Kanso — note_edit_mock_page.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/utils/context_extensions.dart';
import '../../models/data/design_layout.dart';
import '../widgets/note_edit_pane.dart';

/// A note as Markdown source with the keyboard up, on a phone.
class NoteEditMockPage extends StatelessWidget {
  const NoteEditMockPage({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      resizeToAvoidBottomInset: false,
      backgroundColor: context.colors.editor,
      body: const NoteEditPane(layout: DesignLayout.phone),
    );
  }
}
