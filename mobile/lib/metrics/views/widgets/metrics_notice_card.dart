// Kanso — metrics_notice_card.dart
// Created 2026-10-31.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:url_launcher/url_launcher.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_links.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_message_card.dart';

/// The one-time notice for the anonymous daily count, with the same wording
/// as the desktop app and links to the full explanation and its source.
class MetricsNoticeCard extends StatelessWidget {
  const MetricsNoticeCard({required this.onOk, super.key});

  final VoidCallback onOk;

  Future<void> _open(Uri uri) async {
    try {
      await launchUrl(uri, mode: LaunchMode.externalApplication);
    } on Object {
      // The notice is still readable if no browser can open the link.
    }
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return KansoMessageCard(
      icon: KansoIcons.info,
      title: l10n.metricsNoticeTitle,
      message: l10n.metricsNoticeMessage,
      width: KansoSizes.messageCardWidth + 2 * KansoSpacing.lg,
      actions: <Widget>[
        Wrap(
          alignment: WrapAlignment.center,
          spacing: KansoSpacing.xs,
          runSpacing: KansoSpacing.xs,
          children: <Widget>[
            KansoButton(
              label: l10n.metricsNoticeWhat,
              variant: KansoButtonVariant.quiet,
              compact: true,
              expanded: false,
              onPressed: () => _open(KansoLinks.metrics),
            ),
            KansoButton(
              label: l10n.metricsNoticeCode,
              variant: KansoButtonVariant.quiet,
              compact: true,
              expanded: false,
              onPressed: () => _open(KansoLinks.metricsCode),
            ),
          ],
        ),
        KansoButton(label: l10n.metricsNoticeOk, onPressed: onOk),
      ],
    );
  }
}
