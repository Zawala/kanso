// Kanso — tag_color.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:ui';

/// The colours a tag can be given, as the desktop app stores them.
///
/// The stored value is bright; what is drawn is calmed by `mutedTagColor`.
enum TagColor {
  grey(0x767676),
  red(0xDB2828),
  orange(0xF2711C),
  yellow(0xE5B800),
  olive(0xB5CC18),
  green(0x21BA45),
  teal(0x00B5AD),
  blue(0x2185D0),
  violet(0x6435C9),
  purple(0xA333C8),
  pink(0xE03997),
  brown(0xA5673F);

  const TagColor(this.rgb);

  /// The stored colour, `0xRRGGBB`.
  final int rgb;

  /// The stored colour, opaque.
  Color get color => Color(0xFF000000 | rgb);
}
