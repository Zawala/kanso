// Kanso — note_list_page.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/place_labels.dart';
import '../../../shell/views/widgets/shell_page.dart';

/// The notes of a library place.
class NoteListPage extends StatelessWidget {
  const NoteListPage({required this.place, super.key});

  final ListPlace place;

  @override
  Widget build(BuildContext context) {
    return ShellPage(
      title: placeLabel(context.l10n, place),
      backgroundColor: context.colors.list,
    );
  }
}
