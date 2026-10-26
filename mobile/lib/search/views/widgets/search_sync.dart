// Kanso — search_sync.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/models/data/search_scope_kind.dart';
import '../../state/search/search_bloc.dart';

/// Tells the `SearchBloc` above it when the address of the search changes to
/// words or a place to look that it is not already searching: a link to a
/// search, opened while one is. An address that only adds or drops the open
/// note leaves the search as it is.
class SearchSync extends StatefulWidget {
  const SearchSync({
    required this.query,
    required this.scope,
    required this.notebookId,
    required this.child,
    super.key,
  });

  final String query;
  final SearchScopeKind scope;
  final String? notebookId;
  final Widget child;

  @override
  State<SearchSync> createState() => _SearchSyncState();
}

class _SearchSyncState extends State<SearchSync> {
  @override
  void didUpdateWidget(SearchSync oldWidget) {
    super.didUpdateWidget(oldWidget);
    final changed =
        oldWidget.query != widget.query ||
        oldWidget.scope != widget.scope ||
        oldWidget.notebookId != widget.notebookId;
    if (!changed) {
      return;
    }
    final bloc = context.read<SearchBloc>();
    final state = bloc.state;
    if (state.query != widget.query ||
        state.scope != widget.scope ||
        state.notebookId != widget.notebookId) {
      bloc.add(
        SearchEvent.started(widget.query, widget.scope, widget.notebookId),
      );
    }
  }

  @override
  Widget build(BuildContext context) => widget.child;
}
