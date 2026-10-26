// Kanso — navigation_history_listener.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/browse_location.dart';
import '../../../core/state/navigation_history/navigation_history_bloc.dart';

/// Opens the note going back or forward landed on, where the person is, and
/// tells the history it has.
class NavigationHistoryListener extends StatelessWidget {
  const NavigationHistoryListener({
    required this.location,
    required this.child,
    super.key,
  });

  /// Where the person is; All Notes when it is neither a place nor a search.
  final BrowseLocation? location;
  final Widget child;

  @override
  Widget build(BuildContext context) {
    return BlocListener<NavigationHistoryBloc, NavigationHistoryState>(
      listenWhen: (before, after) =>
          after.pendingNoteId != null &&
          after.pendingNoteId != before.pendingNoteId,
      listener: (context, state) {
        final here = location ?? const BrowseLocation.place(ListPlace.all());
        context.go(here.withNote(state.pendingNoteId!).location);
        context.read<NavigationHistoryBloc>().add(
          const NavigationHistoryEvent.consumed(),
        );
      },
      child: child,
    );
  }
}
