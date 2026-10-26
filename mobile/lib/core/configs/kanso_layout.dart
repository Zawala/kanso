// Kanso — kanso_layout.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math' as math;
import 'dart:ui';

import '../models/data/layout_class.dart';
import 'kanso_sizes.dart';

/// Where the app changes from one pane to two to three, from the widths of
/// the panes in the design.
abstract final class KansoLayout {
  /// 400: the least width the note pane is given.
  static const double noteMinWidth = 400;

  /// 600: the shortest side from which a window can have more than one
  /// pane. A phone, even turned sideways, has less.
  static const double tabletShortestSide = 600;

  /// 740: the width from which the note list and the note fit side by side.
  static const double doubleMinWidth = KansoSizes.listPane + noteMinWidth;

  /// 1020: the width from which the library fits beside them too.
  static const double tripleMinWidth =
      KansoSizes.libraryPane + KansoSizes.listPane + noteMinWidth;

  /// The panes a window of [size] has room for.
  ///
  /// An iPad mini in portrait (744) and an 11 inch iPad in portrait (834)
  /// have two; in landscape and a 13 inch iPad in portrait (1024) three;
  /// half of a Split View (about 590) one.
  static LayoutClass classFor(Size size) {
    if (size.shortestSide < tabletShortestSide) {
      return LayoutClass.single;
    }
    if (size.width >= tripleMinWidth) {
      return LayoutClass.triple;
    }
    if (size.width >= doubleMinWidth) {
      return LayoutClass.double;
    }
    return LayoutClass.single;
  }

  /// The width the note pane gets in a window of [width] laid out as
  /// [layout]: what the library and the list leave.
  static double noteWidth(double width, LayoutClass layout) {
    final taken = switch (layout) {
      LayoutClass.single => 0.0,
      LayoutClass.double => KansoSizes.listPane,
      LayoutClass.triple => KansoSizes.libraryPane + KansoSizes.listPane,
    };
    return math.max(0, width - taken);
  }
}
