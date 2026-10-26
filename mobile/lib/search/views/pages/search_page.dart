// Kanso — search_page.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/models/data/search_scope_kind.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../shell/views/widgets/shell_back_button.dart';
import '../../../shell/views/widgets/shell_page.dart';

/// A search: the words, where to look, and the notebook it was opened from.
class SearchPage extends StatelessWidget {
  const SearchPage({
    this.query = '',
    this.scope = SearchScopeKind.all,
    this.notebookId,
    super.key,
  });

  final String query;
  final SearchScopeKind scope;
  final String? notebookId;

  @override
  Widget build(BuildContext context) {
    return ShellPage(
      title: context.l10n.searchTitle,
      leading: const ShellBackButton(),
      backgroundColor: context.colors.list,
    );
  }
}
