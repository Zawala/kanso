// Kanso — note_list_navigation_sync.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/state/list_navigation/list_navigation_bloc.dart';
import '../../state/note_list/note_list_bloc.dart';

/// Tells the `ListNavigationBloc` which notes the list above it holds, in
/// order, whenever that changes, so the keyboard can step through them.
class NoteListNavigationSync extends StatelessWidget {
  const NoteListNavigationSync({required this.child, super.key});

  final Widget child;

  @override
  Widget build(BuildContext context) {
    return BlocListener<NoteListBloc, NoteListState>(
      listenWhen: (before, after) => !_sameNotes(before, after),
      listener: (context, state) {
        context.read<ListNavigationBloc>().add(
          ListNavigationEvent.listShown(<String>[
            for (final row in state.rows) row.id,
          ]),
        );
      },
      child: child,
    );
  }

  static bool _sameNotes(NoteListState before, NoteListState after) {
    if (before.rows.length != after.rows.length) {
      return false;
    }
    for (var i = 0; i < before.rows.length; i++) {
      if (before.rows[i].id != after.rows[i].id) {
        return false;
      }
    }
    return true;
  }
}
