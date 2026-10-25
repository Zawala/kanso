// Kanso — kanso_segmented.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_radii.dart';
import '../../utils/context_extensions.dart';
import 'kanso_segment.dart';
import 'kanso_segment_cell.dart';

/// A row of choices of which exactly one is [selected], drawn by Kanso.
///
/// [expanded] gives every segment the same share of the width; otherwise
/// the control is as wide as its segments need.
class KansoSegmented<T> extends StatelessWidget {
  const KansoSegmented({
    required this.segments,
    required this.selected,
    required this.onChanged,
    this.expanded = false,
    super.key,
  });

  final List<KansoSegment<T>> segments;
  final T selected;
  final ValueChanged<T> onChanged;
  final bool expanded;

  /// The gap between the track's edge and its segments.
  static const double track = 2;

  @override
  Widget build(BuildContext context) {
    final cells = <Widget>[
      for (final segment in segments)
        KansoSegmentCell<T>(
          segment: segment,
          active: segment.value == selected,
          onTap: () => onChanged(segment.value),
        ),
    ];
    return Container(
      padding: const EdgeInsets.all(track),
      decoration: BoxDecoration(
        color: context.colors.fill,
        borderRadius: KansoRadii.mediumBorder,
      ),
      child: Row(
        mainAxisSize: expanded ? MainAxisSize.max : MainAxisSize.min,
        children: expanded
            ? <Widget>[for (final cell in cells) Expanded(child: cell)]
            : cells,
      ),
    );
  }
}
