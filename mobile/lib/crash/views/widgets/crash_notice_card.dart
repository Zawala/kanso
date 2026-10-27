// Kanso — crash_notice_card.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_message_card.dart';
import '../../models/data/crash_send_outcome.dart';
import '../../models/data/pending_crash_report.dart';
import '../../utils/crash_labels.dart';
import 'crash_report_text.dart';

/// The card shown after a launch that found crash reports: it says in plain
/// words what a report holds and asks whether to send it. Show report opens
/// the exact text.
class CrashNoticeCard extends StatelessWidget {
  const CrashNoticeCard({
    required this.reports,
    required this.showsReport,
    required this.sending,
    required this.outcome,
    required this.onSend,
    required this.onDiscard,
    required this.onToggleReport,
    super.key,
  });

  final List<PendingCrashReport> reports;
  final bool showsReport;

  /// Whether a send is under way: the buttons wait.
  final bool sending;

  /// How the last send ended, when it did not work.
  final CrashSendOutcome? outcome;
  final VoidCallback onSend;
  final VoidCallback onDiscard;
  final VoidCallback onToggleReport;

  /// The tallest the text of the reports grows before it scrolls.
  static const double maxTextHeight = 180;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final failure = outcome;
    return KansoMessageCard(
      icon: KansoIcons.shield,
      title: l10n.crashNoticeTitle,
      message: l10n.crashNoticeMessage,
      width: KansoSizes.messageCardWidth + 2 * KansoSpacing.lg,
      actions: <Widget>[
        if (showsReport)
          ConstrainedBox(
            constraints: const BoxConstraints(maxHeight: maxTextHeight),
            child: SingleChildScrollView(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: <Widget>[
                  for (final report in reports) ...<Widget>[
                    CrashReportText(text: report.body),
                    const SizedBox(height: KansoSpacing.xs),
                  ],
                ],
              ),
            ),
          ),
        if (failure != null && failure != CrashSendOutcome.sent)
          Text(
            l10n.crashOutcomeMessage(failure),
            textAlign: TextAlign.center,
            style: context.typography.secondary.copyWith(
              color: context.colors.danger,
            ),
          ),
        KansoButton(
          label: sending ? l10n.crashReportSending : l10n.crashNoticeSend,
          onPressed: sending ? null : onSend,
        ),
        KansoButton(
          label: l10n.crashNoticeDiscard,
          variant: KansoButtonVariant.secondary,
          onPressed: sending ? null : onDiscard,
        ),
        KansoButton(
          label: showsReport ? l10n.crashReportHide : l10n.crashReportShow,
          variant: KansoButtonVariant.quiet,
          onPressed: onToggleReport,
        ),
      ],
    );
  }
}
