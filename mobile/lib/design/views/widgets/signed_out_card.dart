// Kanso — signed_out_card.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_message_card.dart';

/// The card shown when the session on this device has ended: what
/// happened, that the notes are safe, and one clear action.
class SignedOutCard extends StatelessWidget {
  const SignedOutCard({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return KansoMessageCard(
      icon: KansoIcons.logOut,
      title: l10n.errorSignedOutTitle,
      message: l10n.errorSignedOutMessage,
      actions: <Widget>[
        KansoButton(label: l10n.errorSignInAgain, onPressed: () {}),
        KansoButton(
          label: l10n.errorNotNow,
          variant: KansoButtonVariant.quiet,
          onPressed: () {},
        ),
      ],
    );
  }
}
