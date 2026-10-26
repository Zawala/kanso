// Kanso — note_list_pane.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/browse_location.dart';
import '../../../core/models/data/search_scope_kind.dart';
import '../../../core/state/library_index/library_index_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/place_labels.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_message_card.dart';
import '../../models/data/note_list_status.dart';
import '../../state/note_list/note_list_bloc.dart';
import 'empty_place_pane.dart';
import 'list_options_sheet.dart';
import 'loading_list_pane.dart';
import 'note_list_top_bar.dart';
import 'note_list_view.dart';

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

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    return ColoredBox(
      color: colors.list,
      child: BlocBuilder<LibraryIndexBloc, LibraryIndexState>(
        builder: (context, index) {
          return BlocBuilder<NoteListBloc, NoteListState>(
            builder: (context, state) {
              final bloc = context.read<NoteListBloc>();
              final title = placeTitle(l10n, state.place, index);
              final here = BrowseLocation.place(state.place);
              final ready = state.status == NoteListStatus.ready;
              return Column(
                children: <Widget>[
                  NoteListTopBar(
                    title: title,
                    subtitle: ready
                        ? l10n.noteListCount(state.rows.length)
                        : null,
                    onSearch: () => context.go(
                      BrowseLocation.search(
                        scope: state.place is ListPlace_Notebook
                            ? SearchScopeKind.notebook
                            : SearchScopeKind.all,
                        notebookId: switch (state.place) {
                          ListPlace_Notebook(:final id) => id,
                          _ => null,
                        },
                      ).location,
                    ),
                    onOptions: () => showKansoBottomSheet<void>(
                      context,
                      builder: (_) => BlocProvider<NoteListBloc>.value(
                        value: bloc,
                        child: const ListOptionsSheet(),
                      ),
                    ),
                  ),
                  Expanded(
                    child: switch (state.status) {
                      NoteListStatus.loading => const LoadingListPane(),
                      NoteListStatus.failed => Center(
                        child: Padding(
                          padding: const EdgeInsets.all(KansoSpacing.md),
                          child: KansoMessageCard(
                            icon: KansoIcons.files,
                            title: l10n.noteListFailedTitle,
                            message: l10n.noteListFailedMessage,
                            actions: <Widget>[
                              KansoButton(
                                label: l10n.retry,
                                onPressed: () =>
                                    bloc.add(const NoteListEvent.retried()),
                              ),
                            ],
                          ),
                        ),
                      ),
                      NoteListStatus.ready when state.rows.isEmpty =>
                        EmptyPlacePane(
                          place: state.place,
                          name: switch (state.place) {
                            ListPlace_Notebook(:final id) =>
                              index.notebookName(id) ?? title,
                            ListPlace_Tag(:final id) =>
                              index.tag(id)?.name ?? title,
                            _ => title,
                          },
                          narrowed: state.options.narrowed,
                        ),
                      NoteListStatus.ready => NoteListView(
                        state: state,
                        index: index,
                        location: here,
                        openNoteId: openNoteId,
                      ),
                    },
                  ),
                ],
              );
            },
          );
        },
      ),
    );
  }
}
