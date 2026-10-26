// Kanso — layout_class_reporter.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

import '../../configs/kanso_layout.dart';
import '../../router/layout_class_notifier.dart';

/// Tells [notifier] how big the window is whenever that changes, so the
/// panes are laid out for the new size in the frame that shows it. It draws
/// [child] unchanged.
///
/// The window's own metrics are listened to, because a notifier may not
/// change while the tree is being built; the size the tree is built with is
/// checked as well, in case the two differ.
class LayoutClassReporter extends StatefulWidget {
  const LayoutClassReporter({
    required this.notifier,
    required this.child,
    super.key,
  });

  final LayoutClassNotifier notifier;
  final Widget child;

  @override
  State<LayoutClassReporter> createState() => LayoutClassReporterState();
}

/// The state of a [LayoutClassReporter]: it watches the window.
class LayoutClassReporterState extends State<LayoutClassReporter>
    with WidgetsBindingObserver {
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
  }

  @override
  void didChangeMetrics() {
    final view = View.maybeOf(context);
    if (view != null) {
      widget.notifier.update(view.physicalSize / view.devicePixelRatio);
    }
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final size = MediaQuery.sizeOf(context);
    if (KansoLayout.classFor(size) != widget.notifier.value) {
      WidgetsBinding.instance.addPostFrameCallback(
        (_) => widget.notifier.update(size),
      );
    }
    return widget.child;
  }
}
