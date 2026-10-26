// Kanso — kanso_check_row.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_icons.dart';
import '../../configs/kanso_sizes.dart';
import '../../configs/kanso_spacing.dart';
import '../../utils/context_extensions.dart';
import 'kanso_icon.dart';
import 'kanso_pressable.dart';

/// A row of a sheet that is one of several choices: its [label], and a
/// check mark in the accent colour when it is the one in use.
class KansoCheckRow extends StatelessWidget {
  const KansoCheckRow({
    required this.label,
    required this.checked,
    this.icon,
    this.indent = 0,
    this.onTap,
    super.key,
  });

  final String label;
  final bool checked;
  final KansoIcons? icon;

  /// How far the label is moved in, in points: a notebook inside another.
  final double indent;

  /// What choosing the row does; with none it is drawn dimmed.
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final glyph = icon;
    final available = onTap != null;
    return Semantics(
      checked: checked,
      child: KansoPressable(
        onTap: onTap,
        child: ConstrainedBox(
          constraints: const BoxConstraints(minHeight: KansoSizes.settingsRow),
          child: Opacity(
            opacity: available ? 1 : 0.5,
            child: Padding(
              padding: EdgeInsetsDirectional.only(
                start: KansoSpacing.md + indent,
                end: KansoSpacing.md,
                top: KansoSpacing.xs,
                bottom: KansoSpacing.xs,
              ),
              child: Row(
                children: <Widget>[
                  if (glyph != null) ...<Widget>[
                    KansoIcon(glyph, color: colors.textSecondary),
                    const SizedBox(width: KansoSpacing.sm),
                  ],
                  Expanded(child: Text(label, style: context.typography.body)),
                  const SizedBox(width: KansoSpacing.xs),
                  SizedBox(
                    width: KansoSizes.icon,
                    child: checked
                        ? KansoIcon(KansoIcons.check, color: colors.accent)
                        : null,
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}
