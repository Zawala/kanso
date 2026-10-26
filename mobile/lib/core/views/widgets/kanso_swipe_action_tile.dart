// Kanso — kanso_swipe_action_tile.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../models/data/kanso_swipe_action.dart';
import '../../utils/context_extensions.dart';
import '../../utils/contrast.dart';
import 'kanso_icon.dart';
import 'kanso_pressable.dart';

/// One action behind a swiped row: its picture and word on its colour, in
/// whichever of white and near-black reads better on that colour.
class KansoSwipeActionTile extends StatelessWidget {
  const KansoSwipeActionTile({required this.action, super.key});

  final KansoSwipeAction action;

  @override
  Widget build(BuildContext context) {
    final ink = contrastRatio(action.background, Colors.white) >= 3
        ? Colors.white
        : const Color(0xFF1E1B16);
    return KansoPressable(
      onTap: action.onPressed,
      semanticLabel: action.label,
      child: ColoredBox(
        color: action.background,
        child: Center(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: <Widget>[
              KansoIcon(action.icon, color: ink),
              Text(
                action.label,
                maxLines: 1,
                style: context.typography.meta.copyWith(color: ink),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
