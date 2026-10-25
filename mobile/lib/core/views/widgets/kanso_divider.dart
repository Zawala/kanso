// Kanso — kanso_divider.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../utils/context_extensions.dart';

/// A one point line in the divider colour.
///
/// [indent] leaves space at the start, for a list whose lines begin under
/// the text rather than under the icon.
class KansoDivider extends StatelessWidget {
  const KansoDivider({
    this.indent = 0,
    this.endIndent = 0,
    this.vertical = false,
    super.key,
  });

  final double indent;
  final double endIndent;

  /// A vertical line, between panes side by side.
  final bool vertical;

  @override
  Widget build(BuildContext context) {
    final line = ColoredBox(color: context.colors.divider);
    if (vertical) {
      return SizedBox(width: 1, height: double.infinity, child: line);
    }
    return Padding(
      padding: EdgeInsetsDirectional.only(start: indent, end: endIndent),
      child: SizedBox(width: double.infinity, height: 1, child: line),
    );
  }
}
