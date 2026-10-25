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
/// The track is a quiet fill and the chosen segment a soft wash of the
/// accent colour, so the control stays calm beside the text. It is 44
/// points tall, a full touch target. [expanded] gives every segment the
/// same share of the width; otherwise the control is as wide as its
/// segments need. [semanticLabel] names the whole control for a screen
/// reader.
class KansoSegmented<T> extends StatelessWidget {
  const KansoSegmented({
    required this.segments,
    required this.selected,
    required this.onChanged,
    this.expanded = false,
    this.semanticLabel,
    super.key,
  });

  final List<KansoSegment<T>> segments;
  final T selected;
  final ValueChanged<T> onChanged;
  final bool expanded;
  final String? semanticLabel;

  /// The weight of the track's fill, so it stays quieter than a button.
  static const double trackOpacity = 0.55;

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
    return Semantics(
      container: true,
      label: semanticLabel,
      child: Container(
        padding: const EdgeInsets.all(track),
        decoration: BoxDecoration(
          color: context.colors.fill.withValues(alpha: trackOpacity),
          borderRadius: KansoRadii.mediumBorder,
        ),
        child: Row(
          mainAxisSize: expanded ? MainAxisSize.max : MainAxisSize.min,
          children: expanded
              ? <Widget>[for (final cell in cells) Expanded(child: cell)]
              : cells,
        ),
      ),
    );
  }
}
