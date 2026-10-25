// Kanso — kanso_grouped_section.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_radii.dart';
import '../../configs/kanso_spacing.dart';
import '../../utils/context_extensions.dart';
import 'kanso_divider.dart';

/// Rows in one rounded surface, with hairlines between them, an optional
/// [title] above and an optional [footer] note below.
class KansoGroupedSection extends StatelessWidget {
  const KansoGroupedSection({
    required this.children,
    this.title,
    this.footer,
    this.dividerIndent = KansoSpacing.md,
    super.key,
  });

  final String? title;
  final String? footer;
  final List<Widget> children;

  /// How far the hairlines are inset from the start.
  final double dividerIndent;

  @override
  Widget build(BuildContext context) {
    final heading = title;
    final note = footer;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        if (heading != null)
          Padding(
            padding: const EdgeInsetsDirectional.only(
              start: KansoSpacing.md,
              bottom: KansoSpacing.xs,
            ),
            child: Text(heading, style: context.typography.sectionLabel),
          ),
        DecoratedBox(
          decoration: BoxDecoration(
            color: context.groupedSurface,
            borderRadius: KansoRadii.largeBorder,
          ),
          child: ClipRRect(
            borderRadius: KansoRadii.largeBorder,
            child: Column(
              children: <Widget>[
                for (var i = 0; i < children.length; i++) ...<Widget>[
                  if (i > 0) KansoDivider(indent: dividerIndent),
                  children[i],
                ],
              ],
            ),
          ),
        ),
        if (note != null)
          Padding(
            padding: const EdgeInsetsDirectional.only(
              start: KansoSpacing.md,
              end: KansoSpacing.md,
              top: KansoSpacing.xs,
            ),
            child: Text(note, style: context.typography.meta),
          ),
      ],
    );
  }
}
