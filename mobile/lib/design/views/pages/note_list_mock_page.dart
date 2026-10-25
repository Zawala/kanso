// Kanso — note_list_mock_page.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/utils/context_extensions.dart';
import '../../models/data/design_layout.dart';
import '../widgets/note_list_pane.dart';

/// The note list on a phone.
class NoteListMockPage extends StatelessWidget {
  const NoteListMockPage({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      resizeToAvoidBottomInset: false,
      backgroundColor: context.colors.list,
      body: const NoteListPane(layout: DesignLayout.phone),
    );
  }
}
