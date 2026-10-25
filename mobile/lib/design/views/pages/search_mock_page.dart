// Kanso — search_mock_page.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/utils/context_extensions.dart';
import '../widgets/search_pane.dart';

/// Search results on a phone.
class SearchMockPage extends StatelessWidget {
  const SearchMockPage({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      resizeToAvoidBottomInset: false,
      backgroundColor: context.colors.list,
      body: const SearchPane(),
    );
  }
}
