// Kanso — loading_list_mock_page.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../models/data/design_layout.dart';
import '../widgets/loading_list_pane.dart';

/// The list of notes on a phone while it loads: bars where the rows will
/// be.
class LoadingListMockPage extends StatelessWidget {
  const LoadingListMockPage({super.key});

  @override
  Widget build(BuildContext context) {
    return const Scaffold(
      resizeToAvoidBottomInset: false,
      body: LoadingListPane(layout: DesignLayout.phone),
    );
  }
}
