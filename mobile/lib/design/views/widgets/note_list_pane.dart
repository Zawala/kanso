// Kanso — note_list_pane.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_effects.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_divider.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../core/views/widgets/kanso_section_header.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../models/data/design_layout.dart';
import '../../models/data/mock_data.dart';
import 'mock_note_row.dart';

/// The list of notes, grouped by date, with a pinned row first.
///
/// On a phone it has a menu button that opens the library, search and more
/// in its bar, and a button for a new note afloat in the corner. On a
/// tablet the new-note button is in the bar and [selectedTitle] marks the
/// note open beside it.
class NoteListPane extends StatelessWidget {
  const NoteListPane({
    required this.layout,
    this.selectedTitle,
    super.key,
  });

  final DesignLayout layout;
  final String? selectedTitle;

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
          KansoTopBar(
            backgroundColor: colors.list,
            leading: phone
                ? KansoIconButton(
                    icon: KansoIcons.menu,
                    semanticLabel: l10n.openLibrary,
                    onPressed: () {},
                  )
                : null,
            title: l10n.libraryAllNotes,
            subtitle: l10n.noteListCount(MockData.allNotesCount),
            trailing: phone
                ? <Widget>[
                    KansoIconButton(
                      icon: KansoIcons.search,
                      semanticLabel: l10n.noteSearch,
                      onPressed: () {},
                    ),
                    KansoIconButton(
                      icon: KansoIcons.ellipsis,
                      semanticLabel: l10n.noteListMore,
                      onPressed: () {},
                    ),
                  ]
                : <Widget>[
                    KansoIconButton(
                      icon: KansoIcons.plus,
                      semanticLabel: l10n.noteListNewNote,
                      onPressed: () {},
                    ),
                  ],
          ),
          Expanded(
            child: Stack(
              children: <Widget>[
                ListView(
                  padding: EdgeInsets.only(
                    top: KansoSpacing.xxs,
                    bottom: bottom + (phone ? 88 : KansoSpacing.md),
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
                if (phone)
                  PositionedDirectional(
                    end: KansoSpacing.md,
                    bottom: bottom + KansoSpacing.md,
                    child: KansoPressable(
                      onTap: () {},
                      semanticLabel: l10n.noteListNewNote,
                      child: Container(
                        width: 56,
                        height: 56,
                        alignment: Alignment.center,
                        decoration: BoxDecoration(
                          color: colors.button,
                          shape: BoxShape.circle,
                          boxShadow: <BoxShadow>[
                            BoxShadow(
                              color: Colors.black.withValues(
                                alpha: KansoEffects.shadow,
                              ),
                              blurRadius: 16,
                              offset: const Offset(0, 6),
                            ),
                          ],
                        ),
                        child: KansoIcon(
                          KansoIcons.plus,
                          size: KansoSizes.icon + 4,
                          color: colors.buttonText,
                        ),
                      ),
                    ),
                  ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
