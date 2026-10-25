// Kanso — note_preview_mock_page.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/utils/context_extensions.dart';
import '../../models/data/design_layout.dart';
import '../widgets/note_preview_pane.dart';

/// A note rendered, on a phone.
class NotePreviewMockPage extends StatelessWidget {
  const NotePreviewMockPage({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      resizeToAvoidBottomInset: false,
      backgroundColor: context.colors.editor,
      body: const NotePreviewPane(layout: DesignLayout.phone),
    );
  }
}
