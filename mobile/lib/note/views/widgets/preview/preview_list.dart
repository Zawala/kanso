// Kanso — preview_list.dart
// Created 2026-10-30.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../../../../core/bridge/api/types.dart';
import '../../../../core/configs/kanso_icons.dart';
import '../../../../core/views/widgets/kanso_icon.dart';
import 'preview_blocks.dart';
import 'preview_style.dart';

/// A list: bullets, or numbers from [start], or task boxes, each item's
/// blocks beside its marker. A [tight] list has no paragraph gaps.
class PreviewList extends StatelessWidget {
  const PreviewList({
    required this.ordered,
    required this.start,
    required this.tight,
    required this.items,
    super.key,
  });

  final bool ordered;
  final int start;
  final bool tight;
  final List<PreviewListItem> items;

  static const List<String> _bullets = <String>['•', '◦', '▪'];

  @override
  Widget build(BuildContext context) {
    final scope = PreviewScope.of(context);
    final style = scope.style;
    final size = style.size;
    final between = tight ? 0.25 * size : math.max(0.25 * size, style.gap);
    final marker = style.body.copyWith(color: style.taskDone);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      mainAxisSize: MainAxisSize.min,
      children: <Widget>[
        for (var at = 0; at < items.length; at++) ...<Widget>[
          if (at > 0) SizedBox(height: between),
          Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: <Widget>[
              if (items[at].task case final task?)
                PreviewTaskCheckbox(checked: task.checked, index: task.index)
              else
                SizedBox(
                  width: 1.4 * size,
                  child: Padding(
                    padding: EdgeInsets.only(right: 0.35 * size),
                    child: Text(
                      ordered
                          ? '${start + at}.'
                          : _bullets[scope.depth % _bullets.length],
                      textAlign: TextAlign.right,
                      style: marker,
                    ),
                  ),
                ),
              Expanded(
                child: PreviewScope.nested(
                  context,
                  depth: scope.depth + 1,
                  style: items[at].task?.checked ?? false
                      ? style.withText(style.taskDone)
                      : null,
                  child: PreviewBlocks(items[at].blocks, tight: tight),
                ),
              ),
            ],
          ),
        ],
      ],
    );
  }
}

/// A task's box, ticked or not, as the page draws it. A box with an
/// [index] can be tapped, which asks to tick it (`kanso-task:<index>`); one
/// without can't.
class PreviewTaskCheckbox extends StatelessWidget {
  const PreviewTaskCheckbox({required this.checked, this.index, super.key});

  final bool checked;
  final int? index;

  @override
  Widget build(BuildContext context) {
    final scope = PreviewScope.of(context);
    final style = scope.style;
    final line =
        MediaQuery.textScalerOf(context).scale(style.size) *
        (style.body.height ?? 1.45);
    final box = Container(
      width: 15,
      height: 15,
      decoration: BoxDecoration(
        color: checked ? style.accent : null,
        border: Border.all(
          color: checked ? style.accent : style.taskDone,
          width: 1.5,
        ),
        borderRadius: BorderRadius.circular(4),
      ),
      alignment: Alignment.center,
      child: checked
          ? KansoIcon(
              KansoIcons.check,
              size: 12,
              color: style.palette.dark
                  ? const Color(0xFF000000)
                  : const Color(0xFFFFFFFF),
            )
          : null,
    );
    final tappable = index;
    return Semantics(
      container: true,
      checked: checked,
      enabled: tappable != null,
      button: tappable != null,
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: tappable == null
            ? null
            : () => scope.onLink('kanso-task:$tappable'),
        child: SizedBox(
          width: 15 + 8 + 8,
          height: math.max(line, 15),
          child: Align(alignment: Alignment.centerLeft, child: box),
        ),
      ),
    );
  }
}
