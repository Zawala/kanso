// Kanso — preview_due_chip.dart
// Created 2026-10-30.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../../core/bridge/api/types.dart';
import 'preview_style.dart';

/// A due date as the page draws it: a small rounded chip in its tone,
/// [size] the size of the text it is in.
class PreviewDueChip extends StatelessWidget {
  const PreviewDueChip({
    required this.label,
    required this.title,
    required this.tone,
    required this.size,
    this.onTap,
    super.key,
  });

  final String label;
  final String title;
  final DueTone tone;
  final double size;

  /// What a tap on it does; nothing takes the tap without one.
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) {
    final look = PreviewScope.of(context).style;
    final red = look.palette.dark
        ? const Color(0xFFF2766B)
        : const Color(0xFFB3261E);
    Color tint(Color color) => previewOver(color, look.background, 0.16);
    final (Color text, Color fill) = switch (tone) {
      DueTone.plain => (look.muted, look.surface),
      DueTone.today => (look.accent, tint(look.accent)),
      DueTone.overdue => (red, tint(red)),
      DueTone.done => (
        look.muted,
        previewOver(look.surface, look.background, 0.6),
      ),
    };
    final fontSize = size * 0.8667;
    final Widget box = Container(
      padding: EdgeInsets.symmetric(horizontal: fontSize * 0.55),
      decoration: BoxDecoration(
        color: fill,
        borderRadius: BorderRadius.circular(999),
      ),
      child: Text(
        label,
        style: look.body.copyWith(
          fontSize: fontSize,
          height: 1.5,
          color: text,
          fontWeight: FontWeight.w400,
          fontStyle: FontStyle.normal,
          decoration: TextDecoration.none,
        ),
      ),
    );
    final tap = onTap;
    final chip = tap == null ? box : GestureDetector(onTap: tap, child: box);
    return title.isEmpty ? chip : Tooltip(message: title, child: chip);
  }
}
