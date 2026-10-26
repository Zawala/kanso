// Kanso — note_list_page.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/place_labels.dart';
import '../../../core/views/widgets/kanso_drawer_scope.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../shell/views/widgets/shell_page.dart';

/// The notes of a library place.
class NoteListPage extends StatelessWidget {
  const NoteListPage({required this.place, super.key});

  final ListPlace place;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    // Where the library is a drawer, a button opens it.
    final drawer = KansoDrawerScope.maybeOf(context);
    return ShellPage(
      title: placeLabel(l10n, place),
      leading: drawer == null
          ? null
          : KansoIconButton(
              icon: KansoIcons.menu,
              semanticLabel: l10n.openLibrary,
              onPressed: drawer.open,
            ),
      backgroundColor: context.colors.list,
    );
  }
}
