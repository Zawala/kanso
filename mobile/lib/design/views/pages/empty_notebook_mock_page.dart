// Kanso — empty_notebook_mock_page.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../models/data/design_layout.dart';
import '../widgets/empty_notebook_pane.dart';

/// A notebook with no notes on a phone: a quiet message and a button for
/// the first note.
class EmptyNotebookMockPage extends StatelessWidget {
  const EmptyNotebookMockPage({super.key});

  @override
  Widget build(BuildContext context) {
    return const Scaffold(
      resizeToAvoidBottomInset: false,
      body: EmptyNotebookPane(layout: DesignLayout.phone),
    );
  }
}
