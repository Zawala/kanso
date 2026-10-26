// Kanso — note_top_bar.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';

/// The bar above a note. On a phone it has the way back to the list; on a
/// tablet, with the list beside the note, the history's back and forward.
/// Whatever the note has to offer goes in [trailing].
///
/// A way that leads nowhere ([onGoBack] and [onGoForward] when there is no
/// note behind or ahead) is drawn dimmed.
class NoteTopBar extends StatelessWidget {
  const NoteTopBar({
    required this.phone,
    this.onBack,
    this.onGoBack,
    this.onGoForward,
    this.trailing = const <Widget>[],
    super.key,
  });

  /// Whether it is a phone's bar, with the note on a page of its own.
  final bool phone;

  /// Back to the list, on a phone.
  final VoidCallback? onBack;

  /// Back and forward through the notes opened, on a tablet.
  final VoidCallback? onGoBack;
  final VoidCallback? onGoForward;
  final List<Widget> trailing;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return KansoTopBar(
      bordered: false,
      leading: phone
          ? KansoIconButton(
              icon: KansoIcons.chevronLeft,
              semanticLabel: l10n.noteBack,
              onPressed: onBack,
            )
          : Row(
              mainAxisSize: MainAxisSize.min,
              children: <Widget>[
                KansoIconButton(
                  icon: KansoIcons.arrowLeft,
                  semanticLabel: l10n.noteBack,
                  onPressed: onGoBack,
                ),
                KansoIconButton(
                  icon: KansoIcons.arrowRight,
                  semanticLabel: l10n.noteForward,
                  onPressed: onGoForward,
                ),
              ],
            ),
      trailing: trailing,
    );
  }
}
