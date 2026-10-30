// Kanso — bundled_font_repository.dart
// Created 2026-10-30.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/foundation.dart';

/// One face of a font the app ships, as a page embeds it.
@immutable
class BundledFontFace {
  const BundledFontFace({
    required this.weight,
    required this.bytes,
  });

  /// The CSS `font-weight` the face covers: one weight, or a range for a
  /// variable font (`100 900`).
  final String weight;

  /// The TrueType file.
  final List<int> bytes;
}

/// The files of the fonts the app ships, for the note's preview page, which
/// cannot see the fonts Flutter registered.
// ignore: one_member_abstracts
abstract interface class BundledFontRepository {
  /// The upright faces of [family], or none when the app does not ship it.
  Future<List<BundledFontFace>> faces(String family);
}
