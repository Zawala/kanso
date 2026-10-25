// Kanso — skeleton_note_row.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/views/widgets/kanso_skeleton.dart';

/// A row of the note list while its note loads: bars where the title, the
/// two lines of text and the quiet line of facts will be, with the same
/// padding and rhythm as the real row so nothing jumps when notes arrive.
///
/// [titleFraction] and [textFraction] vary the bars' widths, so a column of
/// rows does not look stamped.
class SkeletonNoteRow extends StatelessWidget {
  const SkeletonNoteRow({
    this.titleFraction = 0.5,
    this.textFraction = 0.8,
    super.key,
  });

  /// The title bar's share of the row's width.
  final double titleFraction;

  /// The second line's share of the row's width.
  final double textFraction;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(
        horizontal: KansoSpacing.md,
        vertical: KansoSpacing.sm,
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          SizedBox(
            height: 20,
            child: Align(
              alignment: AlignmentDirectional.centerStart,
              child: FractionallySizedBox(
                widthFactor: titleFraction,
                child: const KansoSkeleton(height: 14),
              ),
            ),
          ),
          const SizedBox(height: KansoSpacing.xxs),
          SizedBox(
            height: 36,
            child: Column(
              mainAxisAlignment: MainAxisAlignment.spaceEvenly,
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: <Widget>[
                const KansoSkeleton(height: 11),
                Align(
                  alignment: AlignmentDirectional.centerStart,
                  child: FractionallySizedBox(
                    widthFactor: textFraction,
                    child: const KansoSkeleton(height: 11),
                  ),
                ),
              ],
            ),
          ),
          const SizedBox(height: KansoSpacing.xs),
          const SizedBox(
            height: 22,
            child: Row(
              children: <Widget>[
                KansoSkeleton(width: 88, height: 10),
                SizedBox(width: KansoSpacing.xs),
                KansoSkeleton(width: 44, height: 16),
                Spacer(),
                KansoSkeleton(width: 32, height: 10),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
