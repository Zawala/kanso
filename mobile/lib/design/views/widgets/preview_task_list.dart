// Kanso — preview_task_list.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_checkbox.dart';
import '../../models/data/mock_task_item.dart';
import 'markdown_inline_text.dart';

/// A task list, rendered: a checkbox for each item, the done ones struck
/// through and quiet.
class PreviewTaskList extends StatelessWidget {
  const PreviewTaskList({required this.items, super.key});

  final List<MockTaskItem> items;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final body = context.typography.body;
    final lineHeight = (body.fontSize ?? 15) * (body.height ?? 1.4);
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
                  height: lineHeight,
                  child: Center(child: KansoCheckbox(value: item.done)),
                ),
                const SizedBox(width: KansoSpacing.sm),
                Expanded(
                  child: MarkdownInlineText(
                    item.text,
                    style: item.done
                        ? body.copyWith(
                            color: colors.textMuted,
                            decoration: TextDecoration.lineThrough,
                            decorationColor: colors.textMuted,
                          )
                        : body,
                  ),
                ),
              ],
            ),
          ),
      ],
    );
  }
}
