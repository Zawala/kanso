// Kanso — kanso_search_field.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_icons.dart';
import '../../configs/kanso_radii.dart';
import '../../configs/kanso_sizes.dart';
import '../../configs/kanso_spacing.dart';
import '../../utils/context_extensions.dart';
import 'kanso_caret.dart';
import 'kanso_icon.dart';
import 'kanso_pressable.dart';

/// A search field, drawn by Kanso.
///
/// It shows [query], or [hint] when there is none. [focused] draws the
/// accent outline and a caret; [onClear] adds a clear button while there is
/// a query.
class KansoSearchField extends StatelessWidget {
  const KansoSearchField({
    required this.hint,
    this.query = '',
    this.focused = false,
    this.onClear,
    this.clearLabel,
    super.key,
  });

  final String hint;
  final String query;
  final bool focused;
  final VoidCallback? onClear;

  /// What the clear button says to a screen reader.
  final String? clearLabel;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final typography = context.typography;
    final hasQuery = query.isNotEmpty;
    return Container(
      height: KansoSizes.searchField,
      padding: const EdgeInsetsDirectional.only(start: KansoSpacing.sm),
      decoration: BoxDecoration(
        color: colors.fill,
        borderRadius: KansoRadii.mediumBorder,
        border: Border.all(
          color: focused ? colors.accent : colors.fill,
          width: focused ? 1.5 : 1,
        ),
      ),
      child: Row(
        children: <Widget>[
          KansoIcon(
            KansoIcons.search,
            size: KansoSizes.iconSmall + 2,
            color: colors.textMuted,
          ),
          const SizedBox(width: KansoSpacing.xs),
          Expanded(
            child: Row(
              children: <Widget>[
                Flexible(
                  child: Text(
                    hasQuery ? query : hint,
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: hasQuery
                        ? typography.body
                        : typography.body.copyWith(color: colors.textMuted),
                  ),
                ),
                if (focused) ...<Widget>[
                  const SizedBox(width: 1),
                  const KansoCaret(height: 18),
                ],
              ],
            ),
          ),
          if (hasQuery && onClear != null)
            KansoPressable(
              onTap: onClear,
              semanticLabel: clearLabel,
              child: SizedBox.square(
                dimension: KansoSizes.searchField,
                child: Center(
                  child: KansoIcon(
                    KansoIcons.close,
                    size: KansoSizes.iconSmall,
                    color: colors.textMuted,
                  ),
                ),
              ),
            ),
        ],
      ),
    );
  }
}
