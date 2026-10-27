// Kanso — crash_report_row.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../models/data/pending_crash_report.dart';
import '../../utils/crash_labels.dart';
import 'crash_report_text.dart';

/// One crash report on the Crash reports page: its type, its day and where
/// it came from; open, its exact text and the actions.
class CrashReportRow extends StatelessWidget {
  const CrashReportRow({
    required this.report,
    required this.open,
    required this.sending,
    required this.onToggle,
    required this.onSend,
    required this.onDelete,
    super.key,
  });

  final PendingCrashReport report;
  final bool open;
  final bool sending;
  final VoidCallback onToggle;
  final VoidCallback onSend;
  final VoidCallback onDelete;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final typography = context.typography;
    final l10n = context.l10n;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        KansoPressable(
          onTap: onToggle,
          child: ConstrainedBox(
            constraints: const BoxConstraints(
              minHeight: KansoSizes.settingsRow,
            ),
            child: Padding(
              padding: const EdgeInsets.symmetric(
                horizontal: KansoSpacing.md,
                vertical: KansoSpacing.xs,
              ),
              child: Row(
                children: <Widget>[
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: <Widget>[
                        Text(
                          report.report.type,
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                          style: typography.body,
                        ),
                        Text(
                          l10n.crashReportSubtitle(
                            report.report.day,
                            l10n.crashKindName(report.report.kind),
                          ),
                          style: typography.meta.copyWith(
                            color: colors.textMuted,
                          ),
                        ),
                      ],
                    ),
                  ),
                  KansoIcon(
                    open ? KansoIcons.chevronDown : KansoIcons.chevronRight,
                    size: KansoSizes.iconSmall,
                    color: colors.textMuted,
                  ),
                ],
              ),
            ),
          ),
        ),
        if (open)
          Padding(
            padding: const EdgeInsets.fromLTRB(
              KansoSpacing.md,
              0,
              KansoSpacing.md,
              KansoSpacing.md,
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: <Widget>[
                CrashReportText(text: report.body),
                const SizedBox(height: KansoSpacing.sm),
                Row(
                  children: <Widget>[
                    Expanded(
                      child: KansoButton(
                        label: sending
                            ? l10n.crashReportSending
                            : l10n.crashReportSend,
                        compact: true,
                        onPressed: sending ? null : onSend,
                      ),
                    ),
                    const SizedBox(width: KansoSpacing.xs),
                    Expanded(
                      child: KansoButton(
                        label: l10n.crashReportDelete,
                        variant: KansoButtonVariant.danger,
                        compact: true,
                        onPressed: sending ? null : onDelete,
                      ),
                    ),
                  ],
                ),
              ],
            ),
          ),
      ],
    );
  }
}
