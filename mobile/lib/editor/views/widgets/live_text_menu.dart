// Kanso — live_text_menu.dart
// Created 2026-10-30.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_text_menu_toolbar.dart';

/// The Live editor's text menu, drawn as the field's is: the screen's own
/// buttons, Open link when the caret is in a link (once the core has said
/// where it leads), then [buttons] (Cut, Copy, Paste, Select All, Comment).
class LiveTextMenu extends StatefulWidget {
  const LiveTextMenu({
    required this.anchors,
    required this.buttons,
    this.extra = const <(String, VoidCallback)>[],
    this.link,
    this.onOpen,
    super.key,
  });

  final TextSelectionToolbarAnchors anchors;
  final List<(String, VoidCallback)> buttons;

  /// The screen's buttons, first.
  final List<(String, VoidCallback)> extra;

  /// Where the link at the caret leads, when the core answers.
  final Future<LinkTarget?>? link;

  /// The person chose Open link.
  final ValueChanged<LinkTarget>? onOpen;

  @override
  State<LiveTextMenu> createState() => _LiveTextMenuState();
}

class _LiveTextMenuState extends State<LiveTextMenu> {
  late Future<LinkTarget?> link;

  @override
  void initState() {
    super.initState();
    link = widget.link ?? Future<LinkTarget?>.value();
  }

  @override
  Widget build(BuildContext context) {
    return FutureBuilder<LinkTarget?>(
      future: link,
      builder: (context, snapshot) {
        final found = snapshot.data;
        final open = widget.onOpen;
        final buttons = <(String, VoidCallback)>[
          ...widget.extra,
          if (found != null && open != null)
            (context.l10n.menuOpenLink, () => open(found)),
          ...widget.buttons,
        ];
        if (buttons.isEmpty) {
          return const SizedBox.shrink();
        }
        return KansoTextMenuToolbar(anchors: widget.anchors, buttons: buttons);
      },
    );
  }
}
