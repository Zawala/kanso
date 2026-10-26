// Kanso — list_note_layout.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

import '../../../core/configs/kanso_sizes.dart';
import '../../../core/views/widgets/kanso_divider.dart';

/// Two panes side by side: the list at its fixed width and the note, which
/// takes the rest.
class ListNoteLayout extends StatelessWidget {
  const ListNoteLayout({required this.list, required this.note, super.key});

  final Widget list;
  final Widget note;

  @override
  Widget build(BuildContext context) {
    return Row(
      children: <Widget>[
        SizedBox(width: KansoSizes.listPane, child: list),
        const KansoDivider(vertical: true),
        Expanded(child: note),
      ],
    );
  }
}
