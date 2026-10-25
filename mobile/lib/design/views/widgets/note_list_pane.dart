// Kanso — note_list_pane.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_divider.dart';
import '../../../core/views/widgets/kanso_section_header.dart';
import '../../models/data/design_layout.dart';
import '../../models/data/mock_data.dart';
import 'mock_note_row.dart';
import 'new_note_button.dart';
import 'note_list_top_bar.dart';

/// The list of notes, grouped by date, with a pinned row first.
///
/// On a phone it has a menu button that opens the library, search and more
/// in its bar, and a button for a new note afloat in the corner. The list
/// has room below its last row for that button, and its rows fade out
/// beneath it, so nothing is hidden behind it for good. On a tablet the
/// new-note button is in the bar and [selectedTitle] marks the note open
/// beside it.
class NoteListPane extends StatelessWidget {
  const NoteListPane({required this.layout, this.selectedTitle, super.key});

  final DesignLayout layout;
  final String? selectedTitle;

  /// How far below its last row a phone's list scrolls: the floating
  /// button, with a gutter above and below it.
  static const double floatingClearance =
      KansoSizes.floatingButton + 2 * KansoSpacing.md;

  /// How far above the clearance the fade begins.
  static const double fadeReach = 56;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    final phone = layout == DesignLayout.phone;
    final bottom = MediaQuery.paddingOf(context).bottom;
    return ColoredBox(
      color: colors.list,
      child: Column(
        children: <Widget>[
          NoteListTopBar(
            layout: layout,
            title: l10n.libraryAllNotes,
            subtitle: l10n.noteListCount(MockData.allNotesCount),
          ),
          Expanded(
            child: Stack(
              children: <Widget>[
                ListView(
                  padding: EdgeInsets.only(
                    top: KansoSpacing.xxs,
                    bottom:
                        bottom + (phone ? floatingClearance : KansoSpacing.md),
                  ),
                  children: <Widget>[
                    for (final group in MockData.noteGroups) ...<Widget>[
                      KansoSectionHeader(
                        label: group.label == 'Pinned'
                            ? l10n.libraryPinned
                            : group.label,
                      ),
                      for (var i = 0; i < group.notes.length; i++) ...<Widget>[
                        if (i > 0) const KansoDivider(indent: KansoSpacing.md),
                        MockNoteRow(
                          note: group.notes[i],
                          selected: group.notes[i].title == selectedTitle,
                        ),
                      ],
                    ],
                  ],
                ),
                if (phone) ...<Widget>[
                  PositionedDirectional(
                    start: 0,
                    end: 0,
                    bottom: 0,
                    height: bottom + floatingClearance + fadeReach,
                    child: IgnorePointer(
                      child: DecoratedBox(
                        decoration: BoxDecoration(
                          gradient: LinearGradient(
                            begin: Alignment.topCenter,
                            end: Alignment.bottomCenter,
                            colors: <Color>[
                              colors.list.withValues(alpha: 0),
                              colors.list.withValues(alpha: 0.9),
                              colors.list.withValues(alpha: 0.96),
                            ],
                            stops: const <double>[0, 0.65, 1],
                          ),
                        ),
                      ),
                    ),
                  ),
                  PositionedDirectional(
                    end: KansoSpacing.md,
                    bottom: bottom + KansoSpacing.md,
                    child: const NewNoteButton(),
                  ),
                ],
              ],
            ),
          ),
        ],
      ),
    );
  }
}
