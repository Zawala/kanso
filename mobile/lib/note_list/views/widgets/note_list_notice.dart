// Kanso — note_list_notice.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_shadows.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_pressable.dart';

/// A short line afloat at the foot of the list that says why a change to
/// notes did not happen, and goes when it has been read.
class NoteListNotice extends StatelessWidget {
  const NoteListNotice({
    required this.error,
    required this.onDismiss,
    super.key,
  });

  final ApplicationError error;
  final VoidCallback onDismiss;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    return Semantics(
      liveRegion: true,
      child: Container(
        padding: const EdgeInsetsDirectional.only(
          start: KansoSpacing.md,
          top: KansoSpacing.xxs,
          bottom: KansoSpacing.xxs,
        ),
        decoration: BoxDecoration(
          color: colors.elevated,
          borderRadius: KansoRadii.mediumBorder,
          border: Border.all(color: colors.divider),
          boxShadow: KansoShadows.floating(Theme.brightnessOf(context)),
        ),
        child: Row(
          children: <Widget>[
            Expanded(
              child: Text(
                error is LockedError
                    ? l10n.noteListChangeLocked
                    : l10n.noteListChangeFailed,
                style: context.typography.secondary.copyWith(
                  color: colors.text,
                ),
              ),
            ),
            KansoPressable(
              onTap: onDismiss,
              child: Padding(
                padding: const EdgeInsets.all(KansoSpacing.sm),
                child: Text(
                  l10n.noteListDismiss,
                  style: context.typography.body.copyWith(
                    color: colors.accent,
                  ),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
