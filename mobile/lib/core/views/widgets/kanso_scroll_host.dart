// Kanso — kanso_scroll_host.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

/// Owns the [ScrollController] of a scrolling [builder] and disposes it.
///
/// With [startAtEnd] the view opens scrolled to its end, which is how a
/// long page is shown at its foot.
class KansoScrollHost extends StatefulWidget {
  const KansoScrollHost({
    required this.builder,
    this.startAtEnd = false,
    super.key,
  });

  final Widget Function(BuildContext context, ScrollController controller)
  builder;
  final bool startAtEnd;

  @override
  State<KansoScrollHost> createState() => _KansoScrollHostState();
}

class _KansoScrollHostState extends State<KansoScrollHost> {
  final ScrollController controller = ScrollController();

  @override
  void initState() {
    super.initState();
    if (widget.startAtEnd) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (controller.hasClients) {
          controller.jumpTo(controller.position.maxScrollExtent);
        }
      });
    }
  }

  @override
  void dispose() {
    controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => widget.builder(context, controller);
}
