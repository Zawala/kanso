// Kanso — keyboard_placeholder.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_sizes.dart';
import '../../../core/utils/context_extensions.dart';

/// A plain block the height of a phone's keyboard, standing in for it in
/// the mock: no keys, nothing that belongs to an operating system.
class KeyboardPlaceholder extends StatelessWidget {
  const KeyboardPlaceholder({super.key});

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      height: KansoSizes.keyboardHeight,
      width: double.infinity,
      child: ColoredBox(color: context.colors.selected),
    );
  }
}
