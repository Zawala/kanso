// Kanso — search_navigation_sync.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/state/list_navigation/list_navigation_bloc.dart';
import '../../state/search/search_bloc.dart';

/// Tells the `ListNavigationBloc` which notes the results above it hold, in
/// order, whenever that changes, so the keyboard can step through them.
class SearchNavigationSync extends StatelessWidget {
  const SearchNavigationSync({required this.child, super.key});

  final Widget child;

  @override
  Widget build(BuildContext context) {
    return BlocListener<SearchBloc, SearchState>(
      listenWhen: (before, after) => !sameNotes(before, after),
      listener: (context, state) {
        context.read<ListNavigationBloc>().add(
          ListNavigationEvent.listShown(<String>[
            for (final hit in state.hits) hit.row.id,
          ]),
        );
      },
      child: child,
    );
  }

  static bool sameNotes(SearchState before, SearchState after) {
    if (before.hits.length != after.hits.length) {
      return false;
    }
    for (var i = 0; i < before.hits.length; i++) {
      if (before.hits[i].row.id != after.hits[i].row.id) {
        return false;
      }
    }
    return true;
  }
}
