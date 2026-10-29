// Kanso — backup_row.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/list_labels.dart';
import '../../../core/views/widgets/kanso_dialog.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_progress.dart';
import '../../state/backups/backups_bloc.dart';

/// A daily copy cannot be deleted; on-request copies can.
class BackupRow extends StatelessWidget {
  const BackupRow({required this.backup, required this.deleting, super.key});

  final BackupFile backup;
  final bool deleting;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final size = backup.bytes < 1024
        ? l10n.fileSizeBytes(backup.bytes.toString())
        : backup.bytes < 1024 * 1024
        ? l10n.fileSizeKilobytes((backup.bytes / 1024).toStringAsFixed(1))
        : l10n.fileSizeMegabytes(
            (backup.bytes / (1024 * 1024)).toStringAsFixed(1),
          );
    return Row(
      children: <Widget>[
        const Padding(
          padding: EdgeInsets.symmetric(horizontal: KansoSpacing.md),
          child: KansoIcon(KansoIcons.databaseBackup),
        ),
        Expanded(
          child: Padding(
            padding: const EdgeInsets.symmetric(vertical: KansoSpacing.sm),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: <Widget>[
                Text(
                  dateLabel(l10n, backup.madeAtMs),
                  style: context.typography.listTitle,
                ),
                const SizedBox(height: 2),
                Text(
                  l10n.backupsKindManualKept(
                    backup.manual
                        ? l10n.backupsKindManual
                        : l10n.backupsKindDaily,
                    size,
                  ),
                  style: context.typography.meta,
                ),
              ],
            ),
          ),
        ),
        if (backup.manual)
          Padding(
            padding: const EdgeInsetsDirectional.only(end: KansoSpacing.xs),
            child: deleting
                ? const Padding(
                    padding: EdgeInsets.all(KansoSpacing.sm),
                    child: KansoSpinner(),
                  )
                : KansoIconButton(
                    icon: KansoIcons.trash,
                    semanticLabel: l10n.backupsDelete,
                    onPressed: () async {
                      final confirmed = await showKansoConfirm(
                        context,
                        title: l10n.backupsDeleteTitle,
                        body: l10n.backupsDeleteMessage(
                          dateLabel(l10n, backup.madeAtMs),
                        ),
                        cancelLabel: l10n.backupsDeleteCancel,
                        confirmLabel: l10n.backupsDeleteConfirm,
                        danger: true,
                      );
                      if (confirmed && context.mounted) {
                        context.read<BackupsBloc>().add(
                          BackupsEvent.delete(backup.path),
                        );
                      }
                    },
                  ),
          ),
      ],
    );
  }
}
