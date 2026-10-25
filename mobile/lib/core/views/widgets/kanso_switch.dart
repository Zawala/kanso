// Kanso — kanso_switch.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_effects.dart';
import '../../configs/kanso_motion.dart';
import '../../configs/kanso_radii.dart';
import '../../configs/kanso_sizes.dart';
import '../../utils/context_extensions.dart';
import 'kanso_pressable.dart';

/// An on/off switch, drawn by Kanso, the same on every platform.
///
/// Without [onChanged] it is drawn dimmed and does nothing.
class KansoSwitch extends StatelessWidget {
  const KansoSwitch({required this.value, this.onChanged, super.key});

  final bool value;
  final ValueChanged<bool>? onChanged;

  static const double inset = 2;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final changed = onChanged;
    const knob = KansoSizes.switchHeight - inset * 2;
    final duration = KansoMotion.resolve(context, KansoMotion.short);
    return Semantics(
      toggled: value,
      child: KansoPressable(
        onTap: changed == null ? null : () => changed(!value),
        child: Opacity(
          opacity: changed == null ? KansoEffects.disabled : 1,
          child: AnimatedContainer(
            duration: duration,
            curve: KansoMotion.standard,
            width: KansoSizes.switchWidth,
            height: KansoSizes.switchHeight,
            padding: const EdgeInsets.all(inset),
            alignment: value ? Alignment.centerRight : Alignment.centerLeft,
            decoration: BoxDecoration(
              color: value ? colors.button : colors.selected,
              borderRadius: KansoRadii.pillBorder,
            ),
            child: Container(
              width: knob,
              height: knob,
              decoration: BoxDecoration(
                color: value ? colors.buttonText : colors.textMuted,
                shape: BoxShape.circle,
              ),
            ),
          ),
        ),
      ),
    );
  }
}
