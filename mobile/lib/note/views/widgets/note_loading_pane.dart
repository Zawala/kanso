// Kanso — note_loading_pane.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_skeleton.dart';

/// A note while it is read: still bars where its title, facts and lines of
/// text will be.
class NoteLoadingPane extends StatelessWidget {
  const NoteLoadingPane({required this.gutter, super.key});

  /// The space either side of the text.
  final double gutter;

  /// How wide each line of text is, as a share of the pane; it repeats.
  static const List<double> lineFractions = <double>[
    1,
    0.92,
    0.97,
    0.6,
    1,
    0.85,
  ];

  @override
  Widget build(BuildContext context) {
    return Semantics(
      label: context.l10n.noteLoading,
      container: true,
      child: ListView(
        physics: const NeverScrollableScrollPhysics(),
        padding: EdgeInsets.fromLTRB(
          gutter,
          KansoSpacing.xxs,
          gutter,
          KansoSpacing.lg,
        ),
        children: <Widget>[
          const FractionallySizedBox(
            alignment: AlignmentDirectional.centerStart,
            widthFactor: 0.6,
            child: KansoSkeleton(height: 24),
          ),
          const SizedBox(height: KansoSpacing.sm),
          const FractionallySizedBox(
            alignment: AlignmentDirectional.centerStart,
            widthFactor: 0.45,
            child: KansoSkeleton(height: 14),
          ),
          const SizedBox(height: KansoSpacing.xs),
          const FractionallySizedBox(
            alignment: AlignmentDirectional.centerStart,
            widthFactor: 0.3,
            child: KansoSkeleton(),
          ),
          const SizedBox(height: KansoSpacing.lg),
          for (var i = 0; i < 12; i++) ...<Widget>[
            FractionallySizedBox(
              alignment: AlignmentDirectional.centerStart,
              widthFactor: lineFractions[i % lineFractions.length],
              child: const KansoSkeleton(),
            ),
            const SizedBox(height: KansoSpacing.sm),
          ],
        ],
      ),
    );
  }
}
