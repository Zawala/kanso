// Kanso — note_list_top_bar.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../models/data/design_layout.dart';

/// The bar above a list of notes: the title with a quiet line under it. On
/// a phone it has the menu that opens the library, search and more; on a
/// tablet it has the button for a new note.
class NoteListTopBar extends StatelessWidget {
  const NoteListTopBar({
    required this.layout,
    required this.title,
    this.subtitle,
    super.key,
  });

  final DesignLayout layout;
  final String title;
  final String? subtitle;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final phone = layout == DesignLayout.phone;
    return KansoTopBar(
      backgroundColor: context.colors.list,
      leading: phone
          ? KansoIconButton(
              icon: KansoIcons.menu,
              semanticLabel: l10n.openLibrary,
              onPressed: () {},
            )
          : null,
      title: title,
      subtitle: subtitle,
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
    );
  }
}
