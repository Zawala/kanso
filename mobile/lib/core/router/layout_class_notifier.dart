// Kanso — layout_class_notifier.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:ui';

import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart' show WidgetsBinding;
import 'package:injectable/injectable.dart';

import '../configs/kanso_layout.dart';
import '../models/data/layout_class.dart';

/// The [LayoutClass] of the window now, for the router to rebuild its pages
/// when it changes (a rotation, a Split View, a Stage Manager window, a
/// fold).
///
/// The app tells it the window's size at the top of the widget tree (see
/// `LayoutClassReporter`); it starts from the size of the window the app
/// opened in, so the first frame is already laid out right.
///
/// It also knows whether the device is a tablet, which does not change with
/// the window: a tablet's Split View half is a narrow window, and still a
/// tablet's.
@singleton
class LayoutClassNotifier extends ValueNotifier<LayoutClass> {
  LayoutClassNotifier() : tabletDevice = _isTablet(), super(_openingClass());

  /// A notifier that starts as [initial], for tests, on a tablet if
  /// [tabletDevice].
  @visibleForTesting
  LayoutClassNotifier.starting(super.initial, {this.tabletDevice = false});

  /// Whether the device's display has a shortest side of a tablet's. Settings
  /// is a sheet on a tablet and a page on a phone, whatever the width of the
  /// window.
  final bool tabletDevice;

  /// Takes the window's [size]; listeners hear of it only if the class
  /// changed.
  void update(Size size) {
    value = KansoLayout.classFor(size);
  }

  static bool _isTablet() {
    final display =
        WidgetsBinding.instance.platformDispatcher.implicitView?.display;
    if (display == null) {
      return false;
    }
    final size = display.size / display.devicePixelRatio;
    return size.shortestSide >= KansoLayout.tabletShortestSide;
  }

  static LayoutClass _openingClass() {
    final view = WidgetsBinding.instance.platformDispatcher.implicitView;
    if (view == null) {
      return LayoutClass.single;
    }
    return KansoLayout.classFor(view.physicalSize / view.devicePixelRatio);
  }
}
