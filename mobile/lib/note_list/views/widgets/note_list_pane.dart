// Kanso — note_list_pane.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/state/library_index/library_index_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../journal/views/widgets/journal_opener.dart';
import '../../../journal/views/widgets/journal_strip.dart';
import '../../state/note_list/note_list_bloc.dart';
import 'note_list_bar.dart';
import 'note_list_body.dart';
import 'note_list_notice.dart';

/// The notes of a place: a bar, and under it the list grouped by date, or
/// while it is read the shape of one, or why there is none.
///
/// It reads the `NoteListBloc` and `LibraryIndexBloc` above it. On a phone
/// the bar has the menu that opens the library, search and more; on a
/// tablet with the library beside it, only more. [openNoteId] is the note
/// open beside the list.
class NoteListPane extends StatelessWidget {
  const NoteListPane({this.openNoteId, super.key});

  final String? openNoteId;

  /// Whether [place] is the Journal notebook.
  static bool _isJournal(ListPlace place, LibraryIndexState index) =>
      place is ListPlace_Notebook &&
      index.notebook(place.id)?.role == NotebookRole.journal;

  @override
  Widget build(BuildContext context) {
    return ColoredBox(
      color: context.colors.list,
      child: BlocBuilder<LibraryIndexBloc, LibraryIndexState>(
        builder: (context, index) {
          return BlocBuilder<NoteListBloc, NoteListState>(
            builder: (context, state) {
              final bloc = context.read<NoteListBloc>();
              final error = state.actionError;
              return JournalOpener(
                child: PopScope(
                // Back, on a phone that has one, ends choosing before it
                // leaves.
                canPop: !state.selecting,
                onPopInvokedWithResult: (didPop, result) {
                  if (!didPop) {
                    bloc.add(const NoteListEvent.selectionCleared());
                  }
                },
                child: Stack(
                  children: <Widget>[
                    Column(
                      children: <Widget>[
                        NoteListBar(state: state, index: index),
                        if (_isJournal(state.place, index))
                          const JournalStrip(),
                        Expanded(
                          child: NoteListBody(
                            state: state,
                            index: index,
                            openNoteId: openNoteId,
                          ),
                        ),
                      ],
                    ),
                    if (error != null)
                      PositionedDirectional(
                        start: KansoSpacing.md,
                        end: KansoSpacing.md,
                        bottom:
                            MediaQuery.paddingOf(context).bottom +
                            KansoSpacing.md,
                        child: NoteListNotice(
                          error: error,
                          onDismiss: () => bloc.add(
                            const NoteListEvent.errorDismissed(),
                          ),
                        ),
                      ),
                  ],
                ),
                ),
              );
            },
          );
        },
      ),
    );
  }
}
