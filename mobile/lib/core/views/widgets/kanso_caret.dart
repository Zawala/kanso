// Kanso — kanso_caret.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../utils/context_extensions.dart';

/// The text cursor, drawn in the accent colour.
class KansoCaret extends StatelessWidget {
  const KansoCaret({this.height = 20, super.key});

  final double height;

  @override
  Widget build(BuildContext context) {
    return Container(
      width: 2,
      height: height,
      decoration: BoxDecoration(
        color: context.colors.accent,
        borderRadius: BorderRadius.circular(1),
      ),
    );
  }
}
