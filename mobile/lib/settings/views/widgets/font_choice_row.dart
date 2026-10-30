// Kanso — font_choice_row.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/configs/kanso_typography.dart';
import '../../../core/models/data/kanso_font_family.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_pressable.dart';

/// One font to choose: its name and a short sample, both set in that font,
/// with a check on the one in use.
///
/// The device's own font ([KansoFontFamily.system]) is the exception. It
/// differs on every platform, so its row is drawn in the interface font,
/// says so plainly, and names what the device uses.
///
/// [FontChoiceRow.downloaded] is a Google Font downloaded to the device,
/// shown the same way under the name it was downloaded as.
class FontChoiceRow extends StatelessWidget {
  const FontChoiceRow({
    required KansoFontFamily this.font,
    required this.sample,
    required this.selected,
    required this.onTap,
    super.key,
  }) : downloadedFamily = null;

  const FontChoiceRow.downloaded({
    required String family,
    required this.sample,
    required this.selected,
    required this.onTap,
    super.key,
  }) : font = null,
       downloadedFamily = family;

  /// The bundled family, or null for a downloaded one.
  final KansoFontFamily? font;

  /// The downloaded Google Font's family, or null for a bundled one.
  final String? downloadedFamily;

  /// The line shown under the name, in the font.
  final String sample;
  final bool selected;
  final VoidCallback onTap;

  /// The least a row measures: a little over a touch target, since it
  /// holds two lines.
  static const double minHeight = 60;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final typography = context.typography;
    final l10n = context.l10n;
    final bundled = font;
    final downloaded = downloadedFamily;
    final system = bundled == KansoFontFamily.system;
    TextStyle set(double size, FontWeight weight, Color color) {
      if (bundled == null || system) {
        return typography.body.copyWith(
          fontFamily: downloaded,
          fontSize: size,
          fontWeight: weight,
          height: 1.35,
          color: color,
        );
      }
      return KansoTypography.fontStyle(
        bundled,
        size: size,
        weight: weight,
        height: 1.35,
        color: color,
      );
    }

    return Semantics(
      selected: selected,
      inMutuallyExclusiveGroup: true,
      child: KansoPressable(
        onTap: onTap,
        child: ConstrainedBox(
          constraints: const BoxConstraints(minHeight: minHeight),
          child: Padding(
            padding: const EdgeInsets.symmetric(
              horizontal: KansoSpacing.md,
              vertical: KansoSpacing.sm,
            ),
            child: Row(
              children: <Widget>[
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: <Widget>[
                      Text(
                        system
                            ? l10n.fontsSystemName
                            : downloaded ?? bundled!.displayName,
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: set(17, FontWeight.w600, colors.text),
                      ),
                      const SizedBox(height: KansoSpacing.xxs),
                      Text(
                        system ? l10n.fontsSystemSample : sample,
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: set(14, FontWeight.w400, colors.textSecondary),
                      ),
                    ],
                  ),
                ),
                SizedBox(
                  width: KansoSizes.touchTarget - KansoSpacing.xs,
                  child: selected
                      ? Align(
                          alignment: AlignmentDirectional.centerEnd,
                          child: KansoIcon(
                            KansoIcons.check,
                            color: colors.accent,
                          ),
                        )
                      : null,
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
