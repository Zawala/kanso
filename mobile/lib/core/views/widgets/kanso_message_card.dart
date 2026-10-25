// Kanso — kanso_message_card.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_icons.dart';
import '../../configs/kanso_sizes.dart';
import '../../configs/kanso_spacing.dart';
import '../../utils/context_extensions.dart';
import 'kanso_card.dart';
import 'kanso_icon.dart';

/// A small centred card that says one thing and offers [actions]: a locked
/// note, an error. Errors never show a raw message; they say what happened
/// and what to do.
class KansoMessageCard extends StatelessWidget {
  const KansoMessageCard({
    required this.title,
    required this.message,
    required this.actions,
    this.icon,
    this.width = KansoSizes.messageCardWidth,
    super.key,
  });

  final String title;
  final String message;
  final List<Widget> actions;
  final KansoIcons? icon;
  final double width;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final typography = context.typography;
    final glyph = icon;
    return ConstrainedBox(
      constraints: BoxConstraints(maxWidth: width),
      child: KansoCard(
        padding: const EdgeInsets.all(KansoSpacing.lg),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: <Widget>[
            if (glyph != null) ...<Widget>[
              Center(
                child: Container(
                  width: KansoSizes.touchTarget + KansoSpacing.xs,
                  height: KansoSizes.touchTarget + KansoSpacing.xs,
                  alignment: Alignment.center,
                  decoration: BoxDecoration(
                    color: colors.fill,
                    shape: BoxShape.circle,
                  ),
                  child: KansoIcon(
                    glyph,
                    size: KansoSizes.icon + 4,
                    color: colors.textSecondary,
                  ),
                ),
              ),
              const SizedBox(height: KansoSpacing.md),
            ],
            Text(
              title,
              textAlign: TextAlign.center,
              style: typography.heading2,
            ),
            const SizedBox(height: KansoSpacing.xs),
            Text(
              message,
              textAlign: TextAlign.center,
              style: typography.secondary,
            ),
            const SizedBox(height: KansoSpacing.lg),
            for (var i = 0; i < actions.length; i++) ...<Widget>[
              if (i > 0) const SizedBox(height: KansoSpacing.xs),
              actions[i],
            ],
          ],
        ),
      ),
    );
  }
}
