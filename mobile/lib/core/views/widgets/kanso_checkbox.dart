// Kanso — kanso_checkbox.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_icons.dart';
import '../../configs/kanso_sizes.dart';
import '../../utils/context_extensions.dart';
import 'kanso_icon.dart';
import 'kanso_pressable.dart';

/// A checkbox, drawn by Kanso, the same on every platform.
class KansoCheckbox extends StatelessWidget {
  const KansoCheckbox({required this.value, this.onChanged, super.key});

  final bool value;
  final ValueChanged<bool>? onChanged;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final changed = onChanged;
    return Semantics(
      checked: value,
      child: KansoPressable(
        onTap: changed == null ? null : () => changed(!value),
        child: Container(
          width: KansoSizes.checkbox,
          height: KansoSizes.checkbox,
          alignment: Alignment.center,
          decoration: BoxDecoration(
            color: value ? colors.button : null,
            borderRadius: BorderRadius.circular(5),
            border: value
                ? null
                : Border.all(color: colors.textMuted, width: 1.5),
          ),
          child: value
              ? KansoIcon(
                  KansoIcons.check,
                  size: KansoSizes.glyph,
                  color: colors.buttonText,
                )
              : null,
        ),
      ),
    );
  }
}
