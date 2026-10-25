// Kanso — kanso_radii.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/painting.dart';

/// Corner radii: the desktop's 6 and 10, and 14 for sheets and cards.
abstract final class KansoRadii {
  /// 6: rows, fields and small controls.
  static const double small = 6;

  /// 10: buttons and menus.
  static const double medium = 10;

  /// 14: sheets and cards.
  static const double large = 14;

  static const BorderRadius smallBorder = BorderRadius.all(
    Radius.circular(small),
  );
  static const BorderRadius mediumBorder = BorderRadius.all(
    Radius.circular(medium),
  );
  static const BorderRadius largeBorder = BorderRadius.all(
    Radius.circular(large),
  );
}
