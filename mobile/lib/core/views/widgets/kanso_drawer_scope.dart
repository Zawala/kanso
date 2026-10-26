// Kanso — kanso_drawer_scope.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

import 'kanso_drawer_controller.dart';

/// Gives the pages under a `KansoDrawer` its [KansoDrawerController].
class KansoDrawerScope extends InheritedWidget {
  const KansoDrawerScope({
    required this.controller,
    required super.child,
    super.key,
  });

  final KansoDrawerController controller;

  /// The drawer above [context], or null when there is none.
  static KansoDrawerController? maybeOf(BuildContext context) {
    return context
        .getInheritedWidgetOfExactType<KansoDrawerScope>()
        ?.controller;
  }

  @override
  bool updateShouldNotify(KansoDrawerScope oldWidget) =>
      controller != oldWidget.controller;
}
