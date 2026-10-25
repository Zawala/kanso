// Kanso — kanso_segment.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../configs/kanso_icons.dart';

/// One choice of a `KansoSegmented`: a [value], and a [label], an [icon] or
/// both. An icon-only segment needs a [semanticLabel].
class KansoSegment<T> {
  const KansoSegment({
    required this.value,
    this.label,
    this.icon,
    this.semanticLabel,
  });

  final T value;
  final String? label;
  final KansoIcons? icon;
  final String? semanticLabel;
}
