// Kanso — tag_row_color.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:ui';

import '../bridge/api/types.dart';

/// A tag's stored colour as a [Color].
extension TagRowColor on TagRow {
  /// The colour the library stored (`0xRRGGBB`), opaque; drawn calmed.
  Color get tint => Color(0xFF000000 | color);
}
