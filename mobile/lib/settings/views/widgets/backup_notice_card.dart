// Kanso — backup_notice_card.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_message_card.dart';

/// The card that says, once, that the notes on this phone are not in device
/// backups, with one button: OK.
class BackupNoticeCard extends StatelessWidget {
  const BackupNoticeCard({required this.onOk, super.key});

  final VoidCallback onOk;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return KansoMessageCard(
      icon: KansoIcons.info,
      title: l10n.backupNoticeTitle,
      message: l10n.backupNoticeMessage,
      actions: <Widget>[
        KansoButton(label: l10n.backupNoticeOk, onPressed: onOk),
      ],
    );
  }
}
