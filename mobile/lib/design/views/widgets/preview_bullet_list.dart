// Kanso — preview_bullet_list.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import 'markdown_inline_text.dart';

/// A bulleted list, rendered.
class PreviewBulletList extends StatelessWidget {
  const PreviewBulletList({required this.items, super.key});

  final List<String> items;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final body = context.typography.body;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: <Widget>[
        for (final item in items)
          Padding(
            padding: const EdgeInsets.only(bottom: KansoSpacing.xxs),
            child: Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: <Widget>[
                SizedBox(
                  width: KansoSpacing.md,
                  height: (body.fontSize ?? 15) * (body.height ?? 1.4),
                  child: Center(
                    child: Container(
                      width: 5,
                      height: 5,
                      decoration: BoxDecoration(
                        color: colors.textMuted,
                        shape: BoxShape.circle,
                      ),
                    ),
                  ),
                ),
                const SizedBox(width: KansoSpacing.xs),
                Expanded(child: MarkdownInlineText(item, style: body)),
              ],
            ),
          ),
      ],
    );
  }
}
