// Kanso — crash_report_text.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_selection_controls.dart';
import '../../../core/views/widgets/kanso_text_context_menu.dart';

/// The exact text of a crash report, in the code font, on a quiet surface,
/// selectable so it can be copied.
class CrashReportText extends StatelessWidget {
  const CrashReportText({required this.text, super.key});

  final String text;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    return DecoratedBox(
      decoration: BoxDecoration(
        color: colors.fill,
        borderRadius: KansoRadii.mediumBorder,
      ),
      child: Padding(
        padding: const EdgeInsets.all(KansoSpacing.sm),
        child: SelectableText(
          text,
          style: context.typography.monoSmall.copyWith(color: colors.text),
          selectionControls: KansoSelectionControls(),
          contextMenuBuilder: (context, state) =>
              KansoTextContextMenu(state: state),
        ),
      ),
    );
  }
}
