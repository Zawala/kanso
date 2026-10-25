// Kanso — kanso_pressable.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_effects.dart';
import '../../configs/kanso_motion.dart';

/// Makes its [child] tappable, and dims it while a finger is on it.
///
/// There is no ink splash: the feedback is the same on every platform. With
/// no [onTap] the child is drawn as it is and does not react. Give
/// [semanticLabel] to a child that has no text of its own.
class KansoPressable extends StatefulWidget {
  const KansoPressable({
    required this.child,
    this.onTap,
    this.semanticLabel,
    super.key,
  });

  final Widget child;
  final VoidCallback? onTap;
  final String? semanticLabel;

  @override
  State<KansoPressable> createState() => _KansoPressableState();
}

class _KansoPressableState extends State<KansoPressable> {
  final ValueNotifier<bool> pressed = ValueNotifier<bool>(false);

  @override
  void dispose() {
    pressed.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final onTap = widget.onTap;
    return Semantics(
      button: onTap != null,
      enabled: onTap != null,
      label: widget.semanticLabel,
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: onTap,
        onTapDown: onTap == null ? null : (_) => pressed.value = true,
        onTapUp: onTap == null ? null : (_) => pressed.value = false,
        onTapCancel: onTap == null ? null : () => pressed.value = false,
        child: ValueListenableBuilder<bool>(
          valueListenable: pressed,
          builder: (context, isPressed, child) {
            return AnimatedOpacity(
              opacity: isPressed ? KansoEffects.pressed : 1,
              duration: KansoMotion.resolve(context, KansoMotion.fast),
              child: child,
            );
          },
          child: widget.child,
        ),
      ),
    );
  }
}
