// Kanso — loading_list_pane.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_divider.dart';
import 'skeleton_note_row.dart';

/// The list of notes while it loads: rows made of still bars where notes
/// will appear.
class LoadingListPane extends StatelessWidget {
  const LoadingListPane({this.semanticLabel, super.key});

  /// What a screen reader says while it shows; "Loading notes" when null.
  final String? semanticLabel;

  /// How wide each skeleton row's title is, as a share of the row; it
  /// repeats down the list.
  static const List<double> titleFractions = <double>[0.46, 0.62, 0.38, 0.54];

  /// How wide each skeleton row's second line is.
  static const List<double> textFractions = <double>[0.78, 0.9, 0.64, 0.82];

  /// How many rows are drawn: enough to fill a screen.
  static const int rows = 8;

  @override
  Widget build(BuildContext context) {
    return Semantics(
      label: semanticLabel ?? context.l10n.noteListLoading,
      container: true,
      child: ListView(
        physics: const NeverScrollableScrollPhysics(),
        padding: const EdgeInsets.only(top: KansoSpacing.xxs),
        children: <Widget>[
          for (var i = 0; i < rows; i++) ...<Widget>[
            if (i > 0) const KansoDivider(indent: KansoSpacing.md),
            SkeletonNoteRow(
              titleFraction: titleFractions[i % titleFractions.length],
              textFraction: textFractions[i % textFractions.length],
            ),
          ],
        ],
      ),
    );
  }
}
