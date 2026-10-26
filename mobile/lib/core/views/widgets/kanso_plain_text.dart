// Kanso — kanso_plain_text.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

/// Makes text in [child] plain where no page gives it a style: with no
/// `Scaffold` or `Material` above it, text is drawn underlined in yellow.
///
/// The shell's panes, the library drawer, sheets and notices sit above or
/// beside the pages, so they set it themselves.
class KansoPlainText extends StatelessWidget {
  const KansoPlainText({required this.child, super.key});

  final Widget child;

  @override
  Widget build(BuildContext context) {
    return DefaultTextStyle.merge(
      style: const TextStyle(decoration: TextDecoration.none),
      child: child,
    );
  }
}
