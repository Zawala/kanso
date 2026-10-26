// Kanso — tablet_layout.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_sizes.dart';
import '../../../core/views/widgets/kanso_divider.dart';

/// Three panes side by side: the library, the list and the note, apart by
/// hairlines. The library and the list have fixed widths; the note takes
/// the rest.
class TabletLayout extends StatelessWidget {
  const TabletLayout({
    required this.library,
    required this.list,
    required this.note,
    super.key,
  });

  final Widget library;
  final Widget list;
  final Widget note;

  @override
  Widget build(BuildContext context) {
    return Row(
      children: <Widget>[
        SizedBox(width: KansoSizes.libraryPane, child: library),
        const KansoDivider(vertical: true),
        SizedBox(width: KansoSizes.listPane, child: list),
        const KansoDivider(vertical: true),
        Expanded(child: note),
      ],
    );
  }
}
