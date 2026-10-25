// Kanso — locked_note_mock_page.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/utils/context_extensions.dart';
import '../../models/data/design_layout.dart';
import '../widgets/locked_note_pane.dart';

/// A locked note on a phone.
class LockedNoteMockPage extends StatelessWidget {
  const LockedNoteMockPage({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      resizeToAvoidBottomInset: false,
      backgroundColor: context.colors.editor,
      body: const LockedNotePane(layout: DesignLayout.phone),
    );
  }
}
