// Kanso — kanso_section_header.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math' as math;

import 'package:flutter/material.dart';

import '../../configs/kanso_sizes.dart';
import '../../configs/kanso_spacing.dart';
import '../../utils/context_extensions.dart';

/// A section's label above its rows, with optional controls at the end.
///
/// The label is in one quiet style everywhere: the library's "Notebooks",
/// the list's "Today". A header has some air above it, so groups of rows
/// read as groups; with [trailing] controls it is as tall as a touch
/// target.
class KansoSectionHeader extends StatelessWidget {
  const KansoSectionHeader({
    required this.label,
    this.trailing = const <Widget>[],
    this.horizontalPadding = KansoSpacing.md,
    super.key,
  });

  final String label;
  final List<Widget> trailing;
  final double horizontalPadding;

  @override
  Widget build(BuildContext context) {
    final plain = trailing.isEmpty;
    return Container(
      constraints: BoxConstraints(
        minHeight: plain ? KansoSizes.chip : KansoSizes.touchTarget,
      ),
      padding: EdgeInsetsDirectional.only(
        start: horizontalPadding,
        end: math.max(0, horizontalPadding - KansoSpacing.xs),
        top: plain ? KansoSpacing.sm : 0,
        bottom: plain ? KansoSpacing.xxs : 0,
      ),
      child: Row(
        children: <Widget>[
          Expanded(
            child: Text(
              label,
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: context.typography.sectionLabel,
            ),
          ),
          ...trailing,
        ],
      ),
    );
  }
}
