// Kanso — search_page.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/models/data/search_scope_kind.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../injection.dart';
import '../../state/search/search_bloc.dart';
import '../widgets/search_pane.dart';
import '../widgets/search_sync.dart';

/// A search: the words, where to look, and the notebook it was opened from.
/// The words, scope and notebook are the address's to start from; what is
/// typed after is the search's own.
class SearchPage extends StatelessWidget {
  const SearchPage({
    this.query = '',
    this.scope = SearchScopeKind.all,
    this.notebookId,
    this.openNoteId,
    super.key,
  });

  final String query;
  final SearchScopeKind scope;
  final String? notebookId;

  /// The note open beside the results, in a window with room for both.
  final String? openNoteId;

  @override
  Widget build(BuildContext context) {
    return BlocProvider<SearchBloc>(
      create: (_) =>
          getIt<SearchBloc>()
            ..add(SearchEvent.started(query, scope, notebookId)),
      child: SearchSync(
        query: query,
        scope: scope,
        notebookId: notebookId,
        child: Scaffold(
          resizeToAvoidBottomInset: false,
          backgroundColor: context.colors.list,
          body: SearchPane(initialQuery: query, openNoteId: openNoteId),
        ),
      ),
    );
  }
}
