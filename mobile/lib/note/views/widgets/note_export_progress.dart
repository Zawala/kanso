// Kanso — note_export_progress.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_progress.dart';
import '../../../core/views/widgets/kanso_scrim.dart';

/// The part of the screen a sheet points at, from [context]'s box.
Rect? exportOriginOf(BuildContext context) {
  final box = context.findRenderObject();
  if (box is! RenderBox || !box.hasSize) {
    return null;
  }
  return box.localToGlobal(Offset.zero) & box.size;
}

/// What is shown while an export is written: [label] over a barrier that
/// lets nothing be tapped.
class NoteExportProgress extends StatelessWidget {
  const NoteExportProgress({required this.label, super.key});

  final String label;

  @override
  Widget build(BuildContext context) {
    return Stack(
      children: <Widget>[
        const Positioned.fill(child: KansoScrim()),
        Center(
          child: Container(
            padding: const EdgeInsets.symmetric(
              horizontal: KansoSpacing.lg,
              vertical: KansoSpacing.md,
            ),
            decoration: BoxDecoration(
              color: context.colors.elevated,
              borderRadius: KansoRadii.mediumBorder,
            ),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: <Widget>[
                const KansoSpinner(size: 18),
                const SizedBox(width: KansoSpacing.sm),
                Text(label, style: context.typography.body),
              ],
            ),
          ),
        ),
      ],
    );
  }
}
