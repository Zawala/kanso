// Kanso — kanso_swipe_host.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

import 'kanso_swipe_group.dart';
import 'kanso_swipe_scope.dart';

/// Owns the [KansoSwipeGroup] of the list under it, and gives it up when the
/// list goes.
class KansoSwipeHost extends StatefulWidget {
  const KansoSwipeHost({required this.child, super.key});

  final Widget child;

  @override
  State<KansoSwipeHost> createState() => _KansoSwipeHostState();
}

class _KansoSwipeHostState extends State<KansoSwipeHost> {
  final KansoSwipeGroup group = KansoSwipeGroup();

  @override
  void dispose() {
    group.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return KansoSwipeScope(group: group, child: widget.child);
  }
}
